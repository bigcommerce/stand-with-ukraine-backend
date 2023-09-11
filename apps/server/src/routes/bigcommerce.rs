use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::header::LOCATION;
use sqlx::PgPool;

use crate::{
    authentication::{create_jwt, Error},
    bigcommerce::client::HttpAPI,
    configuration::{BaseURL, JWTSecret},
    data::{write_store_as_uninstalled, write_store_credentials},
};

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/bigcommerce")
            .route("/install", web::get().to(install))
            .route("/uninstall", web::get().to(uninstall))
            .route("/load", web::get().to(load)),
    );
}

#[derive(serde::Deserialize)]
struct InstallQuery {
    code: String,
    scope: String,
    context: String,
}

#[derive(thiserror::Error, Debug)]
enum InstallError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for InstallError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[tracing::instrument(
    name = "Process install request",
    skip(query, bigcommerce_client, base_url, db_pool, jwt_secret),
    fields(context=tracing::field::Empty, user_email=tracing::field::Empty)
)]
async fn install(
    query: web::Query<InstallQuery>,
    bigcommerce_client: web::Data<HttpAPI>,
    base_url: web::Data<BaseURL>,
    db_pool: web::Data<PgPool>,
    jwt_secret: web::Data<JWTSecret>,
) -> Result<HttpResponse, InstallError> {
    tracing::Span::current().record("context", &tracing::field::display(&query.context));

    let oauth_credentials = bigcommerce_client
        .authorize_oauth_install(&query.code, &query.scope, &query.context)
        .await
        .context("Failed to validate credentials")
        .map_err(InstallError::UnexpectedError)?;

    tracing::Span::current().record(
        "user_email",
        &tracing::field::display(&oauth_credentials.user.email),
    );

    let store = oauth_credentials
        .get_bigcommerce_store()
        .map_err(InstallError::UnexpectedError)?;

    write_store_credentials(&store, &db_pool)
        .await
        .context("Failed to store credentials in database")
        .map_err(InstallError::UnexpectedError)?;

    let jwt = create_jwt(store.get_store_hash(), jwt_secret.get_ref())
        .context("Failed to encode jwt token")
        .map_err(InstallError::UnexpectedError)?;

    Ok(HttpResponse::Found()
        .append_header((
            LOCATION,
            generate_dashboard_url(&base_url.0, &jwt, store.get_store_hash()),
        ))
        .finish())
}

#[derive(serde::Deserialize)]
struct LoadQuery {
    signed_payload_jwt: String,
}
#[derive(thiserror::Error, Debug)]
enum LoadError {
    #[error("Not store owner.")]
    NotStoreOwnerError,

    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for LoadError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::NotStoreOwnerError | Self::InvalidCredentials(_) => {
                HttpResponse::new(StatusCode::UNAUTHORIZED)
            }
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[tracing::instrument(
    name = "Process load request",
    skip(query, bigcommerce_client, base_url, jwt_secret)
)]
async fn load(
    query: web::Query<LoadQuery>,
    bigcommerce_client: web::Data<HttpAPI>,
    base_url: web::Data<BaseURL>,
    jwt_secret: web::Data<JWTSecret>,
) -> Result<HttpResponse, LoadError> {
    let claims = bigcommerce_client
        .decode_jwt(&query.signed_payload_jwt)
        .map_err(LoadError::InvalidCredentials)?;

    let store_hash = claims
        .get_store_hash()
        .map_err(LoadError::UnexpectedError)?;

    let jwt = create_jwt(store_hash, jwt_secret.as_ref())
        .context("Failed to encode token")
        .map_err(LoadError::UnexpectedError)?;

    Ok(HttpResponse::Found()
        .append_header((
            LOCATION,
            generate_dashboard_url(&base_url.0, &jwt, store_hash),
        ))
        .finish())
}

#[tracing::instrument(
    name = "Process uninstall request",
    skip(query, bigcommerce_client, db_pool)
)]
async fn uninstall(
    query: web::Query<LoadQuery>,
    bigcommerce_client: web::Data<HttpAPI>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, LoadError> {
    let claims = bigcommerce_client
        .decode_jwt(&query.signed_payload_jwt)
        .map_err(LoadError::InvalidCredentials)?;

    if !claims.is_owner() {
        return Err(LoadError::NotStoreOwnerError);
    }

    let store_hash = claims
        .get_store_hash()
        .map_err(LoadError::UnexpectedError)?;

    write_store_as_uninstalled(store_hash, &db_pool)
        .await
        .context("Failed to set store as uninstalled")
        .map_err(LoadError::UnexpectedError)?;

    Ok(HttpResponse::Ok().finish())
}

fn generate_dashboard_url(base_url: &str, token: &str, store_hash: &str) -> String {
    format!("{base_url}/dashboard/?token={token}&store-id={store_hash}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_dashboard_url() {
        let dashboard_url = generate_dashboard_url("test.com", "test.test.test", "test-store");

        assert_eq!(
            dashboard_url,
            "test.com/dashboard/?token=test.test.test&store-id=test-store"
        )
    }
}

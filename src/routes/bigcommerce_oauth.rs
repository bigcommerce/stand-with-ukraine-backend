use actix_web::{
    http::StatusCode,
    web::{self},
    HttpResponse, ResponseError,
};
use anyhow::Context;
use reqwest::header::LOCATION;
use sqlx::PgPool;

use crate::{
    authentication::create_jwt,
    bigcommerce::BCClient,
    configuration::{ApplicationBaseUrl, JWTSecret},
    data::{save_store_credentials, set_store_as_uninstalled},
};

#[derive(serde::Deserialize)]
pub struct InstallQuery {
    code: String,
    scope: String,
    context: String,
}

#[derive(thiserror::Error, Debug)]
pub enum InstallError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for InstallError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            Self::InvalidCredentials(_) => HttpResponse::new(StatusCode::UNAUTHORIZED),
        }
    }
}

#[tracing::instrument(
    name = "Process install request",
    skip(query, bigcommerce_client, base_url, db_pool, jwt_secret), fields(context=tracing::field::Empty, user_email=tracing::field::Empty)
)]
pub async fn install(
    query: web::Query<InstallQuery>,
    bigcommerce_client: web::Data<BCClient>,
    base_url: web::Data<ApplicationBaseUrl>,
    db_pool: web::Data<PgPool>,
    jwt_secret: web::Data<JWTSecret>,
) -> Result<HttpResponse, InstallError> {
    tracing::Span::current().record("context", &tracing::field::display(&query.context));

    let oauth_credentials = bigcommerce_client
        .authorize_oauth_install(&base_url.0, &query.code, &query.scope, &query.context)
        .await
        .context("Failed to validate install query")
        .map_err(InstallError::InvalidCredentials)?;

    tracing::Span::current().record(
        "user_email",
        &tracing::field::display(&oauth_credentials.user.email),
    );

    let store = oauth_credentials
        .get_bigcommerce_store()
        .map_err(InstallError::UnexpectedError)?;

    save_store_credentials(&store, &db_pool)
        .await
        .context("Failed to store credentials in database")
        .map_err(InstallError::UnexpectedError)?;

    let jwt = create_jwt(&store.store_hash, jwt_secret.as_ref())
        .context("Failed to encode jwt token")
        .map_err(InstallError::UnexpectedError)?;

    Ok(HttpResponse::Found()
        .append_header((LOCATION, format!("{}/?token={}", &base_url.0, &jwt)))
        .finish())
}

#[derive(serde::Deserialize)]
pub struct LoadQuery {
    signed_payload_jwt: String,
}

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for LoadError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            Self::InvalidCredentials(_) => HttpResponse::new(StatusCode::UNAUTHORIZED),
        }
    }
}

#[tracing::instrument(
    name = "Process load request",
    skip(query, bigcommerce_client, base_url, jwt_secret)
)]
pub async fn load(
    query: web::Query<LoadQuery>,
    bigcommerce_client: web::Data<BCClient>,
    base_url: web::Data<ApplicationBaseUrl>,
    jwt_secret: web::Data<JWTSecret>,
) -> Result<HttpResponse, LoadError> {
    let store = bigcommerce_client
        .decode_jwt(&query.signed_payload_jwt)
        .context("Failed to decode bigcommerce jwt")
        .map_err(LoadError::InvalidCredentials)?;

    let jwt = create_jwt(&store.store_hash, jwt_secret.as_ref())
        .context("Failed to encode jwt token")
        .map_err(LoadError::UnexpectedError)?;

    Ok(HttpResponse::Found()
        .append_header((LOCATION, format!("{}/?token={}", &base_url.0, &jwt)))
        .finish())
}

#[tracing::instrument(name = "Process uninstall request", skip(query, bigcommerce_client, db_pool), fields(store_hash=tracing::field::Empty, user_email=tracing::field::Empty))]
pub async fn uninstall(
    query: web::Query<LoadQuery>,
    bigcommerce_client: web::Data<BCClient>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, LoadError> {
    let store = bigcommerce_client
        .decode_jwt(&query.signed_payload_jwt)
        .context("Failed to decode bigcommerce jwt")
        .map_err(LoadError::InvalidCredentials)?;

    set_store_as_uninstalled(&store.store_hash, &db_pool)
        .await
        .context("Failed to mark store as uninstalled")
        .map_err(LoadError::UnexpectedError)?;

    Ok(HttpResponse::Ok().finish())
}

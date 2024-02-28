use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};

use crate::{
    authentication::{create_jwt, Error},
    data::{write_store_as_uninstalled, write_store_credentials},
    state::{AppState, SharedState},
};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/install", get(install))
        .route("/uninstall", get(uninstall))
        .route("/load", get(load))
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

impl IntoResponse for InstallError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[tracing::instrument(
    name = "Process install request",
    skip(query, bigcommerce_client, db_pool, jwt_secret, base_url),
    fields(context=tracing::field::Empty, user_email=tracing::field::Empty)
)]
async fn install(
    Query(query): Query<InstallQuery>,
    State(AppState {
        bigcommerce_client,
        db_pool,
        jwt_secret,
        base_url,
        ..
    }): State<AppState>,
) -> Result<Response, InstallError> {
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

    let jwt = create_jwt(store.get_store_hash(), &jwt_secret)
        .context("Failed to encode jwt token")
        .map_err(InstallError::UnexpectedError)?;

    Ok(Redirect::to(&generate_dashboard_url(
        &base_url,
        &jwt,
        store.get_store_hash(),
    ))
    .into_response())
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

impl IntoResponse for LoadError {
    fn into_response(self) -> Response {
        match self {
            Self::NotStoreOwnerError | Self::InvalidCredentials(_) => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

#[tracing::instrument(
    name = "Process load request",
    skip(query, bigcommerce_client, base_url, jwt_secret)
)]
async fn load(
    Query(query): Query<LoadQuery>,
    State(AppState {
        bigcommerce_client,
        base_url,
        jwt_secret,
        ..
    }): State<AppState>,
) -> Result<Response, LoadError> {
    let claims = bigcommerce_client
        .decode_jwt(&query.signed_payload_jwt)
        .map_err(LoadError::InvalidCredentials)?;

    let store_hash = claims
        .get_store_hash()
        .map_err(LoadError::UnexpectedError)?;

    let jwt = create_jwt(store_hash, &jwt_secret)
        .context("Failed to encode token")
        .map_err(LoadError::UnexpectedError)?;

    Ok(Redirect::to(&generate_dashboard_url(&base_url, &jwt, store_hash)).into_response())
}

#[tracing::instrument(
    name = "Process uninstall request",
    skip(query, bigcommerce_client, db_pool)
)]
async fn uninstall(
    Query(query): Query<LoadQuery>,
    State(AppState {
        bigcommerce_client,
        db_pool,
        ..
    }): State<AppState>,
) -> Result<Response, LoadError> {
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

    Ok(StatusCode::OK.into_response())
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

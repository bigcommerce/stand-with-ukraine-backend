use crate::liq_pay::InputQuery;
use crate::state::{App, Shared};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Router;

pub fn router() -> Router<Shared> {
    Router::new().route("/", get(pay))
}

#[derive(thiserror::Error, Debug)]
enum PayError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for PayError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[tracing::instrument(name = "Process pay request", skip(query, liq_pay_client))]
async fn pay(
    Query(query): Query<InputQuery>,
    State(App { liq_pay_client, .. }): State<App>,
) -> Result<Redirect, PayError> {
    let checkout_request = liq_pay_client
        .generate_request_payload(query, "Support BigCommerce colleagues defending Ukraine")?;

    let url = liq_pay_client
        .link(checkout_request)
        .map_err(PayError::UnexpectedError)?;

    Ok(Redirect::to(&url))
}

use crate::liq_pay::InputQuery;
use crate::state::{AppState, SharedState};
use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;

pub fn router() -> Router<SharedState> {
    Router::new().route("/", get(pay))
}

#[tracing::instrument(name = "Process pay request", skip(query, liq_pay_client))]
async fn pay(
    Query(query): Query<InputQuery>,
    State(AppState { liq_pay_client, .. }): State<AppState>,
) -> Redirect {
    let checkout_request = liq_pay_client
        .generate_request_payload(query, "Support BigCommerce colleagues defending Ukraine");

    let url = liq_pay_client.link(checkout_request);

    Redirect::to(&url)
}

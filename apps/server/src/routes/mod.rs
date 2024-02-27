use axum::{response::Response, routing::get, Router};

use crate::configuration::SharedState;

mod bigcommerce;
mod pay;
mod widget;

pub async fn health_check() -> Response {
    Response::new("".into())
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/health_check", get(health_check))
        .nest("/pay", pay::router())
        .nest("/api", widget::router())
        .nest("/bigcommerce", bigcommerce::router())
}

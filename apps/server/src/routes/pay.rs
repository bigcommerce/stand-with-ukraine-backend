use crate::liq_pay::HttpAPI;
use crate::liq_pay::InputQuery;
use actix_web::web::Redirect;
use actix_web::{web, HttpResponse, ResponseError};
use reqwest::StatusCode;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/pay").route(web::get().to(pay)));
}

#[derive(thiserror::Error, Debug)]
enum PayError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for PayError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[tracing::instrument(name = "Process pay request", skip(query, liq_pay))]
async fn pay(
    query: web::Query<InputQuery>,
    liq_pay: web::Data<HttpAPI>,
) -> Result<Redirect, PayError> {
    let checkout_request = liq_pay.generate_request_payload(
        query.into_inner(),
        "Support BigCommerce colleagues defending Ukraine",
    )?;

    let url = liq_pay
        .link(checkout_request)
        .map_err(PayError::UnexpectedError)?;

    Ok(Redirect::to(url))
}

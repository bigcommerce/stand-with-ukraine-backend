use crate::liq_pay::HttpAPI;
use crate::liq_pay::InputQuery;
use actix_web::web::Redirect;
use actix_web::{web, HttpResponse, ResponseError};
use reqwest::StatusCode;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/pay").route(web::get().to(handle)));
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

#[tracing::instrument(name = "Process load request", skip(query, liq_pay_links))]
async fn handle(
    query: web::Query<InputQuery>,
    liq_pay_links: web::Data<HttpAPI>,
) -> Result<Redirect, PayError> {
    let url = liq_pay_links
        .link(
            query.into_inner(),
            "Support BigCommerce colleagues defending Ukraine",
        )
        .map_err(PayError::UnexpectedError)?;

    Ok(Redirect::to(url))
}

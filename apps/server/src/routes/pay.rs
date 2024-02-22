use crate::payment_buttons::action::Action;
use actix_web::web::Redirect;
use actix_web::{web, Responder};
use serde::Deserialize;

use crate::payment_buttons::currency::Currency;
use crate::payment_buttons::language::Language;
use crate::payment_buttons::liqpay_client::LiqPayClient;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/pay").route(web::get().to(handle)));
}

#[derive(Deserialize)]
struct InputQuery {
    language: String,
    currency: String,
    sum: f64,
    action: String,
}

#[tracing::instrument(name = "Process load request", skip(query, liq_pay_links))]
async fn handle(
    query: web::Query<InputQuery>,
    liq_pay_links: web::Data<LiqPayClient>,
) -> impl Responder {
    let url = liq_pay_links.link(
        &query.sum,
        &Language::new(&query.language),
        &Currency::new(&query.currency),
        &Action::new(&query.action),
        "Support BigCommerce colleagues defending Ukraine",
    );
    Redirect::to(url)
}

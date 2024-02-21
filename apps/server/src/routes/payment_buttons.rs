use actix_web::{web, Responder, Result};
use serde::Deserialize;

use crate::payment_buttons::currency::Currency;
use crate::payment_buttons::language::Language;
use crate::payment_buttons::links::Links;
use crate::payment_buttons::liqpay_client::LiqPayClient;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/payment_buttons").route(web::get().to(load)));
}

#[derive(Deserialize)]
struct LoadQuery {
    language: String,
    currency: String,
    sum: String,
}

#[tracing::instrument(name = "Process load request", skip(query, liq_pay_links))]
async fn load(
    query: web::Query<LoadQuery>,
    liq_pay_links: web::Data<LiqPayClient>,
) -> Result<impl Responder> {
    Ok(web::Json(Links::new(
        &Language::new(&query.language),
        &Currency::new(&query.currency),
        &parse_sum(&query.sum),
        &liq_pay_links,
    )))
}

fn parse_sum(sum: &str) -> Vec<f64> {
    sum.split(',')
        .filter_map(|s| s.parse::<f64>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use actix_web::test;
    use secrecy::Secret;

    use super::*;

    #[test]
    async fn test_parse_sum() {
        assert_eq!(
            parse_sum("100,200,300,400"),
            vec![100.0, 200.0, 300.0, 400.0]
        );
    }

    #[test]
    async fn test_index_not_ok() {
        let q = web::Query::<LoadQuery>::from_query("language=ua&currency=usd&sum=100,200,300,400")
            .unwrap();
        let liq_pay_links = web::Data::new(LiqPayClient::new(
            Secret::new("public_key".to_string()),
            Secret::new("private_key".to_string()),
        ));
        let resp = load(q, liq_pay_links).await;
        assert_eq!(resp.is_ok(), true);
    }
}

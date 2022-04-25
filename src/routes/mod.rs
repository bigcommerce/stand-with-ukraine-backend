use actix_cors::Cors;
use actix_web::web;
use actix_web_httpauth::{extractors::bearer::Config, middleware::HttpAuthentication};

use crate::authentication::validate_jwt_bearer_token;

mod api;
mod bigcommerce_oauth;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let bearer_auth_config = Config::default().realm("api-v1").scope("modify");
    let auth_validator = HttpAuthentication::bearer(validate_jwt_bearer_token);

    cfg.service(
        web::scope("/api/v1")
            .app_data(bearer_auth_config)
            .wrap(auth_validator)
            .wrap(Cors::permissive())
            .route("/health_check", web::get().to(api::health_check)),
    );

    cfg.service(
        web::scope("/bigcommerce")
            .route("/install", web::get().to(bigcommerce_oauth::install))
            .route("/uninstall", web::get().to(bigcommerce_oauth::uninstall))
            .route("/load", web::get().to(bigcommerce_oauth::load))
            .route("/logout", web::get().to(bigcommerce_oauth::logout)),
    );
}

use actix_web::web;
use actix_web_httpauth::{extractors::bearer::Config, middleware::HttpAuthentication};

use crate::authentication::validate_jwt_bearer_token;

mod api;
mod bigcommerce;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let bearer_auth_config = Config::default().realm("api-v1").scope("modify");
    let auth_validator = HttpAuthentication::bearer(validate_jwt_bearer_token);

    cfg.service(web::resource("/health_check").route(web::get().to(api::health_check)));

    cfg.service(
        web::scope("/api/v1")
            .app_data(bearer_auth_config)
            .wrap(auth_validator)
            .route("/save", web::post().to(api::save_widget_configuration))
            .route("/publish", web::post().to(api::publish_widget))
            .route("/remove", web::delete().to(api::remove_widget)),
    );

    cfg.service(
        web::scope("/bigcommerce")
            .route("/install", web::get().to(bigcommerce::install))
            .route("/uninstall", web::get().to(bigcommerce::uninstall))
            .route("/load", web::get().to(bigcommerce::load)),
    );
}

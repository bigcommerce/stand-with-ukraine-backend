use actix_web::web;
use actix_web_httpauth::extractors::bearer::Config;

mod api;
mod bigcommerce;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let bearer_auth_config = Config::default().realm("api-v1").scope("modify");

    cfg.service(web::resource("/health_check").route(web::get().to(api::health_check)));

    cfg.service(
        web::scope("/api/v1")
            .app_data(bearer_auth_config)
            .route(
                "/configuration",
                web::post().to(api::save_widget_configuration),
            )
            .route(
                "/configuration",
                web::get().to(api::get_widget_configuration),
            )
            .route("/publish", web::post().to(api::publish_widget))
            .route("/publish", web::get().to(api::get_published_status))
            .route("/publish", web::delete().to(api::remove_widget)),
    );

    cfg.service(
        web::scope("/bigcommerce")
            .route("/install", web::get().to(bigcommerce::install))
            .route("/uninstall", web::get().to(bigcommerce::uninstall))
            .route("/load", web::get().to(bigcommerce::load)),
    );
}

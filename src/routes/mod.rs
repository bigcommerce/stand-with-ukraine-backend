use actix_web::{web, HttpResponse};

mod bigcommerce;
mod widget;

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/health_check").route(web::get().to(health_check)));

    bigcommerce::register_routes(cfg);
    widget::register_routes(cfg);
}

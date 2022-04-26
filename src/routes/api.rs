use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use sqlx::PgPool;

use crate::{
    authentication::AuthClaims,
    data::{read_widget_configuration, write_widget_configuration, WidgetConfiguration},
};

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigurationError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ConfigurationError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

pub async fn save_widget_configuration(
    auth: AuthClaims,
    widget_configuration: web::Json<WidgetConfiguration>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfigurationError> {
    let err = write_widget_configuration(auth.sub.as_str(), &widget_configuration, &db_pool).await;

    match err {
        Err(e) => {
            dbg!(e);

            Err(ConfigurationError::UnexpectedError(anyhow::anyhow!("test")))
        }
        Ok(_) => Ok(HttpResponse::Ok().finish()),
    }
}

pub async fn get_widget_configuration(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfigurationError> {
    let widget_configuration = read_widget_configuration(auth.sub.as_str(), &db_pool)
        .await
        .map_err(ConfigurationError::UnexpectedError)?;

    Ok(HttpResponse::Ok().json(widget_configuration))
}

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

pub async fn publish_widget() -> Result<HttpResponse, PublishError> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn remove_widget() -> Result<HttpResponse, PublishError> {
    Ok(HttpResponse::Ok().finish())
}

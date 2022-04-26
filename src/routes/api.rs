use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;

use crate::{
    authentication::AuthClaims,
    bigcommerce::BCClient,
    configuration::ApplicationBaseUrl,
    data::{
        read_store_credentials, read_widget_configuration, write_store_published,
        write_widget_configuration, WidgetConfiguration,
    },
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

pub fn generate_script_content(
    widget_configuration: &WidgetConfiguration,
    base_url: &ApplicationBaseUrl,
) -> Result<String, serde_json::Error> {
    Ok(format!(
        r#"
    <script>window.SWU_CONFIG = {};</script>
    <script src="{}/widget/index.js"></script>
    "#,
        serde_json::to_string(widget_configuration)?,
        base_url.0
    ))
}

pub async fn publish_widget(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
    base_url: web::Data<ApplicationBaseUrl>,
    bigcommerce_client: web::Data<BCClient>,
) -> Result<HttpResponse, PublishError> {
    let store_hash = auth.sub.as_str();
    let widget_configuration = read_widget_configuration(store_hash, &db_pool)
        .await
        .map_err(PublishError::UnexpectedError)?;

    let script_content = generate_script_content(&widget_configuration, &base_url)
        .context("Failed to generate script content")
        .map_err(PublishError::UnexpectedError)?;

    let store = read_store_credentials(store_hash, &db_pool)
        .await
        .context("Failed to get store credentials")
        .map_err(PublishError::UnexpectedError)?;

    bigcommerce_client
        .create_script(store, script_content)
        .await
        .context("Failed to create script in BigCommerce")
        .map_err(PublishError::UnexpectedError)?;

    write_store_published(store_hash, true, &db_pool)
        .await
        .context("Failed to set store as published")
        .map_err(PublishError::UnexpectedError)?;

    Ok(HttpResponse::Ok().finish())
}

pub async fn remove_widget(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
    bigcommerce_client: web::Data<BCClient>,
) -> Result<HttpResponse, PublishError> {
    let store_hash = auth.sub.as_str();

    let store = read_store_credentials(store_hash, &db_pool)
        .await
        .context("Failed to get store credentials")
        .map_err(PublishError::UnexpectedError)?;

    bigcommerce_client
        .remove_scripts(store)
        .await
        .context("Failed to remove scripts in BigCommerce")
        .map_err(PublishError::UnexpectedError)?;

    write_store_published(store_hash, false, &db_pool)
        .await
        .context("Failed to set store as not published")
        .map_err(PublishError::UnexpectedError)?;

    Ok(HttpResponse::Ok().finish())
}

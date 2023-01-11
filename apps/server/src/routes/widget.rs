use crate::{
    authentication::AuthClaims,
    bigcommerce::client::BCClient,
    configuration::BaseURL,
    data::{
        read_store_credentials, read_store_published, read_widget_configuration,
        write_charity_visited_event, write_general_feedback, write_store_published,
        write_unpublish_feedback, write_widget_configuration, write_widget_event, CharityEvent,
        FeedbackForm, WidgetConfiguration, WidgetEvent,
    },
};

use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use actix_web_httpauth::extractors::bearer::Config;
use anyhow::Context;
use sqlx::PgPool;

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    let bearer_auth_config = Config::default().realm("api-v1").scope("modify");

    cfg.service(
        web::scope("/api/v1")
            .app_data(bearer_auth_config)
            .route("/configuration", web::post().to(save_widget_configuration))
            .route("/configuration", web::get().to(get_widget_configuration))
            .route("/publish", web::post().to(publish_widget))
            .route("/publish", web::get().to(get_published_status))
            .route("/publish", web::delete().to(remove_widget))
            .route("/preview", web::get().to(preview_widget)),
    );

    let cors = actix_cors::Cors::permissive();

    cfg.service(
        web::scope("/api/v2")
            .wrap(cors)
            .route("/widget-event", web::post().to(log_widget_event))
            .route("/charity-event", web::post().to(log_charity_event))
            .route("/feedback-form", web::post().to(submit_general_feedback)),
    );
}

#[derive(thiserror::Error, Debug)]
enum ConfigurationError {
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

#[tracing::instrument(name = "Save widget configuration", skip(auth, db_pool))]
async fn save_widget_configuration(
    auth: AuthClaims,
    widget_configuration: web::Json<WidgetConfiguration>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfigurationError> {
    write_widget_configuration(auth.sub.as_str(), &widget_configuration, &db_pool)
        .await
        .map_err(ConfigurationError::UnexpectedError)?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get widget configuration", skip(auth, db_pool))]
async fn get_widget_configuration(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfigurationError> {
    let widget_configuration = read_widget_configuration(auth.sub.as_str(), &db_pool)
        .await
        .map_err(ConfigurationError::UnexpectedError)?;

    Ok(HttpResponse::Ok().json(widget_configuration))
}

#[derive(thiserror::Error, Debug)]
enum PublishError {
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

#[tracing::instrument(
    name = "Publish the widget",
    skip(auth, base_url, db_pool, bigcommerce_client)
)]
async fn publish_widget(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
    base_url: web::Data<BaseURL>,
    bigcommerce_client: web::Data<BCClient>,
) -> Result<HttpResponse, PublishError> {
    let store_hash = auth.sub.as_str();
    let widget_configuration = read_widget_configuration(store_hash, &db_pool)
        .await
        .map_err(PublishError::UnexpectedError)?;

    let script = widget_configuration
        .generate_script(store_hash, &base_url)
        .context("Failed to generate script content")
        .map_err(PublishError::UnexpectedError)?;

    let store = read_store_credentials(store_hash, &db_pool)
        .await
        .map_err(PublishError::UnexpectedError)?;

    let existing_script = bigcommerce_client
        .try_get_script_with_name(&store, script.get_name())
        .await
        .map_err(PublishError::UnexpectedError)?;

    match existing_script {
        Some(existing_script) => {
            bigcommerce_client
                .update_script(&store, &existing_script.uuid, &script)
                .await
        }
        None => bigcommerce_client.create_script(&store, &script).await,
    }
    .map_err(PublishError::UnexpectedError)?;

    write_store_published(store_hash, true, &db_pool)
        .await
        .context("Failed to set store as published")
        .map_err(PublishError::UnexpectedError)?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(serde::Deserialize)]
struct Feedback {
    reason: Option<String>,
}

#[tracing::instrument(
    name = "Remove widget",
    skip(auth, db_pool, bigcommerce_client, feedback)
)]
async fn remove_widget(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
    bigcommerce_client: web::Data<BCClient>,
    feedback: web::Query<Feedback>,
) -> Result<HttpResponse, PublishError> {
    let store_hash = auth.sub.as_str();

    let store = read_store_credentials(store_hash, &db_pool)
        .await
        .context("Failed to get store credentials")
        .map_err(PublishError::UnexpectedError)?;

    bigcommerce_client
        .remove_all_scripts(&store)
        .await
        .context("Failed to remove scripts in BigCommerce")
        .map_err(PublishError::UnexpectedError)?;

    write_store_published(store_hash, false, &db_pool)
        .await
        .context("Failed to set store as not published")
        .map_err(PublishError::UnexpectedError)?;

    let feedback = feedback.into_inner();
    if let Some(reason) = feedback.reason {
        write_unpublish_feedback(store_hash, reason.as_str(), &db_pool)
            .await
            .context("Failed to record unpublish feedback")
            .map_err(PublishError::UnexpectedError)?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Preview widget", skip(auth, db_pool, bigcommerce_client))]
async fn preview_widget(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
    bigcommerce_client: web::Data<BCClient>,
) -> Result<HttpResponse, PublishError> {
    let store_hash = auth.sub.as_str();

    let store = read_store_credentials(store_hash, &db_pool)
        .await
        .context("Failed to get store credentials")
        .map_err(PublishError::UnexpectedError)?;

    let store_information = bigcommerce_client
        .get_store_information(&store)
        .await
        .context("Failed to get store information")
        .map_err(PublishError::UnexpectedError)?;

    Ok(HttpResponse::Ok().json(store_information))
}

#[tracing::instrument(name = "Get widget status", skip(auth, db_pool))]
async fn get_published_status(
    auth: AuthClaims,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, PublishError> {
    let store_hash = auth.sub.as_str();

    let store_status = read_store_published(store_hash, &db_pool)
        .await
        .context("Failed to get store status")
        .map_err(PublishError::UnexpectedError)?;

    Ok(HttpResponse::Ok().json(store_status))
}

#[tracing::instrument(name = "Log charity event", skip(db_pool))]
async fn log_charity_event(
    event: web::Query<CharityEvent>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    if let Err(error) = write_charity_visited_event(&event.into_inner(), &db_pool).await {
        tracing::warn!("Error while saving event {}", error);
    };

    HttpResponse::Ok().finish()
}

#[tracing::instrument(name = "Save feedback form", skip(db_pool))]
async fn submit_general_feedback(
    event: web::Query<FeedbackForm>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    if let Err(error) = write_general_feedback(&event.into_inner(), &db_pool).await {
        tracing::warn!("Error while saving event {}", error);
    };

    HttpResponse::Ok().finish()
}

#[tracing::instrument(name = "Log widget event", skip(db_pool))]
async fn log_widget_event(
    event: web::Query<WidgetEvent>,
    db_pool: web::Data<PgPool>,
) -> HttpResponse {
    if let Err(error) = write_widget_event(&event.into_inner(), &db_pool).await {
        tracing::warn!("Error while saving event {}", error);
    };

    HttpResponse::Ok().finish()
}

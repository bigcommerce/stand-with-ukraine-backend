use crate::{
    authentication::AuthClaims,
    data::{
        read_store_credentials, read_store_published, read_widget_configuration,
        write_charity_visited_event, write_general_feedback, write_store_published,
        write_universal_widget_event, write_unpublish_feedback, write_widget_configuration,
        write_widget_event, CharityEvent, FeedbackForm, UniversalConfiguratorEvent,
        WidgetConfiguration, WidgetEvent,
    },
    state::{AppState, SharedState},
};

use tower_http::cors::CorsLayer;

use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};

pub fn router() -> Router<SharedState> {
    let v1_router = Router::new()
        .route("/configuration", post(save_widget_configuration))
        .route("/configuration", get(get_widget_configuration))
        .route("/publish", post(publish_widget))
        .route("/publish", get(get_published_status))
        .route("/publish", delete(remove_widget))
        .route("/preview", get(preview_widget));

    let cors = CorsLayer::permissive();

    let v2_router = Router::new()
        .layer(cors)
        .route("/widget-event", post(log_widget_event))
        .route("/charity-event", post(log_charity_event))
        .route("/feedback-form", post(submit_general_feedback))
        .route(
            "/universal-event",
            post(submit_universal_configurator_event),
        );

    Router::new().nest("/v1", v1_router).nest("/v2", v2_router)
}

#[derive(thiserror::Error, Debug)]
enum ConfigurationError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for ConfigurationError {
    #[tracing::instrument(name = "configuration error")]
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[tracing::instrument(name = "save widget configuration", skip(auth, db_pool))]
async fn save_widget_configuration(
    auth: AuthClaims,
    State(AppState { db_pool, .. }): State<AppState>,
    Json(widget_configuration): Json<WidgetConfiguration>,
) -> Result<Response, ConfigurationError> {
    let store_hash = auth.sub.as_str();

    write_widget_configuration(store_hash, &widget_configuration, &db_pool)
        .await
        .map_err(ConfigurationError::UnexpectedError)?;

    Ok(StatusCode::OK.into_response())
}

#[tracing::instrument(name = "get widget configuration", skip(auth, db_pool))]
async fn get_widget_configuration(
    auth: AuthClaims,
    State(AppState { db_pool, .. }): State<AppState>,
) -> Result<Response, ConfigurationError> {
    let store_hash = auth.sub.as_str();

    let widget_configuration = read_widget_configuration(store_hash, &db_pool)
        .await
        .map_err(ConfigurationError::UnexpectedError)?;

    Ok(Json(widget_configuration).into_response())
}

#[derive(thiserror::Error, Debug)]
enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for PublishError {
    #[tracing::instrument(name = "publish error")]
    fn into_response(self) -> Response {
        match self {
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

#[tracing::instrument(
    name = "publish widget",
    skip(auth, db_pool, base_url, bigcommerce_client)
)]
async fn publish_widget(
    auth: AuthClaims,
    State(AppState {
        db_pool,
        base_url,
        bigcommerce_client,
        ..
    }): State<AppState>,
) -> Result<Response, PublishError> {
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

    Ok(StatusCode::OK.into_response())
}

#[derive(serde::Deserialize)]
struct Feedback {
    reason: Option<String>,
}

#[tracing::instrument(
    name = "remove widget",
    skip(auth, db_pool, bigcommerce_client, feedback)
)]
async fn remove_widget(
    auth: AuthClaims,
    State(AppState {
        db_pool,
        bigcommerce_client,
        ..
    }): State<AppState>,
    Query(feedback): Query<Feedback>,
) -> Result<Response, PublishError> {
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

    if let Some(reason) = feedback.reason {
        write_unpublish_feedback(store_hash, reason.as_str(), &db_pool)
            .await
            .context("Failed to record unpublish feedback")
            .map_err(PublishError::UnexpectedError)?;
    }

    Ok(StatusCode::OK.into_response())
}

#[tracing::instrument(name = "preview widget", skip(auth, db_pool, bigcommerce_client))]
async fn preview_widget(
    auth: AuthClaims,
    State(AppState {
        db_pool,
        bigcommerce_client,
        ..
    }): State<AppState>,
) -> Result<Response, PublishError> {
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

    Ok(Json(store_information).into_response())
}

#[tracing::instrument(name = "get widget status", skip(auth, db_pool))]
async fn get_published_status(
    auth: AuthClaims,
    State(AppState { db_pool, .. }): State<AppState>,
) -> Result<Response, PublishError> {
    let store_hash = auth.sub.as_str();

    let store_status = read_store_published(store_hash, &db_pool)
        .await
        .context("Failed to get store status")
        .map_err(PublishError::UnexpectedError)?;

    Ok(Json(store_status).into_response())
}

#[tracing::instrument(name = "log charity event", skip(db_pool))]
async fn log_charity_event(
    Query(event): Query<CharityEvent>,
    State(AppState { db_pool, .. }): State<AppState>,
) -> Response {
    if let Err(error) = write_charity_visited_event(&event, &db_pool).await {
        tracing::warn!("error while saving event {}", error);
    };

    StatusCode::OK.into_response()
}

#[tracing::instrument(name = "save feedback form", skip(db_pool))]
async fn submit_general_feedback(
    Query(event): Query<FeedbackForm>,
    State(AppState { db_pool, .. }): State<AppState>,
) -> Response {
    if let Err(error) = write_general_feedback(&event, &db_pool).await {
        tracing::warn!("error while saving event {}", error);
    };

    StatusCode::OK.into_response()
}

#[tracing::instrument(name = "save universal configurator event", skip(db_pool))]
async fn submit_universal_configurator_event(
    Query(event): Query<UniversalConfiguratorEvent>,
    State(AppState { db_pool, .. }): State<AppState>,
) -> Response {
    if let Err(error) = write_universal_widget_event(&event, &db_pool).await {
        tracing::warn!("error while saving event {}", error);
    };

    StatusCode::OK.into_response()
}

#[tracing::instrument(name = "log widget event", skip(db_pool))]
async fn log_widget_event(
    Query(event): Query<WidgetEvent>,
    State(AppState { db_pool, .. }): State<AppState>,
) -> Response {
    if let Err(error) = write_widget_event(&event, &db_pool).await {
        tracing::warn!("error while saving event {}", error);
    };

    StatusCode::OK.into_response()
}

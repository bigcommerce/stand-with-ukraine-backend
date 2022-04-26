use actix_web::{http::StatusCode, HttpResponse, ResponseError};

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(thiserror::Error, Debug)]
pub enum SaveOrPublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for SaveOrPublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

pub async fn save_widget_configuration() -> Result<HttpResponse, SaveOrPublishError> {
    Ok(HttpResponse::Ok().finish())
}

pub async fn publish_widget() -> Result<HttpResponse, SaveOrPublishError> {
    Ok(HttpResponse::Ok().finish())
}

#[derive(thiserror::Error, Debug)]
pub enum RemoveError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for RemoveError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnexpectedError(_) => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

pub async fn remove_widget() -> Result<HttpResponse, RemoveError> {
    Ok(HttpResponse::Ok().finish())
}

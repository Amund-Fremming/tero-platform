use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use tracing::error;

use crate::{auth::auth_models::Permission, client::gamesession_client::GameSessionClientError};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Sqlx failed: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Api error: {1}")]
    Api(StatusCode, String),

    #[error("Permission error")]
    Permission(Permission),

    #[error("Access denied error")]
    AccessDenied,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JWT verification error: {0}")]
    JwtVerification(String),

    #[error("Gust Cache error: {0}")]
    Cache(#[from] gustcache::CacheError),

    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing env var: {0}")]
    MissingEnv(String),

    #[error("GameSessionClient error: {0}")]
    GameSessionClientError(#[from] GameSessionClientError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServerError::Sqlx(e) => {
                error!("Sqlx failed with error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, String::new())
            }
            ServerError::Internal(e) => {
                error!("Internal server error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, String::new())
            }
            ServerError::Api(sc, msg) => {
                error!("Api error: {} - {}", sc, msg);
                (sc, msg)
            }
            ServerError::Permission(missing) => {
                error!("Missing permission: {:?}", missing);
                (
                    StatusCode::FORBIDDEN,
                    format!("Missing permission: {:?}", missing),
                )
            }
            ServerError::NotFound(e) => {
                error!("Entity not found: {}", e);
                (StatusCode::NOT_FOUND, e)
            }
            ServerError::AccessDenied => {
                error!("Access denied for requesting entity");
                (StatusCode::FORBIDDEN, String::from("Access denied"))
            }
            ServerError::Request(e) => {
                error!("Failed to send request: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("Failed to access third party"),
                )
            }
            ServerError::JwtVerification(e) => {
                error!("Failed to verify JWT: {}", e);
                (StatusCode::UNAUTHORIZED, String::new())
            }
            ServerError::Json(e) => {
                error!("Json error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, String::new())
            }
            ServerError::Cache(e) => {
                error!("Gust Cache error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, String::new())
            }
            ServerError::MissingEnv(e) => {
                error!("Missing env var: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, String::new())
            }
            ServerError::GameSessionClientError(e) => {
                error!("GameSessionClient error: {}", e);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    String::from("Upstream service unavailable"),
                )
            }
        }
        .into_response()
    }
}

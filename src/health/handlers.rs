use std::sync::Arc;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    health::db,
    server::{app_state::AppState, server_error::ServerError},
};

pub fn health_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(health))
        .route("/detailed", get(health_detailed))
        .with_state(state.clone())
}

async fn health() -> impl IntoResponse {
    "OK".into_response()
}

async fn health_detailed(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ServerError> {
    let db_platform = "healthy".to_string(); // TODO - check real

    let db_status = match db::health_check(state.get_pool()).await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    let session_status = match state
        .get_session_client()
        .health_check(state.get_client())
        .await
    {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    };

    let json = json!({
        "platform": db_platform,
        "database": db_status,
        "session": session_status,
    });

    Ok((StatusCode::OK, Json(json)))
}

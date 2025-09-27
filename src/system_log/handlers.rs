use std::sync::Arc;

use axum::{Extension, Json, Router, extract::State, response::IntoResponse, routing::post};
use reqwest::StatusCode;

use crate::{
    auth::models::{PermissionCtx, SubjectId},
    server::{app_state::AppState, error::ServerError},
    system_log::models::SystemLogRequest,
};

pub fn log_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_system_log))
        .with_state(state)
}

async fn create_system_log(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(permission_ctx): Extension<PermissionCtx>,
    Json(request): Json<SystemLogRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let mut builder = state.audit();

    if let Some(action) = request.action {
        builder = builder.action(action);
    };

    if let Some(ceverity) = request.ceverity {
        builder = builder.ceverity(ceverity);
    }

    if let Some(description) = request.description {
        builder = builder.description(description);
    }

    if let Some(metadata) = request.metadata {
        builder = builder.metadata(metadata);
    }

    builder.log_async();

    Ok(StatusCode::CREATED)
}

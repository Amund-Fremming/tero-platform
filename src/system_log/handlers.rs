use std::sync::Arc;

use tracing::{error, info};

use axum::{Extension, Json, Router, extract::State, response::IntoResponse, routing::post};
use reqwest::StatusCode;

use crate::{
    auth::models::{Claims, Permission, SubjectId},
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
    Extension(claims): Extension<Claims>,
    Json(request): Json<SystemLogRequest>,
) -> Result<impl IntoResponse, ServerError> {
    match subject_id {
        SubjectId::Guest(_) | SubjectId::Registered(_) => {
            error!("User tried writing a system log");
            return Err(ServerError::AccessDenied);
        }
        SubjectId::Integration(int_name) => {
            if let Some(missing) = claims.missing_permission([Permission::WriteSystemLog]) {
                return Err(ServerError::Permission(missing));
            }

            info!("Integration {} is writing a system log", int_name);
        }
    };

    let mut builder = state.audit();

    if let Some(action) = request.action {
        builder = builder.action(action);
    };

    if let Some(ceverity) = request.ceverity {
        builder = builder.ceverity(ceverity);
    }

    if let Some(description) = request.description {
        builder = builder.description(&description);
    }

    if let Some(metadata) = request.metadata {
        builder = builder.metadata(metadata);
    }

    builder.log_async();

    Ok(StatusCode::CREATED)
}

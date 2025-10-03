use std::sync::Arc;

use tracing::{error, info};

use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;

use crate::{
    auth::models::{Claims, Permission, SubjectId},
    common::{app_state::AppState, error::ServerError},
    system_log::{
        db,
        models::{CreateSyslogRequest, SyslogPageQuery},
    },
};

pub fn log_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_system_log))
        .route("/", get(get_system_log_page))
        .with_state(state)
}

async fn get_system_log_page(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<SyslogPageQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(_) = subject_id else {
        error!("Unauthorized subject tried reading system logs");
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::ReadAdmin]) {
        return Err(ServerError::Permission(missing));
    }

    let page = db::get_system_log_page(state.get_pool(), query).await?;
    Ok((StatusCode::OK, Json(page)))
}

async fn create_system_log(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateSyslogRequest>,
) -> Result<impl IntoResponse, ServerError> {
    match &subject_id {
        SubjectId::Guest(id) | SubjectId::Registered(id) => {
            error!("User {} tried writing a system log", id);
            return Err(ServerError::AccessDenied);
        }
        SubjectId::Integration(int_name) => {
            if let Some(missing) = claims.missing_permission([Permission::WriteSystemLog]) {
                return Err(ServerError::Permission(missing));
            }

            info!("Integration {} is writing a system log", int_name);
        }
    };

    let mut builder = state.audit().subject(subject_id);

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

    if let Some(function) = request.function {
        builder = builder.function(&function);
    }

    builder.log_async();

    Ok(StatusCode::CREATED)
}

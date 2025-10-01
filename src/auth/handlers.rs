use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post, put},
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::{
        db::{self},
        models::{Auth0User, Claims, Permission, PutUserRequest, SubjectId},
    },
    server::{app_state::AppState, error::ServerError},
    system_log::models::{Action, LogCeverity},
};

pub fn public_auth_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create_guest_user))
        .with_state(state)
}

pub fn protected_auth_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_user_from_subject))
        .route(
            "/{user_id}",
            patch(patch_user)
                .delete(delete_user)
                .post(auth0_trigger_endpoint),
        )
        .route("/list", get(list_all_users))
        .route("/activity/{user_id}", put(patch_user_activity))
        .with_state(state)
}

// TODO - strip user if guest
async fn get_user_from_subject(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(_claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = match subject_id {
        SubjectId::Guest(user_id) | SubjectId::Registered(user_id) => user_id,
        SubjectId::Integration(_) => {
            return Err(ServerError::AccessDenied);
        }
    };

    let Some(user) = db::get_user_by_id(state.get_pool(), &user_id).await? else {
        error!("Unexpected: user id was previously fetched but is now missing.");
        state
            .audit()
            .subject(subject_id)
            .action(Action::Read)
            .ceverity(LogCeverity::Critical)
            .function_name("get_user_from_subject")
            .description("Unexpected: user id was previously fetched but is now missing.")
            .log_async();

        return Err(ServerError::NotFound("User not found".into()));
    };

    Ok((StatusCode::OK, Json(user)))
}

async fn create_guest_user(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ServerError> {
    let guest_id = db::create_guest_user(state.get_pool()).await?;
    Ok((StatusCode::CREATED, Json(guest_id)))
}

async fn patch_user(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
    Json(put_request): Json<PutUserRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(actual_user_id) = subject else {
        return Err(ServerError::AccessDenied);
    };

    if let None = claims.missing_permission([Permission::WriteAdmin]) {
        db::patch_user_by_id(state.get_pool(), &user_id, put_request).await?;
        return Ok(StatusCode::OK);
    }

    if actual_user_id != user_id {
        return Err(ServerError::AccessDenied);
    }

    db::patch_user_by_id(state.get_pool(), &actual_user_id, put_request).await?;

    Ok(StatusCode::OK)
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(actual_user_id) = subject else {
        return Err(ServerError::AccessDenied);
    };

    if let None = claims.missing_permission([Permission::WriteAdmin]) {
        db::delete_user_by_id(state.get_pool(), &user_id).await?;
        return Ok(StatusCode::OK);
    }

    if actual_user_id != user_id {
        return Err(ServerError::AccessDenied);
    }

    db::delete_user_by_id(state.get_pool(), &actual_user_id).await?;

    Ok(StatusCode::OK)
}

async fn patch_user_activity(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(_claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(user_id) = subject_id else {
        return Err(ServerError::AccessDenied);
    };

    db::update_user_activity(state.get_pool(), user_id).await?;
    Ok(StatusCode::OK)
}

// TODO - delete
pub async fn auth0_trigger_endpoint(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Json(auth0_user): Json<Auth0User>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Integration(_) = subject else {
        return Err(ServerError::AccessDenied);
    };

    info!("Auth0 post registration trigger was triggered");
    db::create_registered_user(state.get_pool(), &auth0_user).await?;

    Ok(())
}

pub async fn list_all_users(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(_) = subject else {
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::ReadAdmin]) {
        return Err(ServerError::Permission(missing));
    }

    let users = db::list_all_users(state.get_pool()).await?;
    Ok((StatusCode::OK, Json(users)))
}

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post, put},
};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::{
        db::{self},
        models::{Auth0User, Claims, EnsureGuestQuery, Permission, PutUserRequest, SubjectId},
    },
    common::{app_state::AppState, error::ServerError},
    system_log::models::{Action, LogCeverity, SubjectType},
};

pub fn public_auth_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ensure", post(ensure_guest_user))
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
        .route("/valid-token", get(validate_token))
        .route("/stats", get(get_user_activity_stats))
        .route("/activity/{user_id}", put(patch_user_activity))
        .with_state(state)
}

async fn get_user_from_subject(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(_claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let (user_id, is_guest) = match subject_id {
        SubjectId::Guest(user_id) => (user_id, true),
        SubjectId::Registered(user_id) => (user_id, false),
        SubjectId::Integration(_) => {
            return Err(ServerError::AccessDenied);
        }
    };

    let Some(mut user) = db::get_user_by_id(state.get_pool(), &user_id).await? else {
        error!("Unexpected: user id was previously fetched but is now missing.");
        state
            .syslog()
            .subject(subject_id)
            .action(Action::Read)
            .ceverity(LogCeverity::Critical)
            .function("get_user_from_subject")
            .description("Unexpected: user id was previously fetched but is now missing.")
            .log_async();

        return Err(ServerError::NotFound("User not found".into()));
    };

    if is_guest {
        user = user.strip();
    }

    Ok((StatusCode::OK, Json(user)))
}

async fn validate_token(
    Extension(subject_id): Extension<SubjectId>,
) -> Result<impl IntoResponse, ServerError> {
    let valid_type = match subject_id {
        SubjectId::Guest(_) => SubjectType::GuestUser,
        SubjectId::Registered(_) => SubjectType::RegisteredUser,
        SubjectId::Integration(_) => SubjectType::Integration,
    };

    Ok((StatusCode::OK, Json(valid_type)))
}

async fn ensure_guest_user(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EnsureGuestQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let guest_id = match query.guest_id {
        None => db::create_guest_user(state.get_pool()).await?,
        Some(mut guest_id) => {
            let exists = db::guest_user_exists(state.get_pool(), guest_id).await?;
            if !exists {
                guest_id = db::create_guest_user(state.get_pool()).await?;
            }

            guest_id
        }
    };

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

    if put_request.name.is_none() && put_request.email.is_none() && put_request.birth_date.is_none()
    {
        info!("User tried patching without a payload");
        return Ok(StatusCode::OK);
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
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(_) = subject_id else {
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::ReadAdmin]) {
        return Err(ServerError::Permission(missing));
    }

    let users = db::list_all_users(state.get_pool()).await?;
    Ok((StatusCode::OK, Json(users)))
}

async fn get_user_activity_stats(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(_) = subject_id else {
        error!("Unauthorized guest user or integration tried accessing admin endpoint");
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::ReadAdmin]) {
        error!("Unauthorized user tried accessing admin endpoint");
        return Err(ServerError::Permission(missing));
    }

    let stats = db::get_user_activity_stats(state.get_pool()).await?;

    Ok((StatusCode::OK, Json(stats)))
}

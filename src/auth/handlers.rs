use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post, put},
};
use sqlx::{Pool, Postgres};
use tracing::info;

use crate::{
    auth::models::{Auth0User, PutUserRequest, SubjectId},
    auth::{
        db,
        models::{Permission, PermissionCtx},
    },
    server::{app_state::AppState, server_error::ServerError},
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

pub async fn get_user_from_subject(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(_permissions): Extension<PermissionCtx>,
) -> Result<impl IntoResponse, ServerError> {
    let option = match subject {
        SubjectId::Guest(id) => db::get_user_by_guest_id(state.get_pool(), id).await?,
        SubjectId::Registered(id) => db::get_user_by_auth0_id(state.get_pool(), id).await?,
        SubjectId::Auth0 => {
            return Err(ServerError::AccessDenied);
        }
    };

    let user = option.ok_or(ServerError::NotFound("User".into()))?;
    Ok((StatusCode::OK, Json(user)))
}

pub async fn create_guest_user(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ServerError> {
    let guest_id = db::create_guest_user(state.get_pool()).await?;
    Ok((StatusCode::CREATED, Json(guest_id)))
}

pub async fn patch_user(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(permission_ctx): Extension<PermissionCtx>,
    Path(user_id): Path<i32>,
    Json(put_request): Json<PutUserRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(auth0_id) = subject else {
        return Err(ServerError::AccessDenied);
    };

    if permission_ctx.has(Permission::WriteAdmin) {
        db::patch_user_by_id(state.get_pool(), user_id, put_request).await?;
        return Ok(StatusCode::NO_CONTENT);
    }

    ensure_user_owns_data(state.get_pool(), user_id, auth0_id).await?;
    db::patch_user_by_id(state.get_pool(), user_id, put_request).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(permission_ctx): Extension<PermissionCtx>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(auth0_id) = subject else {
        return Err(ServerError::AccessDenied);
    };

    if permission_ctx.has(Permission::WriteAdmin) {
        db::delete_user_by_id(state.get_pool(), user_id).await?;
    }

    ensure_user_owns_data(state.get_pool(), user_id, auth0_id).await?;
    db::delete_user_by_id(state.get_pool(), user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn patch_user_activity(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(_permission_ctx): Extension<PermissionCtx>,
    Path(user_id): Path<i32>,
) -> Result<(), ServerError> {
    if let SubjectId::Auth0 = subject {
        return Err(ServerError::AccessDenied);
    };

    db::update_user_activity(state.get_pool(), user_id).await?;
    Ok(())
}

pub async fn auth0_trigger_endpoint(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Json(auth0_user): Json<Auth0User>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Auth0 = subject else {
        return Err(ServerError::AccessDenied);
    };

    info!("Auth0 post registration trigger was triggered");
    db::create_registered_user(state.get_pool(), &auth0_user).await?;

    Ok(())
}

pub async fn list_all_users(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(permission_ctx): Extension<PermissionCtx>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(_) = subject else {
        return Err(ServerError::Api(
            StatusCode::FORBIDDEN,
            "Not allowed".into(),
        ));
    };

    if permission_ctx.has(Permission::ReadAdmin) {
        return Err(ServerError::Permission(Permission::ReadAdmin));
    }

    let users = db::list_all_users(state.get_pool()).await?;
    Ok((StatusCode::OK, Json(users)))
}

// Helper function
async fn ensure_user_owns_data(
    pool: &Pool<Postgres>,
    user_id: i32,
    auth0_id: String,
) -> Result<(), ServerError> {
    let target_user = db::get_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| ServerError::AccessDenied)?;

    if target_user.auth0_id != Some(auth0_id) {
        return Err(ServerError::AccessDenied);
    }

    Ok(())
}

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::{
        db::{self},
        models::{
            Auth0User, Claims, EnsureUserQuery, ListUsersQuery, PatchUserRequest, Permission,
            RestrictedConfig, SubjectId, UserRole,
        },
    },
    common::{app_state::AppState, error::ServerError, models::ClientPopup},
    config::config::CONFIG,
    system_log::models::{Action, LogCeverity, SubjectType},
};

pub fn public_auth_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ensure", post(ensure_pseudo_user))
        .route("/popup", get(get_client_popup))
        .with_state(state)
}

pub fn protected_auth_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_base_user_from_subject))
        .route(
            "/{user_id}",
            delete(delete_user)
                .patch(patch_user)
                .post(cleanup_subject_pseudo_id),
        )
        .route("/list", get(list_all_users))
        .route("/valid-token", get(validate_token))
        .route("/stats", get(get_user_activity_stats))
        .route("/config", get(get_config))
        .route("/popup", put(update_client_popup))
        .with_state(state)
}

async fn cleanup_subject_pseudo_id(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(_claims): Extension<Claims>,
    Path(pseudo_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::BaseUser(_) = subject_id else {
        return Err(ServerError::AccessDenied);
    };

    tokio::spawn(async move {
        //  TODO - If base with pseudo id x exists skip, if not delete it
        if let Ok(None) = db::get_base_user_by_id(state.get_pool(), &pseudo_id).await {
            // User doesn't exist, can optionally clean up pseudo user
            // Currently no delete_pseudo_user function exists, so we skip this
        }
    });

    Ok(StatusCode::OK)
}

async fn get_base_user_from_subject(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = match subject_id {
        SubjectId::BaseUser(user_id) => user_id,
        SubjectId::Integration(_) | SubjectId::PseudoUser(_) => {
            return Err(ServerError::AccessDenied);
        }
    };

    let Some(user) = db::get_base_user_by_id(state.get_pool(), &user_id).await? else {
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

    let wrapped = match claims.missing_permission([Permission::ReadAdmin, Permission::WriteAdmin]) {
        Some(_missing) => UserRole::BaseUser(user),
        None => UserRole::Admin(user),
    };

    Ok((StatusCode::OK, Json(wrapped)))
}

// TODO - delete ??
async fn validate_token(
    Extension(subject_id): Extension<SubjectId>,
) -> Result<impl IntoResponse, ServerError> {
    let valid_type = match subject_id {
        SubjectId::BaseUser(_) => SubjectType::RegisteredUser,
        SubjectId::Integration(_) => SubjectType::Integration,
        _ => return Err(ServerError::AccessDenied),
    };

    Ok((StatusCode::OK, Json(valid_type)))
}

async fn ensure_pseudo_user(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EnsureUserQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let pseudo_id = match query.pseudo_id {
        None => db::create_pseudo_user(state.get_pool()).await?,
        Some(mut pseudo_id) => {
            let exists = db::pseudo_user_exists(state.get_pool(), pseudo_id).await?;
            if exists {
                return Ok((StatusCode::OK, Json(pseudo_id)));
            }

            pseudo_id = db::create_pseudo_user(state.get_pool()).await?;
            pseudo_id
        }
    };

    let pool = state.get_pool().clone();
    tokio::spawn(async move {
        if let Err(e) = db::update_pseudo_user_activity(&pool, pseudo_id).await {
            let _ = state
                .syslog()
                .action(Action::Update)
                .ceverity(LogCeverity::Warning)
                .function("ensure_pseudo_user")
                .description("Failed to update pseudo user activity")
                .metadata(json!({"error": e.to_string()}))
                .log();
        };
    });

    Ok((StatusCode::CREATED, Json(pseudo_id)))
}

/*
Update this to have id, wo only return no content if a admin updates another user id than itslef, now a admin cannot update its own values without gvetting blank back
*/
async fn patch_user(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<PatchUserRequest>,
) -> Result<Response, ServerError> {
    let SubjectId::BaseUser(uid) = subject else {
        return Err(ServerError::AccessDenied);
    };

    if claims
        .missing_permission([Permission::WriteAdmin])
        .is_none()
        && user_id != uid
    {
        db::patch_base_user_by_id(state.get_pool(), &user_id, request).await?;
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    if request == PatchUserRequest::default() {
        info!("User tried patching without a payload");
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    let user = db::patch_base_user_by_id(state.get_pool(), &uid, request).await?;
    Ok((StatusCode::OK, Json(user)).into_response())
}

// NOT TESTED
async fn delete_user(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::BaseUser(actual_user_id) = subject else {
        return Err(ServerError::AccessDenied);
    };

    if let None = claims.missing_permission([Permission::WriteAdmin]) {
        db::delete_base_user_by_id(state.get_pool(), &user_id).await?;
        return Ok(StatusCode::OK);
    }

    if actual_user_id != user_id {
        return Err(ServerError::AccessDenied);
    }

    db::delete_base_user_by_id(state.get_pool(), &actual_user_id).await?;
    Ok(StatusCode::OK)
}

// TODO - delete
pub async fn auth0_trigger_endpoint(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Json(auth0_user): Json<Auth0User>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Integration(_intname) = subject else {
        return Err(ServerError::AccessDenied);
    };

    info!(
        "Auth0 post registration trigger was triggered for {}",
        auth0_user.email.clone().unwrap_or("[no email]".to_string())
    );
    let mut tx = state.get_pool().begin().await?;
    let bid = db::create_base_user(&mut tx, &auth0_user).await?;
    let pid = db::tx_create_pseudo_user(&mut tx, bid).await?;

    if bid != pid {
        return Err(ServerError::Internal("Failed to create user pair".into()));
    }

    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(pid)))
}

// NOT TESTED
pub async fn list_all_users(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListUsersQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::BaseUser(_) = subject_id else {
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::ReadAdmin]) {
        return Err(ServerError::Permission(missing));
    }

    let users = db::list_base_users(state.get_pool(), query).await?;
    Ok((StatusCode::OK, Json(users)))
}

// NOT TESTED
async fn get_user_activity_stats(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::BaseUser(_) = subject_id else {
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

// NOT TESTED
async fn get_config(
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::BaseUser(_) = subject_id else {
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::ReadAdmin]) {
        return Err(ServerError::Permission(missing));
    }

    let config = RestrictedConfig {
        auth0_domain: CONFIG.auth0.domain.clone(),
        gs_domain: CONFIG.server.gs_domain.clone(),
    };

    Ok((StatusCode::OK, Json(config)))
}

// NOT TESTED
async fn update_client_popup(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ClientPopup>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::BaseUser(_user_id) = subject_id else {
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::WriteAdmin]) {
        return Err(ServerError::Permission(missing));
    }

    let manager = state.get_popup_manager();
    let popup = manager.update(payload).await;

    Ok((StatusCode::OK, Json(popup)))
}

pub async fn get_client_popup(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ServerError> {
    let popup = state.get_popup_manager().read().await;
    Ok((StatusCode::OK, Json(popup)))
}

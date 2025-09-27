use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::post,
};
use reqwest::StatusCode;
use uuid::Uuid;

use tracing::error;

use crate::{
    auth::{
        db::get_user_id_by_auth0_id,
        models::{Permission, PermissionCtx, SubjectId},
    },
    game::{
        db::{self, increment_times_played},
        models::{CreateGameRequest, CreateSessionRequest, GameType, PagedRequest},
    },
    key_vault::models::{KEY_VAULT, KeyPair},
    quiz::{
        db::{get_quiz_session_by_id, persist_quiz_session},
        models::QuizSession,
    },
    server::{app_state::AppState, error::ServerError},
    spin::{
        db::{get_spin_session_by_game_id, tx_persist_spin_session},
        models::SpinSession,
    },
};

pub fn games_routes(state: Arc<AppState>) -> Router {
    let a = Router::new()
        .route("/{game_type}/page", post(get_game_page))
        .route("/{game_type}/create", post(create_game_session))
        .route("/{game_type}/join/{game_id}", post(join_game_session))
        .route("/{game_type}/free-key", post(free_game_key))
        .route(
            "/{game_type}/initiate/{game_id}",
            post(initiate_game_session),
        )
        .with_state(state.clone());

    let session_routes = Router::new()
        .route("/persist", post(persist_game_session))
        .with_state(state.clone());

    Router::new().nest("/a", a).nest("/session", session_routes)
}

/* Game handlers */

async fn join_game_session(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Path(game_type): Path<GameType>,
    Path(game_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = match subject_id {
        SubjectId::Guest(uid) => uid,
        SubjectId::Registered(aid) => get_user_id_by_auth0_id(state.get_pool(), &aid).await?,
        _ => return Err(ServerError::AccessDenied),
    };

    let gs_client = state.get_session_client();
    let response = gs_client
        .join_game_session(state.get_client(), game_type, user_id, game_id)
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

async fn create_game_session(
    State(state): State<Arc<AppState>>,
    Path(game_type): Path<GameType>,
    Json(request): Json<CreateGameRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let client = state.get_client();
    let gs_client = state.get_session_client();
    let response = gs_client
        .create_game_session(client, game_type, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn initiate_game_session(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<SubjectId>,
    Path((game_type, game_id)): Path<(GameType, Uuid)>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = match subject {
        SubjectId::Guest(id) => id,
        SubjectId::Registered(id) => get_user_id_by_auth0_id(state.get_pool(), &id).await?,
        _ => return Err(ServerError::AccessDenied),
    };

    let client = state.get_client();
    let gs_client = state.get_session_client();
    let key = KEY_VAULT.create_key(state.get_pool()).await?;

    let response = match game_type {
        GameType::Spin => {
            let mut session =
                get_spin_session_by_game_id(state.get_pool(), user_id, &game_id).await?;

            session.set_key(key);
            gs_client
                .initiate_gamesession(game_type, session, client)
                .await?
        }
        GameType::Quiz => {
            let mut session = get_quiz_session_by_id(state.get_pool(), &game_id).await?;

            session.set_key(key);
            gs_client
                .initiate_gamesession(game_type, session, client)
                .await?
        }
    };

    Ok((StatusCode::OK, Json(response)))
}

async fn get_game_page(
    State(state): State<Arc<AppState>>,
    Path(game_type): Path<GameType>,
    Json(request): Json<PagedRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let response = db::get_game_page(state.get_pool(), game_type, request).await?;
    Ok((StatusCode::OK, Json(response)))
}

/* Session routes */

async fn persist_game_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, ServerError> {
    // TODO - add m2m integration check here
    let pool = state.get_pool();

    match request.game_type {
        GameType::Spin => {
            let session: SpinSession = serde_json::from_value(request.payload)?;
            match session.times_played {
                0 => increment_times_played(pool, GameType::Spin, &session.id).await?,
                _ => {
                    let mut tx = pool.begin().await?;
                    tx_persist_spin_session(&mut tx, &session).await?;
                }
            }
        }
        GameType::Quiz => {
            let session: QuizSession = serde_json::from_value(request.payload)?;
            match session.times_played {
                0 => increment_times_played(pool, GameType::Quiz, &session.id).await?,
                _ => persist_quiz_session(pool, &session).await?,
            }
        }
    }

    return Ok(StatusCode::OK);
}

async fn free_game_key(
    State(_state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(permission_ctx): Extension<PermissionCtx>,
    Path(game_type): Path<GameType>,
    Json(key_pair): Json<KeyPair>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Integration(_) = subject_id else {
        error!(
            "User tried accessing integration endpoint: POST /games/{}/free-key",
            game_type.to_string()
        );
        return Err(ServerError::AccessDenied);
    };

    if permission_ctx.has(Permission::WriteGame) {
        return Err(ServerError::Permission(Permission::WriteGame));
    }

    KEY_VAULT.remove_key(&key_pair.id).await;
    Ok(StatusCode::OK)
}

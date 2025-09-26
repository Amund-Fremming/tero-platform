use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::post,
};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    auth::{db::get_user_id_by_auth0_id, models::SubjectId},
    game::{
        db,
        models::{CreateGameRequest, CreateSessionRequest, GameType, PagedRequest},
    },
    key_vault::key_vault::KeyPair,
    quiz::{
        db::{get_quiz_session_by_id, increment_quiz_game, tx_persist_quizsession},
        models::QuizSession,
    },
    server::{app_state::AppState, server_error::ServerError},
    spin::{
        db::{get_spin_session_by_game_id, increment_spin_game, tx_persist_spinsession},
        models::SpinSession,
    },
};

pub fn games_routes(state: Arc<AppState>) -> Router {
    let a = Router::new()
        .route("/page/{game_type}", post(get_game_page))
        .route("/create/{game_type}", post(create_game_session))
        .route("/join/{game_type}/{game_id}", post(join_game_session))
        .route(
            "/initiate/{game_type}/{game_id}",
            post(initiate_game_session),
        )
        .with_state(state.clone());

    let session_routes = Router::new()
        .route("/persist", post(persist_game_session))
        .route("/free-key", post(free_game_key))
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
    let key = state.get_key_vault().create_key(state.get_pool()).await?;

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

    match request.game_type {
        GameType::Spin => {
            let session: SpinSession = serde_json::from_value(request.payload)?;
            match session.times_played {
                0 => increment_spin_game(state.get_pool(), &session.id).await?,
                _ => {
                    let mut tx = state.get_pool().begin().await?;
                    tx_persist_spinsession(&mut tx, &session).await?;
                }
            }
        }
        GameType::Quiz => {
            let session: QuizSession = serde_json::from_value(request.payload)?;
            match session.times_played {
                0 => increment_quiz_game(state.get_pool(), &session.id).await?,
                _ => {
                    let mut tx = state.get_pool().begin().await?;
                    tx_persist_quizsession(&mut tx, &session).await?;
                }
            }
        }
    }

    return Ok(StatusCode::OK);
}

async fn free_game_key(
    State(state): State<Arc<AppState>>,
    Json(key_pair): Json<KeyPair>,
) -> Result<impl IntoResponse, ServerError> {
    // TODO - add m2m integration check here
    state.get_key_vault().remove_key(&key_pair.id).await;
    Ok(StatusCode::OK)
}

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::post,
};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    auth::{db::get_user_id_by_auth0_id, user_models::Subject},
    common::{
        app_state::AppState,
        models::{CreateGameRequest, GameSessionRequest, GameType, PagedRequest, PagedResponse},
        server_error::ServerError,
    },
    quiz::{
        db::{get_quiz_page, get_quiz_session_by_id, tx_persist_quizsession},
        models::QuizSession,
    },
    spin::{
        db::{get_spin_page, get_spin_session_by_id, tx_persist_spinsession},
        models::SpinSession,
    },
};

pub fn common_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/session/persist", post(persist_gamesession))
        .route("/search/{game_type}", post(typed_search))
        .route("/create/{game_type}", post(create_gamesession))
        .route("/initiate/{game_type}", post(initiate_gamesession))
        .with_state(state)
}

async fn create_gamesession(
    State(state): State<Arc<AppState>>,
    Path(game_type): Path<GameType>,
    Json(request): Json<CreateGameRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let client = state.get_client();
    let gs_client = state.get_session_client();
    let response = gs_client
        .create_gamesession(client, game_type, request)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

async fn initiate_gamesession(
    State(state): State<Arc<AppState>>,
    Extension(subject): Extension<Subject>,
    Path(game_type): Path<GameType>,
    Path(game_id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = match subject {
        Subject::Guest(id) => id,
        Subject::Registered(id) => get_user_id_by_auth0_id(state.get_pool(), &id).await?,
        _ => return Err(ServerError::AccessDenied),
    };

    let client = state.get_client();
    let gs_client = state.get_session_client();

    let response = match game_type {
        GameType::Spin => {
            let session = get_spin_session_by_id(state.get_pool(), user_id, &game_id).await?;
            gs_client
                .initiate_gamesession(game_type, session, client)
                .await?
        }
        GameType::Quiz => {
            let session = get_quiz_session_by_id(state.get_pool(), &game_id).await?;
            gs_client
                .initiate_gamesession(game_type, session, client)
                .await?
        }
    };

    Ok((StatusCode::OK, Json(response)))
}

async fn persist_gamesession(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GameSessionRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let mut tx = state.get_pool().begin().await?;

    match request.game_type {
        GameType::Spin => {
            let gamesession: SpinSession = serde_json::from_value(request.payload)?;
            tx_persist_spinsession(&mut tx, &gamesession).await?;
        }
        GameType::Quiz => {
            let gamesession: QuizSession = serde_json::from_value(request.payload)?;
            tx_persist_quizsession(&mut tx, &gamesession).await?;
        }
    }

    Ok(())
}

async fn typed_search(
    State(state): State<Arc<AppState>>,
    Path(game_type): Path<GameType>,
    Json(request): Json<PagedRequest>,
) -> Result<Response, ServerError> {
    let response = match game_type {
        GameType::Quiz => {
            let quizzes = state
                .get_quiz_cache()
                .get(&request, || get_quiz_page(state.get_pool(), &request))
                .await?;

            PagedResponse::from_quizzes(quizzes)
        }
        GameType::Spin => {
            let spinners = state
                .get_spin_cache()
                .get(&request, || get_spin_page(state.get_pool(), &request))
                .await?;

            PagedResponse::from_spinners(spinners)
        }
    };

    Ok((StatusCode::OK, Json(response)).into_response())
}

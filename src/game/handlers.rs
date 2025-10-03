use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;
use uuid::Uuid;

use tracing::error;

use crate::{
    auth::models::{Claims, Permission, SubjectId},
    client::gs_client::InteractiveGameResponse,
    common::{app_state::AppState, error::ServerError},
    config::config::CONFIG,
    game::{
        db::{self, increment_times_played},
        models::{
            CreateGameRequest, GameConverter, GameEnvelope, GamePageQuery, GameType,
            SavedGamePageQuery,
        },
    },
    key_vault::models::{JoinKeySet, KEY_VAULT},
    quiz::{
        db::{get_quiz_session_by_id, persist_quiz_session},
        models::QuizSession,
    },
    spin::{
        db::{get_spin_session_by_game_id, tx_persist_spin_session},
        models::SpinSession,
    },
};

pub fn game_routes(state: Arc<AppState>) -> Router {
    let generic_routes = Router::new()
        .route("/{game_type}/page", post(get_game_page))
        .route("/{game_type}/create", post(create_interactive_game))
        .route("/{game_type}/{game_id}", post(delete_game))
        .route("/{game_type}/free-key", post(free_game_key))
        .route("/{game_type}/save/{game_id}", post(save_game))
        .route("/saved", get(get_saved_games_page))
        .with_state(state.clone());

    let standalone_routes = Router::new()
        .route(
            "/{game_type}/initiate/{game_id}",
            get(initiate_standalone_game),
        )
        .route("/persist", post(persist_standalone_game))
        .with_state(state.clone());

    let interactive_routes = Router::new()
        .route("/persist", post(persist_interactive_game))
        .route(
            "/{game_type}/initiate/{game_id}",
            post(initiate_interactive_game),
        )
        .route("/{game_type}/join/{game_id}", post(join_interactive_game))
        .with_state(state.clone());

    Router::new()
        .nest("/", generic_routes)
        .nest("/static", standalone_routes)
        .nest("/session", interactive_routes)
}

async fn delete_game(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Path((game_type, game_id)): Path<(GameType, Uuid)>,
) -> Result<impl IntoResponse, ServerError> {
    if let SubjectId::Integration(_) | SubjectId::Guest(_) = subject_id {
        return Err(ServerError::AccessDenied);
    }

    if let Some(missing) = claims.missing_permission([Permission::WriteAdmin]) {
        return Err(ServerError::Permission(missing));
    }

    db::delete_game(state.get_pool(), &game_type, game_id).await?;
    Ok(StatusCode::OK)
}

async fn join_interactive_game(
    State(_state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Path((game_type, join_word)): Path<(GameType, String)>,
) -> Result<impl IntoResponse, ServerError> {
    if let SubjectId::Integration(id) = subject_id {
        error!("Integration {} tried accessing user endpoint", id);
        return Err(ServerError::AccessDenied);
    }

    let hub_address = format!(
        "{}hubs/{}",
        CONFIG.server.session_domain,
        game_type.to_string()
    );

    let response = InteractiveGameResponse {
        join_word,
        hub_address,
    };

    Ok((StatusCode::OK, Json(response)))
}

async fn create_interactive_game(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Path(game_type): Path<GameType>,
    Json(request): Json<CreateGameRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = match subject_id {
        SubjectId::Guest(id) | SubjectId::Registered(id) => id,
        _ => return Err(ServerError::AccessDenied),
    };

    let client = state.get_client();
    let gs_client = state.get_session_client();
    let join_key = KEY_VAULT.create_key(state.get_pool()).await?;

    let payload = match game_type {
        GameType::Spin => {
            let session = SpinSession::from_create_request(request, user_id);
            session.to_json_value()?
        }
        GameType::Quiz => {
            let session = QuizSession::from_create_request(request);
            session.to_json_value()?
        }
    };

    let join_word = join_key.join_word.clone();
    let envelope = GameEnvelope {
        game_type: game_type.clone(),
        host_id: user_id,
        join_key,
        payload,
    };

    gs_client.create_interactive_game(client, &envelope).await?;

    let hub_address = format!(
        "{}/hubs/{}",
        CONFIG.server.session_domain,
        game_type.to_string()
    );

    let response = InteractiveGameResponse {
        join_word,
        hub_address,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn initiate_standalone_game(
    State(state): State<Arc<AppState>>,
    Extension(_subject_id): Extension<SubjectId>,
    Path((game_type, game_id)): Path<(GameType, Uuid)>,
) -> Result<impl IntoResponse, ServerError> {
    let response = match game_type {
        GameType::Quiz => get_quiz_session_by_id(state.get_pool(), &game_id).await?,
        _ => {
            return Err(ServerError::Api(
                StatusCode::BAD_REQUEST,
                "This game does not have static support".into(),
            ));
        }
    };

    return Ok((StatusCode::OK, Json(response)));
}

async fn initiate_interactive_game(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Path((game_type, game_id)): Path<(GameType, Uuid)>,
) -> Result<impl IntoResponse, ServerError> {
    let user_id = match subject_id {
        SubjectId::Guest(id) | SubjectId::Registered(id) => id,
        _ => return Err(ServerError::AccessDenied),
    };

    let client = state.get_client();
    let gs_client = state.get_session_client();
    let join_key = KEY_VAULT.create_key(state.get_pool()).await?;

    let payload = match game_type {
        GameType::Spin => {
            let session = get_spin_session_by_game_id(state.get_pool(), user_id, game_id).await?;
            session.to_json_value()?
        }
        _ => {
            return Err(ServerError::Api(
                StatusCode::BAD_REQUEST,
                "This game does not have session support".into(),
            ));
        }
    };

    let join_word = join_key.join_word.clone();
    let envelope = GameEnvelope {
        game_type: game_type.clone(),
        host_id: user_id,
        join_key,
        payload,
    };

    gs_client.initiate_gamesession(client, &envelope).await?;

    let hub_address = format!(
        "{}/hubs/{}",
        CONFIG.server.session_domain,
        game_type.to_string()
    );

    let response = InteractiveGameResponse {
        join_word,
        hub_address,
    };

    Ok((StatusCode::OK, Json(response)))
}

async fn get_game_page(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Path(game_type): Path<GameType>,
    Json(request): Json<GamePageQuery>,
) -> Result<impl IntoResponse, ServerError> {
    if let SubjectId::Integration(_) = subject_id {
        return Err(ServerError::AccessDenied);
    }

    let response = db::get_game_page(state.get_pool(), game_type, request).await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn persist_standalone_game(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Json(request): Json<GameEnvelope>,
) -> Result<impl IntoResponse, ServerError> {
    if let SubjectId::Integration(id) = subject_id {
        error!("Integration {} tried to store a static game", id);
        return Err(ServerError::AccessDenied);
    }

    match request.game_type {
        GameType::Quiz => {
            let session: QuizSession = serde_json::from_value(request.payload)?;
            persist_quiz_session(state.get_pool(), &session).await?;
        }
        _ => {
            return Err(ServerError::Api(
                StatusCode::BAD_REQUEST,
                "This game does not have static persist support".into(),
            ));
        }
    }

    Ok(StatusCode::CREATED)
}

// Only called from tero-session
async fn persist_interactive_game(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<GameEnvelope>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Integration(_) = subject_id else {
        error!("User tried to persist game session");
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::WriteGame]) {
        return Err(ServerError::Permission(missing));
    }

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
            increment_times_played(pool, GameType::Quiz, &session.id).await?;
        }
    }

    return Ok(StatusCode::CREATED);
}

async fn free_game_key(
    Extension(subject_id): Extension<SubjectId>,
    Extension(claims): Extension<Claims>,
    Json(key_pair): Json<JoinKeySet>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Integration(_) = subject_id else {
        error!("User tried to free game keys/word");
        return Err(ServerError::AccessDenied);
    };

    if let Some(missing) = claims.missing_permission([Permission::WriteGame]) {
        return Err(ServerError::Permission(missing));
    }

    KEY_VAULT.remove_key(&key_pair.combined_id).await;
    Ok(StatusCode::OK)
}

async fn save_game(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Path((game_type, game_id)): Path<(GameType, Uuid)>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(user_id) = subject_id else {
        error!("Unregistered user or integration tried saving a game");
        return Err(ServerError::AccessDenied);
    };

    db::save_game(state.get_pool(), &game_type, user_id, game_id).await?;
    Ok(StatusCode::CREATED)
}

async fn get_saved_games_page(
    State(state): State<Arc<AppState>>,
    Extension(subject_id): Extension<SubjectId>,
    Query(query): Query<SavedGamePageQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let SubjectId::Registered(user_id) = subject_id else {
        error!("Unregistered user or integration tried saving a game");
        return Err(ServerError::AccessDenied);
    };

    let page = db::get_saved_games_page(state.get_pool(), user_id, query).await?;
    Ok((StatusCode::OK, Json(page)))
}

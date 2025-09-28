use std::collections::HashMap;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
};
use dotenv::dotenv;
use sqlx::{Pool, Postgres, pool};
use tokio::sync::Mutex;
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use crate::{
    auth::handlers::{protected_auth_routes, public_auth_routes},
    config::config::CONFIG,
    game::handlers::game_routes,
    health::handlers::health_routes,
    integration::{
        db,
        models::{INTEGRATION_IDS, INTEGRATION_NAMES, IntegrationName},
    },
    mw::{auth_mw::auth_mw, request_mw::request_mw},
    server::{app_state::AppState, error::ServerError},
    system_log::handlers::log_routes,
};

mod auth;
mod client;
mod config;
mod game;
mod health;
mod integration;
mod key_vault;
mod mw;
mod quiz;
mod server;
mod spin;
mod system_log;

#[tokio::main]
async fn main() {
    // Initialize .env
    dotenv().ok();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global tracing");

    // Initialize state
    let state = AppState::from_connection_string(&CONFIG.database_url)
        .await
        .unwrap_or_else(|e| panic!("{}", e));

    // Initiate integrations
    load_integrations(state.get_pool())
        .await
        .expect("Failed to load integrations");

    // Run migrations
    if let Err(e) = sqlx::migrate!().run(state.get_pool()).await {
        error!("Failed to run migrations: {}", e);
        return;
    }

    // Initialize routes
    let public_routes = Router::new()
        .nest("/health", health_routes(state.clone()))
        .nest("/guest-user", public_auth_routes(state.clone()))
        .nest("/game", game_routes(state.clone()))
        .nest("/log", log_routes(state.clone()));

    let protected_routes = Router::new()
        .nest("/user", protected_auth_routes(state.clone()))
        .layer(from_fn_with_state(state.clone(), auth_mw));

    let app = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(from_fn(request_mw));

    // Initialize webserver
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", CONFIG.server.address, CONFIG.server.port))
            .await
            .unwrap();

    info!(
        "Server listening on address: {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn load_integrations(pool: &Pool<Postgres>) -> Result<(), ServerError> {
    let integrations = db::list_integrations(pool).await?;

    let integration_names: HashMap<String, IntegrationName> = integrations
        .iter()
        .map(|i| (i.subject.clone(), i.name.clone()))
        .collect();
    let integration_ids: HashMap<IntegrationName, Uuid> = integrations
        .iter()
        .map(|i| (i.name.clone(), i.id))
        .collect();

    {
        let mut lock = INTEGRATION_IDS.lock().await;
        *lock = integration_ids;
    }

    {
        let mut lock = INTEGRATION_NAMES.lock().await;
        *lock = integration_names;
    }

    Ok(())
}

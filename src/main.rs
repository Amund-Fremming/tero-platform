use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
};
use dotenv::dotenv;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::FmtSubscriber;

use crate::{
    auth::handlers::{protected_auth_routes, public_auth_routes},
    config::config::CONFIG,
    game::handlers::games_routes,
    health::handlers::health_routes,
    mw::{auth_mw::auth_mw, request_mw::request_mw},
    server::app_state::AppState,
};

mod auth;
mod client;
mod config;
mod game;
mod health;
mod key_vault;
mod mw;
mod quiz;
mod server;
mod spin;

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

    // Initialize routes
    let public_routes = Router::new()
        .nest("/health", health_routes(state.clone()))
        .nest("/guest-user", public_auth_routes(state.clone()))
        .nest("/games", games_routes(state.clone()));

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

use std::sync::Arc;

use tracing::{error, info};

use gustcache::GustCache;
use reqwest::Client;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    auth::db,
    client::gs_client::GSClient,
    common::{error::ServerError, models::PagedResponse},
    config::config::CONFIG,
    game::models::GameBase,
    system_log::{
        builder::SystemLogBuilder,
        models::{Action, LogCeverity},
    },
};

#[derive(Clone)]
pub struct AppState {
    pool: Pool<Postgres>,
    jwks: Jwks,
    client: Client,
    gs_client: GSClient,
    page_cache: Arc<GustCache<Vec<PagedResponse<GameBase>>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwks {
    pub keys: [Jwk; 2],
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    pub kid: String,
    pub n: String,
    pub e: String,
    pub kty: String,
    pub alg: String,
    #[serde(rename(deserialize = "use"))]
    pub use_: String,
}

impl AppState {
    pub async fn from_connection_string(connection_string: &str) -> Result<Arc<Self>, ServerError> {
        let pool = Pool::<Postgres>::connect(&connection_string).await?;
        let client = Client::new();
        let gs_client = GSClient::new(&CONFIG.server.gs_domain);

        let jwks_url = format!("{}.well-known/jwks.json", CONFIG.auth0.domain);
        let response = client.get(jwks_url).send().await?;
        let jwks = response.json::<Jwks>().await?;
        let page_cache = Arc::new(GustCache::from_ttl(chrono::Duration::minutes(2)));

        let state = Arc::new(Self {
            pool,
            jwks,
            client,
            gs_client,
            page_cache,
        });

        Ok(state)
    }

    pub fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    pub fn get_jwks(&self) -> &Jwks {
        &self.jwks
    }

    pub fn get_cache(&self) -> &Arc<GustCache<Vec<PagedResponse<GameBase>>>> {
        &self.page_cache
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub fn get_gs_client(&self) -> &GSClient {
        &self.gs_client
    }

    pub fn audit(&self) -> SystemLogBuilder {
        SystemLogBuilder::new(self.get_pool())
    }

    pub fn sync_user(&self, user_id: Uuid, guest_id: Uuid) {
        let state = self.clone();

        tokio::spawn(async move {
            let Ok(mut tx) = state.get_pool().begin().await else {
                error!("Failed to start database transaction");
                state
                    .audit()
                    .action(Action::Other)
                    .ceverity(LogCeverity::Critical)
                    .description("Failed to start database transaction")
                    .log_async();

                return;
            };

            if let Err(e) = db::tx_sync_user(&mut tx, user_id, guest_id).await {
                error!("Sync failed: {}", e);
                let msg = format!(
                    "Failed to sync user with user_id: {}, and guest_id: {}",
                    user_id, guest_id
                );
                state
                    .audit()
                    .action(Action::Sync)
                    .ceverity(LogCeverity::Critical)
                    .description(&msg)
                    .log_async();

                return;
            }

            info!("User was synced successfully");
        });
    }
}

use std::{sync::Arc, time::Duration};

use serde_json::json;

use reqwest::Client;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    auth::{db, models::Jwks},
    client::gs_client::GSClient,
    common::{
        cache::GustCache,
        error::ServerError,
        key_vault::KeyVault,
        models::{PagedResponse, PopupManager},
    },
    config::config::CONFIG,
    game::{db::delete_non_active_games, models::GameBase},
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
    page_cache: Arc<GustCache<PagedResponse<GameBase>>>,
    key_vault: Arc<KeyVault>,
    popup_manager: PopupManager,
}

impl AppState {
    pub async fn from_connection_string(connection_string: &str) -> Result<Arc<Self>, ServerError> {
        let pool = Pool::<Postgres>::connect(&connection_string).await?;
        let client = Client::new();
        let gs_client = GSClient::new(&CONFIG.server.gs_domain);

        let jwks_url = format!("{}.well-known/jwks.json", CONFIG.auth0.domain);
        let response = client.get(jwks_url).send().await?;
        let jwks = response.json::<Jwks>().await?;
        let page_cache = Arc::new(GustCache::from_ttl(120));
        let key_vault = Arc::new(KeyVault::load_words(&pool).await?);
        let popup_manager = PopupManager::new();

        let state = Arc::new(Self {
            pool,
            jwks,
            client,
            gs_client,
            page_cache,
            key_vault,
            popup_manager,
        });

        Ok(state)
    }

    pub fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    pub fn get_jwks(&self) -> &Jwks {
        &self.jwks
    }

    pub fn get_cache(&self) -> &Arc<GustCache<PagedResponse<GameBase>>> {
        &self.page_cache
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub fn get_gs_client(&self) -> &GSClient {
        &self.gs_client
    }

    pub fn syslog(&self) -> SystemLogBuilder {
        SystemLogBuilder::new(self.get_pool())
    }

    pub fn get_vault(&self) -> &KeyVault {
        &self.key_vault
    }

    pub fn get_popup_manager(&self) -> &PopupManager {
        &self.popup_manager
    }

    pub fn spawn_game_cleanup(&self) {
        let pool = self.get_pool().clone();
        let mut interval = tokio::time::interval(Duration::from_secs(86_400));

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                if let Err(e) = delete_non_active_games(&pool).await {
                    let _ = SystemLogBuilder::new(&pool)
                        .action(Action::Delete)
                        .ceverity(LogCeverity::Info)
                        .description("Failed to purge inactive games")
                        .metadata(json!({"error": e.to_string()}))
                        .log()
                        .await;
                }
            }
        });
    }

    pub fn sync_user(&self, base_id: Uuid, pseudo_id: Uuid) {
        let pool = self.get_pool().clone();

        tokio::spawn(async move {
            if let Err(e) = db::set_pseudo_user_id(&pool, base_id, pseudo_id).await {
                let _ = SystemLogBuilder::new(&pool)
                    .action(Action::Sync)
                    .ceverity(LogCeverity::Warning)
                    .description("Failed to update pseudo user id to sync user")
                    .metadata(json!({"error": e.to_string()}))
                    .function("sync_user")
                    .log();
            }
        });
    }
}

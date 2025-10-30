use std::{sync::Arc, time::Duration};

use serde_json::json;

use reqwest::Client;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    auth::{
        db::{create_pseudo_user, pseudo_user_exists, set_base_user_id, try_delete_pseudo_user},
        models::Jwks,
    },
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

    pub fn spawn_sync_user(&self, base_id: Uuid, pseudo_id: Uuid) {
        let pool = self.get_pool().clone();

        tokio::spawn(async move {
            // Delete old synced pseudo_user if exists
            if let Err(e) = try_delete_pseudo_user(&pool, &base_id).await {
                let _ = SystemLogBuilder::new(&pool)
                    .action(Action::Delete)
                    .ceverity(LogCeverity::Warning)
                    .description("Failed to delete zombie pseudo user, this needs manual deletion")
                    .metadata(json!({"pseudo_user_id": base_id, "error": e.to_string()}))
                    .function("try_delete_pseudo_user")
                    .log();
            }

            let pseudo_exists = match pseudo_user_exists(&pool, pseudo_id).await {
                Ok(exists) => exists,
                Err(e) => {
                    let _ = SystemLogBuilder::new(&pool)
                        .action(Action::Read)
                        .ceverity(LogCeverity::Warning)
                        .description("Failed to read if pseudo user exists")
                        .function("pseudo_user_exists")
                        .metadata(json!({"base_user_id": base_id, "error": e.to_string()}))
                        .log();

                    false
                }
            };

            if pseudo_exists {
                if let Err(e) = set_base_user_id(&pool, pseudo_id, base_id).await {
                    let _ = SystemLogBuilder::new(&pool)
                        .action(Action::Update)
                        .ceverity(LogCeverity::Critical)
                        .description("Failed to sync pseudo user with base user by patching base user id")
                        .function("set_base_user_id")
                        .metadata(json!({"pseudo_user_id": pseudo_id, "base_user_id": base_id, "error": e.to_string()}))
                        .log();
                };
                return;
            };

            if let Err(e) = create_pseudo_user(&pool, Some(pseudo_id)).await {
                let _ = SystemLogBuilder::new(&pool)
                    .action(Action::Create)
                    .ceverity(LogCeverity::Critical)
                    .description("Failed to pseudo user when syncing")
                    .function("set_base_user_id")
                    .metadata(json!({"pseudo_user_id": pseudo_id, "error": e.to_string()}))
                    .log();
            };
        });
    }
}

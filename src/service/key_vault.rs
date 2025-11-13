use std::{
    sync::Arc,
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
};

use dashmap::DashMap;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use sqlx::{Pool, Postgres};
use tracing::{debug, error};

use crate::{
    db::key_vault::get_word_sets,
    models::system_log::{LogAction, LogCeverity},
    service::system_log_builder::SystemLogBuilder,
};

#[derive(Debug, thiserror::Error)]
pub enum KeyVaultError {
    #[error("No more available words")]
    FullCapasity,

    #[error("Failed to load words: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Word sets differ in length")]
    IncompatibleLength,

    #[error("Failed to get created at time: {0}")]
    TimeError(#[from] SystemTimeError),
}

pub struct KeyVault {
    word_count: u8,
    active_keys: Arc<DashMap<(String, String), u64>>,
    prefix_words: Arc<Vec<String>>,
    suffix_words: Arc<Vec<String>>,
}

impl KeyVault {
    pub async fn load_words(pool: &Pool<Postgres>) -> Result<Self, KeyVaultError> {
        let (db_prefix, db_suffix) = get_word_sets(pool).await?;

        if db_prefix.len() != db_suffix.len() {
            return Err(KeyVaultError::IncompatibleLength);
        }

        let vault = Self {
            word_count: db_prefix.len() as u8,
            active_keys: Arc::new(DashMap::new()),
            prefix_words: Arc::new(Vec::from(db_prefix)),
            suffix_words: Arc::new(Vec::from(db_suffix)),
        };

        vault.spawn_vault_cleanup(pool);
        Ok(vault)
    }

    pub fn key_active(&self, key: &(String, String)) -> bool {
        self.active_keys.contains_key(&key)
    }

    pub fn remove_key(&self, key: (String, String)) {
        self.active_keys.remove(&key);
    }

    fn random_idx(&self) -> Result<(usize, usize), KeyVaultError> {
        let mut rng = ChaCha8Rng::from_os_rng();
        let prefix_idx = rng.random_range(0..self.word_count as usize);
        let suffix_idx = rng.random_range(0..self.word_count as usize);

        Ok((prefix_idx, suffix_idx))
    }

    pub fn create_key(&self, pool: &Pool<Postgres>) -> Result<String, KeyVaultError> {
        for _ in 0..100 {
            let Ok((idx1, idx2)) = self.random_idx() else {
                break; // Log outside loop
            };

            let key = (
                self.prefix_words[idx1].clone(),
                self.suffix_words[idx2].clone(),
            );

            if self.key_active(&key) {
                continue;
            }

            let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            self.active_keys.insert(key.clone(), created_at);
            return Ok(format!("{} {}", key.0, key.1));
        }

        for i in 0..self.prefix_words.len() {
            for j in 0..self.suffix_words.len() {
                let key = (self.prefix_words[i].clone(), self.suffix_words[j].clone());

                if self.key_active(&key) {
                    continue;
                }

                let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                self.active_keys.insert(key.clone(), created_at);
                return Ok(format!("{} {}", key.0, key.1));
            }
        }

        SystemLogBuilder::new(pool)
            .action(LogAction::Create)
            .ceverity(LogCeverity::Critical)
            .function("create_key")
            .description("Library failed to create random id.")
            .log_async();

        Err(KeyVaultError::FullCapasity)
    }

    fn spawn_vault_cleanup(&self, pool: &Pool<Postgres>) {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        let active_keys = self.active_keys.clone();
        let pool = pool.clone();

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                debug!("KeyVault is cleaning up its keys");

                let Ok(time) = SystemTime::now().duration_since(UNIX_EPOCH) else {
                    error!("Failed to obtain system time when cleaning up the vault");
                    SystemLogBuilder::new(&pool)
                        .action(LogAction::Other)
                        .ceverity(LogCeverity::Critical)
                        .function("spawn_vault_cleanup")
                        .description("Failed to obtain system time")
                        .log_async();

                    continue;
                };

                let keys_before = active_keys.len();
                let timeout_threshold = time.as_secs() - 3600;
                active_keys.retain(|_, &mut created_at| created_at > timeout_threshold);
                let keys_after = active_keys.len();
                let removed_keys = keys_before - keys_after;

                if removed_keys > 0 {
                    SystemLogBuilder::new(&pool)
                        .action(LogAction::Delete)
                        .ceverity(LogCeverity::Warning)
                        .function("spawn_vault_cleanup")
                        .description(&format!("Cleaned up {} expired keys", removed_keys))
                        .metadata(json!({
                            "warning": "Indicates game crash or unexpected exit - keys should be freed on game start.",              
                        }))
                        .log_async();
                }
            }
        });
    }
}

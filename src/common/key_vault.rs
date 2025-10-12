use std::{
    sync::Arc,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH},
};

use dashmap::DashMap;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sqlx::{Pool, Postgres};

use crate::{
    common::db,
    system_log::{
        builder::SystemLogBuilder,
        models::{Action, LogCeverity},
    },
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
        let (db_prefix, db_suffix) = db::get_word_sets(pool).await?;

        if db_prefix.len() != db_suffix.len() {
            return Err(KeyVaultError::IncompatibleLength);
        }

        Ok(Self {
            word_count: db_prefix.len() as u8,
            active_keys: Arc::new(DashMap::new()),
            prefix_words: Arc::new(Vec::from(db_prefix)),
            suffix_words: Arc::new(Vec::from(db_suffix)),
        })
    }

    pub fn key_active(&self, key: &(String, String)) -> bool {
        self.active_keys.contains_key(&key)
    }

    pub fn remove_key(&self, key: (String, String)) {
        self.active_keys.remove(&key);
    }

    async fn random_idx(&self) -> Result<(usize, usize), KeyVaultError> {
        let mut rng = ChaCha8Rng::from_os_rng();
        let prefix_idx = rng.random_range(0..self.word_count as usize);
        let suffix_idx = rng.random_range(0..self.word_count as usize);

        Ok((prefix_idx, suffix_idx))
    }

    pub async fn create_key(&self, syslog: SystemLogBuilder) -> Result<String, KeyVaultError> {
        for _ in 0..100 {
            let Ok((idx1, idx2)) = self.random_idx().await else {
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
                let key = (
                    self.prefix_words[i].to_string(),
                    self.suffix_words[j].to_string(),
                );

                if self.key_active(&key) {
                    continue;
                }

                let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
                self.active_keys.insert(key.clone(), created_at);
                return Ok(format!("{} {}", key.0, key.1));
            }
        }

        syslog
            .action(Action::Create)
            .ceverity(LogCeverity::Critical)
            .function("create_key")
            .description("Library failed to create random id.")
            .log_async();

        Err(KeyVaultError::FullCapasity)
    }

    // TODO
    /*
       Cleanup words that are outdated
       change the create vault to have a ref to pool inside, change from param sysslog, to creating its own if it needs its instead, may be better
    */
    fn spawn_vault_cleanup() {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));

        tokio::spawn(async move {
            loop {
                // TODO
                interval.tick().await;
                debug!("KeyVault is cleaning up its keys");
            }
        });
    }
}

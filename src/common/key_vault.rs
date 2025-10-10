use std::{collections::HashSet, sync::Arc};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sqlx::{Pool, Postgres};
use tokio::sync::RwLock;

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
}

pub struct KeyVault {
    prefix_len: u8,
    suffix_len: u8,
    active_keys: Arc<RwLock<HashSet<(String, String)>>>,
    prefix_words: Arc<RwLock<Vec<String>>>,
    suffix_words: Arc<RwLock<Vec<String>>>,
}

impl KeyVault {
    pub async fn load_words(pool: &Pool<Postgres>) -> Result<Self, KeyVaultError> {
        let mut vault = Self {
            prefix_len: 0,
            suffix_len: 0,
            active_keys: Arc::new(RwLock::new(HashSet::new())),
            prefix_words: Arc::new(RwLock::new(Vec::new())),
            suffix_words: Arc::new(RwLock::new(Vec::new())),
        };

        let (db_prefix, db_suffix) = db::get_word_sets(pool).await?;

        vault.prefix_len = db_prefix.len() as u8;
        {
            let mut lock = vault.prefix_words.write().await;
            *lock = db_prefix;
        }

        vault.suffix_len = db_suffix.len() as u8;
        {
            let mut lock = vault.suffix_words.write().await;
            *lock = db_suffix;
        }

        Ok(vault)
    }

    pub async fn key_active(&self, tuple: (String, String)) -> bool {
        let lock = self.active_keys.read().await;
        lock.get(&tuple).is_some()
    }

    pub async fn remove_key(&self, tuple: (String, String)) {
        {
            let mut lock = self.active_keys.write().await;
            lock.remove(&tuple);
        }
    }

    pub async fn create_key(&self, syslog: SystemLogBuilder) -> Result<String, KeyVaultError> {
        let prefix_lock = self.prefix_words.read().await;
        let suffix_lock = self.suffix_words.read().await;
        let active_lock = self.active_keys.read().await;

        for _ in 0..100 {
            let Ok((idx1, idx2)) = self.random_idx().await else {
                break; // Log outside loop
            };

            let key = (prefix_lock[idx1].to_string(), suffix_lock[idx2].to_string());

            if !active_lock.contains(&key) {
                drop(active_lock);
                let mut active_lock = self.active_keys.write().await;
                active_lock.insert(key.clone());
                return Ok(format!("{} {}", key.0, key.1));
            }
        }

        for i in 0..prefix_lock.len() {
            for j in 0..suffix_lock.len() {
                let key = (prefix_lock[i].to_string(), suffix_lock[j].to_string());

                if !active_lock.contains(&key) {
                    drop(active_lock);
                    let mut active_lock = self.active_keys.write().await;
                    active_lock.insert(key.clone());
                    return Ok(format!("{} {}", key.0, key.1));
                }
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

    async fn random_idx(&self) -> Result<(usize, usize), KeyVaultError> {
        match (self.prefix_len, self.suffix_len) {
            (0, 0) => {
                return Err(KeyVaultError::FullCapasity);
            }
            (1, 1) => return Ok((0, 0)),
            _ => {}
        }

        let mut rng = ChaCha8Rng::from_os_rng();
        let prefix_idx = rng.random_range(0..self.prefix_len as usize);
        let suffix_idx = rng.random_range(0..self.suffix_len as usize);

        Ok((prefix_idx, suffix_idx))
    }
}

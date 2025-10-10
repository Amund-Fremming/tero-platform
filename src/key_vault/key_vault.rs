use std::{collections::HashSet, sync::Arc};

use once_cell::sync::Lazy;
use rand::seq::IndexedRandom;
use sqlx::{Pool, Postgres, pool};
use tokio::sync::{Mutex, RwLock};
use tracing::Value;

use crate::{common::error::ServerError, key_vault::db};

pub static KEY_VAULT: Lazy<KeyVault> = Lazy::new(|| KeyVault::new(0.2, 100));

#[derive(Clone)]
pub struct KeyVault {
    refill_threshold: f32,
    word_count: usize,
    active_keys: Arc<RwLock<HashSet<(String, String)>>>,
    prefix_words: Arc<RwLock<Vec<String>>>,
    suffix_words: Arc<RwLock<Vec<String>>>,
}

impl KeyVault {
    pub fn new(refill_threshold: f32, word_count: usize) -> Self {
        Self {
            refill_threshold,
            word_count,
            active_keys: Arc::new(RwLock::new(HashSet::new())),
            prefix_words: Arc::new(RwLock::new(Vec::new())),
            suffix_words: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn load_words(&self, pool: &Pool<Postgres>) -> Result<(), ServerError> {
        let (db_prefix, db_suffix) = db::get_word_sets(pool).await?;
        {
            let mut lock = self.prefix_words.write().await;
            *lock = db_prefix;
        }

        {
            let mut lock = self.suffix_words.write().await;
            *lock = db_suffix;
        }

        Ok(())
    }

    pub async fn remove_key(&self, prefix: String, suffix: String) {
        {
            let tuple = (prefix.clone(), suffix.clone());
            let mut lock = self.active_keys.write().await;
            lock.remove(&tuple);
        }
        {
            let mut lock = self.prefix_words.write().await;
            lock.push(prefix);
        }
        {
            let mut lock = self.suffix_words.write().await;
            lock.push(suffix);
        }
    }

    pub async fn create_key(&self, pool: &Pool<Postgres>) -> Result<String, ServerError> {
        self.refill(pool).await;

        let idx = self.random_idx()?;
        let prefix = {
            let mut lock = self.prefix_words.write().await;
            lock.remove(idx)
        };

        let idx = self.random_idx()?;
        let suffix = {
            let mut lock = self.suffix_words.write().await;
            lock.remove(idx)
        };

        let key = format!("{prefix} {suffix}");
        {
            let mut lock = self.active_keys.write().await;
            lock.insert((prefix, suffix));
        }

        Ok(key)
    }

    async fn refill(&self, pool: &Pool<Postgres>) -> Result<(), ServerError> {
        let prefix_size = {
            let lock = self.prefix_words.read().await;
            (lock.len() / 100) as f32
        };

        let suffix_size = {
            let lock = self.suffix_words.read().await;
            (lock.len() / 100) as f32
        };

        if prefix_size > self.refill_threshold && suffix_size > self.refill_threshold {
            return Ok(());
        }

        self.spawn_refill_task(pool);
        Ok(())
    }

    fn spawn_refill_task(&self, pool: &Pool<Postgres>) {
        let vault = self.clone();
        let pool = pool.clone();

        tokio::spawn(async move {
            let (used_prefix, used_suffix) = vault.used_words().await;

            match db::get_word_sets(&pool).await {
                Ok((mut db_prefix, mut db_suffix)) => {
                    {
                        db_prefix.retain(|word| !used_prefix.contains(word));
                        let mut lock = vault.prefix_words.write().await;
                        *lock = db_prefix;
                    }

                    {
                        db_suffix.retain(|word| !used_suffix.contains(word));
                        let mut lock = vault.suffix_words.write().await;
                        *lock = db_suffix;
                    }
                }
                Err(_e) => {
                    // LOG
                }
            }
        });
    }

    async fn used_words(&self) -> (HashSet<String>, HashSet<String>) {
        let active_slots = {
            let lock = self.active_keys.read().await;
            lock.clone()
        };

        let mut slot_one_used = HashSet::new();
        let mut slot_two_used = HashSet::new();

        for tuple in active_slots {
            slot_one_used.insert(tuple.0);
            slot_two_used.insert(tuple.1);
        }

        (slot_one_used, slot_two_used)
    }

    fn random_idx(&self) -> Result<usize, ServerError> {
        let mut rng = rand::rng();
        let range: Vec<usize> = (0..self.word_count - 1).collect();
        let Some(n) = range.choose(&mut rng) else {
            // TODO - add syslog
            return Err(ServerError::Internal(
                "Rand failed to get random id foid for key".into(),
            ));
        };

        Ok(*n)
    }
}

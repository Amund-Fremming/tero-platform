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

    #[error("Word sets differ in length")]
    IncompatibleLength,
}

pub struct KeyVault {
    word_count: u8,
    active_keys: Arc<RwLock<HashSet<(String, String)>>>,
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
            active_keys: Arc::new(RwLock::new(HashSet::new())),
            prefix_words: Arc::new(Vec::from(db_prefix)),
            suffix_words: Arc::new(Vec::from(db_suffix)),
        })
    }

    pub async fn key_active(&self, tuple: (String, String)) -> bool {
        let lock = self.active_keys.read().await;
        lock.contains(&tuple)
    }

    pub async fn remove_key(&self, tuple: (String, String)) {
        let mut lock = self.active_keys.write().await;
        lock.remove(&tuple);
    }

    /*
       Performance upgrade
       - If alot of clients try to create a key at the same time they will lock eachother out, causing this to be slow
       - One solution is to clone the working arrays and release locks, then obtaining again when you write
            - the active still neds to not be cloned

        - prefix and suffix words might not need a lock, we can just use a arc on them, they are static when loaded
    */
    pub async fn create_key(&self, syslog: SystemLogBuilder) -> Result<String, KeyVaultError> {
        let active_lock = self.active_keys.read().await;

        for _ in 0..100 {
            let Ok((idx1, idx2)) = self.random_idx().await else {
                break; // Log outside loop
            };
            println!("1");
            let key = (
                self.prefix_words[idx1].clone(),
                self.suffix_words[idx2].clone(),
            );
            {
                if !active_lock.contains(&key) {
                    println!("2");
                    drop(active_lock);
                    let mut active_lock = self.active_keys.write().await;
                    active_lock.insert(key.clone());
                    return Ok(format!("{} {}", key.0, key.1));
                }
            }
        }

        let active_lock = self.active_keys.read().await;
        for i in 0..self.prefix_words.len() {
            for j in 0..self.suffix_words.len() {
                println!("3");
                let key = (
                    self.prefix_words[i].to_string(),
                    self.suffix_words[j].to_string(),
                );

                println!("4");
                if !active_lock.contains(&key) {
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
        match self.word_count {
            0 => return Err(KeyVaultError::FullCapasity),
            1 => return Ok((0, 0)),
            _ => {}
        }

        let mut rng = ChaCha8Rng::from_os_rng();
        let prefix_idx = rng.random_range(0..self.word_count as usize);
        let suffix_idx = rng.random_range(0..self.word_count as usize);

        Ok((prefix_idx, suffix_idx))
    }
}

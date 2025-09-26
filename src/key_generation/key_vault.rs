use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tracing::error;

use crate::{common::server_error::ServerError, key_generation::db};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct JoinKey {
    pub id: String,
    pub word: String,
}

pub struct KeyVault {
    pub in_use: Arc<RwLock<HashSet<String>>>,
}

impl KeyVault {
    pub fn new() -> Self {
        Self {
            in_use: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Format: `S1_0001 S2_0002`
    pub fn remove_key(&self, combined_id: &str) -> Result<(), ServerError> {
        let mut lock = self.in_use.write().map_err(|e| {
            error!("KeyVault write-lock error: {}", e);
            ServerError::PoisonError
        })?;

        lock.remove(combined_id);
        Ok(())
    }

    /// Creates a new unique key
    pub async fn create_key(&self, pool: &Pool<Postgres>) -> Result<String, ServerError> {
        let lock = self.in_use.read().map_err(|e| {
            error!("KeyVault read-lock error: {}", e);
            ServerError::PoisonError
        })?;

        let slot1_id: String;
        let slot2_id: String;
        loop {
            let s1 = Self::get_random_id(1)?;
            let s2 = Self::get_random_id(2)?;
            let combined_id = format!("{s1} {s2}");

            if let Some(_) = lock.get(&combined_id) {
                drop(lock);
                slot1_id = s1;
                slot2_id = s2;
                break;
            };
        }

        let mut lock = self.in_use.write().map_err(|e| {
            error!("KeyVault write-lock error: {}", e);
            ServerError::PoisonError
        })?;

        let combined_id = format!("{slot1_id} {slot2_id}");
        lock.insert(combined_id);
        let join_key = db::get_word_set(pool, &[&slot1_id, &slot2_id]).await?;

        Ok(join_key)
    }

    fn get_random_id(slot: u8) -> Result<String, ServerError> {
        let mut rng = rand::rng();
        let range: Vec<u32> = (0..1000).collect();
        let Some(n) = range.choose(&mut rng) else {
            return Err(ServerError::Internal(
                "Rand failed to get random id foid for key".into(),
            ));
        };

        Ok(format!("S{slot}_{n}"))
    }
}

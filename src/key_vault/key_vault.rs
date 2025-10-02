use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use rand::seq::IndexedRandom;
use sqlx::{Pool, Postgres};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    key_vault::{db, models::KeyPair},
    server::error::ServerError,
};

pub struct KeyVault {
    pub in_use: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl KeyVault {
    pub fn new() -> Self {
        Self {
            in_use: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Format: `S1_0001 S2_0002`
    pub async fn remove_key(&self, combined_id: &str) {
        let mut lock = self.in_use.write().await;
        lock.remove(combined_id);
    }

    /// Creates a new unique key
    pub async fn create_key(
        &self,
        pool: &Pool<Postgres>,
        game_id: Uuid,
    ) -> Result<KeyPair, ServerError> {
        let lock = self.in_use.read().await;

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

        let mut lock = self.in_use.write().await;
        let combined_id = format!("{slot1_id} {slot2_id}");
        lock.insert(combined_id.clone(), game_id);
        let join_key = db::get_word_set(pool, &[&slot1_id, &slot2_id]).await?;

        Ok(KeyPair {
            id: combined_id,
            key: join_key,
        })
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

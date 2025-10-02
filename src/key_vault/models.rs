use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::key_vault::key_vault::KeyVault;

pub static KEY_VAULT: Lazy<KeyVault> = Lazy::new(|| KeyVault::new());

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct JoinKey {
    pub id: String,
    pub word: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub id: String,
    pub key: String,
}

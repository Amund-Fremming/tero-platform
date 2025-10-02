use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::key_vault::key_vault::KeyVault;

pub static KEY_VAULT: Lazy<KeyVault> = Lazy::new(|| KeyVault::new());

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinKeySet {
    pub combined_id: String,
    pub join_word: String,
}

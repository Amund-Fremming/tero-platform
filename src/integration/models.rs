use core::fmt;
use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

static INTEGRATIONS: Lazy<Mutex<HashMap<String, IntegrationName>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Integration {
    pub id: Uuid,
    pub subject: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IntegrationName {
    Auth0,
    Session,
}

impl fmt::Display for IntegrationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrationName::Auth0 => write!(f, "auth0"),
            IntegrationName::Session => write!(f, "session"),
        }
    }
}

impl IntegrationName {
    pub async fn from_subject(
        subject: String,
        integrations: &Mutex<HashMap<String, IntegrationName>>,
    ) -> Option<IntegrationName> {
        let lock = integrations.lock().await;
        lock.get(&subject).cloned()
    }
}

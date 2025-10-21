use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::common::error::ServerError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PagedResponse<T> {
    items: Vec<T>,
    has_next: bool,
}

impl<T> PagedResponse<T> {
    pub fn new(items: Vec<T>, has_next: bool) -> Self {
        Self { items, has_next }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientPopup {
    pub heading: String,
    pub paragraph: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct PopupManager {
    popup: Arc<RwLock<ClientPopup>>,
}

impl PopupManager {
    pub fn new() -> Self {
        Self {
            popup: Arc::new(RwLock::new(ClientPopup {
                heading: "Velkommen".to_string(),
                paragraph: "Takk for at du har lastet ned appen vår!".to_string(),
                active: false,
            })),
        }
    }

    pub async fn update(&self, update: ClientPopup) -> Result<ClientPopup, ServerError> {
        let mut lock = self.popup.write().map_err(|_| {
            ServerError::Internal("Failed to toggle popup message because of lock error".into())
        })?;

        *lock = update.clone();
        Ok(update)
    }
}

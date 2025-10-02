use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::game::models::GameEnvelope;

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveGameResponse {
    pub join_word: String,
    pub hub_address: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GameSessionClientError {
    #[error("Http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Api error: {0} - {1}")]
    ApiError(StatusCode, String),

    #[error("Failed to serialize object: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct GameSessionClient {
    domain: String,
}

impl GameSessionClient {
    pub fn new(domain: impl Into<String>) -> Self {
        let domain = domain.into();

        Self { domain }
    }

    pub async fn health_check(&self, client: &Client) -> Result<(), GameSessionClientError> {
        let response = client.get(format!("{}/health", self.domain)).send().await?;
        if !response.status().is_success() {
            error!("Failed heath check on session microservice");
            return Err(GameSessionClientError::ApiError(
                StatusCode::SERVICE_UNAVAILABLE,
                "Failed to reach session microservice".into(),
            ));
        }
        info!("GameSession microservice is healthy");

        Ok(())
    }

    pub async fn create_interactive_game(
        &self,
        client: &Client,
        envelope: &GameEnvelope,
    ) -> Result<(), GameSessionClientError> {
        let uri = format!("{}session/create", self.domain);
        self.send_json(client, &uri, envelope).await
    }

    pub async fn initiate_gamesession(
        &self,
        client: &Client,
        envelope: &GameEnvelope,
    ) -> Result<(), GameSessionClientError> {
        let uri = "/games/initiate".to_string();
        self.send_json(client, &uri, envelope).await
    }

    async fn send_json<T: Serialize>(
        &self,
        client: &Client,
        uri: &str,
        body: T,
    ) -> Result<(), GameSessionClientError> {
        info!("GameSessionClient sending request to: {}", uri);
        let url = format!("{}/{}", self.domain, uri);
        let response = client
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or("No body".into());
        if !status.is_success() {
            error!("GameSessionClient request failed: {} - {}", status, body);
            return Err(GameSessionClientError::ApiError(status, body));
        }

        Ok(())
    }
}

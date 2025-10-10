use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{client::gs_client_error::GSClientError, game::models::GameEnvelope};

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveGameResponse {
    pub key_word: String,
    pub hub_address: String,
}

#[derive(Debug, Clone)]
pub struct GSClient {
    domain: String,
}

impl GSClient {
    pub fn new(domain: impl Into<String>) -> Self {
        let domain = domain.into();

        Self { domain }
    }

    pub async fn health_check(&self, client: &Client) -> Result<(), GSClientError> {
        let response = client.get(format!("{}health", self.domain)).send().await?;
        if !response.status().is_success() {
            return Err(GSClientError::ApiError(
                StatusCode::SERVICE_UNAVAILABLE,
                "Failed to reach game session microservice".into(),
            ));
        }

        Ok(())
    }

    pub async fn create_interactive_game(
        &self,
        client: &Client,
        envelope: &GameEnvelope,
    ) -> Result<(), GSClientError> {
        let uri = format!("{}session/create", self.domain);
        self.send_json(client, &uri, envelope).await
    }

    pub async fn initiate_game_session(
        &self,
        client: &Client,
        envelope: &GameEnvelope,
    ) -> Result<(), GSClientError> {
        let uri = "/games/initiate".to_string();
        self.send_json(client, &uri, envelope).await
    }

    async fn send_json<T: Serialize>(
        &self,
        client: &Client,
        uri: &str,
        body: T,
    ) -> Result<(), GSClientError> {
        info!("GSClient sending request to: {}", uri);
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
            error!("GSClient request failed: {} - {}", status, body);
            return Err(GSClientError::ApiError(status, body));
        }

        Ok(())
    }
}

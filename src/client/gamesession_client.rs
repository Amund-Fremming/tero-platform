use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    common::models::{CreateGameRequest, GameSessionRequest, GameType, Identify},
    quiz::models::QuizSession,
    spin::models::SpinSession,
};

//#[serde(untagged)]
#[derive(Debug, Serialize, Deserialize)]
pub enum GameApiWrapper {
    Quiz(QuizSession),
    Spin(SpinSession),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateSessionResponse {
    game_id: Uuid,
    hub_address: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GameSessionClientError {
    #[error("Failed to initialize game: {0}")]
    Initialize(String),

    #[error("Failed to initialize game: {0}")]
    Create(String),

    #[error("Http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Api error: {0} - {1}")]
    ApiError(StatusCode, String),

    #[error("Failed to serialize object: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug)]
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

    pub async fn create_gamesession(
        &self,
        client: &Client,
        game_type: GameType,
        request: CreateGameRequest,
    ) -> Result<InitiateSessionResponse, GameSessionClientError> {
        let (game_id, payload) = match game_type {
            GameType::Spin => {
                let session = SpinSession::from_create_request(request);
                let value = serde_json::to_value(&session)?;
                (session.id, value)
            }
            GameType::Quiz => {
                let session = QuizSession::from_create_request(request);
                let value = serde_json::to_value(&session)?;
                (session.id, value)
            }
        };

        let uri = format!("{}/session/create", game_type.to_string());
        let request = GameSessionRequest { game_type, payload };
        self.send_json(client, &uri, request).await?;

        Ok(InitiateSessionResponse {
            game_id,
            hub_address: format!("{}/{}", self.domain, uri),
        })
    }

    pub async fn initiate_gamesession<T>(
        &self,
        game_type: GameType,
        gamesession: T,
        client: &Client,
    ) -> Result<InitiateSessionResponse, GameSessionClientError>
    where
        T: Serialize + Identify,
    {
        let payload = serde_json::to_value(&gamesession)?;
        let uri = format!("{}/session/initiate", game_type.to_string());
        let request = GameSessionRequest { game_type, payload };
        self.send_json(client, &uri, request).await?;

        Ok(InitiateSessionResponse {
            game_id: gamesession.get_id(),
            hub_address: format!("{}/{}", self.domain, uri),
        })
    }

    async fn send_json<T: Serialize>(
        &self,
        client: &Client,
        uri: &str,
        body: T,
    ) -> Result<(), GameSessionClientError> {
        info!("GameSessionClient sending request to: {}", uri);
        let response = client
            .post(format!("{}/{}", self.domain, uri))
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

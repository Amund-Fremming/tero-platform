use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    game::models::{
        CreateGameRequest, CreateSessionRequest, GameType, Identify, JoinSessionRequest,
    },
    quiz::models::QuizSession,
    spin::models::SpinSession,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSessionResponse {
    game_id: Uuid,
    hub_address: String,
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

    pub async fn join_game_session(
        &self,
        client: &Client,
        game_type: GameType,
        user_id: Uuid,
        game_id: Uuid,
    ) -> Result<GameSessionResponse, GameSessionClientError> {
        let uri = "session/join".to_string();
        let hub_name = game_type.to_string();
        let request = JoinSessionRequest {
            user_id,
            game_id,
            game_type,
        };
        self.send_json(client, &uri, &request).await?;

        Ok(GameSessionResponse {
            game_id,
            hub_address: format!("{}hubs/{}", self.domain, hub_name),
        })
    }

    pub async fn create_game_session(
        &self,
        client: &Client,
        game_type: GameType,
        request: CreateGameRequest,
    ) -> Result<GameSessionResponse, GameSessionClientError> {
        let hub_name = game_type.to_string();
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

        let uri = format!("{}session/create", self.domain);
        let request = CreateSessionRequest { game_type, payload };
        self.send_json(client, &uri, request).await?;

        Ok(GameSessionResponse {
            game_id,
            hub_address: format!("{}hubs/{}", self.domain, hub_name),
        })
    }

    pub async fn initiate_gamesession<T>(
        &self,
        game_type: GameType,
        gamesession: T,
        client: &Client,
    ) -> Result<GameSessionResponse, GameSessionClientError>
    where
        T: Serialize + Identify,
    {
        let hub_name = game_type.to_string();
        let payload = serde_json::to_value(&gamesession)?;
        let uri = "session/initiate".to_string();
        let request = CreateSessionRequest { game_type, payload };
        self.send_json(client, &uri, request).await?;

        Ok(GameSessionResponse {
            game_id: gamesession.get_id(),
            hub_address: format!("{}hubs/{}", self.domain, hub_name),
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

use std::sync::Arc;

use gustcache::GustCache;
use reqwest::Client;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{
    client::gamesession_client::GameSessionClient, common::server_error::ServerError,
    config::config::CONFIG, key_generation::key_vault::KeyVault, quiz::models::QuizGame,
    spin::models::SpinGame,
};

pub struct AppState {
    pool: Pool<Postgres>,
    jwks: Jwks,
    client: Client,
    gs_client: GameSessionClient,
    key_vault: KeyVault,
    quiz_cache: GustCache<Vec<QuizGame>>,
    spin_cache: GustCache<Vec<SpinGame>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwks {
    pub keys: [Jwk; 2],
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    pub kid: String,
    pub n: String,
    pub e: String,
    pub kty: String,
    pub alg: String,
    #[serde(rename(deserialize = "use"))]
    pub use_: String,
}

impl AppState {
    pub async fn from_connection_string(connection_string: &str) -> Result<Arc<Self>, ServerError> {
        let pool = Pool::<Postgres>::connect(&connection_string).await?;
        let client = Client::new();
        let gs_client = GameSessionClient::new(&CONFIG.server.session_domain);

        let jwks_url = format!("{}.well-known/jwks.json", CONFIG.auth0.domain);
        let response = client.get(jwks_url).send().await?;
        let jwks = response.json::<Jwks>().await?;

        let key_vault = KeyVault::new();
        let quiz_cache = GustCache::from_ttl(chrono::Duration::minutes(2));
        let spin_cache = GustCache::from_ttl(chrono::Duration::minutes(2));

        let state = Arc::new(Self {
            pool,
            jwks,
            client,
            gs_client,
            key_vault,
            quiz_cache,
            spin_cache,
        });

        Ok(state)
    }

    pub fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    pub fn get_jwks(&self) -> &Jwks {
        &self.jwks
    }

    pub fn get_key_vault(&self) -> &KeyVault {
        &self.key_vault
    }

    pub fn get_quiz_cache(&self) -> &GustCache<Vec<QuizGame>> {
        &self.quiz_cache
    }

    pub fn get_spin_cache(&self) -> &GustCache<Vec<SpinGame>> {
        &self.spin_cache
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    pub fn get_session_client(&self) -> &GameSessionClient {
        &self.gs_client
    }
}

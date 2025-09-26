use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    game::models::{CreateGameRequest, GameBase, GameCategory, Identify},
    key_vault::key_vault::KeyPair,
};

impl Into<GameBase> for SpinGame {
    fn into(self) -> GameBase {
        GameBase {
            id: self.id,
            name: self.name,
            description: self.description,
            category: self.category,
            iterations: self.iterations,
            times_played: self.times_played,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SpinGame {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GameCategory,
    pub iterations: i32,
    pub times_played: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Round {
    id: Uuid,
    spinner_id: i32,
    participants: i32,
    read_before: bool,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpinSession {
    pub id: Uuid,
    pub join_key: Option<KeyPair>,
    pub host_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GameCategory,
    pub iterations: i32,
    pub times_played: i32,

    // metadata
    // players
    pub rounds: Vec<Round>,
}

impl Identify for SpinSession {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl SpinSession {
    pub fn from_create_request(request: CreateGameRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            join_key: None,
            host_id: request.host_id,
            name: request.name,
            description: request.description,
            category: request.category.unwrap_or(GameCategory::Default),
            iterations: 0,
            times_played: 0,
            rounds: vec![],
        }
    }

    pub fn from_game_and_rounds(host_id: Uuid, game: SpinGame, rounds: Vec<Round>) -> Self {
        Self {
            id: game.id,
            join_key: None,
            host_id,
            name: game.name,
            description: game.description,
            category: game.category,
            iterations: game.iterations,
            times_played: game.times_played,
            rounds,
        }
    }

    pub fn to_game_and_rounds(&self) -> (SpinGame, Vec<Round>) {
        let rounds = self.rounds.iter().map(|r| r.clone()).collect();
        let game = SpinGame {
            id: self.id,
            name: self.name.to_string(),
            description: self.description.clone(),
            category: self.category.clone(),
            iterations: self.iterations,
            times_played: self.times_played,
        };

        (game, rounds)
    }

    pub fn set_key(&mut self, pair: KeyPair) {
        self.join_key = Some(pair);
    }
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::models::{CreateGameRequest, GameBase, GameCategory, GameConverter};

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

impl GameConverter for SpinSession {
    fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
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

impl SpinSession {
    pub fn from_create_request(request: CreateGameRequest, host_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            host_id: host_id,
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
}

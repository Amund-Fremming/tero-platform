use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::models::{CreateGameRequest, GameCategory, GameConverter, GameType};

impl GameConverter for QuizSession {
    fn to_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct QuizSession {
    pub base_id: Uuid,
    pub quiz_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub game_type: GameType,
    pub category: GameCategory,
    pub iterations: i32,
    pub current_iteration: i32,
    pub questions: Vec<String>,
    pub times_played: i32,
}

impl QuizSession {
    pub fn from_create_request(request: CreateGameRequest) -> Self {
        Self {
            base_id: Uuid::new_v4(),
            quiz_id: Uuid::new_v4(),
            name: request.name,
            description: request.description,
            game_type: GameType::Quiz,
            category: request.category.unwrap_or(GameCategory::Default),
            iterations: 0,
            current_iteration: 0,
            questions: vec![],
            times_played: 0,
        }
    }
}

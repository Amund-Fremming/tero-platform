use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    game::models::{CreateGameRequest, GameBase, GameCategory, Identify},
    key_vault::models::KeyPair,
};

impl Into<GameBase> for QuizGame {
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
pub struct QuizGame {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GameCategory,
    pub iterations: i32,
    pub times_played: i32,
    pub questions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizSession {
    pub id: Uuid,
    pub join_key: Option<KeyPair>,
    pub name: String,
    pub description: Option<String>,
    pub category: GameCategory,
    pub iterations: u8,
    pub current_iteration: u8,
    pub questions: Vec<String>,
    pub times_played: i32,
}

impl Identify for QuizSession {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl QuizSession {
    pub fn from_create_request(request: CreateGameRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            join_key: None,
            name: request.name,
            description: request.description,
            category: request.category.unwrap_or(GameCategory::Default),
            iterations: 0,
            current_iteration: 0,
            questions: vec![],
            times_played: 0,
        }
    }

    pub fn from_game(quiz: QuizGame) -> Self {
        Self {
            id: quiz.id,
            join_key: None,
            name: quiz.name,
            description: quiz.description,
            category: quiz.category,
            iterations: u8::try_from(quiz.iterations).ok().unwrap(),
            current_iteration: 0,
            questions: quiz.questions,
            times_played: quiz.times_played,
        }
    }

    pub fn set_key(&mut self, key: KeyPair) {
        self.join_key = Some(key);
    }
}

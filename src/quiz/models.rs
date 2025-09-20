use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::models::{CreateGameRequest, GameCategory, Identify};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Quiz {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GameCategory,
    pub iterations: i32,
    pub times_played: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateQuizRequest {
    name: String,
    description: Option<String>,
    category: Option<GameCategory>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Question {
    id: i32,
    quiz_id: i32,
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizSession {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GameCategory,
    pub iterations: u8,
    pub current_iteration: u8,
    pub questions: Vec<String>,
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
            name: request.name,
            description: request.description,
            category: request.category.unwrap_or(GameCategory::Default),
            iterations: 0,
            current_iteration: 0,
            questions: vec![],
        }
    }

    pub fn from_game_and_questions(quiz: Quiz, mut questions: Vec<Question>) -> Self {
        Self {
            id: quiz.id,
            name: quiz.name,
            description: quiz.description,
            category: quiz.category,
            iterations: u8::try_from(quiz.iterations).ok().unwrap(),
            current_iteration: 0,
            questions: questions.iter_mut().map(|q| q.title.clone()).collect(),
        }
    }
}

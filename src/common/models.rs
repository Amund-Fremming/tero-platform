use core::fmt;
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{quiz::models::Quiz, spin::models::SpinGame};

pub trait Identify {
    fn get_id(&self) -> Uuid;
}

#[derive(Debug, Serialize, Deserialize, Hash, Clone, sqlx::Type)]
#[sqlx(type_name = "game_category", rename_all = "lowercase")]
pub enum GameCategory {
    #[serde(rename(deserialize = "warm_up"))]
    Warmup,
    #[serde(rename(deserialize = "casual"))]
    Casual,
    #[serde(rename(deserialize = "spicy"))]
    Spicy,
    #[serde(rename(deserialize = "dangerous"))]
    Dangerous,
    #[serde(rename(deserialize = "ladies"))]
    Ladies,
    #[serde(rename(deserialize = "boys"))]
    Boys,
    #[serde(rename(deserialize = "default"))]
    Default,
}

impl GameCategory {
    pub fn as_str(&self) -> &str {
        match self {
            GameCategory::Warmup => "warm_up",
            GameCategory::Casual => "casual",
            GameCategory::Spicy => "spicy",
            GameCategory::Dangerous => "dangerous",
            GameCategory::Ladies => "ladies",
            GameCategory::Boys => "boys",
            GameCategory::Default => "default",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub enum GameType {
    Quiz,
    Spin,
}

impl fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::Quiz => write!(f, "quiz"),
            GameType::Spin => write!(f, "spin"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct PagedRequest {
    pub category: Option<GameCategory>,
    pub page_num: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameBase {
    id: Uuid,
    name: String,
    description: Option<String>,
    category: GameCategory,
    iterations: i32,
}

impl From<Quiz> for GameBase {
    fn from(value: Quiz) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            category: value.category,
            iterations: value.iterations,
        }
    }
}

impl From<SpinGame> for GameBase {
    fn from(value: SpinGame) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            category: value.category,
            iterations: value.iterations,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PagedResponse {
    games: Vec<GameBase>,
}

impl PagedResponse {
    pub fn from_quizzes(quizzes: Vec<Quiz>) -> Self {
        Self {
            games: quizzes.into_iter().map(|q| q.into()).collect(),
        }
    }

    pub fn from_spinners(spinners: Vec<SpinGame>) -> Self {
        Self {
            games: spinners.into_iter().map(|s| s.into()).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSessionRequest {
    pub game_type: GameType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameRequest {
    pub host_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<GameCategory>,
}

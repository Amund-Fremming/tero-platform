use core::fmt;
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GameBase {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: GameCategory,
    pub iterations: i32,
    pub times_played: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PagedResponse {
    games: Vec<GameBase>,
    has_next: bool,
}

impl PagedResponse {
    pub fn new(games: Vec<GameBase>, has_next: bool) -> Self {
        Self { games, has_next }
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

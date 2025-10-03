use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PagedResponse<T> {
    games: Vec<T>,
    has_next: bool,
}

impl<T> PagedResponse<T> {
    pub fn new(games: Vec<T>, has_next: bool) -> Self {
        Self { games, has_next }
    }
}

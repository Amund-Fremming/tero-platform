use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::common::error::ServerError;

// Placed here private because it fucked me up big time
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
struct JoinKey {
    pub id: String,
    pub word: String,
}

pub async fn get_word_sets(
    pool: &Pool<Postgres>,
) -> Result<(Vec<String>, Vec<String>), ServerError> {
    let prefix_fut =
        sqlx::query_scalar::<_, String>("SELECT word FROM prefix_word").fetch_all(pool);

    let suffix_fut =
        sqlx::query_scalar::<_, String>("SELECT word FROM prefix_word").fetch_all(pool);

    let (prefix_result, suffix_result): (
        Result<Vec<String>, sqlx::Error>,
        Result<Vec<String>, sqlx::Error>,
    ) = tokio::join!(prefix_fut, suffix_fut);

    Ok((prefix_result?, suffix_result?))
}

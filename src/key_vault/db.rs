use sqlx::{Pool, Postgres};

use crate::{key_vault::key_vault::JoinKey, server::server_error::ServerError};

pub async fn get_word_set(pool: &Pool<Postgres>, keys: &[&str; 2]) -> Result<String, ServerError> {
    let keys = sqlx::query_as::<_, JoinKey>(
        r#"
        SELECT id, word
        FROM "join_key"
        WHERE id = ANY($1)
        LIMIT 2
        "#,
    )
    .bind(keys)
    .fetch_all(pool)
    .await?;

    if keys.len() != 2 {
        return Err(ServerError::Internal("Missing join keys".into()));
    }

    let join_key = format!("{} {}", keys[0].word, keys[1].word);
    Ok(join_key)
}

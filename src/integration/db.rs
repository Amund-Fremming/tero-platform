use sqlx::{Pool, Postgres};

use crate::integration::models::Integration;

pub async fn list_integrations(pool: &Pool<Postgres>) -> Result<Vec<Integration>, sqlx::Error> {
    sqlx::query_as!(
        Integration,
        r#"
        SELECT id, subject, name as "name: _"
        FROM "integration"
        "#,
    )
    .fetch_all(pool)
    .await
}

use sqlx::{Pool, Postgres};

use crate::integration::models::Integration;

pub async fn list_integrations(pool: &Pool<Postgres>) -> Result<Vec<Integration>, sqlx::Error> {
    sqlx::query_as::<_, Integration>(
        r#"
        SELECT id, subject, name
        FROM "integration"
        "#,
    )
    .fetch_all(pool)
    .await
}

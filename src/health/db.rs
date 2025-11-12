use sqlx::{Pool, Postgres};

pub async fn health_check(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    let _ = sqlx::query_scalar!("SELECT 1 as one").fetch_one(pool).await?;
    Ok(())
}

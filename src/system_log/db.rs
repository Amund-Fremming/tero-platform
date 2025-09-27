use sqlx::{Pool, Postgres};

use crate::{
    server::error::ServerError,
    system_log::models::{LogAction, LogCeverity, SubjectType},
};

pub async fn create_system_log(
    pool: &Pool<Postgres>,
    subject_id: &str,
    subject_type: &SubjectType,
    action: &LogAction,
    ceverity: &LogCeverity,
    file_name: &str,
    description: &str,
    metadata: &Option<serde_json::Value>,
) -> Result<(), ServerError> {
    let row = sqlx::query(
        r#"
        INSERT INTO "system_log" (subject_id, action, ceverity, file_name, description, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(subject_id)
    .bind(subject_type)
    .bind(action)
    .bind(ceverity)
    .bind(file_name)
    .bind(description)
    .bind(metadata)
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal("Failed to create system log".into()));
    }

    Ok(())
}

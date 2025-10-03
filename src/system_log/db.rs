use chrono::Utc;
use sqlx::{Pool, Postgres};

use crate::{
    common::{error::ServerError, models::PagedResponse},
    config::config::CONFIG,
    system_log::models::{Action, LogCeverity, SubjectType, SyslogPageRequest, SystemLog},
};

pub async fn get_system_log_page(
    pool: &Pool<Postgres>,
    request: SyslogPageRequest,
) -> Result<PagedResponse<SystemLog>, sqlx::Error> {
    let mut query = format!(
        r#"
        SELECT id, subject_id, subject_type, action, ceverity, function, description, metadata
        FROM "system_log"
        "#
    );

    let mut conditions = Vec::new();

    if let Some(subject_type) = request.subject_type {
        conditions.push(format!("subject_type = '{}'", subject_type));
    }

    if let Some(action) = request.action {
        conditions.push(format!("action = '{}'", action));
    }

    if let Some(ceverity) = request.ceverity {
        conditions.push(format!("ceverity = '{}'", ceverity));
    }

    let page_size = CONFIG.server.page_size as u16;
    let offset = page_size * request.page_num;
    let limit = page_size + 1;

    query.push_str(&format!(
        r#"
        WHERE {} 
        LIMIT {} OFFSET {}
        ORDER BY created_at DESC
        "#,
        conditions.join(" AND "),
        limit,
        offset
    ));

    let logs = sqlx::query_as::<_, SystemLog>(&query)
        .fetch_all(pool)
        .await?;

    let has_next = logs.len() < limit as usize;
    let page = PagedResponse::new(logs, has_next);

    Ok(page)
}

pub async fn create_system_log(
    pool: &Pool<Postgres>,
    subject_id: &str,
    subject_type: &SubjectType,
    action: &Action,
    ceverity: &LogCeverity,
    file_name: &str,
    description: &str,
    metadata: &Option<serde_json::Value>,
) -> Result<(), ServerError> {
    let row = sqlx::query(
        r#"
        INSERT INTO "system_log" (subject_id, action, ceverity, file_name, description, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(subject_id)
    .bind(subject_type)
    .bind(action)
    .bind(ceverity)
    .bind(file_name)
    .bind(description)
    .bind(metadata)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal("Failed to create system log".into()));
    }

    Ok(())
}

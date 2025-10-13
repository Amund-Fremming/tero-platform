use chrono::Utc;
use sqlx::{Pool, Postgres};

use crate::{
    common::{db_query_builder::DBQueryBuilder, error::ServerError, models::PagedResponse},
    config::config::CONFIG,
    system_log::models::{Action, LogCeverity, SubjectType, SyslogPageQuery, SystemLog},
};

pub async fn get_system_log_page(
    pool: &Pool<Postgres>,
    request: SyslogPageQuery,
) -> Result<PagedResponse<SystemLog>, sqlx::Error> {
    let page_size = CONFIG.server.page_size as u16;
    let logs = DBQueryBuilder::select(
        r#"
            id,
            subject_id,
            subject_type,
            action,
            ceverity,
            function,
            description,
            metadata
        "#,
    )
    .from("system_log")
    .where_opt("subject_type", &request.subject_type)
    .where_opt("action", &request.action)
    .where_opt("ceverity", &request.ceverity)
    .offset(page_size * request.page_num)
    .limit(page_size + 1)
    .order_desc("created_at")
    .build()
    .build_query_as::<SystemLog>()
    .fetch_all(pool)
    .await?;

    let has_next = logs.len() < (page_size + 1) as usize;
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

use chrono::Utc;
use sqlx::{Pool, Postgres};

use crate::{
    config::config::CONFIG,
    models::{
        error::ServerError,
        popup_manager::PagedResponse,
        system_log::{LogAction, LogCategoryCount, LogCeverity, SubjectType, SyslogPageQuery, SystemLog},
    },
    service::db_query_builder::DBQueryBuilder,
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
    action: &LogAction,
    ceverity: &LogCeverity,
    file_name: &str,
    description: &str,
    metadata: &Option<serde_json::Value>,
) -> Result<(), ServerError> {
    let created_at = Utc::now();
    let row = sqlx::query!(
        r#"
        INSERT INTO "system_log" (subject_id, subject_type, action, ceverity, file_name, description, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        subject_id,
        subject_type as _,
        action as _,
        ceverity as _,
        file_name,
        description,
        metadata as _,
        created_at
    )
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal("Failed to create system log".into()));
    }

    Ok(())
}

pub async fn get_log_category_count(
    pool: &Pool<Postgres>,
) -> Result<LogCategoryCount, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct CountRow {
        info: i64,
        warning: i64,
        critical: i64,
    }

    let result = sqlx::query_as::<_, CountRow>(
        r#"
        SELECT 
            COUNT(*) FILTER (WHERE ceverity = 'info') as info,
            COUNT(*) FILTER (WHERE ceverity = 'warning') as warning,
            COUNT(*) FILTER (WHERE ceverity = 'critical') as critical
        FROM system_log
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(LogCategoryCount {
        info: result.info,
        warning: result.warning,
        critical: result.critical,
    })
}

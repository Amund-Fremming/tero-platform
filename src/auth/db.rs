use chrono::Utc;
use serde_json::json;
use sqlx::{Pool, Postgres, QueryBuilder, query, query_as};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    auth::models::{
        ActivityStats, Auth0User, AverageUserStats, BaseUser, ListUsersQuery, PatchUserRequest,
        RecentUserStats,
    },
    common::{error::ServerError, models::PagedResponse},
    config::config::CONFIG,
    game::models::Gender,
    system_log::{
        builder::SystemLogBuilder,
        models::{Action, LogCeverity},
    },
};


pub async fn ensure_pseudo_user(pool: &Pool<Postgres>, id: Uuid) {
    let result = sqlx::query(
        r#"
        INSERT INTO "pseudo_user" (id, last_active)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(Utc::now())
    .execute(pool)
    .await;

    match result {
        Err(e) => {
            let _ = SystemLogBuilder::new(pool)
                .action(Action::Create)
                .ceverity(LogCeverity::Critical)
                .function("ensure_psuedo_user")
                .description("Failed to do insert on pseudo user. Should not fail")
                .metadata(json!({"error": e.to_string()}))
                .log();
        }
        Ok(row) => {
            if row.rows_affected() != 0 {
                let _ = SystemLogBuilder::new(pool)
                    .action(Action::Create)
                    .ceverity(LogCeverity::Warning)
                    .function("ensure_psuedo_user")
                    .description("User had pseudo user that did not exist, so a new was created. This will cause ghost users")
                    .log();
            }
        }
    };
}

pub async fn get_base_user_by_auth0_id(
    pool: &Pool<Postgres>,
    auth0_id: &str,
) -> Result<Option<BaseUser>, sqlx::Error> {
    sqlx::query_as::<_, BaseUser>(
        r#"
        SELECT id, username, auth0_id, birth_date, gender, email,
            email_verified, family_name, updated_at, given_name, created_at
        FROM "base_user"
        WHERE auth0_id = $1
        "#,
    )
    .bind(auth0_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_base_user_by_id(
    pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Option<BaseUser>, sqlx::Error> {
    sqlx::query_as::<_, BaseUser>(
        r#"
        SELECT id, username, auth0_id, birth_date, gender, email,
            email_verified, family_name, updated_at, given_name, created_at
        FROM "base_user"
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn pseudo_user_exists(pool: &Pool<Postgres>, id: Uuid) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, Uuid>("SELECT id FROM \"pseudo_user\" WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(exists.is_some())
}

pub async fn create_pseudo_user(
    pool: &Pool<Postgres>,
    id: Option<Uuid>,
) -> Result<Uuid, ServerError> {
    let id = id.unwrap_or(Uuid::new_v4());

    let pseudo_id: Uuid = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO "pseudo_user" (id, last_active)
        VALUES ($1, $2)
        RETURNING id;
        "#,
    )
    .bind(id)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    Ok(pseudo_id)
}


pub async fn create_base_user(
    pool: &Pool<Postgres>,
    auth0_user: &Auth0User,
) -> Result<(), ServerError> {
    let email = auth0_user.email.clone().unwrap_or("Kenneth".to_string());
    let split = email.splitn(2, '@').next().unwrap_or("Kenneth").to_string();

    let username = match &auth0_user.username {
        Some(username) => username.to_string(),
        None => split,
    };

    // Extract names safely, with fallbacks to username split
    let given_name: &str = auth0_user
        .given_name
        .as_deref()
        .unwrap_or_else(|| username.split('.').next().unwrap_or("John"));

    let family_name: &str = auth0_user
        .family_name
        .as_deref()
        .unwrap_or_else(|| username.split('.').nth(1).unwrap_or("Doe"));

    let result = sqlx::query(
        r#"
        INSERT INTO "base_user" (id, username, auth0_id, gender, email, email_verified, updated_at, family_name, given_name, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&username)
    .bind(&auth0_user.auth0_id)
    .bind(Gender::Unknown)
    .bind(&auth0_user.email)
    .bind(&auth0_user.email_verified)
    .bind(&auth0_user.updated_at)
    .bind(family_name)  
    .bind(given_name)   
    .bind(&auth0_user.created_at)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        error!("Failed to create registered user");
        return Err(ServerError::Internal(
            "Failed to create registered user".into(),
        ));
    }

    Ok(())
}

pub async fn update_pseudo_user_activity(
    pool: &Pool<Postgres>,
    id: Uuid,
) -> Result<(), ServerError> {
    let row = sqlx::query(
        r#"
        UPDATE "pseudo_user"
        SET last_active = $1
        WHERE id = $2
        "#,
    )
    .bind(&Utc::now())
    .bind(&id)
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        warn!("Query failed, no user with id: {}", id);
        return Err(ServerError::NotFound("User does not exist".into()));
    }

    Ok(())
}

pub async fn patch_base_user_by_id(
    pool: &Pool<Postgres>,
    user_id: &Uuid,
    request: PatchUserRequest,
) -> Result<BaseUser, ServerError> {
    let mut builder: QueryBuilder<'_, Postgres> = sqlx::QueryBuilder::new("UPDATE base_user SET ");
    let mut separator = builder.separated(", ");

    if let Some(username) = request.username {
        separator.push("username = ").push_bind_unseparated(username);
    }

    if let Some(gname) = request.given_name {
        separator.push("given_name = ").push_bind_unseparated(gname);
    }

    if let Some(fname) = request.family_name {
        separator.push("family_name = ").push_bind_unseparated(fname);
    }

    if let Some(gender) = request.gender {
        separator.push("gender = ").push_bind_unseparated(gender);
    }

    if let Some(birth_date) = request.birth_date {
        separator.push("birth_date = ").push_bind_unseparated(birth_date);
    }

    builder.push(" WHERE id = ").push_bind(user_id);  // Also fixed: use 'id', not 'user_id'
    builder.push(" RETURNING id, username, auth0_id, birth_date, gender, email, email_verified, family_name, updated_at, given_name, created_at");
    let result: BaseUser = builder.build_query_as().fetch_one(pool).await?;
    
    Ok(result)
}

pub async fn delete_base_user_by_id(pool: &Pool<Postgres>, id: &Uuid) -> Result<(), ServerError> {
    let result = query(
        r#"
        DELETE FROM "base_user" WHERE id = $1;
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        warn!("Query failed, no game with id: {}", id);
        return Err(ServerError::NotFound("User does not exist".into()));
    }

    Ok(())
}

pub async fn list_base_users(
    pool: &Pool<Postgres>,
    query: ListUsersQuery,
) -> Result<PagedResponse<BaseUser>, sqlx::Error> {
    let offset = CONFIG.server.page_size * query.page_num;
    let limit = CONFIG.server.page_size + 1;

    let items = query_as::<_, BaseUser>(
        r#"
        SELECT id, username, auth0_id, gender, email, email_verified, updated_at, family_name, given_name, created_at
        FROM "base_user"
        OFFSET = $1 LIMIT = $2
        ORDER BY created_at DESC
        "#,
    )
    .bind(offset as i32)
    .bind(limit as i32)
    .fetch_all(pool)
    .await?;

    let has_next = items.len() > CONFIG.server.page_size as usize;
    let response = PagedResponse::new(items, has_next);

    Ok(response)
}

pub async fn get_user_activity_stats(pool: &Pool<Postgres>) -> Result<ActivityStats, sqlx::Error> {
    let recent_fut = sqlx::query_as::<_, RecentUserStats>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE last_active >= date_trunc('month', CURRENT_DATE)) AS this_month_users,
            COUNT(*) FILTER (WHERE last_active >= date_trunc('week', CURRENT_DATE)) AS this_week_users,
            COUNT(*) FILTER (WHERE last_active >= CURRENT_DATE) AS todays_users
        FROM pseudo_user
        "#
    )
    .fetch_one(pool);

    let average_fut = sqlx::query_as::<_, AverageUserStats>(
        r#"
        SELECT
            COALESCE((
                SELECT AVG(cnt)::float8 
                FROM (
                    SELECT COUNT(*) AS cnt 
                    FROM pseudo_user 
                    WHERE last_active >= CURRENT_DATE - INTERVAL '6 months'
                    GROUP BY date_trunc('month', last_active)
                ) t
            ), 0) AS avg_month_users,
            COALESCE((
                SELECT AVG(cnt)::float8 
                FROM (
                    SELECT COUNT(*) AS cnt 
                    FROM pseudo_user 
                    WHERE last_active >= CURRENT_DATE - INTERVAL '8 weeks'
                    GROUP BY date_trunc('week', last_active)
                ) t
            ), 0) AS avg_week_users,
            COALESCE((
                SELECT AVG(cnt)::float8 
                FROM (
                    SELECT COUNT(*) AS cnt 
                    FROM pseudo_user 
                    WHERE last_active >= CURRENT_DATE - INTERVAL '30 days'
                    GROUP BY last_active::date
                ) t
            ), 0) AS avg_daily_users
        "#
    )
    .fetch_one(pool);

    let total_game_count_fut =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM game_base").fetch_one(pool);

    let total_user_count_fut =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM base_user").fetch_one(pool);

    let (recent, average, total_game_count, total_user_count): (
        Result<RecentUserStats, sqlx::Error>,
        Result<AverageUserStats, sqlx::Error>,
        Result<i64, sqlx::Error>,
        Result<i64, sqlx::Error>,
    ) = tokio::join!(
        recent_fut,
        average_fut,
        total_game_count_fut,
        total_user_count_fut
    );

    Ok(ActivityStats {
        total_game_count: total_game_count?,
        total_user_count: total_user_count?,
        recent: recent?,
        average: average?,
    })
}

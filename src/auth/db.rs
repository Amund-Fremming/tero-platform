use chrono::Utc;
use serde_json::json;
use sqlx::{Pool, Postgres, QueryBuilder, Row, query, query_as};
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

pub async fn delete_pseudo_user(pool: &Pool<Postgres>, id: &Uuid) -> Result<(), ServerError> {
    let row = sqlx::query(
        r#"
        DELETE FROM "pseudo_user"
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal("Failed to delete pseudo user".into()));
    }

    Ok(())
}

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

pub async fn set_pseudo_user_id(
    pool: &Pool<Postgres>,
    new_id: Uuid,
    old_id: Uuid,
) -> Result<(), ServerError> {
    let row = sqlx::query(
        r#"
        UPDATE "pseudo_user"
        SET id = $1
        WHERE id = $2
        "#,
    )
    .bind(new_id)
    .bind(old_id)
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal("Failed to sync users".into()));
    }

    Ok(())
}

pub async fn get_base_user_by_auth0_id(
    pool: &Pool<Postgres>,
    auth0_id: &str,
) -> Result<Option<BaseUser>, sqlx::Error> {
    sqlx::query_as::<_, BaseUser>(
        r#"
        SELECT id, username, auth0_id, guest_id, birth_date, gender, email,
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
        SELECT id, username, auth0_id, guest_id, birth_date, gender, email,
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

pub async fn create_pseudo_user(pool: &Pool<Postgres>) -> Result<Uuid, ServerError> {
    let row = sqlx::query(
        r#"
        INSERT INTO "pseudo_user" (id, last_active)
        VALUES ($1, $2)
        RETURNING id;
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    if row.len() == 0 {
        return Err(ServerError::Internal("Failed to create guest id".into()));
    }

    let guest_id = row.get("guest_id");
    Ok(guest_id)
}

pub async fn create_base_user(
    pool: &Pool<Postgres>,
    auth0_user: &Auth0User,
) -> Result<(), ServerError> {
    let username = match &auth0_user.username {
        Some(username) => username.to_string(),
        None => {
            let email = auth0_user.email.clone().unwrap_or("Kenneth".to_string());
            let username = email.splitn(2, '@').next().unwrap_or("Kenneth").to_string();
            username
        }
    };

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
    .bind(&auth0_user.family_name)
    .bind(&auth0_user.given_name)
    .bind(&auth0_user.created_at)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        error!("Failed to create registered user");
        return Err(ServerError::NotFound(
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
    put_request: PatchUserRequest,
) -> Result<(), ServerError> {
    let mut builder: QueryBuilder<'_, Postgres> = sqlx::QueryBuilder::new("UPDATE user SET ");
    let mut separator = builder.separated(", ");

    if let Some(name) = put_request.name {
        separator.push_unseparated("name = ").push_bind(name);
    }

    if let Some(email) = put_request.email {
        separator.push_unseparated("email = ").push_bind(email);
    }

    if let Some(birth_date) = put_request.birth_date {
        separator
            .push_unseparated("birth_date = ")
            .push_bind(birth_date);
    }

    builder.push(" WHERE user_id = ").push_bind(user_id);
    let result = builder.build().execute(pool).await?;

    if result.rows_affected() == 0 {
        warn!("Query failed, no user with id: {}", user_id);
        return Err(ServerError::NotFound("User does not exist".into()));
    }

    Ok(())
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

// TODO - update to count base + pseudo
pub async fn get_user_activity_stats(pool: &Pool<Postgres>) -> Result<ActivityStats, sqlx::Error> {
    let recent_fut = sqlx::query_as::<_, RecentUserStats>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE last_active >= date_trunc('month', CURRENT_DATE)) AS this_month_users,
            COUNT(*) FILTER (WHERE last_active >= date_trunc('week', CURRENT_DATE)) AS this_week_users,
            COUNT(*) FILTER (WHERE last_active = CURRENT_DATE) AS todays_users,
        FROM users
        "#
    )
    .fetch_one(pool);

    let average_fut = sqlx::query_as::<_, AverageUserStats>(
        r#"
        SELECT
            (SELECT AVG(cnt) FROM (SELECT COUNT(*) AS cnt FROM users GROUP BY date_trunc('month', created_at)) t) AS avg_month_users,
            (SELECT AVG(cnt) FROM (SELECT COUNT(*) AS cnt FROM users GROUP BY date_trunc('week', created_at)) t) AS avg_week_users,
            (SELECT AVG(cnt) FROM (SELECT COUNT(*) AS cnt FROM users GROUP BY created_at) t) AS avg_daily_users
        "#
    )
    .fetch_one(pool);

    let total_game_count_fut =
        sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM games").fetch_one(pool);

    let total_user_count_fut =
        sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM base_user").fetch_one(pool);

    let (recent, average, total_game_count, total_user_count): (
        Result<RecentUserStats, sqlx::Error>,
        Result<AverageUserStats, sqlx::Error>,
        Result<i32, sqlx::Error>,
        Result<i32, sqlx::Error>,
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

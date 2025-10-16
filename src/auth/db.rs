use chrono::Utc;
use sqlx::{Pool, Postgres, QueryBuilder, Row, Transaction, query, query_as};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    auth::models::{
        ActivityStats, Auth0User, AverageUserStats, PutUserRequest, RecentUserStats, User,
        UserKeys, UserType,
    },
    common::error::ServerError,
};

pub async fn tx_sync_user(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    guest_id: Uuid,
) -> Result<(), ServerError> {
    let delete_row = sqlx::query(
        r#"
        DELETE FROM "user"
        WHERE guest_id = $1
        "#,
    )
    .bind(guest_id)
    .execute(&mut **tx)
    .await?;

    let insert_row = sqlx::query(
        r#"
        UPDATE "user"
        SET guest_id = $1
        WHERE id = $2
        "#,
    )
    .bind(guest_id)
    .bind(user_id)
    .execute(&mut **tx)
    .await?;

    if delete_row.rows_affected() == 0 || insert_row.rows_affected() == 0 {
        return Err(ServerError::Internal(
            "Failed to sync user to the database".into(),
        ));
    }

    Ok(())
}

pub async fn get_user_id_from_guest_id(
    pool: &Pool<Postgres>,
    guest_id: &Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT id
        FROM "user"
        WHERE guest_id = $1
        "#,
    )
    .bind(guest_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_user_keys_from_auth0_id(
    pool: &Pool<Postgres>,
    auth0_id: &str,
) -> Result<UserKeys, ServerError> {
    let option = sqlx::query_as::<_, UserKeys>(
        r#"
        SELECT id, auth0_id, guest_id
        FROM "user"
        WHERE auth0_id = $1
        "#,
    )
    .bind(auth0_id)
    .fetch_optional(pool)
    .await?;

    let Some(keys) = option else {
        return Err(ServerError::OutOfSync(format!(
            "User id is out of sync with auth0_id {}",
            auth0_id
        )));
    };

    Ok(keys)
}

pub async fn get_user_by_id(
    pool: &Pool<Postgres>,
    user_id: &Uuid,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, auth0_id, guest_id, user_type, last_active, birth_date, gender, email,
            email_verified, family_name, "updated_at", "given_name", "created_at"
        FROM "user"
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn guest_user_exists(pool: &Pool<Postgres>, id: Uuid) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, Uuid>("SELECT guest_id FROM \"user\" WHERE guest_id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(exists.is_some())
}

pub async fn create_guest_user(pool: &Pool<Postgres>) -> Result<Uuid, ServerError> {
    let row = sqlx::query(
        r#"
        INSERT INTO "user" (guest_id, user_type, last_active)
        VALUES ($1, $2, $3)
        RETURNING guest_id;
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(UserType::Guest)
    .bind(Utc::now())
    .fetch_one(pool)
    .await?;

    if row.len() == 0 {
        return Err(ServerError::Internal("Failed to create guest id".into()));
    }

    let guest_id = row.get("guest_id");
    Ok(guest_id)
}

pub async fn create_registered_user(
    pool: &Pool<Postgres>,
    auth0_user: &Auth0User,
) -> Result<(), ServerError> {
    let fullname = format!(
        "{} {}",
        auth0_user.given_name.as_deref().unwrap_or(""),
        auth0_user.family_name.as_deref().unwrap_or("")
    );

    let result = sqlx::query(
        r#"
        INSERT INTO "user" (auth0_id, user_type, name, email)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&auth0_user.auth0_id)
    .bind(&UserType::Registered)
    .bind(&fullname)
    .bind(&auth0_user.email)
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

pub async fn update_user_activity(pool: &Pool<Postgres>, user_id: Uuid) -> Result<(), ServerError> {
    let row = sqlx::query(
        r#"
        UPDATE "user"
        SET last_active = $1
        WHERE id = $2
        "#,
    )
    .bind(&Utc::now())
    .bind(&user_id)
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        warn!("Query failed, no user with id: {}", user_id);
        return Err(ServerError::NotFound("User does not exist".into()));
    }

    Ok(())
}

pub async fn patch_user_by_id(
    pool: &Pool<Postgres>,
    user_id: &Uuid,
    put_request: PutUserRequest,
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

pub async fn delete_user_by_id(pool: &Pool<Postgres>, user_id: &Uuid) -> Result<(), ServerError> {
    let result = query(
        r#"
        DELETE FROM "user" WHERE id = $1;
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        warn!("Query failed, no game with id: {}", user_id);
        return Err(ServerError::NotFound("User does not exist".into()));
    }

    Ok(())
}

pub async fn list_all_users(pool: &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error> {
    query_as::<_, User>(r#"SELECT * FROM "user""#)
        .fetch_all(pool)
        .await
}

pub async fn get_user_activity_stats(pool: &Pool<Postgres>) -> Result<ActivityStats, sqlx::Error> {
    let recent_fut = sqlx::query_as::<_, RecentUserStats>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE created_at >= date_trunc('month', CURRENT_DATE)) AS this_month_users,
            COUNT(*) FILTER (WHERE created_at >= date_trunc('week', CURRENT_DATE)) AS this_week_users,
            COUNT(*) FILTER (WHERE created_at = CURRENT_DATE) AS todays_users,
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
        sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM user").fetch_one(pool);

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

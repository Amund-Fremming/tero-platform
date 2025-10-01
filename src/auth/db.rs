use chrono::Utc;
use sqlx::{Pool, Postgres, Row, Transaction, query, query_as};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    auth::models::{Auth0User, PutUserRequest, User, UserKeys, UserType},
    server::error::ServerError,
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
        SELECT user_id
        FROM "user"
        WHERE guest_id = &1
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
    sqlx::query_as::<_, User>(r#"SELECT * FROM "user" WHERE id = $1"#)
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn create_guest_user(pool: &Pool<Postgres>) -> Result<Uuid, sqlx::Error> {
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
        SET last_updated = $1
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
    let mut query: String = String::from("UPDATE user SET");
    let mut conditions: Vec<String> = Vec::new();

    if let Some(name) = put_request.name {
        conditions.push(format!("name = {}", name));
    }

    if let Some(email) = put_request.email {
        conditions.push(format!("email = {}", email));
    }

    if let Some(birth_date) = put_request.birth_date {
        conditions.push(format!("birth_date = {}", birth_date));
    }

    query.push_str(conditions.join(", ").as_str());
    query.push_str(format!("WHERE id = {}", user_id).as_str());

    let result = sqlx::query(&query).execute(pool).await?;

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

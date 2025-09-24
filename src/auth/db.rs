use chrono::Utc;
use sqlx::{Pool, Postgres, Row, query, query_as};
use uuid::Uuid;

use crate::{
    auth::models::{Auth0User, PutUserRequest, User, UserType},
    common::server_error::ServerError,
};

pub async fn get_user_id_by_auth0_id(
    pool: &Pool<Postgres>,
    auth0_id: &str,
) -> Result<Uuid, ServerError> {
    let option = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id from "user"
        WHERE auth0_id = $1
        "#,
    )
    .bind(auth0_id)
    .fetch_optional(pool)
    .await?;

    let Some(uuid) = option else {
        return Err(ServerError::OutOfSync(format!(
            "User id is out of sync with auth0_id {}",
            auth0_id
        )));
    };

    Ok(uuid)
}

pub async fn get_user_by_id(
    pool: &Pool<Postgres>,
    user_id: i32,
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
        return Err(ServerError::Internal(
            "No rows affected when auth0 triggered creating a new user".into(),
        ));
    }

    Ok(())
}

pub async fn update_user_activity(pool: &Pool<Postgres>, user_id: i32) -> Result<(), ServerError> {
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
        return Err(ServerError::Internal(
            "No rows affected when updating user activity".into(),
        ));
    }

    Ok(())
}

pub async fn get_user_by_auth0_id(
    pool: &Pool<Postgres>,
    auth0_id: String,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * from user WHERE auth0_id = $1")
        .bind(&auth0_id)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_by_guest_id(
    pool: &Pool<Postgres>,
    guest_id: Uuid,
) -> Result<Option<User>, sqlx::Error> {
    let option = sqlx::query_as::<_, User>("SELECT * from user WHERE id = $1")
        .bind(&guest_id)
        .fetch_optional(pool)
        .await?;

    match option {
        Some(u) => Ok(Some(u.strip())),
        None => Ok(None),
    }
}

pub async fn patch_user_by_id(
    pool: &Pool<Postgres>,
    user_id: i32,
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
        return Err(ServerError::Internal(
            "No rows affected when updating user".into(),
        ));
    }

    Ok(())
}

pub async fn delete_user_by_id(pool: &Pool<Postgres>, user_id: i32) -> Result<(), ServerError> {
    let result = query(
        r#"
        DELETE FROM "user" WHERE id = $1;
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ServerError::Internal(
            "No rows affected when deleting user".into(),
        ));
    }

    Ok(())
}

pub async fn list_all_users(pool: &Pool<Postgres>) -> Result<Vec<User>, sqlx::Error> {
    query_as::<_, User>(r#"SELECT * FROM "user""#)
        .fetch_all(pool)
        .await
}

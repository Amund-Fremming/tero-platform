use chrono::Utc;
use sqlx::{Pool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    common::error::ServerError,
    spin::models::{SpinGame, SpinSession},
};

pub async fn get_spin_session_by_game_id(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    game_id: Uuid,
) -> Result<SpinSession, ServerError> {
    let game = sqlx::query_as::<_, SpinGame>(
        r#"
        SELECT
            base.id AS base_id,
            spin.id AS spin_id,
            base.name,
            base.description,
            base.game_type,
            base.category,
            base.iterations,
            base.times_played
            base.last_played
            spin.rounds
        FROM "game_base" base
        JOIN "spin_game" spin
        ON base.id = spin.base_id
        WHERE base.id = $1
        "#,
    )
    .bind(game_id)
    .fetch_one(pool)
    .await?;

    let session = SpinSession::from_game(user_id, game);
    Ok(session)
}

pub async fn tx_persist_spin_session(
    tx: &mut Transaction<'_, Postgres>,
    session: &SpinSession,
) -> Result<(), ServerError> {
    let game_row = sqlx::query(
        r#"
        INSERT INTO "game_base" (id, name, description, game_type, category, iterations, times_played, last_played)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(&session.base_id)
    .bind(&session.name)
    .bind(&session.description)
    .bind(&session.category)
    .bind(&session.iterations)
    .bind(&session.times_played)
    .bind(Utc::now())
    .execute(&mut **tx)
    .await?;

    let round_row = sqlx::query(
        r#"
        INSERT INTO "spin_game" (id, base_id, rounds)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&session.base_id)
    .bind(&session.rounds)
    .execute(&mut **tx)
    .await?;

    if game_row.rows_affected() == 0 || round_row.rows_affected() == 0 {
        return Err(ServerError::Internal(
            "Failed to persist spin game session".into(),
        ));
    }

    Ok(())
}

use sqlx::{Pool, Postgres, Transaction};
use tracing::error;
use uuid::Uuid;

use crate::{
    server::error::ServerError,
    spin::models::{Round, SpinGame, SpinSession},
};

pub async fn get_spin_session_by_game_id(
    pool: &Pool<Postgres>,
    host_id: Uuid,
    game_id: Uuid,
) -> Result<SpinSession, ServerError> {
    let game = sqlx::query_as::<_, SpinGame>(
        r#"
        SELECT id, host_id, name, description, category, iterations, times_played
        FROM spinner
        WHERE id = $1
        "#,
    )
    .bind(game_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Spinner with id {} was not found",
        game_id
    )))?;

    let rounds = sqlx::query_as::<_, Round>(
        r#"
        SELECT id, spinner_id, participants, read_before, title
        FROM round
        WHERE spinner_id = $1 
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await?;

    let session = SpinSession::from_game_and_rounds(host_id, game, rounds);

    Ok(session)
}

pub async fn tx_persist_spin_session(
    tx: &mut Transaction<'_, Postgres>,
    session: &SpinSession,
) -> Result<(), ServerError> {
    let (game, rounds) = session.to_game_and_rounds();

    let game_row = sqlx::query(
        r#"
        INSERT INTO "spin_game" (id, name, description, category, iterations, times_played)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&game.id)
    .bind(&game.name)
    .bind(&game.description)
    .bind(&game.category)
    .bind(&game.iterations)
    .bind(&game.times_played)
    .execute(&mut **tx)
    .await?;

    if rounds.is_empty() {}

    let round_row = sqlx::query(
        r#"
        INSERT INTO "spin_game_rounds" (id, spin_game_id, participants, consent)
        VALUES ($1, $2, $3, $4)
    "#,
    )
    .execute(&mut **tx)
    .await?;

    if game_row.rows_affected() == 0 || round_row.rows_affected() == 0 {
        error!("Failed to persist spin session");
        return Err(ServerError::Internal(
            "Failed to persist spin session".into(),
        ));
    }

    Ok(())
}

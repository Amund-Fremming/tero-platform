use chrono::Utc;
use sqlx::{Pool, Postgres, Transaction};
use uuid::Uuid;

use crate::{common::error::ServerError, quiz::models::QuizSession};

pub async fn get_quiz_session_by_id(
    pool: &Pool<Postgres>,
    base_id: &Uuid,
) -> Result<QuizSession, ServerError> {
    let session = sqlx::query_as::<_, QuizSession>(
        r#"
        SELECT 
            base.id AS base_id,
            quiz.id AS quiz_id,
            base.name,
            base.description,
            base.game_type,
            base.category,
            base.iterations,
            base.times_played,
            0 AS current_iteration,
            quiz.questions
        FROM "game_base" base
        JOIN "quiz_game" quiz
        ON base.id = quiz.base_id
        WHERE base.id = $1
        "#,
    )
    .bind(base_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Quiz with id {} does not exist",
        base_id
    )))?;

    Ok(session)
}

// TODO - join and do asycn
pub async fn tx_persist_quiz_session(
    tx: &mut Transaction<'_, Postgres>,
    session: &QuizSession,
) -> Result<(), ServerError> {
    let times_played = 1;

    let base_row = sqlx::query(
        r#"
        INSERT INTO "game_base" (id, name, description, category, iterations, times_played, last_played)
        VALUES ($1, $2, $3, $4, $5, $6, &7)
        "#
    )
    .bind(&session.quiz_id)
    .bind(&session.name)
    .bind(&session.description)
    .bind(&session.category)
    .bind(session.iterations as i32)
    .bind(&times_played)
    .bind(Utc::now())
    .execute(&mut **tx)
    .await?;

    let quiz_row = sqlx::query(
        r#"
        INSERT INTO "quiz_game" (id, base_id, questions)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(&session.base_id)
    .bind(&session.quiz_id)
    .bind(&session.questions)
    .execute(&mut **tx)
    .await?;

    if base_row.rows_affected() == 0 || quiz_row.rows_affected() == 0 {
        return Err(ServerError::Internal(
            "Failed to persist quiz session".into(),
        ));
    }

    Ok(())
}

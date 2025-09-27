use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
    quiz::models::{QuizGame, QuizSession},
    server::error::ServerError,
};

pub async fn get_quiz_session_by_id(
    pool: &Pool<Postgres>,
    quiz_id: &Uuid,
) -> Result<QuizSession, ServerError> {
    let quiz = sqlx::query_as::<_, QuizGame>(
        r#"
        SELECT id, name, description, category, iterations, times_played, questions
        FROM "quiz_game"
        WHERE id = $1
        "#,
    )
    .bind(quiz_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Quiz with id {} does not exist",
        quiz_id
    )))?;

    let session = QuizSession::from_game(quiz);
    Ok(session)
}

pub async fn persist_quiz_session(
    pool: &Pool<Postgres>,
    session: &QuizSession,
) -> Result<(), ServerError> {
    let times_played = 1;

    let row = sqlx::query(
        r#"
        INSERT INTO "quiz_game" (id, name, description, category, iterations, times_played, questions)
        VALUES ($1, $2, $3, $4, $5, $6, &7)
        "#
    )
    .bind(&session.id)
    .bind(&session.name)
    .bind(&session.description)
    .bind(&session.category)
    .bind(session.iterations as i32)
    .bind(&times_played)
    .bind(&session.questions)
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal("Failed to persist quiz game".into()));
    }

    Ok(())
}

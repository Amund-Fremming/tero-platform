use chrono::Utc;
use sqlx::{Pool, Postgres};
use tracing::warn;
use uuid::Uuid;

use crate::{
    common::{error::ServerError, models::PagedResponse},
    config::config::CONFIG,
    game::models::{GameBase, GamePageRequest, GameType},
};

pub async fn get_game_page(
    pool: &Pool<Postgres>,
    game_type: GameType,
    request: GamePageRequest,
) -> Result<PagedResponse<GameBase>, sqlx::Error> {
    let mut sql = format!(
        r#"
        SELECT id, name, description, category, iterations, times_played
        FROM {}
        ORDER BY times_played DESC
        "#,
        game_type.to_string()
    );

    let mut query = Vec::new();
    let page_size = CONFIG.server.page_size as u16;
    let offset = page_size * request.page_num;
    let limit = page_size + 1;

    if let Some(category) = &request.category {
        query.push(format!(" category = '{}'", category.as_str()));
    };

    query.push(format!("LIMIT {} OFFSET {} ", limit, offset));
    sql.push_str(format!("WHERE {}", query.join(" AND ")).as_str());

    let games = sqlx::query_as::<_, GameBase>(&sql).fetch_all(pool).await?;

    let has_next = games.len() < limit as usize;
    let page = PagedResponse::new(games, has_next);

    Ok(page)
}

pub async fn increment_times_played(
    pool: &Pool<Postgres>,
    game_type: GameType,
    game_id: &Uuid,
) -> Result<(), ServerError> {
    let query = format!(
        r#"
        UPDATE {}
        SET times_played = times_played + 1, last_played = $1
        WHERE id = $2
        "#,
        game_type.to_string()
    );

    let row = sqlx::query(&query)
        .bind(Utc::now())
        .bind(game_id)
        .execute(pool)
        .await?;

    if row.rows_affected() == 0 {
        warn!("Query failed, no game with id: {}", game_id);
        return Err(ServerError::NotFound("Game does not exist".into()));
    }

    Ok(())
}

pub async fn delete_game(
    pool: &Pool<Postgres>,
    game_type: &GameType,
    id: &Uuid,
) -> Result<(), ServerError> {
    let query = format!(
        r#"
        DELETE FROM {}
        WHERE id = $1
        "#,
        game_type.to_string()
    );

    let row = sqlx::query(&query).bind(id).execute(pool).await?;

    if row.rows_affected() == 0 {
        warn!("Query failed, no game with id: {}", id);
        return Err(ServerError::Internal("Failed to delete game".into()));
    }

    Ok(())
}

use sqlx::{Pool, Postgres};
use tracing::warn;
use uuid::Uuid;

use crate::{
    game::models::{GameBase, GameType, PagedRequest, PagedResponse},
    server::error::ServerError,
};

pub async fn get_game_page(
    pool: &Pool<Postgres>,
    game_type: GameType,
    request: PagedRequest,
) -> Result<PagedResponse, sqlx::Error> {
    let mut sql = format!(
        r#"
        SELECT id, name, description, category, iterations, times_played
        FROM {}
        ORDER BY times_played DESC
        "#,
        game_type.to_string()
    );

    let mut query = Vec::new();
    let offset = 20 * request.page_num;
    let limit = 21;

    if let Some(category) = &request.category {
        query.push(format!(" category = '{}'", category.as_str()));
    };

    query.push(format!("LIMIT {} OFFSET {} ", limit, offset));
    sql.push_str(format!("WHERE {}", query.join(" AND ")).as_str());

    let games = sqlx::query_as::<_, GameBase>(&sql).fetch_all(pool).await?;

    let has_next = games.len() < 21;
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
        SET times_played = times_played + 1
        WHERE id = $1
        "#,
        game_type.to_string()
    );
    let row = sqlx::query(&query).bind(game_id).execute(pool).await?;

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

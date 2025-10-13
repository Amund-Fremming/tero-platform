use chrono::{Duration, Utc};
use sqlx::{Pool, Postgres};
use tracing::warn;
use uuid::Uuid;

use crate::{
    common::{db_query_builder::DBQueryBuilder, error::ServerError, models::PagedResponse},
    config::config::CONFIG,
    game::models::{GameBase, GamePageQuery, GameType, SavedGamePageQuery},
};

pub async fn delete_non_active_games(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    let timeout = Utc::now() - Duration::days(24);
    sqlx::query(
        r#"
        DELETE FROM "game_base"
        WHERE last_played < $1
        "#,
    )
    .bind(timeout)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_game_page(
    pool: &Pool<Postgres>,
    request: &GamePageQuery,
) -> Result<PagedResponse<GameBase>, sqlx::Error> {
    let page_size = CONFIG.server.page_size as u16;
    let games = DBQueryBuilder::select(
        r#"
        SELECT 
            id,
            name,
            description,
            game_type,
            category,
            iterations,
            times_played,
            last_played
            "#,
    )
    .from("game_base")
    .r#where("game_type", &request.game_type)
    .where_opt("category", &request.category)
    .offset(page_size * request.page_num)
    .limit(page_size + 1)
    .order_desc("times_played")
    .build()
    .build_query_as::<GameBase>()
    .fetch_all(pool)
    .await?;

    let has_next = games.len() < (page_size + 1) as usize;
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
    id: Uuid,
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

pub async fn save_game(
    pool: &Pool<Postgres>,
    game_type: &GameType,
    user_id: Uuid,
    base_id: Uuid,
) -> Result<(), ServerError> {
    let base_id_fut = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM "game_base"
        WHERE id $1
        "#,
    )
    .bind(&base_id)
    .fetch_one(pool);

    let query = format!(
        r#"
        SELECT id
        FROM {}
        WHERE id = $1
        "#,
        game_type
    );

    let game_id_fut = sqlx::query_scalar::<_, Uuid>(&query).fetch_one(pool);

    let (base_id, game_id): (Result<Uuid, sqlx::Error>, Result<Uuid, sqlx::Error>) =
        tokio::join!(base_id_fut, game_id_fut);

    let row = sqlx::query(
        r#"
        INSERT INTO "saved_game" (user_id, base_id, game_id, game_type)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(user_id)
    .bind(base_id?)
    .bind(game_id?)
    .bind(game_type)
    .execute(pool)
    .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal(
            "Failed to insert to table `saved_game`".into(),
        ));
    }

    Ok(())
}

pub async fn delete_saved_game(
    pool: &Pool<Postgres>,
    game_type: &GameType,
    user_id: Uuid,
    saved_id: Uuid,
) -> Result<(), ServerError> {
    let query = format!(
        r#"
        DELETE FROM {}
        WHERE user_id = $1 AND id = $2
        "#,
        game_type
    );

    let row = sqlx::query(&query)
        .bind(&user_id)
        .bind(&saved_id)
        .execute(pool)
        .await?;

    if row.rows_affected() == 0 {
        return Err(ServerError::Internal(
            "Failed to delete from table `saved_game`".into(),
        ));
    }

    Ok(())
}

pub async fn get_saved_games_page(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    query: SavedGamePageQuery,
) -> Result<PagedResponse<GameBase>, ServerError> {
    let page_size = CONFIG.server.page_size;
    let limit = page_size + 1;
    let offset = query.page_num * page_size;

    let query = format!(
        r#"
        SELECT
            base.id,
            base.name,
            base.description,
            base.game_type,
            base.category,
            base.iterations,
            base.times_played,
            base.last_played
        FROM "game_base" base
        JOIN "saved_game" saved
        ON base.id = saved.game_id
        WHERE saved.user_id = $1
        LIMIT {} OFFSET {}
        "#,
        limit, offset
    );

    let games = sqlx::query_as::<_, GameBase>(&query)
        .bind(&user_id)
        .fetch_all(pool)
        .await?;

    let has_next = games.len() < limit as usize;
    let page = PagedResponse::new(games, has_next);

    Ok(page)
}

use sqlx::{Pool, Postgres};

use crate::game::models::{GameBase, GameType, PagedRequest, PagedResponse};

pub async fn get_game_page(
    pool: &Pool<Postgres>,
    game_type: GameType,
    request: PagedRequest,
) -> Result<PagedResponse, sqlx::Error> {
    let mut sql = format!(
        r#"
        SELECT id, name, description, category, iterations, times_played
        FROM {}
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

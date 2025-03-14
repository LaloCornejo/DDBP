use axum::{
    extract::{Path, State},
    Json,
};

use sqlx::PgPool;
use uuid::Uuid;
use crate::db::models::{Post, Node};
use tracing::info;

pub async fn get_posts(
    State(pool): State<PgPool>,
) -> Json<Vec<Post>> {
    info!("Fetching posts");
    let posts = sqlx::query_as!(
        Post,
        "SELECT id, content, author, created_at, updated_at, origin_node 
FROM posts 
ORDER BY created_at DESC 
LIMIT 100"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(posts)
}

pub async fn get_post(
    Path(id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Json<Option<Post>> {
    info!("Fetching post with id: {}", id);
    let post = sqlx::query_as!(
        Post, 
        "SELECT id, content, author, created_at, updated_at, origin_node 
FROM posts 
WHERE id = $1",
        id
    )
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    Json(post)
}

pub async fn get_nodes(
    State(pool): State<PgPool>,
) -> Json<Vec<Node>> {
    info!("Fetching nodes");
    let nodes = sqlx::query_as!(
        Node,
        "SELECT id, url, last_seen FROM nodes"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    Json(nodes)
}

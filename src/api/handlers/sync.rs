use axum::{
    extract::State,
    Json,
};
use sqlx::PgPool;
use crate::db::models::Post;

pub async fn sync_data(
    State(pool): State<PgPool>,
    Json(posts): Json<Vec<Post>>,
) -> Json<serde_json::Value> {
    for post in posts {
        sqlx::query!(
            "INSERT INTO posts (id, content, author, created_at, updated_at, origin_node) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING",
            post.id,
            post.content,
            post.author,
            post.created_at,
            post.updated_at,
            post.origin_node,
        )
        .execute(&pool)
        .await
        .unwrap_or_default();
    }

    Json(serde_json::json!({ "status": "synced" }))
}

use axum::{
    extract::State,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    db::models::{CreatePostRequest, Post, Node},
    config::Config,
    cluster::replication,
};
use chrono::Utc;

pub async fn create_post(
    State(pool): State<PgPool>,
    Json(request): Json<CreatePostRequest>,
) -> Json<Post> {
    let config = Config::from_env().unwrap();

    let post = Post {
        id: Uuid::new_v4(),
        content: request.content,
        author: request.author,
        created_at: Utc::now(),
        updated_at: None,
        origin_node: config.node_id.clone(),
    };

    sqlx::query!(
        "INSERT INTO posts (id, content, author, created_at, updated_at, origin_node) VALUES ($1, $2, $3, $4, $5, $6)",
        post.id,
        post.content,
        post.author,
        post.created_at,
        post.updated_at,
        post.origin_node,
    )
    .execute(&pool)
    .await
    .unwrap();

    tokio::spawn(replication::async_post_to_nodes(post.clone(), config.cluster_nodes));

    Json(post)
}

pub async fn register_node(
    State(pool): State<PgPool>,
    Json(request): Json<Node>,
) -> Json<Node> {
    let node = Node {
        id: Uuid::new_v4().to_string(),
        url: request.url.clone(),
        last_seen: Utc::now(),
    };

    sqlx::query!(
        "INSERT INTO nodes (id, url, last_seen) VALUES ($1, $2, $3)",
        node.id,
        node.url,
        node.last_seen,
    )
    .execute(&pool)
    .await
    .unwrap();

    Json(node)
}

pub async fn sync_data(
    State(pool): State<PgPool>,
    Json(post): Json<Post>,
) -> Json<Post> {
    sqlx::query!(
        "INSERT INTO posts (id, content, author, created_at, updated_at, origin_node) VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO NOTHING",
        post.id,
        post.content,
        post.author,
        post.created_at,
        post.updated_at,
        post.origin_node,
    )
    .execute(&pool)
    .await
    .unwrap();

    Json(post)
}

use axum::extract::State;
use axum::Json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    db::models::{CreatePostRequest, Post, Node},
    config::Config,
    cluster::{replication, node::register_with_node},
};
use chrono::Utc;
use tracing::info;

pub async fn create_post(
    State(pool): State<PgPool>,
    Json(request): Json<CreatePostRequest>,
) -> Json<Post> {
    info!("Creating post with content: {} and author: {}", request.content, request.author);
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

    tokio::spawn(replication::sync_post_to_nodes(post.clone(), config.cluster_nodes.clone()));

    Json(post)
}

pub async fn register_node(
    State(pool): State<PgPool>,
    Json(request): Json<Node>,
) -> Json<Node> {
    info!("Registering node with URL: {}", request.url);

    let existing_node = sqlx::query_as!(
        Node,
        "SELECT id, url, last_seen FROM nodes WHERE url = $1",
        request.url
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    if let Some(mut node) = existing_node {
        info!("Node with URL: {} already exists with ID: {}. Updating last_seen timestamp.", node.url, node.id);

        node.last_seen = Utc::now();
        sqlx::query!(
            "UPDATE nodes SET last_seen = $1 WHERE id = $2",
            node.last_seen,
            node.id
        )
        .execute(&pool)
        .await
        .unwrap();

        return Json(node);
    }

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

    // Register with other nodes
    let config = Config::from_env().unwrap();
    for node_url in config.cluster_nodes {
        let node_id = node.id.clone();
        let node_url = node_url.to_owned();
        let node_url2 = node.url.clone();
        tokio::spawn(async move {
            if let Err(err) = register_with_node(&node_id, &node_url2, &node_url).await {
                info!("Failed to register with node: {}. Error: {:?}", node_url, err);
            }
        });
    }

    info!("Successfully registered node with ID: {} and URL: {}", node.id, node.url);

    Json(node)
}

pub async fn sync_data(
    State(pool): State<PgPool>,
    Json(post): Json<Post>,
) -> Json<Post> {
    info!("Syncing post with ID: {} to local database", post.id);
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

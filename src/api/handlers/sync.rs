use axum::{
    extract::State,
    Json,
};
use sqlx::PgPool;
use crate::db::models::Post;
use crate::config::Config;
use crate::cluster::replication::sync_post_to_nodes;
use tracing::info;

pub async fn sync_data(
    State(pool): State<PgPool>,
) -> Json<serde_json::Value> {
    info!("Syncing data to all nodes(sync.rs)");
    let config = Config::from_env().unwrap();
    let nodes = config.cluster_nodes.clone();

    let posts = sqlx::query_as!(Post, "SELECT * FROM posts")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    for post in posts {
        // Sync post to all nodes
        tokio::spawn(sync_post_to_nodes(post.clone(), nodes.clone()));
    }

    Json(serde_json::json!({ "status": "synced" }))
}

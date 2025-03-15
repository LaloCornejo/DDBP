use sqlx::PgPool;
use uuid::Uuid;
use crate::{
    db::models::Post,
    config::Config,
};
use futures::future;

pub async fn create_post_across_nodes(
    post: Post,
    config: &Config,
    pools: &[PgPool],
) -> Result<(), sqlx::Error> {
    let futures = pools.iter().map(|pool| {
        let post = post.clone();
        async move {
            sqlx::query!(
                "INSERT INTO posts (id, content, author, created_at, origin_node) 
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) DO NOTHING",
                post.id,
                post.content,
                post.author,
                post.created_at,
                post.origin_node
            )
                .execute(pool)
                .await
        }
    });

    // Execute insert on all database nodes in parallel
    let results = future::join_all(futures).await;

    // Check for errors
    for result in results {
        if let Err(e) = result {
            tracing::error!("Error syncing post to a node: {}", e);
            // We continue even if one node fails
        }
    }

    Ok(())
}

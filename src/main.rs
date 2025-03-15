mod config;
mod db;
mod api;
mod cluster;

use sqlx::PgPool;
use futures::future;
use crate::db::distributed_post::create_post_across_nodes;
use crate::db::migrations;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = config::Config::from_env()?;

    // Setup primary database connection for this node
    tracing::info!("Connecting to primary database at {}", config.database_url);
    let primary_pool = db::connect(&config.database_url).await?;

    // Run migrations on primary database
    migrations::run_migrations(&primary_pool).await?;

    // Connect to all database nodes in the cluster
    tracing::info!("Connecting to all {} database nodes in cluster", config.database_urls.len());
    let connect_futures = config.database_urls.iter().map(|url| db::connect(url));
    let all_pools: Vec<PgPool> = future::join_all(connect_futures)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();

    tracing::info!("Successfully connected to {} database nodes", all_pools.len());

    // Store the connection pools for future use (optional)
    // You could store these in a global variable or pass them to your API handlers

    // Start API server
    let app = api::create_router(primary_pool);

    // Start the server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Starting API server on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

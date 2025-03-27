mod config;
mod db;
mod api;
mod cluster;

use sqlx::PgPool;
use crate::config::Config;
use crate::db::{run_migrations, connection::create_pool};
use crate::cluster::discovery::start_discovery_service;
use futures::future;
use tracing::info;
use dotenv::dotenv;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables
    dotenv().ok();
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create connection pool
    let pool = create_pool(&config).await?;

    // Run migrations
    run_migrations(&pool).await?;

    // Start the node discovery service
    tokio::spawn(start_discovery_service(config.clone()));

    info!("Connecting to primary database at {}", config.database_url);

    let primary_pool = db::connect(&config.database_url).await?;

    // Run migrations on primary database
    db::migrations::run_migrations(&primary_pool).await?;

    // Connect to all database nodes in the cluster
    info!("Connecting to all {} database nodes in cluster", config.database_urls.len());
    let connect_futures = config.database_urls.iter().map(|url| db::connect(url));
    let all_pools: Vec<PgPool> = future::join_all(connect_futures)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();

    info!("Successfully connected to {} database nodes", all_pools.len());

    // Start API server
    let app = api::create_router(primary_pool);

    // Start the server
    let addr = format!("{}:{}", config.host, config.port);
    info!("Starting API server on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

mod config;
mod db;
mod api;
mod cluster;

use sqlx::PgPool;
use crate::config::Config;
use crate::db::run_migrations;
use crate::db::connection::create_pool;
use crate::cluster::discovery::start_discovery_service;
use futures::future;
use crate::db::migrations;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env().unwrap();   // Load environment variables
    dotenv::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create connection pool
    let pool = create_pool(&config).await?;
    
    // Run migrations
    run_migrations(&pool).await?;

    // Start the node discovery service
    tokio::spawn(start_discovery_service(config.clone()));   // Setup primary database connection for this node

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

mod config;
mod db;
mod api;
mod cluster;

#[tokio::main]
async fn main() -> Result<(),  Box<dyn std::error::Error>> {
    // Load env 
    dotenv::dotenv().ok();

    // Init loging 
    tracing_subscriber::fmt::init();

    // Load config
    let config = config::Config::from_env()?;

    // Setup db connection
    let db_pool = db::connect(&config.database_url).await?;

    //Run migrations
    db::run_migrations(&db_pool).await?;

    //Start API server
    let app = api::create_router(db_pool);

    // Start the server 
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Starting server at http://{}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

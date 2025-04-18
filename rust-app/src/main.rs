use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use mongodb::{
    options::{
        AuthMechanism, ClientOptions, Credential, ResolverConfig, RetryClientOptions, RetryPolicy,
        ServerAddress,
    },
    Client,
};
use std::{env, time::Duration};
use tracing::{info, Level};
use tracing_subscriber;

mod errors;
mod handlers;
mod models;
mod state;

use handlers::*;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting application");

    let mongo_uri = env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://admin:password@localhost:27017/social_media_db".to_string());

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    // Configure MongoDB client options with better defaults for reliability
    let mut client_options = ClientOptions::parse(mongo_uri).await.unwrap();

    // Configure connection pooling
    client_options.max_pool_size = Some(20);
    client_options.min_pool_size = Some(5);
    client_options.max_idle_time = Some(Duration::from_secs(60));

    // Configure timeouts
    client_options.connect_timeout = Some(Duration::from_secs(10));
    client_options.server_selection_timeout = Some(Duration::from_secs(15));
    // client_options.socket_timeout = Some(Duration::from_secs(30));

    // Configure retry options
    let retry_options = RetryClientOptions::builder()
        .with_max_time(Duration::from_secs(30))
        .with_max_retries(3)
        .with_retry_reads(true)
        .with_retry_writes(true)
        .build();
    client_options.retry = Some(retry_options);

    // Configure heartbeat to detect server issues quickly
    client_options.heartbeat_freq = Some(Duration::from_secs(15));

    info!("Connecting to MongoDB");
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("social_media_db");

    let app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check_handler))
            .route("/create_user", web::post().to(create_user_handler))
            .route("/create_post", web::post().to(create_post_handler))
    })
    .bind((host, port))?
    .run()
    .await
}

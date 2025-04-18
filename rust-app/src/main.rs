use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client};
use std::env;
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

    let client_options = ClientOptions::parse(mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("social_media_db");

    let app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check_handler))
            .route("/create_user", web::post().to(create_user_handler))
            .route("/create_post", web::post().to(create_post_handler))
        // Add other routes here
    })
    .bind((host, port))?
    .run()
    .await
}

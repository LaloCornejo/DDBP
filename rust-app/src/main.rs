use crate::handlers::{
    clean_database_handler, create_comment_handler, create_post_handler, create_user_handler,
    follow_user_handler, health_check_handler, populate_database_handler,
};
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use dotenv::dotenv;
use mongodb::{
    options::{ClientOptions, ReadConcern, ReadPreference, ReadPreferenceOptions, WriteConcern},
    Client,
};
use std::{env, time::Duration};
use tracing::{info, Level};
use tracing_subscriber;

mod errors;
mod handlers;
mod models;
mod state;

// use handlers::*;
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
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);

    // Configure MongoDB client options with better defaults for reliability
    let mut client_options = ClientOptions::parse(mongo_uri).await.unwrap();

    // Configure connection pooling
    client_options.max_pool_size = Some(20);
    client_options.min_pool_size = Some(5);
    client_options.max_idle_time = Some(Duration::from_secs(60));

    // Configure timeouts
    client_options.connect_timeout = Some(Duration::from_secs(10));
    client_options.server_selection_timeout = Some(Duration::from_secs(15));

    // Configure read/write concerns for better reliability
    client_options.read_concern = Some(ReadConcern::majority());
    client_options.write_concern = Some(
        WriteConcern::builder()
            .w(mongodb::options::Acknowledgment::Majority)
            .build(),
    );

    // Set read preference to SecondaryPreferred
    let options = ReadPreferenceOptions::default();
    client_options.selection_criteria = Some(
        ReadPreference::SecondaryPreferred {
            options: Some(options),
        }
        .into(),
    );

    // Configure retry behavior with available options
    client_options.retry_reads = Some(true);
    client_options.retry_writes = Some(true);

    // Configure heartbeat to detect server issues quickly
    client_options.heartbeat_freq = Some(Duration::from_secs(15));

    info!("Connecting to MongoDB with enhanced configuration");
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("social_media_db");
    let _app_state = web::Data::new(AppState { db });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization", "Accept", "Origin", "X-Requested-With", "Access-Control-Request-Method", "Access-Control-Request-Headers"])
                    .expose_headers(vec!["Content-Length", "Content-Type"])
                    .supports_credentials()
                    .max_age(3600)
            )
            .app_data(_app_state.clone())
            .route("/api/posts", web::get().to(handlers::get_posts_handler))
            .route(
                "/api/posts/{id}",
                web::get().to(handlers::get_post_by_id_handler),
            )
            .route("/api/users", web::get().to(handlers::get_users_handler))
            .route(
                "/api/users/{id}",
                web::get().to(handlers::get_user_by_id_handler),
            )
            .route(
                "/api/comments",
                web::get().to(handlers::get_comments_handler),
            )
            .route(
                "/api/comments/{id}",
                web::get().to(handlers::get_comment_by_id_handler),
            )
            .route(
                "/api/comments/post/{post_id}",
                web::get().to(handlers::get_comments_by_post_id_handler),
            )
            .route(
                "/api/comments/user/{user_id}",
                web::get().to(handlers::get_comments_by_user_id_handler),
            )
            .route(
                "/api/users/following/{user_id}",
                web::get().to(handlers::get_following_users_handler),
            )
            .route(
                "/api/users/followers/{user_id}",
                web::get().to(handlers::get_followers_users_handler),
            )
            .route(
                "/api/users/posts/{user_id}",
                web::get().to(handlers::get_posts_by_user_id_handler),
            )
            .route(
                "/api/posts/comments/{post_id}",
                web::get().to(handlers::get_comments_by_post_id_handler),
            )
            .route("/api/create_user", web::post().to(create_user_handler))
            .route("/api/create_post", web::post().to(create_post_handler))
            .route(
                "/api/create_comment",
                web::post().to(create_comment_handler),
            )
            .route("/api/follow_user", web::post().to(follow_user_handler))
            .route("/api/health", web::get().to(health_check_handler))
            .route(
                "/api/test/populate",
                web::post().to(populate_database_handler),
            )
            .route("/api/test/clean", web::post().to(clean_database_handler))
    })
    .bind((host, port))?
    .run()
    .await
}

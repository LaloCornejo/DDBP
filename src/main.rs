use actix_web::{App, HttpResponse, HttpServer, Responder, Result, web};
use mongodb::{Client, bson::Document, bson::doc, options::ClientOptions};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use tracing::{Level, error, info};
use tracing_subscriber;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    email: String,
    password_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    user_id: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Comment {
    post_id: String,
    user_id: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Follow {
    follower_id: String,
    following_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    message: String,
}

struct AppState {
    db: mongodb::Database,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting application...");
    info!("Connecting to MongoDB replica set...");

    let mut client_options = match ClientOptions::parse(
        "mongodb://admin:password@127.0.0.1:27017/?replicaSet=rs0",
    )
    .await
    {
        Ok(options) => {
            info!("Successfully parsed MongoDB connection string");
            options
        }
        Err(e) => {
            error!("Failed to parse MongoDB connection string: {}", e);
            std::process::exit(1);
        }
    };

    client_options.server_selection_timeout = Some(Duration::from_secs(30));
    client_options.connect_timeout = Some(Duration::from_secs(20));
    client_options.retry_writes = Some(true);
    client_options.retry_reads = Some(true);
    client_options.app_name = Some("social-media-app".to_string());

    let client = match Client::with_options(client_options) {
        Ok(client) => {
            info!("Successfully created MongoDB client");
            client
        }
        Err(e) => {
            error!("Failed to create MongoDB client: {}", e);
            std::process::exit(1);
        }
    };

    info!("Testing MongoDB connection with ping...");
    match client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await
    {
        Ok(_) => info!("Successfully connected to MongoDB replica set"),
        Err(e) => {
            error!("Failed to ping MongoDB: {}", e);
            error!("Warning: Continuing despite ping failure. Check MongoDB connectivity.");
        }
    };

    let db = client.database("social_media_db");

    info!("Testing access to social_media_db...");
    match db.list_collection_names(None).await {
        Ok(collections) => info!(
            "Successfully accessed social_media_db. Collections: {:?}",
            collections
        ),
        Err(e) => {
            error!(
                "Warning: Failed to list collections in social_media_db: {}",
                e
            );
            error!("Will attempt to create collections on first use");
        }
    };

    let app_state = web::Data::new(AppState { db });

    info!("Server starting at http://127.0.0.1:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/create_user", web::post().to(create_user_handler))
            .route("/create_post", web::post().to(create_post_handler))
            .route("/create_comment", web::post().to(create_comment_handler))
            .route("/follow_user", web::post().to(follow_user_handler))
            .route("/get_post/{id}", web::get().to(get_post_handler))
            .route("/health", web::get().to(health_check_handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

async fn create_user_handler(
    user: web::Json<User>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let collection = state.db.collection::<Document>("users");

    let user_id = Uuid::new_v4().to_string();
    let user_doc = doc! {
        "_id": &user_id,
        "username": &user.username,
        "email": &user.email,
        "password_hash": &user.password_hash,
        "created_at": chrono::Utc::now().to_rfc3339()
    };

    match collection.insert_one(user_doc, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(Response {
            message: format!("User created successfully with ID: {}", user_id),
        })),
        Err(e) => {
            error!("Error creating user: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response {
                message: "Failed to create user".to_string(),
            }))
        }
    }
}

async fn create_post_handler(
    post: web::Json<Post>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let collection = state.db.collection::<Document>("posts");

    let post_id = Uuid::new_v4().to_string();
    let post_doc = doc! {
        "_id": &post_id,
        "user_id": &post.user_id,
        "content": &post.content,
        "created_at": chrono::Utc::now().to_rfc3339()
    };

    match collection.insert_one(post_doc, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(Response {
            message: format!("Post created successfully with ID: {}", post_id),
        })),
        Err(e) => {
            error!("Error creating post: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response {
                message: "Failed to create post".to_string(),
            }))
        }
    }
}

async fn create_comment_handler(
    comment: web::Json<Comment>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let collection = state.db.collection::<Document>("comments");

    let comment_id = Uuid::new_v4().to_string();
    let comment_doc = doc! {
        "_id": &comment_id,
        "post_id": &comment.post_id,
        "user_id": &comment.user_id,
        "content": &comment.content,
        "created_at": chrono::Utc::now().to_rfc3339()
    };

    match collection.insert_one(comment_doc, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(Response {
            message: format!("Comment created successfully with ID: {}", comment_id),
        })),
        Err(e) => {
            error!("Error creating comment: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response {
                message: "Failed to create comment".to_string(),
            }))
        }
    }
}

async fn follow_user_handler(
    follow: web::Json<Follow>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let collection = state.db.collection::<Document>("follows");

    let follow_doc = doc! {
        "follower_id": &follow.follower_id,
        "following_id": &follow.following_id,
        "created_at": chrono::Utc::now().to_rfc3339()
    };

    match collection.insert_one(follow_doc, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(Response {
            message: "Follow action recorded successfully".to_string(),
        })),
        Err(e) => {
            error!("Error recording follow action: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response {
                message: "Failed to record follow action".to_string(),
            }))
        }
    }
}

async fn get_post_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let id = path.into_inner();
    println!("Retrieving post with ID: {}", id);

    let collection = state.db.collection::<Document>("posts");

    let filter = doc! { "_id": id.clone() };

    match collection.find_one(filter, None).await {
        Ok(Some(doc)) => {
            println!("Post found: {}", doc.get_str("content").unwrap_or_default());
            let post = Post {
                user_id: doc.get_str("user_id").unwrap_or_default().to_string(),
                content: doc.get_str("content").unwrap_or_default().to_string(),
            };
            Ok(HttpResponse::Ok().json(post))
        }
        Ok(None) => {
            println!("Post not found with ID: {}", id);
            Ok(HttpResponse::NotFound().json(Response {
                message: format!("Post not found with ID: {}", id),
            }))
        }
        Err(e) => {
            eprintln!("Error retrieving post: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response {
                message: format!("Failed to retrieve post: {}", e),
            }))
        }
    }
}

// Health check endpoint
async fn health_check_handler(state: web::Data<AppState>) -> Result<impl Responder> {
    match state.db.run_command(doc! {"ping": 1}, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(Response {
            message: "Service is healthy".to_string(),
        })),
        Err(e) => {
            error!("Health check failed: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(Response {
                message: "Service is unhealthy".to_string(),
            }))
        }
    }
}

use actix_web::{error::ResponseError, web, App, HttpResponse, HttpServer, Responder, Result};
use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use dotenv::dotenv;
use futures_util::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOptions},
    Client,
};
use num_cpus;
use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

use tracing::{debug, error, info, warn, Level};
use tracing_subscriber;
#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    email: String,
    password_hash: String,
    #[serde(default)]
    bio: Option<String>,
    #[serde(default)]
    profile_picture_url: Option<String>,
    #[serde(skip_deserializing)]
    join_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum PostType {
    Text,
    Image,
    Video,
    Link,
}

impl Default for PostType {
    fn default() -> Self {
        PostType::Text
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    user_id: String,
    content: String,
    #[serde(default)]
    media_urls: Vec<String>,
    #[serde(default)]
    post_type: PostType,
    #[serde(default)]
    like_count: i32,
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
struct Like {
    post_id: String,
    user_id: String,
    created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PaginationParams {
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserProfile {
    id: String,
    username: String,
    bio: Option<String>,
    profile_picture_url: Option<String>,
    join_date: String,
    follower_count: i32,
    following_count: i32,
    post_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct PostDetails {
    id: String,
    user_id: String,
    username: String,
    profile_picture_url: Option<String>,
    content: String,
    media_urls: Vec<String>,
    post_type: PostType,
    created_at: String,
    human_time: String,
    like_count: i32,
    comment_count: i32,
    has_liked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct TimelineResponse {
    posts: Vec<PostDetails>,
    pagination: PaginationMeta,
}

#[derive(Serialize, Deserialize, Debug)]
struct PaginationMeta {
    current_page: u32,
    total_pages: u32,
    total_count: u64,
    has_next: bool,
    has_prev: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserStats {
    post_count: i32,
    comment_count: i32,
    follower_count: i32,
    following_count: i32,
    total_likes_received: i32,
    total_likes_given: i32,
}
#[derive(Serialize, Deserialize, Debug)]
struct Response<T = String> {
    status: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

// Custom error enum for the application
#[derive(Debug)]
enum AppError {
    MongoError(mongodb::error::Error),
    NotFound(String),
    InvalidInput(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MongoError(e) => write!(f, "Database error: {}", e),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::MongoError(e) => {
                error!("Database error: {}", e);
                HttpResponse::InternalServerError().json(Response::<()> {
                    status: "error".to_string(),
                    message: "A database error occurred".to_string(),
                    data: None,
                })
            }
            AppError::NotFound(msg) => {
                info!("Not found: {}", msg);
                HttpResponse::NotFound().json(Response::<()> {
                    status: "error".to_string(),
                    message: msg.clone(),
                    data: None,
                })
            }
            AppError::InvalidInput(msg) => {
                info!("Invalid input: {}", msg);
                HttpResponse::BadRequest().json(Response::<()> {
                    status: "error".to_string(),
                    message: msg.clone(),
                    data: None,
                })
            }
        }
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(error: mongodb::error::Error) -> Self {
        AppError::MongoError(error)
    }
}

// Application state to be shared across handlers
struct AppState {
    db: mongodb::Database,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Initialize the logger
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting social media API application");

    // Get MongoDB connection string from environment variable or use default
    let mongo_uri = env::var("MONGO_URI")
        .unwrap_or_else(|_| {
            warn!("MONGO_URI environment variable not set, using default value");
            "mongodb://admin:password@central-mongodb:27017,secondary-mongodb-1:27017,secondary-mongodb-2:27017/social_media_db?replicaSet=rs0&authSource=social_media_db".to_string()
        });

    info!(
        "Connecting to MongoDB at: {}",
        mongo_uri.replace("admin:password", "admin:****")
    );

    // Configure MongoDB client options
    let mut client_options = match ClientOptions::parse(&mongo_uri).await {
        Ok(options) => {
            info!("MongoDB connection string parsed successfully");
            options
        }
        Err(e) => {
            error!("Failed to parse MongoDB connection string: {}", e);
            panic!("Cannot start application without database connection");
        }
    };

    // Set MongoDB client timeout options
    client_options.connect_timeout = Some(Duration::from_secs(30));
    client_options.server_selection_timeout = Some(Duration::from_secs(30));
    client_options.app_name = Some("social_media_api".to_string());

    // Create MongoDB client
    let client = match Client::with_options(client_options) {
        Ok(client) => {
            info!("Successfully created MongoDB client");
            client
        }
        Err(e) => {
            error!("Failed to create MongoDB client: {}", e);
            panic!("Cannot start application without database connection");
        }
    };

    // Test MongoDB connection with ping
    info!("Testing MongoDB connection with ping...");
    let ping_result = tokio::time::timeout(
        Duration::from_secs(10),
        client.database("admin").run_command(doc! {"ping": 1}),
    )
    .await;

    // Handle ping result
    match ping_result {
        Ok(Ok(_)) => info!("Successfully connected to MongoDB replica set"),
        Ok(Err(e)) => {
            error!("Failed to ping MongoDB: {}", e);
            info!("Waiting for 5 seconds before retrying...");
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Try a second time
            info!("Attempting second ping with timeout...");
            match tokio::time::timeout(
                Duration::from_secs(10),
                client.database("admin").run_command(doc! {"ping": 1}),
            )
            .await
            {
                Ok(Ok(_)) => info!("Second ping attempt successful!"),
                Ok(Err(e2)) => {
                    error!("Second ping attempt also failed: {}", e2);
                    warn!("Will continue despite connection issues");
                }
                Err(_) => {
                    error!("Second ping attempt timed out after 10 seconds");
                    warn!("Will continue despite timeout");
                }
            }
        }
        Err(_) => {
            error!("Ping operation timed out after 10 seconds");
            warn!("Will continue despite timeout");
        }
    };

    // Access the database and verify we can list collections
    let db = client.database("social_media_db");

    info!("Testing access to social_media_db...");
    match db.list_collection_names().await {
        Ok(collections) => {
            info!(
                "Successfully accessed social_media_db. Collections: {:?}",
                collections
            );
        }
        Err(e) => {
            warn!("Failed to list collections in social_media_db: {}", e);
            info!("Will attempt to create collections on first use");
        }
    }

    // Create app state with database reference
    let app_state = web::Data::new(AppState { db });

    // Get host and port from environment or use defaults
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_address = format!("{}:{}", host, port);

    info!("Server starting at http://{}", bind_address);

    // Configure and start HTTP server
    // Configure and start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/create_user", web::post().to(create_user_handler))
            .route("/create_post", web::post().to(create_post_handler))
            .route("/create_comment", web::post().to(create_comment_handler))
            .route("/follow_user", web::post().to(follow_user_handler))
            .route("/like_post", web::post().to(like_post_handler))
            .route("/get_post/{id}", web::get().to(get_post_handler))
            .route("/get_user/{id}", web::get().to(get_user_profile_handler))
            .route("/user/{id}/timeline", web::get().to(get_user_timeline_handler))
            .route("/user/{id}/stats", web::get().to(get_user_stats_handler))
            .route("/posts/trending", web::get().to(get_trending_posts_handler))
            .route("/test/populate", web::post().to(populate_test_data_handler))
            .route("/test/clear", web::post().to(clear_test_data_handler))
            .route("/health", web::get().to(health_check_handler))
    .bind(&bind_address)?
    .workers(num_cpus::get()) // Use optimal number of workers based on available CPUs
    .shutdown_timeout(30) // Set shutdown timeout to 30 seconds
    .run()
    .await
}
async fn clear_test_data_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Clearing all test data");
    
    // Verify database connection before proceeding
    match state.db.run_command(doc! {"ping": 1}).await {
        Ok(_) => info!("Database connection verified, proceeding with test data clearing"),
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(AppError::MongoError(e));
        }
    }
    
    let collections = vec!["users", "posts", "comments", "follows", "likes"];
    for collection in collections.iter() {
        if let Err(e) = state.db.collection::<Document>(collection).drop().await {
            error!("Error dropping collection {}: {}", collection, e);
            return Err(AppError::from(e));
        }
    }
    
    info!("Test data cleared successfully");
    Ok(HttpResponse::Ok().json(Response::<()> {
        status: "success".to_string(),
        message: "All test data cleared successfully".to_string(),
        data: None,
    }))
}

async fn create_user_handler(
    user: web::Json<User>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Creating new user with username: {}", user.username);

    // Validate user input
    if user.username.is_empty() {
        return Err(AppError::InvalidInput(
            "Username cannot be empty".to_string(),
        ));
    }

    if user.email.is_empty() || !user.email.contains('@') {
        return Err(AppError::InvalidInput("Invalid email format".to_string()));
    }

    if user.password_hash.is_empty() {
        return Err(AppError::InvalidInput(
            "Password hash cannot be empty".to_string(),
        ));
    }

    let collection = state.db.collection::<Document>("users");

    // Check if user with this email already exists
    let existing_filter = doc! { "email": &user.email };
    if let Ok(Some(_)) = collection.find_one(existing_filter).await {
        return Err(AppError::InvalidInput(format!(
            "User with email {} already exists",
            user.email
        )));
    }

    let user_id = Uuid::new_v4().to_string();
    let user_doc = doc! {
        "_id": &user_id,
        "username": &user.username,
        "email": &user.email,
        "password_hash": &user.password_hash,
        "created_at": chrono::Utc::now().to_rfc3339()
    };

    match collection.insert_one(user_doc).await {
        Ok(_) => {
            info!("User created successfully with ID: {}", user_id);
            Ok(HttpResponse::Created().json(Response {
                status: "success".to_string(),
                message: format!("User created successfully with ID: {}", user_id),
                data: Some(user_id),
            }))
        }
        Err(e) => {
            error!("Error creating user: {}", e);
            Err(AppError::from(e))
        }
    }
}
async fn create_post_handler(
    post: web::Json<Post>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Creating new post for user: {}", post.user_id);

    // Validate post input
    if post.user_id.is_empty() {
        return Err(AppError::InvalidInput(
            "User ID cannot be empty".to_string(),
        ));
    }

    if post.content.is_empty() {
        return Err(AppError::InvalidInput(
            "Post content cannot be empty".to_string(),
        ));
    }

    // Verify user exists
    let users_collection = state.db.collection::<Document>("users");
    let user_filter = doc! { "_id": &post.user_id };

    if let Ok(None) = users_collection.find_one(user_filter).await {
        return Err(AppError::NotFound(format!(
            "User with ID {} not found",
            post.user_id
        )));
    }
    let collection = state.db.collection::<Document>("posts");

    let post_id = Uuid::new_v4().to_string();
    let post_doc = doc! {
        "_id": &post_id,
        "user_id": &post.user_id,
        "content": &post.content,
        "media_urls": &post.media_urls,
        "post_type": match post.post_type {
            PostType::Text => "Text",
            PostType::Image => "Image",
            PostType::Video => "Video",
            PostType::Link => "Link",
        },
        "like_count": 0,
        "created_at": chrono::Utc::now().to_rfc3339()
    };
    };

    match collection.insert_one(post_doc).await {
        Ok(_) => {
            info!("Post created successfully with ID: {}", post_id);
            Ok(HttpResponse::Created().json(Response {
                status: "success".to_string(),
                message: format!("Post created successfully with ID: {}", post_id),
                data: Some(post_id),
            }))
        }
        Err(e) => {
            error!("Error creating post: {}", e);
            Err(AppError::from(e))
        }
    }
}
async fn create_comment_handler(
    comment: web::Json<Comment>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!(
        "Creating new comment on post: {} by user: {}",
        comment.post_id, comment.user_id
    );

    // Validate comment input
    if comment.post_id.is_empty() {
        return Err(AppError::InvalidInput(
            "Post ID cannot be empty".to_string(),
        ));
    }

    if comment.user_id.is_empty() {
        return Err(AppError::InvalidInput(
            "User ID cannot be empty".to_string(),
        ));
    }

    if comment.content.is_empty() {
        return Err(AppError::InvalidInput(
            "Comment content cannot be empty".to_string(),
        ));
    }

    // Verify post exists
    let posts_collection = state.db.collection::<Document>("posts");
    let post_filter = doc! { "_id": &comment.post_id };

    if let Ok(None) = posts_collection.find_one(post_filter).await {
        return Err(AppError::NotFound(format!(
            "Post with ID {} not found",
            comment.post_id
        )));
    }

    // Verify user exists
    let users_collection = state.db.collection::<Document>("users");
    let user_filter = doc! { "_id": &comment.user_id };

    if let Ok(None) = users_collection.find_one(user_filter).await {
        return Err(AppError::NotFound(format!(
            "User with ID {} not found",
            comment.user_id
        )));
    }

    let collection = state.db.collection::<Document>("comments");

    let comment_id = Uuid::new_v4().to_string();
    let comment_doc = doc! {
        "_id": &comment_id,
        "post_id": &comment.post_id,
        "user_id": &comment.user_id,
        "content": &comment.content,
        "created_at": chrono::Utc::now().to_rfc3339()
    };

    match collection.insert_one(comment_doc).await {
        Ok(_) => {
            info!("Comment created successfully with ID: {}", comment_id);
            Ok(HttpResponse::Created().json(Response {
                status: "success".to_string(),
                message: format!("Comment created successfully with ID: {}", comment_id),
                data: Some(comment_id),
            }))
        }
        Err(e) => {
            error!("Error creating comment: {}", e);
            Err(AppError::from(e))
        }
    }
}

async fn follow_user_handler(
    follow: web::Json<Follow>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!(
        "Recording follow action: {} is following {}",
        follow.follower_id, follow.following_id
    );

    // Validate follow input
    if follow.follower_id.is_empty() {
        return Err(AppError::InvalidInput(
            "Follower ID cannot be empty".to_string(),
        ));
    }

    if follow.following_id.is_empty() {
        return Err(AppError::InvalidInput(
            "Following ID cannot be empty".to_string(),
        ));
    }

    if follow.follower_id == follow.following_id {
        return Err(AppError::InvalidInput(
            "Users cannot follow themselves".to_string(),
        ));
    }

    // Verify both users exist
    let users_collection = state.db.collection::<Document>("users");

    // Check follower exists
    let follower_filter = doc! { "_id": &follow.follower_id };
    if let Ok(None) = users_collection.find_one(follower_filter).await {
        return Err(AppError::NotFound(format!(
            "Follower with ID {} not found",
            follow.follower_id
        )));
    }

    // Check following user exists
    let following_filter = doc! { "_id": &follow.following_id };
    if let Ok(None) = users_collection.find_one(following_filter).await {
        return Err(AppError::NotFound(format!(
            "User to follow with ID {} not found",
            follow.following_id
        )));
    }

    // Check if already following
    let follows_collection = state.db.collection::<Document>("follows");
    let existing_follow_filter = doc! {
        "follower_id": &follow.follower_id,
        "following_id": &follow.following_id
    };

    if let Ok(Some(_)) = follows_collection.find_one(existing_follow_filter).await {
        return Err(AppError::InvalidInput(format!(
            "User {} already follows user {}",
            follow.follower_id, follow.following_id
        )));
    }

    let follow_doc = doc! {
        "follower_id": &follow.follower_id,
        "following_id": &follow.following_id,
        "created_at": chrono::Utc::now().to_rfc3339()
    };

    match follows_collection.insert_one(follow_doc).await {
        Ok(_) => {
            info!("Follow action recorded successfully");
            Ok(HttpResponse::Created().json(Response::<()> {
                status: "success".to_string(),
                message: "Follow action recorded successfully".to_string(),
                data: None,
            }))
        }
        Err(e) => {
            error!("Error recording follow action: {}", e);
            Err(AppError::from(e))
        }
    }
}
async fn get_post_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    info!("Retrieving post with ID: {}", id);

    // Validate post ID
    if id.is_empty() {
        return Err(AppError::InvalidInput(
            "Post ID cannot be empty".to_string(),
        ));
    }

    // Check if ID follows UUID format
    if uuid::Uuid::parse_str(&id).is_err() {
        return Err(AppError::InvalidInput(format!(
            "Invalid post ID format: {}",
            id
        )));
    }

    let posts_collection = state.db.collection::<Document>("posts");
    let filter = doc! { "_id": id.clone() };

    match posts_collection.find_one(filter).await {
        Ok(Some(post_doc)) => {
            // Extract post data with proper error handling
            let created_at = match post_doc.get_str("created_at") {
                Ok(val) => val.to_string(),
                Err(_) => {
                    warn!("Missing created_at field in post {}", id);
                    Utc::now().to_rfc3339()
                }
            };

            let user_id = match post_doc.get_str("user_id") {
                Ok(val) => val.to_string(),
                Err(_) => {
                    warn!("Missing user_id field in post {}", id);
                    "unknown".to_string()
                }
            };

            let content = match post_doc.get_str("content") {
                Ok(val) => val.to_string(),
                Err(_) => {
                    warn!("Missing content field in post {}", id);
                    "".to_string()
                }
            };

            info!("Post found: ID {} by user {}", id, user_id);

            // Define post response structure
            #[derive(Serialize)]
            struct PostResponse {
                id: String,
                user_id: String,
                content: String,
                created_at: String,
            }

            let post_response = PostResponse {
                id: id.clone(),
                user_id,
                content,
                created_at,
            };

            // Fetch comments for this post
            let comments_collection = state.db.collection::<Document>("comments");
            let comments_filter = doc! { "post_id": id.clone() };

            #[derive(Serialize)]
            struct CommentResponse {
                id: String,
                user_id: String,
                content: String,
                created_at: String,
            }

            #[derive(Serialize)]
            struct PostWithComments {
                post: PostResponse,
                comments: Vec<CommentResponse>,
            }

            let mut comments = Vec::new();

            // Attempt to fetch comments, but don't fail if we can't get them
            if let Ok(mut cursor) = comments_collection.find(comments_filter).await {
                while let Some(comment_doc_result) = cursor.next().await {
                    if let Ok(comment_doc) = comment_doc_result {
                        let comment_id = comment_doc.get_str("_id").unwrap_or_default().to_string();
                        let comment_user_id = comment_doc
                            .get_str("user_id")
                            .unwrap_or_default()
                            .to_string();
                        let comment_content = comment_doc
                            .get_str("content")
                            .unwrap_or_default()
                            .to_string();
                        let comment_created_at = comment_doc
                            .get_str("created_at")
                            .unwrap_or_default()
                            .to_string();

                        comments.push(CommentResponse {
                            id: comment_id,
                            user_id: comment_user_id,
                            content: comment_content,
                            created_at: comment_created_at,
                        });
                    }
                }
                debug!("Fetched {} comments for post {}", comments.len(), id);
            } else {
                warn!("Failed to fetch comments for post {}", id);
            }
            // Create the complete response with post and comments
            let full_response = PostWithComments {
                post: post_response,
                comments,
            };

            Ok(HttpResponse::Ok().json(Response {
                status: "success".to_string(),
                message: "Post retrieved successfully".to_string(),
                data: Some(full_response),
            }))
        }
        Ok(None) => {
            info!("Post not found with ID: {}", id);
            Err(AppError::NotFound(format!("Post with ID {} not found", id)))
        }
        Err(e) => {
            error!("Error retrieving post: {}", e);
            Err(AppError::from(e))
        }
    }
}

// Health check endpoint with detailed logging
async fn health_check_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Health check requested");

    let timeout_future = tokio::time::timeout(
        Duration::from_secs(5),
        state.db.run_command(doc! {"ping": 1}),
    )
    .await;

    match timeout_future {
        Ok(Ok(_)) => {
            info!("Health check successful");
            Ok(HttpResponse::Ok().json(Response::<()> {
                status: "success".to_string(),
                message: "Service is healthy".to_string(),
                data: None,
            }))
        }
        Ok(Err(e)) => {
            error!("Health check failed: {}", e);
            // Return a service unavailable response but with our standard error format
            Ok(HttpResponse::ServiceUnavailable().json(Response::<()> {
                status: "error".to_string(),
                message: format!("Service is unhealthy: database connection failed"),
                data: None,
            }))
        }
        Err(_) => {
            error!("Health check timed out after 5 seconds");
            Ok(HttpResponse::ServiceUnavailable().json(Response::<()> {
                status: "error".to_string(),
                message: "Service is unhealthy: database connection timeout".to_string(),
                data: None,
            }))
        }
    }
}

async fn like_post_handler(
    like: web::Json<Like>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Recording like for post: {} by user: {}", like.post_id, like.user_id);

    // Validate like input
    if like.post_id.is_empty() {
        return Err(AppError::InvalidInput(
            "Post ID cannot be empty".to_string(),
        ));
    }

    if like.user_id.is_empty() {
        return Err(AppError::InvalidInput(
            "User ID cannot be empty".to_string(),
        ));
    }

    // Verify post exists
    let posts_collection = state.db.collection::<Document>("posts");
    let post_filter = doc! { "_id": &like.post_id };

    let post_doc = match posts_collection.find_one(post_filter.clone()).await {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            return Err(AppError::NotFound(format!(
                "Post with ID {} not found",
                like.post_id
            )));
        }
        Err(e) => {
            error!("Error finding post: {}", e);
            return Err(AppError::from(e));
        }
    };

    // Verify user exists
    let users_collection = state.db.collection::<Document>("users");
    let user_filter = doc! { "_id": &like.user_id };

    if let Ok(None) = users_collection.find_one(user_filter).await {
        return Err(AppError::NotFound(format!(
            "User with ID {} not found",
            like.user_id
        )));
    }

    // Check if already liked
    let likes_collection = state.db.collection::<Document>("likes");
    let existing_like_filter = doc! {
        "post_id": &like.post_id,
        "user_id": &like.user_id
    };

    if let Ok(Some(_)) = likes_collection.find_one(existing_like_filter.clone()).await {
        // Unlike the post by removing the like
        match likes_collection.delete_one(existing_like_filter).await {
            Ok(_) => {
                // Decrement the like count in posts collection
                let update = doc! {
                    "$inc": { "like_count": -1 }
                };
                
                match posts_collection.update_one(post_filter, update).await {
                    Ok(_) => {
                        info!("Post unliked successfully");
                        Ok(HttpResponse::Ok().json(Response::<()> {
                            status: "success".to_string(),
                            message: "Post unliked successfully".to_string(),
                            data: None,
                        }))
                    }
                    Err(e) => {
                        error!("Error updating post like count: {}", e);
                        Err(AppError::from(e))
                    }
                }
            }
            Err(e) => {
                error!("Error removing like: {}", e);
                Err(AppError::from(e))
            }
        }
    } else {
        // Like the post by inserting a new like
        let like_doc = doc! {
            "post_id": &like.post_id,
            "user_id": &like.user_id,
            "created_at": chrono::Utc::now().to_rfc3339()
        };

        match likes_collection.insert_one(like_doc).await {
            Ok(_) => {
                // Increment the like count in posts collection
                let update = doc! {
                    "$inc": { "like_count": 1 }
                };
                
                match posts_collection.update_one(post_filter, update).await {
                    Ok(_) => {
                        info!("Post liked successfully");
                        Ok(HttpResponse::Created().json(Response::<()> {
                            status: "success".to_string(),
                            message: "Post liked successfully".to_string(),
                            data: None,
                        }))
                    }
                    Err(e) => {
                        error!("Error updating post like count: {}", e);
                        Err(AppError::from(e))
                    }
                }
            }
            Err(e) => {
                error!("Error recording like: {}", e);
                Err(AppError::from(e))
            }
        }
    }
}

async fn get_user_profile_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    info!("Retrieving user profile with ID: {}", id);

    // Validate user ID
    if id.is_empty() {
        return Err(AppError::InvalidInput(
            "User ID cannot be empty".to_string(),
        ));
    }

    let users_collection = state.db.collection::<Document>("users");
    let filter = doc! { "_id": id.clone() };

    match users_collection.find_one(filter).await {
        Ok(Some(user_doc)) => {
            // Extract user data
            let username = user_doc.get_str("username").unwrap_or_default().to_string();
            let email = user_doc.get_str("email").unwrap_or_default().to_string();
            let bio = user_doc.get_str("bio").ok().map(|s| s.to_string());
            let profile_picture_url = user_doc.get_str("profile_picture_url").ok().map(|s| s.to_string());
            let join_date = user_doc.get_str("created_at").unwrap_or_default().to_string();
            
            // Count followers
            let follows_collection = state.db.collection::<Document>("follows");
            let follower_filter = doc! { "following_id": &id };
            let follower_count = follows_collection.count_documents(follower_filter).await.unwrap_or(0);
            
            // Count following
            let following_filter = doc! { "follower_id": &id };
            let following_count = follows_collection.count_documents(following_filter).await.unwrap_or(0);
            
            // Count posts
            let posts_collection = state.db.collection::<Document>("posts");
            let posts_filter = doc! { "user_id": &id };
            let post_count = posts_collection.count_documents(posts_filter).await.unwrap_or(0);
            
            let user_profile = UserProfile {
                id: id.clone(),
                username,
                bio,
                profile_picture_url,
                join_date,
                follower_count: follower_count as i32,
                following_count: following_count as i32,
                post_count: post_count as i32,
            };
            
            Ok(HttpResponse::Ok().json(Response {
                status: "success".to_string(),
                message: "User profile retrieved successfully".to_string(),
                data: Some(user_profile),
            }))
        },
        Ok(None) => {
            info!("User not found with ID: {}", id);
            Err(AppError::NotFound(format!("User with ID {} not found", id)))
        },
        Err(e) => {
            error!("Error retrieving user: {}", e);
            Err(AppError::from(e))
        }
    }
}

async fn get_user_timeline_handler(
    path: web::Path<String>,
    query: web::Query<PaginationParams>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let current_user_id = path.into_inner(); // Renamed 'id' for clarity
    info!("Retrieving timeline for user: {}", current_user_id);

    // Set up pagination
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    if page < 1 || limit < 1 || limit > 50 {
        return Err(AppError::InvalidInput(
            "Invalid pagination parameters".to_string(),
        ));
    }
    let skip = (page - 1) * limit;

    // Verify user exists
    let users_collection = state.db.collection::<Document>("users");
    if users_collection
        .find_one(doc! { "_id": &current_user_id })
        .await?
        .is_none()
    {
        return Err(AppError::NotFound(format!(
            "User with ID {} not found",
            current_user_id
        )));
    }

    // Get list of users being followed
    let follows_collection = state.db.collection::<Document>("follows");
    let follows_filter = doc! { "follower_id": &current_user_id };
    let mut following_ids = vec![current_user_id.clone()]; // Include user's own posts

    if let Ok(mut cursor) = follows_collection.find(follows_filter).await {
        while let Some(follow_doc_result) = cursor.next().await {
            if let Ok(follow_doc) = follow_doc_result {
                if let Ok(following_id) = follow_doc.get_str("following_id") {
                    following_ids.push(following_id.to_string());
                }
            }
        }
    }

    // Use aggregation pipeline to fetch posts and related data efficiently
    let posts_collection = state.db.collection::<Document>("posts");
    let pipeline = vec![
        // Match posts from followed users and self
        doc! { "$match": { "user_id": { "$in": &following_ids } } },
        // Sort posts by creation date (newest first)
        doc! { "$sort": { "created_at": -1 } },
        // Apply pagination (skip and limit)
        doc! { "$skip": skip as i64 },
        doc! { "$limit": limit as i64 },
        // Lookup user details
        doc! {
            "$lookup": {
                "from": "users",
                "localField": "user_id",
                "foreignField": "_id",
                "as": "user_info"
            }
        },
        // Deconstruct the user_info array (should only be one user)
        doc! { "$unwind": "$user_info" },
        // Lookup comments count
        doc! {
            "$lookup": {
                "from": "comments",
                "localField": "_id",
                "foreignField": "post_id",
                "as": "comments"
            }
        },
        // Lookup if the current user liked this post
        doc! {
            "$lookup": {
                "from": "likes",
                "let": { "post_id": "$_id" },
                "pipeline": [
                    { "$match": {
                        "$expr": {
                            "$and": [
                                { "$eq": [ "$post_id", "$$post_id" ] },
                                { "$eq": [ "$user_id", &current_user_id ] }
                            ]
                        }
                    }},
                    { "$limit": 1 } // Optimization: we only need to know if one exists
                ],
                "as": "user_like"
            }
        },
        // Project the desired fields for PostDetails
        doc! {
            "$project": {
                "_id": 0, // Exclude the default _id
                "id": "$_id",
                "user_id": "$user_id",
                "username": "$user_info.username",
                "profile_picture_url": "$user_info.profile_picture_url",
                "content": "$content",
                "media_urls": "$media_urls",
                "post_type": "$post_type",
                "created_at": "$created_at",
                "like_count": "$like_count",
                "comment_count": { "$size": "$comments" },
                "has_liked": { "$gt": [ { "$size": "$user_like" }, 0 ] }
            }
        },
    ];

    // Execute the aggregation pipeline
    let mut posts_cursor = match posts_collection.aggregate(pipeline).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error executing timeline aggregation pipeline: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut posts: Vec<PostDetails> = Vec::new();
    while let Some(result) = posts_cursor.next().await {
        match result {
            Ok(doc) => {
                // Deserialize the document into PostDetails struct
                match bson::from_document::<PostDetails>(doc) {
                    Ok(mut post_detail) => {
                        // Calculate human-readable time
                        post_detail.human_time =
                            if let Ok(dt) = DateTime::parse_from_rfc3339(&post_detail.created_at) {
                                format!("{}", HumanTime::from(dt))
                            } else {
                                "some time ago".to_string()
                            };
                        posts.push(post_detail);
                    }
                    Err(e) => {
                        warn!("Failed to deserialize post document: {}", e);
                        // Optionally skip this post or handle the error differently
                    }
                }
            }
            Err(e) => {
                error!("Error reading post from cursor: {}", e);
                // Optionally skip this post or return an error
            }
        }
    }

    // Get total count for pagination meta (run a separate count query)
    let count_filter = doc! { "user_id": { "$in": following_ids } };
    let total_posts = match posts_collection.count_documents(count_filter).await {
        Ok(count) => count,
        Err(e) => {
            error!("Error counting posts for pagination: {}", e);
            return Err(AppError::from(e));
        }
    };

    let total_pages = (total_posts as f64 / limit as f64).ceil() as u32;
    let pagination = PaginationMeta {
        current_page: page,
        total_pages,
        total_count: total_posts,
        has_next: page < total_pages,
        has_prev: page > 1,
    };

    let timeline = TimelineResponse {
        posts,
        pagination,
    };

    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: "Timeline retrieved successfully".to_string(),
        data: Some(timeline),
    }))
}


async fn get_user_stats_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    info!("Retrieving stats for user: {}", id);
    
    // Verify user exists
    let users_collection = state.db.collection::<Document>("users");
    let user_filter = doc! { "_id": &id };
    
    if let Ok(None) = users_collection.find_one(user_filter).await {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }
    
    // Count posts
    let posts_collection = state.db.collection::<Document>("posts");
    let posts_filter = doc! { "user_id": &id };
    let post_count = posts_collection.count_documents(posts_filter).await.unwrap_or(0) as i32;
    
    // Count comments
    let comments_collection = state.db.collection::<Document>("comments");
    let comments_filter = doc! { "user_id": &id };
    let comment_count = comments_collection.count_documents(comments_filter).await.unwrap_or(0) as i32;
    
    // Count followers
    let follows_collection = state.db.collection::<Document>("follows");
    let followers_filter = doc! { "following_id": &id };
    let follower_count = follows_collection.count_documents(followers_filter).await.unwrap_or(0) as i32;
    
    // Count following
    let following_filter = doc! { "follower_id": &id };
    let following_count = follows_collection.count_documents(following_filter).await.unwrap_or(0) as i32;
    
    // Count likes received on user's posts
    let likes_collection = state.db.collection::<Document>("likes");
    let mut total_likes_received = 0;
    if let Ok(mut cursor) = posts_collection.find(posts_filter).await {
        while let Some(post_doc_result) = cursor.next().await {
            if let Ok(post_doc) = post_doc_result {
                if let Ok(post_id) = post_doc.get_str("_id") {
                    let likes_filter = doc! { "post_id": post_id };
                    let likes = likes_collection.count_documents(likes_filter).await.unwrap_or(0) as i32;
                    total_likes_received += likes;
                }
            }
        }
    }
    
    // Count likes given by user
    let likes_given_filter = doc! { "user_id": &id };
    let total_likes_given = likes_collection.count_documents(likes_given_filter).await.unwrap_or(0) as i32;
    
    let stats = UserStats {
        post_count,
        comment_count,
        follower_count,
        following_count,
        total_likes_received,
        total_likes_given,
    };
    
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: "User stats retrieved successfully".to_string(),
        data: Some(stats),
    }))
}

async fn get_trending_posts_handler(
    query: web::Query<PaginationParams>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Retrieving trending posts");
    
    // Set up pagination
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    if page < 1 || limit < 1 || limit > 50 {
        return Err(AppError::InvalidInput("Invalid pagination parameters".to_string()));
    }
    
    // Get posts sorted by like count (highest first)
    let posts_collection = state.db.collection::<Document>("posts");
    
    // Configure options for pagination and sorting by like count
    let skip = (page - 1) * limit;
    let mut options = FindOptions::default();
    options.skip = Some(skip as u64);
    options.limit = Some(limit as i64);
    options.sort = Some(doc! { "like_count": -1, "created_at": -1 }); // Sort by likes, then by date
    
    // Count total posts for pagination info
    let total_posts = match posts_collection.count_documents(doc! {}).await {
        Ok(count) => count,
        Err(e) => {
            error!("Error counting posts: {}", e);
            return Err(AppError::from(e));
        }
    };
    
    let total_pages = (total_posts as f64 / limit as f64).ceil() as u32;
    
    let mut posts = Vec::new();
    
    match posts_collection.find(doc! {}, options).await {
        Ok(mut cursor) => {
            let users_collection = state.db.collection::<Document>("users");
            let comments_collection = state.db.collection::<Document>("comments");
            let likes_collection = state.db.collection::<Document>("likes");
            
            while let Some(post_doc_result) = cursor.next().await {
                if let Ok(post_doc) = post_doc_result {
                    let post_id = post_doc.get_str("_id").unwrap_or_default().to_string();
                    let user_id = post_doc.get_str("user_id").unwrap_or_default().to_string();
                    let content = post_doc.get_str("content").unwrap_or_default().to_string();
                    let created_at = post_doc.get_str("created_at").unwrap_or_default().to_string();
                    
                    // Get post media URLs if they exist
                    let media_urls = if let Ok(media_array) = post_doc.get_array("media_urls") {
                        media_array.iter()
                            .filter_map(|m| m.as_str().map(|s| s.to_string()))
                            .collect()
                    } else {
                        Vec::new()
                    };
                    
                    // Get post type
                    let post_type = post_doc.get_str("post_type").unwrap_or("Text").to_string();
                    
                    // Get like count
                    let like_count = post_doc.get_i32("like_count").unwrap_or(0);
                    
                    // Get comment count
                    let comment_filter = doc! { "post_id": &post_id };
                    let comment_count = comments_collection.count_documents(comment_filter).await.unwrap_or(0) as i32;
                    
                    // Get username and profile picture
                    let user_info = users_collection.find_one(doc! { "_id": &user_id }).await;
                    let (username, profile_picture_url) = if let Ok(Some(user_doc)) = user_info {
                        (
                            user_doc.get_str("username").unwrap_or_default().to_string(),
                            user_doc.get_str("profile_picture_url").ok().map(String::from)
                        )
                    } else {
                        ("Unknown User".to_string(), None)
                    };
                    
                    // Calculate human-readable time
                    let human_time = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&created_at) {
                        format!("{} ago", HumanTime::from(dt))
                    } else {
                        "some time ago".to_string()
                    };
                    
                    posts.push(PostDetails {
                        id: post_id.clone(),
                        user_id,
                        username,
                        profile_picture_url,
                        content,
                        media_urls,
                        post_type: match post_type.as_str() {
                            "Image" => PostType::Image,
                            "Video" => PostType::Video,
                            "Link" => PostType::Link,
                            _ => PostType::Text,
                        },
                        created_at,
                        human_time,
                        like_count,
                        comment_count,
                        has_liked: false, // We don't know the current user in this endpoint
                    });
                }
            }
            
            let pagination = PaginationMeta {
                current_page: page,
                total_pages,
                total_count: total_posts,
                has_next: page < total_pages,
                has_prev: page > 1,
            };
            
            let timeline = TimelineResponse {
                posts,
                pagination,
            };
            
            Ok(HttpResponse::Ok().json(Response {
                status: "success".to_string(),
                message: "Trending posts retrieved successfully".to_string(),
                data: Some(timeline),
            }))
        },
        Err(e) => {
            error!("Error retrieving trending posts: {}", e);
            Err(AppError::from(e))
        }
    }
}

async fn populate_test_data_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Starting test data population");
    
    // Verify database connection before proceeding
    match state.db.run_command(doc! {"ping": 1}).await {
        Ok(_) => info!("Database connection verified, proceeding with test data population"),
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            return Err(AppError::MongoError(e));
        }
    }
    
    let db = &state.db;
    
    // Generate test users
    let mut users = Vec::new();
    for i in 1..=10 {
        let user_id = Uuid::new_v4().to_string();
        let username = format!("test_user_{}", i);
        let email = format!("user{}@test.com", i);
        let bio = format!("Bio for test user {}. This is a sample biography that provides some background information.", i);
        let profile_pic = format!("https://randomuser.me/api/portraits/{}/{}.jpg", 
                              if i % 2 == 0 { "men" } else { "women" }, i);
        
        let user_doc = doc! {
            "_id": &user_id,
            "username": &username,
            "email": &email,
            "password_hash": "test_hash",
            "bio": &bio,
            "profile_picture_url": &profile_pic,
            "created_at": chrono::Utc::now().to_rfc3339()
        };
        
        if let Err(e) = db.collection::<Document>("users").insert_one(user_doc).await {
            error!("Error creating test user: {}", e);
            return Err(AppError::from(e));
        }
        
        users.push(user_id);
    }
    
    // Generate test posts
    let post_types = vec!["Text", "Image", "Video", "Link"];
    let post_contents = vec![
        "Just hanging out with friends today! #fun #weekend",
        "Check out this cool photo I took!",
        "My thoughts on the latest tech trends...",
        "Can't believe what happened today!",
        "Excited to announce my new project!",
        "Who else is watching the game tonight?",
        "The weather is beautiful today!",
        "Looking for recommendations for a good book to read.",
        "Just finished this amazing course.",
        "Celebrating my birthday today!",
    ];
    
    let mut posts = Vec::new();
    
    for user_id in users.iter() {
        for i in 1..=5 {
            let post_id = Uuid::new_v4().to_string();
            let post_type = post_types[i % post_types.len()];
            let content_idx = (i + users.iter().position(|id| id == user_id).unwrap_or(0)) % post_contents.len();
            let content = post_contents[content_idx];
            
            let media_urls = if post_type != "Text" {
                vec![format!("https://picsum.photos/id/{}/{}", 
                    (i * 10 + users.iter().position(|id| id == user_id).unwrap_or(0)) % 1000,
                    if post_type == "Image" { "800/600" } 
                    else if post_type == "Video" { "1280/720" }
                    else { "400/300" }
                )]
            } else {
                Vec::new()
            };
            
            // Create a post from a random time in the past (up to 30 days ago)
            let days_ago = rand::thread_rng().gen_range(0..30);
            let hours_ago = rand::thread_rng().gen_range(0..24);
            let minutes_ago = rand::thread_rng().gen_range(0..60);
            let created_at = (chrono::Utc::now() - chrono::Duration::days(days_ago) 
                - chrono::Duration::hours(hours_ago)
                - chrono::Duration::minutes(minutes_ago)).to_rfc3339();
            
            let post_doc = doc! {
                "_id": &post_id,
                "user_id": user_id,
                "content": content,
                "post_type": post_type,
                "media_urls": &media_urls,
                "like_count": 0,
                "created_at": created_at
            };
            
            if let Err(e) = db.collection::<Document>("posts").insert_one(post_doc).await {
                error!("Error creating test post: {}", e);
                return Err(AppError::from(e));
            }
            
            posts.push(post_id);
        }
    }
    
    // Generate follows relationships
    for (i, follower_id) in users.iter().enumerate() {
        // Each user follows some random users
        let mut to_follow: Vec<usize> = (0..users.len()).filter(|&idx| idx != i).collect();
        to_follow.shuffle(&mut rand::thread_rng());
        
        // Follow 3-7 random users
        let num_to_follow = rand::thread_rng().gen_range(3..=7).min(to_follow.len());
        
        for j in 0..num_to_follow {
            let following_id = &users[to_follow[j]];
            
            let follow_doc = doc! {
                "follower_id": follower_id,
                "following_id": following_id,
                "created_at": chrono::Utc::now().to_rfc3339()
            };
            
            if let Err(e) = db.collection::<Document>("follows").insert_one(follow_doc).await {
                error!("Error creating follow relationship: {}", e);
                return Err(AppError::from(e));
            }
        }
    }
    
    // Generate likes and comments
    let comment_texts = vec![
        "Great post!",
        "I totally agree with this.",
        "Thanks for sharing!",
        "This is so cool!",
        "Interesting perspective.",
        "I've been thinking about this too.",
        "Keep up the great work!",
        "Can't wait to see more!",
        "This made my day!",
        "Very insightful!"
    ];

    // Generate likes and comments for each post
    for post_id in posts.iter() {
        // Random number of likes (3-15)
        let num_likes = rand::thread_rng().gen_range(3..=15).min(users.len());
        let mut likers: Vec<usize> = (0..users.len()).collect();
        likers.shuffle(&mut rand::thread_rng());
        
        for i in 0..num_likes {
            let user_id = &users[likers[i]];
            let like_doc = doc! {
                "post_id": post_id,
                "user_id": user_id,
                "created_at": chrono::Utc::now().to_rfc3339()
            };
            
            if let Err(e) = db.collection::<Document>("likes").insert_one(like_doc).await {
                error!("Error creating like: {}", e);
                return Err(AppError::from(e));
            }
            
            // Update post like count
            let filter = doc! { "_id": post_id };
            let update = doc! { "$inc": { "like_count": 1 } };
            
            if let Err(e) = db.collection::<Document>("posts").update_one(filter, update).await {
                error!("Error updating post like count: {}", e);
                return Err(AppError::from(e));
            }
        }
        
        // Random number of comments (1-5)
        let num_comments = rand::thread_rng().gen_range(1..=5).min(users.len());
        let mut commenters: Vec<usize> = (0..users.len()).collect();
        commenters.shuffle(&mut rand::thread_rng());
        
        for i in 0..num_comments {
            let user_id = &users[commenters[i]];
            let comment_text = comment_texts[rand::thread_rng().gen_range(0..comment_texts.len())];
            
            let comment_doc = doc! {
                "_id": Uuid::new_v4().to_string(),
                "post_id": post_id,
                "user_id": user_id,
                "content": comment_text,
                "created_at": chrono::Utc::now().to_rfc3339()
            };
            
            if let Err(e) = db.collection::<Document>("comments").insert_one(comment_doc).await {
                error!("Error creating comment: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!("Test data population completed successfully");
    Ok(HttpResponse::Ok().json(Response::<()> {
        status: "success".to_string(),
        message: "Test data populated successfully".to_string(),
        data: None,
    }))
}

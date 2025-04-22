use crate::{errors::AppError, models::*, state::AppState};
use actix_web::{web, HttpResponse, Responder, Result};
use chrono::Utc;
use mongodb::bson::{doc, Document};
use rand::seq::SliceRandom;
use rand::Rng;
use tracing::{error, info};
use uuid::Uuid;

// Health Check Handler
pub async fn health_check_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Health check requested");

    // Longer timeout for health checks (15 seconds instead of 5)
    let timeout_duration = std::time::Duration::from_secs(15);

    // Retry logic: try 3 times with a short delay between attempts
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 1..=max_retries {
        info!("Health check attempt {}/{}", attempt, max_retries);

        // Try to ping the database with the configured timeout
        let timeout_future =
            tokio::time::timeout(timeout_duration, state.db.run_command(doc! {"ping": 1})).await;

        match timeout_future {
            Ok(Ok(_)) => {
                // Success! Database is reachable
                info!("Health check successful on attempt {}", attempt);
                return Ok(HttpResponse::Ok().json(Response::<()> {
                    status: "success".to_string(),
                    message: "Service is healthy".to_string(),
                    data: None,
                }));
            }
            Ok(Err(e)) => {
                // Connection established but command failed
                error!("Health check failed on attempt {}: {}", attempt, e);
                last_error = Some(e.to_string());
            }
            Err(_) => {
                // Timeout occurred
                error!("Health check timed out on attempt {}", attempt);
                last_error = Some("database connection timeout".to_string());
            }
        }

        // If we reached the maximum number of retries, break out of the loop
        if attempt == max_retries {
            break;
        }

        // Wait briefly before the next retry
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // If we got here, all retries failed
    let error_message = last_error.unwrap_or_else(|| "unknown database error".to_string());
    error!(
        "Health check failed after {} attempts: {}",
        max_retries, error_message
    );

    Ok(HttpResponse::ServiceUnavailable().json(Response::<()> {
        status: "error".to_string(),
        message: format!("Service is unhealthy: {}", error_message),
        data: None,
    }))
}

// Create User Handler
pub async fn create_user_handler(
    user: web::Json<User>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Creating new user with username: {}", user.username);

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
        "created_at": Utc::now().to_rfc3339(),
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

// Create Post Handler
pub async fn create_post_handler(
    post: web::Json<Post>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Creating new post for user: {}", post.user_id);

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
        "created_at": Utc::now().to_rfc3339(),
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

// Create Comment Handler
pub async fn create_comment_handler(
    comment: web::Json<Comment>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Creating new comment for post: {}", comment.post_id);

    if comment.post_id.is_empty() || comment.user_id.is_empty() || comment.content.is_empty() {
        return Err(AppError::InvalidInput(
            "All fields are required".to_string(),
        ));
    }

    let collection = state.db.collection::<Document>("comments");
    let comment_doc = doc! {
        "post_id": &comment.post_id,
        "user_id": &comment.user_id,
        "content": &comment.content,
        "created_at": Utc::now().to_rfc3339(),
    };

    match collection.insert_one(comment_doc).await {
        Ok(_) => Ok(HttpResponse::Created().json(Response::<()> {
            status: "success".to_string(),
            message: "Comment created successfully".to_string(),
            data: None,
        })),
        Err(e) => Err(AppError::from(e)),
    }
}

// Follow User Handler
pub async fn follow_user_handler(
    follow: web::Json<Follow>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!(
        "User {} is following user {}",
        follow.follower_id, follow.following_id
    );

    if follow.follower_id.is_empty() || follow.following_id.is_empty() {
        return Err(AppError::InvalidInput(
            "Follower and Following IDs are required".to_string(),
        ));
    }

    let collection = state.db.collection::<Document>("follows");
    let follow_doc = doc! {
        "follower_id": &follow.follower_id,
        "following_id": &follow.following_id,
        "created_at": Utc::now().to_rfc3339(),
    };

    match collection.insert_one(follow_doc).await {
        Ok(_) => Ok(HttpResponse::Created().json(Response::<()> {
            status: "success".to_string(),
            message: "Follow relationship created successfully".to_string(),
            data: None,
        })),
        Err(e) => Err(AppError::from(e)),
    }
}

// Helper function to generate random users
fn generate_random_users(count: usize) -> Vec<mongodb::bson::Document> {
    let mut rng = rand::thread_rng();
    let names_pool = vec![
        "Alice", "Bob", "Charlie", "Daisy", "Eve", "Frank", "Grace", "Hank", "Ivy", "Jack",
        "Karen", "Leo", "Mona", "Nina", "Oscar", "Paul", "Quinn", "Rita", "Steve", "Tina", "Uma",
        "Victor", "Wendy", "Xander", "Yara", "Zane",
    ];

    let mut users = Vec::new();

    for _ in 0..count {
        let name = names_pool.choose(&mut rng).unwrap();
        let number: u32 = rng.gen_range(1..1000);
        let username = format!("{}_{}", name, number);
        let email = format!("{}{}@example.com", name.to_lowercase(), number);
        let password_hash = format!("hashed_password_{}", number);
        let bio = format!("Hi, I'm {}! Lover of chaos and creator of mayhem.", name);
        let profile_picture_url = format!("http://fakemedia.com/avatars/{}.png", username);

        users.push(doc! {
            "username": username,
            "email": email,
            "password_hash": password_hash,
            "bio": bio,
            "profile_picture_url": profile_picture_url,
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
    }

    users
}

// Helper function to generate random edgy posts
fn generate_random_posts(user_ids: &[String], count: usize) -> Vec<mongodb::bson::Document> {
    let mut rng = rand::thread_rng();
    let comments_pool = vec![
        "This is the hill I choose to die on.",
        "You won't believe what happened next...",
        "I’m not saying it’s aliens, but it’s aliens.",
        "Chaos is my middle name.",
        "Insert edgy quote here.",
        "This is a hot take, don’t @ me.",
        "I'm just here for the drama.",
        "I should probably delete this later.",
        "Who even needs sleep these days?",
        "Normalize being chaotic good.",
        "Just dropped my mixtape, and it’s fire!",
        "Someone call the FBI, this is too good.",
        "Plot twist: I’m the villain.",
        "This is why we can’t have nice things.",
        "My vibe? Just winging it.",
    ];

    let mut posts = Vec::new();

    for _ in 0..count {
        let user_id = user_ids.choose(&mut rng).unwrap();
        let content = comments_pool.choose(&mut rng).unwrap();
        let media_url = if rng.gen_bool(0.5) {
            Some(format!(
                "http://fakemedia.com/media/random_{}.{}",
                rng.gen_range(1..100),
                if rng.gen_bool(0.5) { "jpg" } else { "mp4" }
            ))
        } else {
            None
        };

        posts.push(doc! {
            "user_id": user_id,
            "content": content,
            "media_urls": media_url.as_ref().map(|url| vec![url]).unwrap_or_default(),
            "post_type": if media_url.is_some() { "Media" } else { "Text" },
            "like_count": rng.gen_range(0..500),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
    }

    posts
}

// Populate Database Handler
pub async fn populate_database_handler(
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Populating database with edgy random test data");

    let users_collection = state.db.collection::<mongodb::bson::Document>("users");
    let posts_collection = state.db.collection::<mongodb::bson::Document>("posts");

    // Generate 25 random users
    let test_users = generate_random_users(100);

    // Insert random users into the users collection
    let mut user_ids = Vec::new();
    for user in &test_users {
        if let Err(e) = users_collection.insert_one(user).await {
            error!("Error inserting test user: {}", e);
            return Err(AppError::from(e));
        }
        if let Some(id) = user.get_str("username").ok() {
            user_ids.push(id.to_string());
        }
    }

    // Generate random posts for the inserted users
    let test_posts = generate_random_posts(&user_ids, 1000);

    // Insert random posts into the posts collection
    for post in test_posts {
        if let Err(e) = posts_collection.insert_one(post).await {
            error!("Error inserting test post: {}", e);
            return Err(AppError::from(e));
        }
    }

    Ok(HttpResponse::Ok().json(Response::<()> {
        status: "success".to_string(),
        message: "Test data populated successfully with 100 & 1000 posts".to_string(),
        data: None,
    }))
}

// Clean Database Handler
pub async fn clean_database_handler(
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Cleaning database of test data");

    let users_collection = state.db.collection::<mongodb::bson::Document>("users");
    let posts_collection = state.db.collection::<mongodb::bson::Document>("posts");

    // Delete all documents in the collections
    if let Err(e) = users_collection.delete_many(doc! {}).await {
        error!("Error cleaning users collection: {}", e);
        return Err(AppError::from(e));
    }

    if let Err(e) = posts_collection.delete_many(doc! {}).await {
        error!("Error cleaning posts collection: {}", e);
        return Err(AppError::from(e));
    }

    Ok(HttpResponse::Ok().json(Response::<()> {
        status: "success".to_string(),
        message: "Test data cleaned successfully".to_string(),
        data: None,
    }))
}

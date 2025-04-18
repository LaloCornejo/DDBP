use crate::{errors::AppError, models::*, state::AppState};
use actix_web::{web, HttpResponse, Responder, Result};
use chrono::Utc;
use mongodb::bson::{doc, Document};
use tracing::{error, info};
use uuid::Uuid;

// Health Check Handler
pub async fn health_check_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Health check requested");
    let timeout_future = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        state.db.run_command(doc! {"ping": 1}),
    )
    .await;

    match timeout_future {
        Ok(Ok(_)) => Ok(HttpResponse::Ok().json(Response::<()> {
            status: "success".to_string(),
            message: "Service is healthy".to_string(),
            data: None,
        })),
        Ok(Err(e)) => {
            error!("Health check failed: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(Response::<()> {
                status: "error".to_string(),
                message: "Service is unhealthy: database connection failed".to_string(),
                data: None,
            }))
        }
        Err(_) => {
            error!("Health check timed out");
            Ok(HttpResponse::ServiceUnavailable().json(Response::<()> {
                status: "error".to_string(),
                message: "Service is unhealthy: database connection timeout".to_string(),
                data: None,
            }))
        }
    }
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

// Add other handlers here (like create_comment_handler, follow_user_handler, etc.)

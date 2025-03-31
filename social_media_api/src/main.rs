use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use mongodb::{Client, options::ClientOptions, bson::doc, bson::Document};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    title: String,
    content: String,
    author: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Media {
    post_id: String,
    media_url: String,
    media_type: String,
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
    // Initialize database connection with improved error handling and connection options
    println!("Starting application...");
    println!("Connecting to MongoDB replica set...");
    
    // Configure more robust connection settings
    let mut client_options = match ClientOptions::parse(
        "mongodb://admin:password@localhost:27017,localhost:27018,localhost:27019/?replicaSet=rs0"
    ).await {
        Ok(options) => {
            println!("Successfully parsed MongoDB connection string");
            options
        },
        Err(e) => {
            eprintln!("Failed to parse MongoDB connection string: {}", e);
            std::process::exit(1);
        }
    };
    
    // Set timeout values
    client_options.server_selection_timeout = Some(Duration::from_secs(30));
    client_options.connect_timeout = Some(Duration::from_secs(20));
    client_options.retry_writes = Some(true);
    client_options.retry_reads = Some(true);
    client_options.app_name = Some("social-media-app".to_string());
    
    // Create MongoDB client
    let client = match Client::with_options(client_options) {
        Ok(client) => {
            println!("Successfully created MongoDB client");
            client
        },
        Err(e) => {
            eprintln!("Failed to create MongoDB client: {}", e);
            std::process::exit(1);
        }
    };

    //println!("{:#?}",client);
    
    // Test connection with a ping to admin database
    println!("Testing MongoDB connection with ping...");
    match client.database("admin").run_command(doc! {"ping": 1}, None).await {
        Ok(_) => println!("Successfully connected to MongoDB replica set"),
        Err(e) => {
            eprintln!("Failed to ping MongoDB: {}", e);
            eprintln!("Warning: Continuing despite ping failure. Check MongoDB connectivity.");
        }
    };
    
    // Create database handle
    let db = client.database("social_media_db");
    
    // Check if the social_media_db database is accessible
    println!("Testing access to social_media_db...");
    match db.list_collection_names(None).await {
        Ok(collections) => println!("Successfully accessed social_media_db. Collections: {:?}", collections),
        Err(e) => {
            eprintln!("Warning: Failed to list collections in social_media_db: {}", e);
            eprintln!("Will attempt to create collections on first use");
        }
    };
    
    // Create app state with database
    let app_state = web::Data::new(AppState { db });
    
    println!("Server starting at http://127.0.0.1:8000");
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/create_post", web::post().to(create_post_handler))
            .route("/add_media", web::post().to(add_media_handler))
            .route("/get_post/{id}", web::get().to(get_post_handler))
            // Add health check endpoint
            .route("/health", web::get().to(health_check_handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

async fn create_post_handler(post: web::Json<Post>, state: web::Data<AppState>) -> Result<impl Responder> {
    println!("Creating post: {}", post.title);
    
    let collection = state.db.collection::<Document>("posts");
    
    let post_id = Uuid::new_v4().to_string();
    let post_doc = doc! {
        "_id": &post_id,
        "title": &post.title,
        "content": &post.content,
        "author": &post.author,
        "created_at": chrono::Utc::now().to_rfc3339()
    };
    
    match collection.insert_one(post_doc, None).await {
        Ok(_) => {
            println!("Post created successfully with ID: {}", post_id);
            Ok(HttpResponse::Ok().json(Response { 
                message: format!("Post created successfully with ID: {}", post_id)
            }))
        },
        Err(e) => {
            eprintln!("Error creating post: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response { 
                message: format!("Failed to create post: {}", e)
            }))
        }
    }
}

async fn add_media_handler(media: web::Json<Media>, state: web::Data<AppState>) -> Result<impl Responder> {
    println!("Adding media for post: {}", media.post_id);
    
    let collection = state.db.collection::<Document>("media");
    
    let media_id = Uuid::new_v4().to_string();
    let media_doc = doc! {
        "_id": &media_id,
        "post_id": &media.post_id,
        "media_url": &media.media_url,
        "media_type": &media.media_type,
        "uploaded_at": chrono::Utc::now().to_rfc3339()
    };
    
    match collection.insert_one(media_doc, None).await {
        Ok(_) => {
            println!("Media added successfully with ID: {}", media_id);
            Ok(HttpResponse::Ok().json(Response { 
                message: format!("Media added successfully with ID: {}", media_id)
            }))
        },
        Err(e) => {
            eprintln!("Error adding media: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response { 
                message: format!("Failed to add media: {}", e)
            }))
        }
    }
}

async fn get_post_handler(path: web::Path<String>, state: web::Data<AppState>) -> Result<impl Responder> {
    let id = path.into_inner();
    println!("Retrieving post with ID: {}", id);
    
    let collection = state.db.collection::<Document>("posts");
    
    let filter = doc! { "_id": id.clone() };
    
    match collection.find_one(filter, None).await {
        Ok(Some(doc)) => {
            println!("Post found: {}", doc.get_str("title").unwrap_or_default());
            let post = Post {
                title: doc.get_str("title").unwrap_or_default().to_string(),
                content: doc.get_str("content").unwrap_or_default().to_string(),
                author: doc.get_str("author").unwrap_or_default().to_string(),
            };
            Ok(HttpResponse::Ok().json(post))
        },
        Ok(None) => {
            println!("Post not found with ID: {}", id);
            Ok(HttpResponse::NotFound().json(Response {
                message: format!("Post not found with ID: {}", id)
            }))
        },
        Err(e) => {
            eprintln!("Error retrieving post: {}", e);
            Ok(HttpResponse::InternalServerError().json(Response {
                message: format!("Failed to retrieve post: {}", e)
            }))
        }
    }
}

// Health check endpoint
async fn health_check_handler(state: web::Data<AppState>) -> Result<impl Responder> {
    match state.db.run_command(doc! {"ping": 1}, None).await {
        Ok(_) => Ok(HttpResponse::Ok().json(Response {
            message: "Service is healthy, MongoDB connection is working".to_string()
        })),
        Err(e) => {
            eprintln!("Health check failed, MongoDB connection error: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(Response {
                message: format!("Service is unhealthy, MongoDB connection failed: {}", e)
            }))
        }
    }
}

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;
use std::sync::Arc;
use std::env;

// Database connection pool
struct AppState {
    db_pool: Pool<Postgres>,
    node_type: String,
    node_id: Uuid,
}

// Models
#[derive(Serialize, Deserialize)]
struct Node {
    node_id: Uuid,
    node_name: String,
    node_url: String,
    node_type: String,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct Post {
    post_id: Uuid,
    user_id: Uuid,
    content: String,
    created_at: chrono::DateTime<chrono::Utc>,
    node_id: Uuid,
}

#[derive(Serialize, Deserialize)]
struct Image {
    image_id: Uuid,
    title: String,
    imaginary_path: String,
    upload_date: chrono::DateTime<chrono::Utc>,
    user_id: Uuid,
    node_id: Uuid,
}

// Request/Response models
#[derive(Serialize, Deserialize)]
struct CreatePostRequest {
    user_id: Uuid,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct CreateImageRequest {
    title: String,
    user_id: Uuid,
    imaginary_path: Option<String>,
}

// API Handlers
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Service is running")
}

async fn get_nodes(data: web::Data<Arc<AppState>>) -> impl Responder {
    match sqlx::query_as!(
        Node,
        r#"SELECT node_id, node_name, node_url, node_type, status FROM nodes"#
    )
        .fetch_all(&data.db_pool)
        .await {
            Ok(nodes) => HttpResponse::Ok().json(nodes),
            Err(e) => {
                eprintln!("Database error: {}", e);
                HttpResponse::InternalServerError().body("Failed to retrieve nodes")
            }
        }
}

async fn create_post(
    data: web::Data<Arc<AppState>>,
    post_req: web::Json<CreatePostRequest>,
) -> impl Responder {
    // Determine which node should handle this post (simple hash-based sharding)
    let target_node_id = if data.node_type == "central" {
        // Determine fragment node based on user_id
        match get_target_node_for_user(&data.db_pool, post_req.user_id).await {
            Ok(node_id) => node_id,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to assign node"),
        }
    } else {
        // We're already on a fragment node
        data.node_id
    };

    // If we're on the central node and need to forward to a fragment
    if data.node_type == "central" && data.node_id != target_node_id {
        return forward_post_creation(&data.db_pool, target_node_id, &post_req).await;
    }

    // Create post in local database
    let post_id = Uuid::new_v4();
    let created_at = chrono::Utc::now();

    match sqlx::query!(
        r#"
        INSERT INTO posts (post_id, user_id, content, created_at, node_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        post_id,
        post_req.user_id,
        post_req.content,
        created_at,
        target_node_id,
    )
        .execute(&data.db_pool)
        .await {
            Ok(_) => {
                // If successful, return the created post
                let post = Post {
                    post_id,
                    user_id: post_req.user_id,
                    content: post_req.content.clone(),
                    created_at,
                    node_id: target_node_id,
                };
                HttpResponse::Created().json(post)
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                HttpResponse::InternalServerError().body("Failed to create post")
            }
        }
}

async fn get_posts(data: web::Data<Arc<AppState>>) -> impl Responder {
    // For simplicity, this just gets posts from the current node
    // In a real system, central node would aggregate from all nodes
    match sqlx::query_as!(
        Post,
        r#"
        SELECT post_id, user_id, content, created_at, node_id
        FROM posts 
        WHERE node_id = $1 OR $2 = 'central'
        ORDER BY created_at DESC 
        LIMIT 100
        "#,
        data.node_id,
        data.node_type
    )
        .fetch_all(&data.db_pool)
        .await {
            Ok(posts) => HttpResponse::Ok().json(posts),
            Err(e) => {
                eprintln!("Database error: {}", e);
                HttpResponse::InternalServerError().body("Failed to retrieve posts")
            }
        }
}

async fn create_image(
    data: web::Data<Arc<AppState>>,
    image_req: web::Json<CreateImageRequest>,
) -> impl Responder {
    // Similar logic to create_post for distribution
    let target_node_id = if data.node_type == "central" {
        // Round-robin allocation for images
        match get_next_fragment_node(&data.db_pool).await {
            Ok(node_id) => node_id,
            Err(_) => return HttpResponse::InternalServerError().body("Failed to assign node"),
        }
    } else {
        // We're already on a fragment node
        data.node_id
    };

    // Forward if needed
    if data.node_type == "central" && data.node_id != target_node_id {
        return forward_image_creation(&data.db_pool, target_node_id, &image_req).await;
    }

    let image_id = Uuid::new_v4();
    let upload_date = chrono::Utc::now();

    // Generate imaginary path if not provided
    let imaginary_path = match &image_req.imaginary_path {
        Some(path) => path.clone(),
        None => {
            // Generate an imaginary path based on title and ID
            format!("/images/user_{}/{}_{}.jpg", 
                image_req.user_id.to_string().split('-').next().unwrap_or("unknown"),
                image_req.title.to_lowercase().replace(" ", "_"),
                image_id.to_string().split('-').next().unwrap_or("img"))
        }
    };

    match sqlx::query!(
        r#"
        INSERT INTO images (image_id, title, imaginary_path, upload_date, user_id, node_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        image_id,
        image_req.title,
        imaginary_path,
        upload_date,
        image_req.user_id,
        target_node_id,
    )
        .execute(&data.db_pool)
        .await {
            Ok(_) => {
                let image = Image {
                    image_id,
                    title: image_req.title.clone(),
                    imaginary_path,
                    upload_date,
                    user_id: image_req.user_id,
                    node_id: target_node_id,
                };
                HttpResponse::Created().json(image)
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                HttpResponse::InternalServerError().body("Failed to create image record")
            }
    }
}

async fn get_images(data: web::Data<Arc<AppState>>) -> impl Responder {
    match sqlx::query_as!(
        Image,
        r#"
        SELECT image_id, title, imaginary_path, upload_date, user_id, node_id
        FROM images
        WHERE node_id = $1 OR $2 = 'central'
        ORDER BY upload_date DESC
        "#,
        data.node_id,
        data.node_type
    )
        .fetch_all(&data.db_pool)
        .await {
            Ok(images) => HttpResponse::Ok().json(images),
            Err(e) => {
                eprintln!("Database error: {}", e);
                HttpResponse::InternalServerError().body("Failed to retrieve images")
            }
        }
}

// Helper functions
async fn get_target_node_for_user(
    pool: &Pool<Postgres>,
    user_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    // Simple hash-based sharding - use last digit of UUID
    let user_id_str = user_id.to_string();
    let last_char = user_id_str.chars().last().unwrap_or('0');
    let shard_num = (last_char.to_digit(16).unwrap_or(0) % 3) + 1; // Get values 1, 2, or 3

    let node = sqlx::query!(
        r#"
        SELECT node_id FROM nodes 
        WHERE node_type = 'fragment' AND node_name = $1
        "#,
        format!("fragment{}", shard_num)
    )
        .fetch_one(pool)
        .await?;

    Ok(node.node_id)
}

async fn get_next_fragment_node(pool: &Pool<Postgres>) -> Result<Uuid, sqlx::Error> {
    // Simple round-robin allocation
    let images_count = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM images"#
    )
        .fetch_one(pool)
        .await?;

    let shard_num = (images_count.count.unwrap_or(0) as i64 % 3) + 1; // Get values 1, 2, or 3

    let node = sqlx::query!(
        r#"
        SELECT node_id FROM nodes 
        WHERE node_type = 'fragment' AND node_name = $1
        "#,
        format!("fragment{}", shard_num)
    )
        .fetch_one(pool)
        .await?;

    Ok(node.node_id)
}

async fn forward_post_creation(
    pool: &Pool<Postgres>,
    target_node_id: Uuid,
    post_req: &CreatePostRequest,
) -> HttpResponse {
    // Get target node URL
    let node = match sqlx::query!(
        r#"SELECT node_url FROM nodes WHERE node_id = $1"#,
        target_node_id
    )
        .fetch_one(pool)
        .await {
            Ok(node) => node,
            Err(e) => {
                eprintln!("Database error: {}", e);
                return HttpResponse::InternalServerError().body("Failed to forward request");
            }
        };

    // Forward the request to the target node
    let client = reqwest::Client::new();
    match client
        .post(&format!("{}/posts", node.node_url))
        .json(&post_req)
        .send()
        .await {
            Ok(response) => {
                // Return the fragment node's response
                match response.status().as_u16() {
                    201 => {
                        match response.json::<Post>().await {
                            Ok(post) => HttpResponse::Created().json(post),
                            Err(_) => HttpResponse::InternalServerError().body("Error parsing response"),
                        }
                    },
                    status_code => HttpResponse::build(
                        actix_web::http::StatusCode::from_u16(status_code).unwrap_or(
                            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
                        )
                    ).body("Error from fragment node"),
                }
            },
            Err(e) => {
                eprintln!("Forward error: {}", e);
                HttpResponse::InternalServerError().body("Failed to forward request")
            }
        }
}

async fn forward_image_creation(
    pool: &Pool<Postgres>,
    target_node_id: Uuid,
    image_req: &CreateImageRequest,
) -> HttpResponse {
    // Similar to forward_post_creation
    let node = match sqlx::query!(
        r#"SELECT node_url FROM nodes WHERE node_id = $1"#,
        target_node_id
    )
        .fetch_one(pool)
        .await {
            Ok(node) => node,
            Err(e) => {
                eprintln!("Database error: {}", e);
                return HttpResponse::InternalServerError().body("Failed to forward request");
            }
        };

    let client = reqwest::Client::new();
    match client
        .post(&format!("{}/images", node.node_url))
        .json(&image_req)
        .send()
        .await {
            Ok(response) => {
                match response.status().as_u16() {
                    201 => {
                        match response.json::<Image>().await {
                            Ok(image) => HttpResponse::Created().json(image),
                            Err(_) => HttpResponse::InternalServerError().body("Error parsing response"),
                        }
                    },
                    status_code => HttpResponse::build(
                        actix_web::http::StatusCode::from_u16(status_code).unwrap_or(
                            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
                        )
                    ).body("Error from fragment node"),
                }
            },
            Err(e) => {
                eprintln!("Forward error: {}", e);
                HttpResponse::InternalServerError().body("Failed to forward request")
            }
        }
}

async fn init_db(db_pool: &Pool<Postgres>, node_type: &str, node_name: &str, node_url: &str) -> Result<Uuid, sqlx::Error> {
    // Create tables if they don't exist
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS nodes (
            node_id UUID PRIMARY KEY,
            node_name TEXT NOT NULL,
            node_url TEXT NOT NULL,
            node_type TEXT NOT NULL,
            status TEXT NOT NULL
        )
        "#
    )
        .execute(db_pool)
        .await?;

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS posts (
            post_id UUID PRIMARY KEY,
            user_id UUID NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            node_id UUID NOT NULL REFERENCES nodes(node_id)
        )
        "#
    )
        .execute(db_pool)
        .await?;

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS images (
            image_id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            imaginary_path TEXT NOT NULL,
            upload_date TIMESTAMPTZ NOT NULL,
            user_id UUID NOT NULL,
            node_id UUID NOT NULL REFERENCES nodes(node_id) 
        )
        "#
    )
        .execute(db_pool)
        .await?;

    // Register this node
    let node_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO nodes (node_id, node_name, node_url, node_type, status)
        VALUES ($1, $2, $3, $4, 'active')
        ON CONFLICT (node_id) DO UPDATE
        SET node_name = $2, node_url = $3, node_type = $4, status = 'active'
        "#,
        node_id,
        node_name,
        node_url,
        node_type
    )
        .execute(db_pool)
        .await?;

    Ok(node_id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get environment variables
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/socialmedia".to_string());

    let node_type = env::var("NODE_TYPE")
        .unwrap_or_else(|_| "central".to_string());

    let node_name = env::var("NODE_NAME")
        .unwrap_or_else(|_| format!("{}", node_type));

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    let base_url = env::var("BASE_URL")
        .unwrap_or_else(|_| format!("http://{}:{}", host, port));

    // Connect to database
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Initialize database and register node
    let node_id = init_db(&db_pool, &node_type, &node_name, &base_url)
        .await
        .expect("Failed to initialize database");

    println!("Starting {} node '{}' on {}:{}", node_type, node_name, host, port);

    // Create app state
    let state = Arc::new(AppState {
        db_pool,
        node_type,
        node_id,
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&state)))
            .route("/health", web::get().to(health_check))
            .route("/nodes", web::get().to(get_nodes))
            .route("/posts", web::post().to(create_post))
            .route("/posts", web::get().to(get_posts))
            .route("/images", web::post().to(create_image))
            .route("/images", web::get().to(get_images))
    })
    .bind((host, port))?
        .run()
        .await
}

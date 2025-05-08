use crate::{errors::AppError, models::*, state::AppState};
use actix_web::{web, HttpResponse, Responder, Result};
use chrono::Utc;
use futures_util::StreamExt;
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

// Helper function to generate random users with more complete profiles
fn generate_random_users(count: usize) -> Vec<mongodb::bson::Document> {
    let mut rng = rand::thread_rng();
    let names_pool = vec![
        "Alice", "Bob", "Charlie", "Daisy", "Eve", "Frank", "Grace", "Hank", "Ivy", "Jack",
        "Karen", "Leo", "Mona", "Nina", "Oscar", "Paul", "Quinn", "Rita", "Steve", "Tina", "Uma",
        "Victor", "Wendy", "Xander", "Yara", "Zane",
    ];

    let bio_templates = vec![
        "Digital creator passionate about {}",
        "Exploring the world of {} one post at a time",
        "Professional {} enthusiast",
        "{} advocate | Content Creator",
        "Living life through the lens of {}",
    ];

    let interests = vec![
        "technology",
        "art",
        "music",
        "photography",
        "travel",
        "food",
        "fitness",
        "gaming",
        "books",
        "nature",
    ];

    let mut users = Vec::new();
    let current_time = chrono::Utc::now();

    for _ in 0..count {
        let name = names_pool.choose(&mut rng).unwrap();
        let number: u32 = rng.gen_range(1..1000);
        let username = format!("{}_{}", name, number);
        let email = format!("{}{}@example.com", name.to_lowercase(), number);

        let bio_template = bio_templates.choose(&mut rng).unwrap();
        let interest = interests.choose(&mut rng).unwrap();
        let bio = bio_template.replace("{}", interest);

        let user_id = Uuid::new_v4().to_string();

        users.push(doc! {
            "_id": &user_id,
            "username": &username,
            "email": &email,
            "password_hash": format!("hashed_password_{}", number),
            "bio": bio,
            // Choose one of the following options:

            "profile_picture_url": format!("https://randomuser.me/api/?gender={}",
                if rng.gen_bool(0.5) { "male" } else { "female" }
            ),

            "join_date": current_time.to_rfc3339(),
            "follower_count": 0, // Will be updated after follows are generated
            "following_count": 0, // Will be updated after follows are generated
            "post_count": 0, // Will be updated after posts are generated
            "total_likes_received": 0, // Will be updated after likes are generated
            "total_likes_given": 0, // Will be updated after likes are generated
        });
    }

    users
}

// Helper function to generate random posts with enhanced details
fn generate_random_posts(user_ids: &[String], count: usize) -> Vec<mongodb::bson::Document> {
    let mut rng = rand::thread_rng();
    let content_templates = vec![
        "Exploring the transformative power of {} and how it is reshaping industries, driving innovation, and creating new opportunities for growth and development. From its origins to its current applications, this post dives deep into the impact of {} on our world.",
        "The future of {} is here, and it's more exciting than ever. In this post, we examine the groundbreaking advancements in {} and their potential to revolutionize the way we live, work, and interact with technology.",
        "What makes {} such a game-changer? This post uncovers the key innovations, challenges, and opportunities that {} brings to the table, and why it is set to redefine the future of technology.",
        "A comprehensive look at the evolution of {} and its journey from a niche concept to a mainstream phenomenon. Discover the stories, breakthroughs, and visionaries behind {} and its role in shaping the future.",
        "10 reasons why {} is not just a buzzword but a transformative force that is driving change across industries. This post explores the practical applications, benefits, and future potential of {}.",
        "How {} is paving the way for a more sustainable, efficient, and connected world. This post delves into the innovations and ideas that are making {} a cornerstone of modern technology.",
        "The untold story of {}: from its humble beginnings to its current status as a revolutionary technology. This post highlights the milestones, challenges, and future prospects of {}.",
        "Behind the scenes of {}: a closer look at the people, ideas, and innovations that are driving its success. This post offers an insider's perspective on the world of {}.",
        "My personal journey with {}: the lessons learned, the challenges faced, and the incredible potential that {} holds for the future. This post is a reflection on the transformative power of {}.",
        "Revolutionizing the world with {}: an in-depth exploration of the technologies, ideas, and innovations that are making {} a driving force in the modern era.",
        "The intersection of {} and everyday life: how this technology is quietly influencing the way we work, communicate, and solve problems. This post examines the subtle yet profound impact of {}.",
        "The challenges and opportunities of {}: a balanced look at the hurdles that need to be overcome and the immense potential that lies ahead for this groundbreaking technology.",
        "How {} is enabling new possibilities in fields like healthcare, education, and entertainment. This post explores the real-world applications of {} and their impact on society.",
        "The role of {} in the global economy: how this technology is creating new markets, disrupting traditional industries, and driving economic growth worldwide.",
        "A visionary look at the future of {}: what lies ahead for this technology, and how it could shape the next decade of innovation and progress.",
    ];

    let titles = vec![
        "The Future of AI",
        "Blockchain Revolution",
        "Sustainable Living",
        "Digital Artistry",
        "Remote Work Culture",
        "Space Exploration",
        "Virtual Reality Experiences",
        "Renewable Energy Solutions",
        "Quantum Computing Breakthroughs",
        "Robotics in Everyday Life",
    ];

    let topics = vec![
        "AI",
        "blockchain",
        "sustainability",
        "digital art",
        "remote work",
        "space exploration",
        "virtual reality",
        "renewable energy",
        "quantum computing",
        "robotics",
    ];

    let post_types = vec![
        PostType::Text,
        PostType::Image,
        PostType::Video,
        PostType::Link,
    ];

    let mut posts = Vec::new();
    let current_time = chrono::Utc::now();

    for _ in 0..count {
        let user_id = user_ids.choose(&mut rng).unwrap();
        let template = content_templates.choose(&mut rng).unwrap();
        let topic = topics.choose(&mut rng).unwrap();
        let content = template.replace("{}", topic);
        let title = titles.choose(&mut rng).unwrap();

        let post_type = post_types.choose(&mut rng).unwrap();
        let media_urls = match post_type {
            PostType::Image => vec![format!(
                "https://picsum.photos/seed/{}/800/600",
                Uuid::new_v4()
            )],
            PostType::Video => vec![format!("https://example.com/videos/{}.mp4", Uuid::new_v4())],
            PostType::Link => vec![format!("https://example.com/article/{}", Uuid::new_v4())],
            PostType::Text => Vec::new(),
        };

        let post_id = Uuid::new_v4().to_string();
        posts.push(doc! {
            "_id": &post_id,
            "user_id": user_id,
            "title": title,
            "content": &content,
            "media_urls": &media_urls,
            "post_type": match post_type {
                PostType::Text => "text",
                PostType::Image => "image",
                PostType::Video => "video",
                PostType::Link => "link",
            },
            "created_at": current_time.to_rfc3339(),
            "like_count": 0, // Will be updated after likes are generated
            "comment_count": 0, // Will be updated after comments are generated
        });
    }

    posts
}

// Helper function to generate random comments
fn generate_random_comments(
    user_ids: &[String],
    post_ids: &[String],
    count: usize,
) -> Vec<Document> {
    let mut rng = rand::thread_rng();
    let comment_texts = vec![
        "This is incredible!",
        "I totally agree with this.",
        "Not sure about this one...",
        "Thanks for sharing!",
        "Interesting perspective.",
        "Can you explain more?",
        "This changed my view completely.",
        "I have a different opinion.",
        "Mind = blown 🤯",
        "This is the content I'm here for!",
        "Well said!",
        "Couldn't agree more.",
        "This is the way.",
        "Very insightful!",
        "You might want to reconsider this.",
    ];

    let mut comments = Vec::new();
    for _ in 0..count {
        let user_id = user_ids.choose(&mut rng).unwrap();
        let post_id = post_ids.choose(&mut rng).unwrap();
        let content = comment_texts.choose(&mut rng).unwrap();

        comments.push(doc! {
            "_id": Uuid::new_v4().to_string(),
            "post_id": post_id,
            "user_id": user_id,
            "content": content,
            "created_at": chrono::Utc::now().to_rfc3339(),
        });
    }
    comments
}

// Helper function to generate random likes
fn generate_random_likes(user_ids: &[String], post_ids: &[String], count: usize) -> Vec<Document> {
    let mut rng = rand::thread_rng();
    let mut likes = Vec::new();
    let mut seen_combinations = std::collections::HashSet::new();

    while likes.len() < count {
        let user_id = user_ids.choose(&mut rng).unwrap();
        let post_id = post_ids.choose(&mut rng).unwrap();

        // Ensure unique user-post combinations for likes
        let combination = format!("{}-{}", user_id, post_id);
        if seen_combinations.insert(combination) {
            likes.push(doc! {
                "_id": Uuid::new_v4().to_string(),
                "user_id": user_id,
                "post_id": post_id,
                "created_at": chrono::Utc::now().to_rfc3339(),
            });
        }
    }
    likes
}

// Helper function to generate random follows
fn generate_random_follows(user_ids: &[String], count: usize) -> Vec<Document> {
    let mut rng = rand::thread_rng();
    let mut follows = Vec::new();
    let mut seen_combinations = std::collections::HashSet::new();

    while follows.len() < count {
        let follower_id = user_ids.choose(&mut rng).unwrap();
        let following_id = user_ids.choose(&mut rng).unwrap();

        // Avoid self-follows and duplicate relationships
        if follower_id != following_id {
            let combination = format!("{}-{}", follower_id, following_id);
            if seen_combinations.insert(combination) {
                follows.push(doc! {
                    "_id": Uuid::new_v4().to_string(),
                    "follower_id": follower_id,
                    "following_id": following_id,
                    "created_at": chrono::Utc::now().to_rfc3339(),
                });
            }
        }
    }
    follows
}

// Populate Database Handler
pub async fn populate_database_handler(
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Populating database with comprehensive test data");

    // Initialize collections
    let users_collection = state.db.collection::<Document>("users");
    let posts_collection = state.db.collection::<Document>("posts");
    let comments_collection = state.db.collection::<Document>("comments");
    let likes_collection = state.db.collection::<Document>("likes");
    let follows_collection = state.db.collection::<Document>("follows");

    // Clean existing data first
    for collection in [
        &users_collection,
        &posts_collection,
        &comments_collection,
        &likes_collection,
        &follows_collection,
    ] {
        if let Err(e) = collection.delete_many(doc! {}).await {
            error!("Error cleaning collection: {}", e);
            return Err(AppError::from(e));
        }
    }

    // Generate and insert users
    let test_users = generate_random_users(150);
    let mut user_ids = Vec::new();

    for user in &test_users {
        match users_collection.insert_one(user.clone()).await {
            Ok(_) => {
                if let Ok(id) = user.get_str("_id") {
                    user_ids.push(id.to_string());
                }
            }
            Err(e) => {
                error!("Error inserting test user: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    // Generate and insert posts
    let test_posts = generate_random_posts(&user_ids, 300);
    let mut post_ids = Vec::new();

    for post in &test_posts {
        match posts_collection.insert_one(post.clone()).await {
            Ok(_) => {
                if let Ok(id) = post.get_str("_id") {
                    post_ids.push(id.to_string());
                }
            }
            Err(e) => {
                error!("Error inserting test post: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    // Generate and insert comments
    let test_comments = generate_random_comments(&user_ids, &post_ids, 500);
    for comment in test_comments {
        if let Err(e) = comments_collection.insert_one(comment).await {
            error!("Error inserting test comment: {}", e);
            return Err(AppError::from(e));
        }
    }

    // Generate and insert likes
    let test_likes = generate_random_likes(&user_ids, &post_ids, 1000);
    for like in test_likes {
        if let Err(e) = likes_collection.insert_one(like).await {
            error!("Error inserting test like: {}", e);
            return Err(AppError::from(e));
        }
    }

    // Generate and insert follows
    let test_follows = generate_random_follows(&user_ids, 400);
    for follow in test_follows {
        if let Err(e) = follows_collection.insert_one(follow).await {
            error!("Error inserting test follow: {}", e);
            return Err(AppError::from(e));
        }
    }

    // Update statistics for users and posts
    for user_id in &user_ids {
        // Count user statistics
        let post_count = posts_collection
            .count_documents(doc! { "user_id": user_id })
            .await? as i32;
        let follower_count = follows_collection
            .count_documents(doc! { "following_id": user_id })
            .await? as i32;
        let following_count = follows_collection
            .count_documents(doc! { "follower_id": user_id })
            .await? as i32;
        let comment_count = comments_collection
            .count_documents(doc! { "user_id": user_id })
            .await? as i32;
        let likes_given = likes_collection
            .count_documents(doc! { "user_id": user_id })
            .await? as i32;

        // Update user document
        users_collection
            .update_one(
                doc! { "_id": user_id },
                doc! {
                    "$set": {
                        "post_count": post_count,
                        "follower_count": follower_count,
                        "following_count": following_count,
                        "comment_count": comment_count,
                        "total_likes_given": likes_given,
                    }
                },
            )
            .await?;
    }

    // Update statistics for posts
    for post_id in &post_ids {
        let comment_count = comments_collection
            .count_documents(doc! { "post_id": post_id })
            .await? as i32;
        let like_count = likes_collection
            .count_documents(doc! { "post_id": post_id })
            .await? as i32;

        posts_collection
            .update_one(
                doc! { "_id": post_id },
                doc! {
                    "$set": {
                        "comment_count": comment_count,
                        "like_count": like_count,
                    }
                },
            )
            .await?;
    }

    Ok(HttpResponse::Ok().json(Response::<()> {
        status: "success".to_string(),
        message: format!(
            "Database populated with {} users, {} posts, {} comments, {} likes, and {} follows. All statistics updated.",
            test_users.len(),
            test_posts.len(),
            500, // comments
            1000, // likes
            400, // follows
        ),
        data: None,
    }))
}

// Clean Database Handler
pub async fn clean_database_handler(
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    info!("Cleaning all collections in the database");

    let collections = vec![
        ("users", "users collection"),
        ("posts", "posts collection"),
        ("comments", "comments collection"),
        ("likes", "likes collection"),
        ("follows", "follows collection"),
    ];

    for (collection_name, description) in collections {
        let collection = state.db.collection::<Document>(collection_name);
        if let Err(e) = collection.delete_many(doc! {}).await {
            error!("Error cleaning {}: {}", description, e);
            return Err(AppError::from(e));
        }
        info!("Successfully cleaned {}", description);
    }

    Ok(HttpResponse::Ok().json(Response::<()> {
        status: "success".to_string(),
        message: "All collections cleaned successfully".to_string(),
        data: None,
    }))
}

// Get All Posts Handler
pub async fn get_posts_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Fetching all posts");

    let collection = state.db.collection::<Document>("posts");
    let mut cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching posts: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut posts = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                posts.push(document);
            }
            Err(e) => {
                error!("Error parsing post document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!("Successfully fetched {} posts", posts.len());
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!("Successfully fetched {} posts", posts.len()),
        data: Some(posts),
    }))
}

// Get Post by ID Handler
pub async fn get_post_by_id_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let post_id = path.into_inner();
    info!("Fetching post with ID: {}", post_id);

    let collection = state.db.collection::<Document>("posts");
    let filter = doc! { "_id": &post_id };

    match collection.find_one(filter).await {
        Ok(Some(post)) => {
            info!("Successfully fetched post with ID: {}", post_id);
            Ok(HttpResponse::Ok().json(Response {
                status: "success".to_string(),
                message: "Post fetched successfully".to_string(),
                data: Some(post),
            }))
        }
        Ok(None) => {
            error!("Post with ID {} not found", post_id);
            Err(AppError::NotFound(format!(
                "Post with ID {} not found",
                post_id
            )))
        }
        Err(e) => {
            error!("Error fetching post: {}", e);
            Err(AppError::from(e))
        }
    }
}

// Get All Users Handler
pub async fn get_users_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Fetching all users");

    let collection = state.db.collection::<Document>("users");
    let mut cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching users: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut users = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                users.push(document);
            }
            Err(e) => {
                error!("Error parsing user document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!("Successfully fetched {} users", users.len());
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!("Successfully fetched {} users", users.len()),
        data: Some(users),
    }))
}

// Get User by ID Handler
pub async fn get_user_by_id_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let user_id = path.into_inner();
    info!("Fetching user with ID: {}", user_id);

    let collection = state.db.collection::<Document>("users");
    let filter = doc! { "_id": &user_id };

    match collection.find_one(filter).await {
        Ok(Some(user)) => {
            info!("Successfully fetched user with ID: {}", user_id);
            Ok(HttpResponse::Ok().json(Response {
                status: "success".to_string(),
                message: "User fetched successfully".to_string(),
                data: Some(user),
            }))
        }
        Ok(None) => {
            error!("User with ID {} not found", user_id);
            Err(AppError::NotFound(format!(
                "User with ID {} not found",
                user_id
            )))
        }
        Err(e) => {
            error!("Error fetching user: {}", e);
            Err(AppError::from(e))
        }
    }
}

// Get All Comments Handler
pub async fn get_comments_handler(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    info!("Fetching all comments");

    let collection = state.db.collection::<Document>("comments");
    let mut cursor = match collection.find(doc! {}).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching comments: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut comments = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                comments.push(document);
            }
            Err(e) => {
                error!("Error parsing comment document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!("Successfully fetched {} comments", comments.len());
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!("Successfully fetched {} comments", comments.len()),
        data: Some(comments),
    }))
}

// Get Comment by ID Handler
pub async fn get_comment_by_id_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let comment_id = path.into_inner();
    info!("Fetching comment with ID: {}", comment_id);

    let collection = state.db.collection::<Document>("comments");
    let filter = doc! { "_id": &comment_id };

    match collection.find_one(filter).await {
        Ok(Some(comment)) => {
            info!("Successfully fetched comment with ID: {}", comment_id);
            Ok(HttpResponse::Ok().json(Response {
                status: "success".to_string(),
                message: "Comment fetched successfully".to_string(),
                data: Some(comment),
            }))
        }
        Ok(None) => {
            error!("Comment with ID {} not found", comment_id);
            Err(AppError::NotFound(format!(
                "Comment with ID {} not found",
                comment_id
            )))
        }
        Err(e) => {
            error!("Error fetching comment: {}", e);
            Err(AppError::from(e))
        }
    }
}

// Get Comments by Post ID Handler
pub async fn get_comments_by_post_id_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let post_id = path.into_inner();
    info!("Fetching comments for post with ID: {}", post_id);

    let collection = state.db.collection::<Document>("comments");
    let filter = doc! { "post_id": &post_id };

    let mut cursor = match collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching comments for post: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut comments = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                comments.push(document);
            }
            Err(e) => {
                error!("Error parsing comment document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!(
        "Successfully fetched {} comments for post {}",
        comments.len(),
        post_id
    );
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!(
            "Successfully fetched {} comments for post {}",
            comments.len(),
            post_id
        ),
        data: Some(comments),
    }))
}

// Get Comments by User ID Handler
pub async fn get_comments_by_user_id_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let user_id = path.into_inner();
    info!("Fetching comments by user with ID: {}", user_id);

    let collection = state.db.collection::<Document>("comments");
    let filter = doc! { "user_id": &user_id };

    let mut cursor = match collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching comments for user: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut comments = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                comments.push(document);
            }
            Err(e) => {
                error!("Error parsing comment document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!(
        "Successfully fetched {} comments by user {}",
        comments.len(),
        user_id
    );
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!(
            "Successfully fetched {} comments by user {}",
            comments.len(),
            user_id
        ),
        data: Some(comments),
    }))
}

// Get Following Users Handler
pub async fn get_following_users_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let user_id = path.into_inner();
    info!("Fetching users followed by user with ID: {}", user_id);

    let collection = state.db.collection::<Document>("follows");
    let filter = doc! { "follower_id": &user_id };

    let mut cursor = match collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching followed users: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut following_users = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                // Extract the followed user ID and fetch their details from users collection
                if let Ok(following_id) = document.get_str("following_id") {
                    let users_collection = state.db.collection::<Document>("users");
                    if let Ok(Some(user_doc)) = users_collection
                        .find_one(doc! { "_id": following_id })
                        .await
                    {
                        following_users.push(user_doc);
                    }
                }
            }
            Err(e) => {
                error!("Error parsing follow document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!(
        "Successfully fetched {} followed users for user {}",
        following_users.len(),
        user_id
    );
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!(
            "Successfully fetched {} followed users",
            following_users.len()
        ),
        data: Some(following_users),
    }))
}

// Get Followers Users Handler
pub async fn get_followers_users_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let user_id = path.into_inner();
    info!("Fetching followers for user with ID: {}", user_id);

    let collection = state.db.collection::<Document>("follows");
    let filter = doc! { "following_id": &user_id };

    let mut cursor = match collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching followers: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut followers = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                // Extract the follower ID and fetch their details from users collection
                if let Ok(follower_id) = document.get_str("follower_id") {
                    let users_collection = state.db.collection::<Document>("users");
                    if let Ok(Some(user_doc)) =
                        users_collection.find_one(doc! { "_id": follower_id }).await
                    {
                        followers.push(user_doc);
                    }
                }
            }
            Err(e) => {
                error!("Error parsing follow document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!(
        "Successfully fetched {} followers for user {}",
        followers.len(),
        user_id
    );
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!("Successfully fetched {} followers", followers.len()),
        data: Some(followers),
    }))
}

// Get Posts by User ID Handler
pub async fn get_posts_by_user_id_handler(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let user_id = path.into_inner();
    info!("Fetching posts for user with ID: {}", user_id);

    // First verify that the user exists
    let users_collection = state.db.collection::<Document>("users");
    match users_collection.find_one(doc! { "_id": &user_id }).await {
        Ok(None) => {
            error!("User with ID {} not found", user_id);
            return Err(AppError::NotFound(format!(
                "User with ID {} not found",
                user_id
            )));
        }
        Err(e) => {
            error!("Error verifying user existence: {}", e);
            return Err(AppError::from(e));
        }
        _ => {} // User exists, continue
    }

    let collection = state.db.collection::<Document>("posts");
    let filter = doc! { "user_id": &user_id };

    let mut cursor = match collection.find(filter).await {
        Ok(cursor) => cursor,
        Err(e) => {
            error!("Error fetching posts for user: {}", e);
            return Err(AppError::from(e));
        }
    };

    let mut posts = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                posts.push(document);
            }
            Err(e) => {
                error!("Error parsing post document: {}", e);
                return Err(AppError::from(e));
            }
        }
    }

    info!(
        "Successfully fetched {} posts for user {}",
        posts.len(),
        user_id
    );
    Ok(HttpResponse::Ok().json(Response {
        status: "success".to_string(),
        message: format!(
            "Successfully fetched {} posts for user {}",
            posts.len(),
            user_id
        ),
        data: Some(posts),
    }))
}

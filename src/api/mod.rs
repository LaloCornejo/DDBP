pub mod routes;
pub mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use routes::*;
use handlers::*;

pub fn create_router(db_pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/posts", get(handlers::read::get_posts))
        .route("/posts", post(handlers::write::create_post))
        .route("/posts/:id", get(handlers::read::get_post))
        .route("/sync", post(handlers::write::sync_data))
        .route("/nodes", get(handlers::read::get_nodes))
        .route("/nodes", post(handlers::write::register_node))
        .with_state(db_pool)
}

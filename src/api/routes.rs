use axum::{Json, extract::State};
use sqlx::PgPool;
use serde_json::json;

pub async fn health_check(State(_pool): State<PgPool>) -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

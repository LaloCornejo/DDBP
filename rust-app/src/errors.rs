use actix_web::{error::ResponseError, HttpResponse};
// use serde::Serialize;
use std::fmt;
use tracing::{error, info};

use crate::models::Response;

#[derive(Debug)]
pub enum AppError {
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

use serde::{Deserialize, Serialize};
// use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub profile_picture_url: Option<String>,
    #[serde(skip_deserializing)]
    pub join_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PostType {
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
pub struct Post {
    pub user_id: String,
    pub content: String,
    #[serde(default)]
    pub media_urls: Vec<String>,
    #[serde(default)]
    pub post_type: PostType,
    #[serde(default)]
    pub like_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    pub post_id: String,
    pub user_id: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Follow {
    pub follower_id: String,
    pub following_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Like {
    pub post_id: String,
    pub user_id: String,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub bio: Option<String>,
    pub profile_picture_url: Option<String>,
    pub join_date: String,
    pub follower_count: i32,
    pub following_count: i32,
    pub post_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostDetails {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub profile_picture_url: Option<String>,
    pub content: String,
    pub media_urls: Vec<String>,
    pub post_type: PostType,
    pub created_at: String,
    pub human_time: String,
    pub like_count: i32,
    pub comment_count: i32,
    pub has_liked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimelineResponse {
    pub posts: Vec<PostDetails>,
    pub pagination: PaginationMeta,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaginationMeta {
    pub current_page: u32,
    pub total_pages: u32,
    pub total_count: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserStats {
    pub post_count: i32,
    pub comment_count: i32,
    pub follower_count: i32,
    pub following_count: i32,
    pub total_likes_received: i32,
    pub total_likes_given: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T = String> {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

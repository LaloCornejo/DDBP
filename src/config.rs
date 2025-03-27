use std::env;

#[derive(Clone)]
pub struct Config {
    pub node_id: String,
    pub host: String,
    pub port: String,
    pub database_url: String,
    pub database_urls: Vec<String>,
    pub node_urls: Vec<String>, // Add this for HTTP URLs
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let node_id = env::var("NODE_ID")?;
        let host = env::var("HOST")?;
        let port = env::var("PORT")?;
        let database_url = env::var("DATABASE_URL")?;
        let database_urls = env::var("DATABASE_URLS")?
            .split(',')
            .map(|s| s.to_string())
            .collect();
        let node_urls = env::var("NODE_URLS")?
            .split(',')
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            node_id,
            host,
            port,
            database_url,
            database_urls,
            node_urls,
        })
    }
}

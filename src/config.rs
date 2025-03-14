use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String, 
    pub port: u16,
    pub node_id: String, 
    pub cluster_nodes: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap(),
            node_id: env::var("NODE_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            cluster_nodes: env::var("CLUSTER_NODES")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        })
    }
}

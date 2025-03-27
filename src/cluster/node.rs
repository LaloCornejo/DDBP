use reqwest::Client;
use crate::db::models::Node;
use chrono::Utc;

pub async fn register_with_node(self_id: &str, self_url: &str, node_url: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    
    let node = Node {
        id: self_id.to_string(),
        url: self_url.to_string(),
        last_seen: Utc::now(),
    };
    
    client.post(format!("{}/nodes", node_url))
        .json(&node)
        .send()
        .await?;
        
    Ok(())
}

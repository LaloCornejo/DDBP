use reqwest::Client;

pub async fn register_with_node(self_id: &str, self_url: &str, node_url: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.post(format!("{}/register", node_url))
        .json(&serde_json::json!({ "id": self_id, "url": self_url }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("Successfully registered with node: {}", node_url);
    } else {
        println!("Failed to register with node: {}", node_url);
    }

    Ok(())
}

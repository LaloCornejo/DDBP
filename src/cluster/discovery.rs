use reqwest::Client;
use crate:: {
    db::models::Node,
    config::Config,
};
use std::time::Duration;

pub async fn start_discovery_service(config: Config){
    let client = Client::new();

    let self_url = format!("http://{}:{}", config.host, config.port);

    loop {
        for node_url in &config.cluster_nodes {
            // Register each know node 
            let _ = super::node::register_with_node(&config.node_id, &self_url, node_url).await;

            // Get all known nodes 
            if let Ok(response) = client.get(format!("{}/nodes", node_url))
                .send()
                    .await {
                        if let Ok(nodes) = response.json::<Vec<Node>>().await {
                            println!("Discovered {} nodes", nodes.len());
                        }
                    }
        }

        // Wait for a bit before trying again
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

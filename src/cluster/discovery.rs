use crate::config::Config;
use crate::cluster::node::register_with_node;
use tokio::time::{sleep, Duration};
use tracing::info;

pub async fn start_discovery_service(config: Config) {
    loop {
        for node_url in &config.cluster_nodes {
            match register_with_node(&config.node_id, &config.host, &node_url).await {
                Ok(_) => info!("Successfully registered with node: {}", node_url),
                Err(err) => info!("Failed to register with node: {}. Error: {:?}", node_url, err),
            }
        }
        sleep(Duration::from_secs(60)).await; // Run every 60 seconds
    }
}

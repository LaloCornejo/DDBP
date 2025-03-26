use crate::config::Config;
use crate::cluster::node::register_with_node;
use tokio::time::{sleep, Duration};
use tracing::info;

pub async fn start_discovery_service(config: Config) {
    loop {
        for node_url in &config.cluster_nodes {
            let node_id = config.node_id.clone();
            let host = config.host.clone();
            let node_url = node_url.to_owned();
            
            tokio::spawn(async move {
                if let Err(err) = register_with_node(&node_id, &host, &node_url).await {
                    info!("Failed to register with node: {}. Error: {:?}", node_url, err);
                }
            });
        }
        sleep(Duration::from_secs(60)).await; // Run every 60 seconds
    }
}

use crate::config::Config;
//use crate::cluster::node::register_with_node;
use tokio::time::Duration;
use reqwest::Client;
use crate::db::models::Node;

pub async fn start_discovery_service(config: Config){
    let client = Client::new();

    let self_url = format!("http://{}:{}", config.host, config.port);

    loop {
        for node_url in &config.cluster_nodes {
            let _ = super::node::register_with_node(&config.node_id, &self_url, node_url).await;

            if let Ok(response) = client.get(format!("{}/nodes", node_url))
                .send()
                    .await {
                        if let Ok(nodes) = response.json::<Vec<Node>>().await {
                            println!("Discovered {} nodes", nodes.len());
                        }
                    }
        }

        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

use reqwest::Client;
use crate ::db::models::Post;

pub async fn async_post_to_nodes(post: Post, nodes: Vec<String>) {
    let client = Client::new();

    for node_url in nodes {
        let sync_url = format!("{}/sync", node_url);

        let _ = client.post(sync_url)
            .json(&vec![post.clone()])
            .send()
            .await;
    }
}

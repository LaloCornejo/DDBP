// src/main.rs
use mongodb::{Client, options::ClientOptions};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // MongoDB server addresses from Podman
    let mongo_servers = vec![
        ("central-mongodb", "127.0.0.1"),
        ("secondary-mongodb-1", "10.88.0.9"),
        ("secondary-mongodb-2", "10.88.0.10"),
    ];
    
    // Process command-line arguments to determine which MongoDB to connect to
    let args: Vec<String> = env::args().collect();
    let server_name = if args.len() > 1 {
        &args[1]
    } else {
        // Default to central if no argument is provided
        "central-mongodb"
    };
    
    // Find the IP address for the requested server
    let ip_address = mongo_servers
        .iter()
        .find(|(name, _)| *name == server_name)
        .map(|(_, ip)| *ip)
        .unwrap_or_else(|| {
            eprintln!("Server '{}' not found. Using central-mongodb.", server_name);
            "127.0.0.1" // Default to central MongoDB
        });
    
    println!("Connecting to MongoDB at {}...", ip_address);
    
    // Create a MongoDB client with the connection string
    let connection_string = format!("mongodb://{}:27017", ip_address);
    let client_options = ClientOptions::parse(&connection_string).await?;
    let client = Client::with_options(client_options)?;
    
    // Ping the database to verify connection
    client
        .database("admin")
        .run_command(mongodb::bson::doc! {"ping": 1}, None)
        .await?;
    
    println!("Successfully connected to MongoDB at {}", ip_address);
    
    // List available databases
    println!("Available databases:");
    let database_names = client.list_database_names(None, None).await?;
    for db_name in database_names {
        println!("- {}", db_name);
    }
    
    Ok(())
}

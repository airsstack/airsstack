// Layer 1: Standard library imports
use std::env;

// Layer 2: Third-party crate imports
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

// Layer 3: Internal module imports
mod config;
mod client;

use config::ClientConfig;
use client::StdioMcpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
    
    info!("Starting STDIO MCP Client Integration Demo");
    
    // Load configuration
    let config = if env::var("USE_MOCK").is_ok() {
        info!("Using mock server configuration");
        ClientConfig::for_mock_server()
    } else {
        info!("Loading configuration from environment");
        ClientConfig::from_env()
    };
    
    info!("Client configuration: {:?}", config);
    
    // Create and run the client
    match StdioMcpClient::new(&config).await {
        Ok(mut client) => {
            info!("Client created successfully");
            
            if let Err(e) = client.run_demo().await {
                error!("Demo failed: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to create client: {}", e);
            std::process::exit(1);
        }
    }
    
    info!("STDIO MCP Client Demo completed successfully");
    Ok(())
}
//! HTTP OAuth2 Client Integration Example
//!
//! Demonstrates OAuth2 authorization code flow with PKCE and MCP operations

// Layer 3: Internal library imports (from lib.rs only)
use http_oauth2_client_integration::{args, Config, FlowOrchestrator};

/// HTTP OAuth2 Client - Demonstrates OAuth2 authorization code flow with PKCE
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    // Parse command line arguments and create configuration
    let matches = args::parse_args();
    let config = Config::from_args(&matches);

    // Create and run the flow orchestrator
    let orchestrator = FlowOrchestrator::new(config);
    orchestrator.run().await
}

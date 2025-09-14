//! OAuth2 MCP Integration Server
//!
//! This is the main entry point for the OAuth2 MCP Integration example.
//! It demonstrates complete OAuth2 authentication and authorization integration
//! with the AirsStack MCP HTTP transport server.
//!
//! Features:
//! - Real OAuth2 JWT token validation using JWKS
//! - Scope-based authorization for MCP methods
//! - Mock JWKS endpoint for testing
//! - MCP-compliant JSON-RPC over HTTP
//! - Test token generation for different scenarios
//! - Complete OAuth2 flow: Authentication → Authorization → Execution
//!
//! Usage:
//!   cargo run --bin oauth2-mcp-server
//!
//! Test with MCP Inspector:
//!   # 1. Get test tokens
//!   curl http://localhost:3002/auth/tokens | jq
//!   
//!   # 2. Use with MCP Inspector
//!   npx @modelcontextprotocol/inspector-cli \
//!     --transport http \
//!     --server-url http://localhost:3001/mcp \
//!     --header "Authorization: Bearer <test-token>"

use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use tracing::info;

// Layer 3: Internal module imports
use airs_mcp::{
    authentication::strategies::oauth2::OAuth2Strategy,
    integration::McpServer,
    oauth2::validator::{Jwt, Scope, Validator},
    transport::adapters::http::{
        auth::{middleware::HttpAuthConfig, oauth2::OAuth2StrategyAdapter},
        axum::AxumHttpServer,
        config::HttpTransportConfig,
        connection_manager::{HealthCheckConfig, HttpConnectionManager},
        HttpTransportBuilder,
    },
};

mod auth_flow;
mod config;
mod jwks;
mod server;
mod tokens;

use config::create_oauth2_config;
use jwks::start_mock_jwks_server;
use server::create_test_environment;
use tokens::TestKeys;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize comprehensive logging for OAuth2 debugging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| {
                    "oauth2_mcp_integration=debug,airs_mcp=debug,oauth2=debug,jsonwebtoken=debug,axum=info".to_string()
                })
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting OAuth2 MCP Integration Server");

    // Generate test keys for JWT operations
    let test_keys = TestKeys::generate()?;

    // Create OAuth2 configuration
    let oauth2_config = create_oauth2_config()?;

    // Start mock JWKS server for JWT validation with proper config
    start_mock_jwks_server(test_keys.clone(), oauth2_config.clone()).await?;

    // Create OAuth2 validators and strategy
    let jwt_validator = Jwt::new(oauth2_config.clone())?;
    let scope_validator = Scope::with_default_mappings();
    let validator = Validator::new(jwt_validator, scope_validator);
    let oauth2_strategy = OAuth2Strategy::new(validator);
    let oauth2_adapter = OAuth2StrategyAdapter::new(oauth2_strategy);

    // Create HTTP authentication configuration
    let auth_config = HttpAuthConfig {
        include_error_details: true,
        auth_realm: "OAuth2 MCP Integration Server".to_string(),
        request_timeout_secs: 30,
        skip_paths: vec![
            "/health".to_string(),
            "/status".to_string(),
            "/metrics".to_string(),
            "/info".to_string(),
        ],
    };

    // Create test environment and MCP handlers
    let (handlers, _temp_dir) = create_test_environment().await?;

    // Create server infrastructure
    let connection_manager = Arc::new(HttpConnectionManager::new(
        1000,
        HealthCheckConfig::default(),
    ));

    // Create HTTP transport configuration
    let transport_config = HttpTransportConfig::new()
        .bind_address("127.0.0.1:3001".parse()?)
        .max_connections(1000)
        .request_timeout(Duration::from_secs(30))
        .enable_buffer_pool()
        .buffer_pool_size(100);

    // Create the OAuth2-enabled MCP server using the complete OAuth2 integration pattern
    // This provides both JWT authentication AND scope-based authorization in one step
    let mut engine = AxumHttpServer::from_parts(connection_manager, transport_config.clone())?
        .with_oauth2_authorization(oauth2_adapter, auth_config);

    // Register the custom MCP handler with specific provider types
    // This uses register_custom_mcp_handler which accepts any McpRequestHandler implementation
    engine.register_custom_mcp_handler(handlers);

    // Use the TransportBuilder pattern to create a properly configured transport
    let transport = HttpTransportBuilder::with_engine(engine)?
        .bind(transport_config.bind_address)
        .await?
        .build()
        .await?;

    // Create MCP server with the properly configured transport
    let mcp_server = McpServer::new(transport);

    // Print server information
    print_server_info();

    // Start the MCP server using the integrated McpServer wrapper
    info!("🚀 Starting OAuth2 MCP Server with integrated transport...");
    mcp_server.start().await?;

    info!("✅ OAuth2 MCP Server running successfully!");
    info!("🌐 Ready to accept MCP requests at http://127.0.0.1:3001/mcp");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("🛑 Shutdown signal received, stopping server...");
    mcp_server.shutdown().await?;

    info!("👋 OAuth2 MCP Server shutdown complete");
    Ok(())
}

fn print_server_info() {
    info!("✅ OAuth2 MCP Server configured successfully with complete OAuth2 integration");
    info!("🔒 Authentication: JWT token validation with JWKS");
    info!("🔐 Authorization: Scope-based method authorization");
    info!("🌐 Server will bind to: http://127.0.0.1:3001");
    info!("📡 MCP Endpoint: http://127.0.0.1:3001/mcp (OAuth2 protected)");
    info!("🏥 Health Check: http://127.0.0.1:3001/health");
    info!("📊 Server Status: http://127.0.0.1:3001/status");
    info!("📈 Server Metrics: http://127.0.0.1:3001/metrics");
    info!("🎫 Test Tokens: http://127.0.0.1:3002/auth/tokens");
    info!("🔑 JWKS Endpoint: http://127.0.0.1:3002/.well-known/jwks.json");
    info!("");
    info!("🔐 OAuth2 Configuration:");
    info!("   • Audience: mcp-server");
    info!("   • Issuer: https://example.com");
    info!("   • Algorithms: RS256");
    info!("   • JWKS URL: http://localhost:3002/.well-known/jwks.json");
    info!("   • Authorization: Scope-based method authorization");
    info!("");
    info!("🛡️ Scope-Based Authorization:");
    info!("   • tools/list → requires 'mcp:tools:*' or 'mcp:*' scope");
    info!("   • tools/call → requires 'mcp:tools:*' or 'mcp:*' scope");
    info!("   • resources/list → requires 'mcp:resources:*' or 'mcp:*' scope");
    info!("   • resources/read → requires 'mcp:resources:*' or 'mcp:*' scope");
    info!("   • prompts/list → requires 'mcp:prompts:*' or 'mcp:*' scope");
    info!("   • prompts/get → requires 'mcp:prompts:*' or 'mcp:*' scope");
    info!("");
    info!("🎯 Testing Instructions:");
    info!("   1. Get test tokens: curl http://localhost:3002/auth/tokens");
    info!("   2. Copy a token for your test scenario");
    info!("   3. Use MCP Inspector with OAuth2:");
    info!("      npx @modelcontextprotocol/inspector-cli \\");
    info!("        --transport http \\");
    info!("        --server-url http://localhost:3001/mcp \\");
    info!("        --header \"Authorization: Bearer <your-token>\"");
    info!("");
    info!("📋 Available Test Scenarios:");
    info!("   • full: Complete access to all MCP operations");
    info!("   • tools: Access to tools operations only");
    info!("   • resources: Access to resources operations only");
    info!("   • readonly: Read-only access to listings");
    info!("");
    info!("🔍 MCP Methods & Required Scopes:");
    info!("   • initialize - Start MCP session (no scope required)");
    info!("   • resources/list - List resources (requires 'mcp:resources:*')");
    info!("   • resources/read - Read resource contents (requires 'mcp:resources:*')");
    info!("   • tools/list - List available tools (requires 'mcp:tools:*')");
    info!("   • tools/call - Execute tools (requires 'mcp:tools:*')");
    info!("   • prompts/list - List available prompts (requires 'mcp:prompts:*')");
    info!("   • prompts/get - Get prompt content (requires 'mcp:prompts:*')");
    info!("");
    info!("Press Ctrl+C to shutdown...");
}

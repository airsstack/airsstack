//! OAuth2 MCP Remote Server - AirsStack Integration Example
//!
//! This example demonstrates OAuth2 authentication integration with AirsStack's
//! MCP Transport infrastructure. It shows how to use:
//!
//! - AxumHttpServer from airs_mcp::transport::adapters::http
//! - OAuth2Strategy from airs_mcp::authentication::strategies::oauth2
//! - OAuth2StrategyAdapter for HTTP transport authentication
//! - AirsStack MCP providers (filesystem, tools, prompts)
//! - Complete OAuth2 JWT validation with JWKS
//!
//! Usage:
//!   cargo run
//!
//! Test with MCP Inspector:
//!   # 1. Get test tokens
//!   curl http://localhost:3003/auth/tokens
//!   
//!   # 2. Use with MCP Inspector
//!   npx @modelcontextprotocol/inspector-cli \
//!     --transport http \
//!     --server-url http://localhost:3001/mcp \
//!     --header "Authorization: Bearer <test-token>"

use tracing::{info, error};
use tracing_subscriber::EnvFilter;
use url::Url;

// Import our modular components
use mcp_remote_server_oauth2::{
    auth::{keys::generate_test_keys, setup::OAuth2Setup},
    config::server::ServerConfig,
    testing::{jwks::MockJwksServer, endpoints::TokenEndpoints},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("RUST_LOG").add_directive("mcp_remote_server_oauth2=info".parse()?))
        .init();

    info!("🚀 Starting OAuth2 MCP Remote Server with AirsStack Integration");

    // Step 1: Generate RSA keys for JWT operations
    let test_keys = generate_test_keys().map_err(|e| {
        error!("Failed to generate test keys: {}", e);
        e
    })?;

    // Step 2: Start mock JWKS server for JWT validation
    MockJwksServer::start(test_keys.clone()).await.map_err(|e| {
        error!("Failed to start JWKS server: {}", e);
        e
    })?;

    // Step 3: Start token generation endpoints for testing
    TokenEndpoints::start(
        test_keys.clone(),
        "mcp-oauth2-remote-server".to_string(),
        "oauth2-mcp-remote-issuer".to_string(),
    ).await.map_err(|e| {
        error!("Failed to start token endpoints: {}", e);
        e
    })?;

    // Step 4: Set up OAuth2 authentication using AirsStack components
    let jwks_url = Url::parse("http://localhost:3002/.well-known/jwks.json")?;
    let oauth2_setup = OAuth2Setup::new(jwks_url).map_err(|e| {
        error!("Failed to setup OAuth2 authentication: {}", e);
        e
    })?;

    // Step 5: Create temporary directory for filesystem provider
    let temp_dir = tempfile::TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create test files for the filesystem provider
    std::fs::write(
        temp_path.join("oauth2-remote-test.txt"), 
        "Hello from OAuth2 protected AirsStack MCP server!"
    )?;
    std::fs::write(
        temp_path.join("config.json"), 
        r#"{"server": "oauth2-mcp-remote", "version": "1.0.0", "auth": "oauth2", "transport": "AxumHttpServer"}"#
    )?;
    std::fs::write(
        temp_path.join("README.md"),
        "# OAuth2 MCP Remote Server\n\nThis file is accessible through OAuth2 protected AirsStack MCP resources."
    )?;

    // Step 6: Configure AirsStack MCP server components
    let server_config = ServerConfig::new(temp_path).await.map_err(|e| {
        error!("Failed to create server configuration: {}", e);
        e
    })?;

    // Step 7: Create the OAuth2-enabled AxumHttpServer
    let server = server_config.create_server(oauth2_setup).await.map_err(|e| {
        error!("Failed to create OAuth2-enabled MCP server: {}", e);
        e
    })?;

    // Log server information
    info!("✅ OAuth2 MCP Remote Server configured successfully");
    info!("🌐 Server will bind to: http://127.0.0.1:3001");
    info!("📡 MCP Endpoint: http://127.0.0.1:3001/mcp (OAuth2 protected)");
    info!("🏥 Health Check: http://127.0.0.1:3001/health");
    info!("📊 Server Status: http://127.0.0.1:3001/status");
    info!("📈 Server Metrics: http://127.0.0.1:3001/metrics");
    info!("🎫 Test Tokens: http://127.0.0.1:3003/auth/tokens");
    info!("🔑 JWKS Endpoint: http://127.0.0.1:3002/.well-known/jwks.json");
    info!("");
    info!("🔐 OAuth2 Configuration:");
    info!("   • Audience: mcp-oauth2-remote-server");
    info!("   • Issuer: oauth2-mcp-remote-issuer");
    info!("   • Algorithms: RS256");
    info!("   • JWKS URL: http://localhost:3002/.well-known/jwks.json");
    info!("");
    info!("🏗️ AirsStack Integration:");
    info!("   • Transport: AxumHttpServer");
    info!("   • Authentication: OAuth2Strategy + OAuth2StrategyAdapter");
    info!("   • Providers: FileSystemResourceProvider, MathToolProvider, CodeReviewPromptProvider");
    info!("   • Infrastructure: CorrelationManager, SessionManager, ConcurrentProcessor");
    info!("");
    info!("🎯 Testing Instructions:");
    info!("   1. Get test tokens: curl http://localhost:3003/auth/tokens");
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
    info!("🔍 MCP Methods Available:");
    info!("   • initialize - Start MCP session");
    info!("   • resources/list - List OAuth2 protected resources");
    info!("   • resources/read - Read OAuth2 protected resources");
    info!("   • tools/list - List OAuth2 protected tools");
    info!("   • tools/call - Execute OAuth2 protected tools");
    info!("   • prompts/list - List OAuth2 protected prompts");
    info!("   • prompts/get - Get OAuth2 protected prompt content");
    info!("");
    info!("Press Ctrl+C to shutdown...");

    // Step 8: Start the AirsStack MCP server
    let addr = "127.0.0.1:3001".parse()?;
    let mut server = server;
    server.bind(addr).await.map_err(|e| {
        error!("Failed to bind server to {}: {}", addr, e);
        e
    })?;
    
    info!("🎉 OAuth2 MCP Server started successfully - ready for testing!");
    
    server.serve().await.map_err(|e| {
        error!("Server error: {}", e);
        e
    })?;

    info!("👋 OAuth2 MCP Server shutdown complete");
    Ok(())
}

//! MCP session management with OAuth2 authentication

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use tracing::{info, warn};

// Layer 3: Internal module imports
use airs_mcp::integration::McpClientBuilder;
use airs_mcp::transport::adapters::http::{AuthMethod, HttpTransportClientBuilder};
use crate::oauth2::OAuth2IntegrationError;

/// MCP session manager with OAuth2 authentication
pub struct McpSession {
    mcp_client: airs_mcp::integration::McpClient<airs_mcp::transport::adapters::http::HttpTransportClient>,
}

impl McpSession {
    /// Create a new MCP session with OAuth2 authentication
    pub async fn new(mcp_server_url: &str, access_token: &str) -> Result<Self, OAuth2IntegrationError> {
        info!("🔧 Creating MCP client with OAuth2 authentication...");
        
        // Create HTTP transport client with OAuth2 authentication
        let transport = HttpTransportClientBuilder::new()
            .endpoint(mcp_server_url)
            .map_err(|e| OAuth2IntegrationError::NetworkError {
                message: format!("Failed to parse MCP server URL: {}", e),
            })?
            .auth(AuthMethod::OAuth2 {
                access_token: access_token.to_string(),
                token_type: Some("Bearer".to_string()),
            })
            .timeout(Duration::from_secs(30))
            .build()
            .await
            .map_err(|e| OAuth2IntegrationError::NetworkError {
                message: format!("Failed to create HTTP transport: {}", e),
            })?;

        // Create MCP client
        let mut mcp_client = McpClientBuilder::new()
            .client_info("oauth2-test-client", "1.0.0")
            .timeout(Duration::from_secs(60))
            .build(transport);

        // Initialize MCP session
        info!("🤝 Initializing MCP session...");
        let server_capabilities = mcp_client.initialize().await.map_err(|e| {
            OAuth2IntegrationError::NetworkError {
                message: format!("Failed to initialize MCP session: {}", e),
            }
        })?;

        info!("✅ MCP session initialized successfully");
        info!("🎯 Server capabilities: {:?}", server_capabilities);

        Ok(Self { mcp_client })
    }

    /// Get mutable reference to the MCP client
    pub fn client_mut(&mut self) -> &mut airs_mcp::integration::McpClient<airs_mcp::transport::adapters::http::HttpTransportClient> {
        &mut self.mcp_client
    }

    /// Close the MCP session
    pub async fn close(mut self) -> Result<(), OAuth2IntegrationError> {
        match self.mcp_client.close().await {
            Ok(_) => {
                info!("✅ MCP client closed successfully");
                Ok(())
            }
            Err(e) => {
                warn!("⚠️  Failed to close MCP client: {}", e);
                Err(OAuth2IntegrationError::NetworkError {
                    message: format!("Failed to close MCP client: {}", e),
                })
            }
        }
    }
}
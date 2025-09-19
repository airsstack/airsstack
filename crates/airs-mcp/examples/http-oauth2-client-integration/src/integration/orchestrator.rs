//! Main flow orchestration for OAuth2 + MCP integration

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio::time::sleep;
use tracing::info;

// Layer 3: Internal module imports
use crate::cli::Config;
use crate::mcp_client::McpOperations;
use crate::oauth2::{OAuth2Flow, OAuth2ClientConfig};
use crate::oauth2_client::{
    simulate_automatic_authorization, simulate_interactive_authorization, TokenManager,
};

/// Main flow orchestrator for OAuth2 + MCP integration
pub struct FlowOrchestrator {
    config: Config,
}

impl FlowOrchestrator {
    /// Create a new flow orchestrator
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run the complete OAuth2 + MCP integration flow
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔐 Starting HTTP OAuth2 Client Integration Example");

        // Initialize OAuth2 flow
        let oauth2_config = OAuth2ClientConfig {
            client_id: self.config.client_id.clone(),
            authorization_endpoint: format!("{}/authorize", self.config.auth_server_url),
            token_endpoint: format!("{}/token", self.config.auth_server_url),
            redirect_uri: "http://localhost:8082/callback".to_string(),
            scope: self.config.scope.clone(),
        };
        let oauth2_flow = OAuth2Flow::new(oauth2_config);
        let token_manager = TokenManager::new(oauth2_flow);

        // Step 1: Generate authorization URL
        info!("🚀 Step 1: Generating authorization URL...");
        let (auth_url, pkce_challenge) = token_manager.generate_authorization_url()?;
        info!("📝 Authorization URL: {}", auth_url);

        // Step 2: Simulate user authorization
        let auth_code = if self.config.interactive {
            simulate_interactive_authorization(&auth_url).await?
        } else {
            simulate_automatic_authorization(&auth_url, &self.config.client_id, &self.config.scope)
                .await?
        };

        info!("✅ Authorization code received: {}", auth_code.code);

        // Step 3: Exchange authorization code for tokens
        info!("🔄 Step 3: Exchanging authorization code for tokens...");
        let token_response = token_manager
            .exchange_code_for_tokens(&auth_code.code, &pkce_challenge)
            .await?;

        // Step 4: Test MCP operations with OAuth2 tokens
        info!("🛠️  Step 4: Testing MCP operations with OAuth2 authentication...");
        McpOperations::test_operations(&self.config.mcp_server_url, &token_response.access_token)
            .await?;

        // Step 5: Test token refresh (if refresh token available)
        if let Some(_refresh_token) = &token_response.refresh_token {
            info!("🔄 Step 5: Testing token refresh...");
            // Wait a bit to demonstrate refresh flow
            sleep(Duration::from_secs(2)).await;

            // For demo purposes, we'll skip the actual refresh since the method doesn't exist yet
            info!("⚠️  Token refresh not implemented in demo");
        }

        info!("🎉 OAuth2 integration example completed successfully!");
        Ok(())
    }
}

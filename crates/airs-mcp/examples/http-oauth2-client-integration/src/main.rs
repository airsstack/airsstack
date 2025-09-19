// Standard library imports
use std::time::Duration;

// Third-party crate imports
use chrono::{Duration as ChronoDuration, Utc};
use clap::{Arg, Command};
use tokio::time::sleep;
use tracing::{info, warn};
use url::Url;

// Internal module imports
use http_oauth2_client_integration::{
    oauth2::flow::OAuth2Flow,
    AuthorizationCode, OAuth2IntegrationError, OAuth2ClientConfig,
};

/// HTTP OAuth2 Client - Demonstrates OAuth2 authorization code flow with PKCE
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🔐 Starting HTTP OAuth2 Client Integration Example");

    let matches = Command::new("http-oauth2-client")
        .version("0.1.0")
        .author("AIRS Stack Contributors")
        .about("HTTP MCP client with OAuth2 authentication")
        .arg(
            Arg::new("auth-server")
                .long("auth-server")
                .value_name("URL")
                .help("OAuth2 authorization server URL")
                .default_value("http://localhost:8080"),
        )
        .arg(
            Arg::new("mcp-server")
                .long("mcp-server")
                .value_name("URL")
                .help("OAuth2-protected MCP server URL")
                .default_value("http://localhost:8081"),
        )
        .arg(
            Arg::new("client-id")
                .long("client-id")
                .value_name("ID")
                .help("OAuth2 client ID")
                .default_value("test-client"),
        )
        .arg(
            Arg::new("scope")
                .long("scope")
                .value_name("SCOPE")
                .help("OAuth2 scope to request")
                .default_value("mcp:read mcp:write"),
        )
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .help("Run in interactive mode with user consent simulation")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let auth_server_url = matches.get_one::<String>("auth-server").unwrap();
    let mcp_server_url = matches.get_one::<String>("mcp-server").unwrap();
    let client_id = matches.get_one::<String>("client-id").unwrap();
    let scope = matches.get_one::<String>("scope").unwrap();
    let interactive = matches.get_flag("interactive");

    info!("📋 Configuration:");
    info!("  Auth Server: {}", auth_server_url);
    info!("  MCP Server: {}", mcp_server_url);
    info!("  Client ID: {}", client_id);
    info!("  Scope: {}", scope);
    info!("  Interactive: {}", interactive);

    // Initialize OAuth2 flow
    let config = OAuth2ClientConfig {
        client_id: client_id.clone(),
        authorization_endpoint: format!("{}/authorize", auth_server_url),
        token_endpoint: format!("{}/token", auth_server_url),
        redirect_uri: "http://localhost:8082/callback".to_string(),
        scope: scope.clone(),
    };
    let oauth2_flow = OAuth2Flow::new(config);

    // Step 1: Generate authorization URL
    info!("🚀 Step 1: Generating authorization URL...");
    let (auth_url, pkce_challenge) = oauth2_flow.generate_authorization_url()?;
    info!("📝 Authorization URL: {}", auth_url);

    // Step 2: Simulate user authorization
    let auth_code = if interactive {
        simulate_interactive_authorization(&auth_url).await?
    } else {
        simulate_automatic_authorization(&auth_url, client_id, scope).await?
    };

    info!("✅ Authorization code received: {}", auth_code.code);

    // Step 3: Exchange authorization code for tokens
    info!("🔄 Step 3: Exchanging authorization code for tokens...");
    let token_response = oauth2_flow.exchange_code_for_tokens(&auth_code.code, &pkce_challenge).await?;
    
    info!("🎫 Tokens received:");
    info!("  Access Token: {}...", &token_response.access_token[..20]);
    if let Some(refresh_token) = &token_response.refresh_token {
        info!("  Refresh Token: {}...", &refresh_token[..20]);
    }
    info!("  Expires At: {}", token_response.expires_at);

    // Step 4: Test MCP operations with OAuth2 tokens
    info!("🛠️  Step 4: Testing MCP operations with OAuth2 authentication...");
    test_mcp_operations(mcp_server_url, &token_response.access_token).await?;

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

/// Simulate interactive authorization flow (user would visit URL in browser)
async fn simulate_interactive_authorization(auth_url: &str) -> Result<AuthorizationCode, OAuth2IntegrationError> {
    info!("🌐 Interactive Authorization Flow:");
    info!("  1. Open this URL in your browser: {}", auth_url);
    info!("  2. Complete the authorization flow");
    info!("  3. Copy the authorization code from the callback URL");
    
    // For demo purposes, simulate getting the code automatically
    sleep(Duration::from_secs(1)).await;
    
    // Extract state parameter for validation
    let url = Url::parse(auth_url).map_err(|e| OAuth2IntegrationError::NetworkError {
        message: format!("Invalid auth URL: {}", e),
    })?;
    
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| "demo-state".to_string());

    // Simulate authorization code response
    Ok(AuthorizationCode {
        code: "demo_auth_code_interactive".to_string(),
        client_id: "test-client".to_string(),
        redirect_uri: "http://localhost:8082/callback".to_string(),
        code_challenge: None,
        code_challenge_method: None,
        scope: "mcp:read mcp:write".to_string(),
        expires_at: Utc::now() + ChronoDuration::hours(1),
        state: Some(state),
    })
}

/// Simulate automatic authorization flow (for testing)
async fn simulate_automatic_authorization(
    auth_url: &str,
    client_id: &str,
    scope: &str,
) -> Result<AuthorizationCode, OAuth2IntegrationError> {
    info!("🤖 Automatic Authorization Flow (Demo Mode):");
    info!("  Simulating user consent for client: {}", client_id);
    info!("  Requested scope: {}", scope);
    
    // Make request to authorization endpoint
    let client = reqwest::Client::new();
    let response = client.get(auth_url).send().await.map_err(|e| {
        OAuth2IntegrationError::NetworkError {
            message: format!("Failed to contact authorization server: {}", e),
        }
    })?;

    if !response.status().is_success() {
        return Err(OAuth2IntegrationError::AuthorizationFailed {
            message: format!("Authorization server returned: {}", response.status()),
        });
    }

    // Extract state parameter for validation
    let url = Url::parse(auth_url).map_err(|e| OAuth2IntegrationError::NetworkError {
        message: format!("Invalid auth URL: {}", e),
    })?;
    
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| "demo-state".to_string());

    info!("✅ Authorization successful! Simulating callback...");
    
    // Simulate authorization code response
    Ok(AuthorizationCode {
        code: "demo_auth_code_automatic".to_string(),
        client_id: "test-client".to_string(),
        redirect_uri: "http://localhost:8082/callback".to_string(),
        code_challenge: None,
        code_challenge_method: None,
        scope: "mcp:read mcp:write".to_string(),
        expires_at: Utc::now() + ChronoDuration::hours(1),
        state: Some(state),
    })
}

/// Test MCP operations using OAuth2 access token
async fn test_mcp_operations(mcp_server_url: &str, access_token: &str) -> Result<(), OAuth2IntegrationError> {
    let client = reqwest::Client::new();
    
    // Test 1: List available tools
    info!("📋 Testing: List available tools");
    let tools_response = client
        .post(&format!("{}/tools/list", mcp_server_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }))
        .send()
        .await
        .map_err(|e| OAuth2IntegrationError::NetworkError {
            message: format!("Failed to list tools: {}", e),
        })?;

    if tools_response.status().is_success() {
        let tools_data: serde_json::Value = tools_response.json().await.map_err(|e| {
            OAuth2IntegrationError::NetworkError {
                message: format!("Failed to parse tools response: {}", e),
            }
        })?;
        info!("✅ Tools list: {}", tools_data);
    } else {
        warn!("⚠️  Tools list failed: {}", tools_response.status());
    }

    // Test 2: List available resources
    info!("📋 Testing: List available resources");
    let resources_response = client
        .post(&format!("{}/resources/list", mcp_server_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/list",
            "params": {}
        }))
        .send()
        .await
        .map_err(|e| OAuth2IntegrationError::NetworkError {
            message: format!("Failed to list resources: {}", e),
        })?;

    if resources_response.status().is_success() {
        let resources_data: serde_json::Value = resources_response.json().await.map_err(|e| {
            OAuth2IntegrationError::NetworkError {
                message: format!("Failed to parse resources response: {}", e),
            }
        })?;
        info!("✅ Resources list: {}", resources_data);
    } else {
        warn!("⚠️  Resources list failed: {}", resources_response.status());
    }

    // Test 3: Test protected operation
    info!("🔒 Testing: Protected operation");
    let protected_response = client
        .post(&format!("{}/test/protected", mcp_server_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "message": "Testing OAuth2 protected endpoint"
        }))
        .send()
        .await
        .map_err(|e| OAuth2IntegrationError::NetworkError {
            message: format!("Failed to access protected endpoint: {}", e),
        })?;

    if protected_response.status().is_success() {
        let protected_data: serde_json::Value = protected_response.json().await.map_err(|e| {
            OAuth2IntegrationError::NetworkError {
                message: format!("Failed to parse protected response: {}", e),
            }
        })?;
        info!("✅ Protected operation: {}", protected_data);
    } else {
        warn!("⚠️  Protected operation failed: {}", protected_response.status());
    }

    info!("🎯 MCP operations testing completed");
    Ok(())
}
//! OAuth2 authorization flow simulation for testing

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{Duration as ChronoDuration, Utc};
use reqwest;
use tokio::time::sleep;
use tracing::info;
use url::Url;

// Layer 3: Internal module imports
use crate::oauth2::{AuthorizationCode, OAuth2IntegrationError};

/// Simulate interactive authorization flow (user would visit URL in browser)
pub async fn simulate_interactive_authorization(auth_url: &str) -> Result<AuthorizationCode, OAuth2IntegrationError> {
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
pub async fn simulate_automatic_authorization(
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
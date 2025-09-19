//! Token management for OAuth2 flow

// Layer 2: Third-party crate imports
use tracing::info;

// Layer 3: Internal module imports
use crate::oauth2::{OAuth2Flow, OAuth2IntegrationError, PkceChallenge, TokenStore};

/// Token manager for OAuth2 flow operations
pub struct TokenManager {
    oauth2_flow: OAuth2Flow,
}

impl TokenManager {
    /// Create a new token manager with OAuth2 flow
    pub fn new(oauth2_flow: OAuth2Flow) -> Self {
        Self { oauth2_flow }
    }

    /// Generate authorization URL with PKCE challenge
    pub fn generate_authorization_url(&self) -> Result<(String, PkceChallenge), OAuth2IntegrationError> {
        self.oauth2_flow.generate_authorization_url()
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code_for_tokens(
        &self,
        authorization_code: &str,
        pkce_challenge: &PkceChallenge,
    ) -> Result<TokenStore, OAuth2IntegrationError> {
        let token_response = self.oauth2_flow.exchange_code_for_tokens(authorization_code, pkce_challenge).await?;
        
        self.log_token_info(&token_response);
        Ok(token_response)
    }

    /// Log token information (safely)
    fn log_token_info(&self, tokens: &TokenStore) {
        info!("🎫 Tokens received:");
        info!("  Access Token: {}...", &tokens.access_token[..20.min(tokens.access_token.len())]);
        if let Some(refresh_token) = &tokens.refresh_token {
            info!("  Refresh Token: {}...", &refresh_token[..20.min(refresh_token.len())]);
        }
        info!("  Expires At: {}", tokens.expires_at);
    }
}
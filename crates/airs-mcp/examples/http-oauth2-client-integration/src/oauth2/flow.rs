// Standard library imports
use std::collections::HashMap;

// Third-party crate imports
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::Value;
use url::Url;

// Internal module imports
use crate::{
    OAuth2ClientConfig, OAuth2IntegrationError, PkceChallenge, TokenResponse, TokenStore,
};
use super::pkce::PkceGenerator;
use super::tokens::TokenManager;

/// OAuth2 authorization code flow implementation
pub struct OAuth2Flow {
    config: OAuth2ClientConfig,
    client: Client,
    token_manager: TokenManager,
    pkce_generator: PkceGenerator,
}

impl OAuth2Flow {
    /// Create a new OAuth2 flow instance
    pub fn new(config: OAuth2ClientConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            token_manager: TokenManager::new(),
            pkce_generator: PkceGenerator::new(),
        }
    }

    /// Generate authorization URL with PKCE challenge
    pub fn generate_authorization_url(&self) -> Result<(String, PkceChallenge), OAuth2IntegrationError> {
        let pkce_challenge = self.pkce_generator.generate_challenge();
        
        let mut auth_url = Url::parse(&self.config.authorization_endpoint)
            .map_err(|_| OAuth2IntegrationError::Configuration {
                message: "Invalid authorization endpoint URL".to_string(),
            })?;

        auth_url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &self.config.client_id)
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("scope", &self.config.scope)
            .append_pair("code_challenge", &pkce_challenge.code_challenge)
            .append_pair("code_challenge_method", &pkce_challenge.code_challenge_method)
            .append_pair("state", &uuid::Uuid::new_v4().to_string());

        Ok((auth_url.to_string(), pkce_challenge))
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code_for_tokens(
        &self,
        authorization_code: &str,
        pkce_challenge: &PkceChallenge,
    ) -> Result<TokenStore, OAuth2IntegrationError> {
        let mut form_data = HashMap::new();
        form_data.insert("grant_type", "authorization_code");
        form_data.insert("code", authorization_code);
        form_data.insert("redirect_uri", &self.config.redirect_uri);
        form_data.insert("client_id", &self.config.client_id);
        form_data.insert("code_verifier", &pkce_challenge.code_verifier);

        let response = self.client
            .post(&self.config.token_endpoint)
            .form(&form_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuth2IntegrationError::OAuth2Flow {
                message: format!("Token exchange failed: {}", error_text),
            });
        }

        let token_response: TokenResponse = response.json().await?;
        
        let expires_at = Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64);
        
        Ok(TokenStore {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at,
            scope: token_response.scope.unwrap_or_else(|| self.config.scope.clone()),
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenStore, OAuth2IntegrationError> {
        let mut form_data = HashMap::new();
        form_data.insert("grant_type", "refresh_token");
        form_data.insert("refresh_token", refresh_token);
        form_data.insert("client_id", &self.config.client_id);

        let response = self.client
            .post(&self.config.token_endpoint)
            .form(&form_data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuth2IntegrationError::OAuth2Flow {
                message: format!("Token refresh failed: {}", error_text),
            });
        }

        let token_response: TokenResponse = response.json().await?;
        
        let expires_at = Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64);
        
        Ok(TokenStore {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.or_else(|| Some(refresh_token.to_string())),
            expires_at,
            scope: token_response.scope.unwrap_or_else(|| self.config.scope.clone()),
        })
    }

    /// Perform complete OAuth2 flow simulation for testing
    pub async fn simulate_complete_flow(&self) -> Result<TokenStore, OAuth2IntegrationError> {
        // Generate authorization URL and PKCE challenge
        let (auth_url, pkce_challenge) = self.generate_authorization_url()?;
        
        tracing::info!("Generated authorization URL: {}", auth_url);
        
        // In a real implementation, the user would visit this URL and authorize
        // For this simulation, we'll directly simulate the authorization response
        let authorization_code = self.simulate_authorization_response(&auth_url).await?;
        
        // Exchange the authorization code for tokens
        self.exchange_code_for_tokens(&authorization_code, &pkce_challenge).await
    }

    /// Simulate authorization server response (for testing)
    async fn simulate_authorization_response(&self, auth_url: &str) -> Result<String, OAuth2IntegrationError> {
        // Parse the authorization URL to extract parameters
        let url = Url::parse(auth_url)
            .map_err(|_| OAuth2IntegrationError::OAuth2Flow {
                message: "Invalid authorization URL".to_string(),
            })?;

        let query_params: HashMap<String, String> = url.query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        // Simulate calling the authorization endpoint
        let response = self.client
            .get(auth_url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OAuth2IntegrationError::OAuth2Flow {
                message: format!("Authorization request failed: {}", response.status()),
            });
        }

        // In this simulation, the mock server should return an authorization code
        // This would normally be extracted from a redirect URL in a real implementation
        let response_text = response.text().await?;
        
        // Try to parse as JSON to get the authorization code
        if let Ok(json_response) = serde_json::from_str::<Value>(&response_text) {
            if let Some(code) = json_response.get("authorization_code") {
                if let Some(code_str) = code.as_str() {
                    return Ok(code_str.to_string());
                }
            }
        }

        // If JSON parsing fails, look for code in the response text
        // This is a fallback for different response formats
        if response_text.contains("authorization_code") {
            // Extract code from response - this is a simplified extraction
            if let Some(start) = response_text.find("\"authorization_code\":\"") {
                let start = start + 21; // Length of "authorization_code":""
                if let Some(end) = response_text[start..].find('"') {
                    return Ok(response_text[start..start + end].to_string());
                }
            }
        }

        Err(OAuth2IntegrationError::OAuth2Flow {
            message: "Could not extract authorization code from response".to_string(),
        })
    }

    /// Get current token store
    pub fn get_token_store(&self) -> Option<&TokenStore> {
        self.token_manager.get_current_tokens()
    }

    /// Set token store
    pub fn set_token_store(&mut self, tokens: TokenStore) {
        self.token_manager.set_tokens(tokens);
    }

    /// Check if current tokens are valid and not expired
    pub fn has_valid_tokens(&self) -> bool {
        self.token_manager.has_valid_tokens()
    }

    /// Get access token for API requests
    pub async fn get_access_token(&mut self) -> Result<String, OAuth2IntegrationError> {
        // Check if we have valid tokens
        if self.token_manager.has_valid_tokens() {
            if let Some(tokens) = self.token_manager.get_current_tokens() {
                return Ok(tokens.access_token.clone());
            }
        }

        // Check if we can refresh the token
        if let Some(tokens) = self.token_manager.get_current_tokens() {
            if let Some(refresh_token) = &tokens.refresh_token {
                tracing::info!("Refreshing access token");
                let new_tokens = self.refresh_access_token(refresh_token).await?;
                self.token_manager.set_tokens(new_tokens.clone());
                return Ok(new_tokens.access_token);
            }
        }

        // No valid tokens and no refresh token - need to perform full OAuth2 flow
        Err(OAuth2IntegrationError::AuthenticationRequired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_flow_creation() {
        let config = OAuth2ClientConfig::default();
        let flow = OAuth2Flow::new(config);
        assert!(!flow.has_valid_tokens());
    }

    #[test]
    fn test_authorization_url_generation() {
        let config = OAuth2ClientConfig::default();
        let flow = OAuth2Flow::new(config);
        
        let result = flow.generate_authorization_url();
        assert!(result.is_ok());
        
        let (auth_url, _pkce) = result.unwrap();
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("client_id=test-client"));
        assert!(auth_url.contains("code_challenge="));
        assert!(auth_url.contains("code_challenge_method=S256"));
    }
}
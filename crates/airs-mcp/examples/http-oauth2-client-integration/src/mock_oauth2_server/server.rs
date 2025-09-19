// OAuth2 server state management

// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// Third-party crate imports
use chrono::Utc;
use jsonwebtoken::EncodingKey;
use serde_json::Value;
use tokio::sync::RwLock;

// Internal module imports
use http_oauth2_client_integration::{
    AuthorizationCode, OAuth2IntegrationError, OAuth2ServerConfig, TokenClaims,
};

/// OAuth2 authorization server state
pub struct OAuth2ServerState {
    /// Server configuration
    pub config: OAuth2ServerConfig,
    /// JWT encoding key for token signing
    pub encoding_key: EncodingKey,
    /// JWKS response for public key distribution
    pub jwks_response: Value,
    /// Active authorization codes (code -> authorization data)
    pub authorization_codes: Arc<RwLock<HashMap<String, AuthorizationCode>>>,
    /// Active tokens (token -> claims)
    pub active_tokens: Arc<RwLock<HashMap<String, TokenClaims>>>,
    /// Registered clients (client_id -> client_data)
    pub registered_clients: HashMap<String, RegisteredClient>,
}

/// Registered OAuth2 client information
#[derive(Debug, Clone)]
pub struct RegisteredClient {
    pub client_id: String,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub name: String,
}

impl OAuth2ServerState {
    /// Create new OAuth2 server state
    pub async fn new(config: OAuth2ServerConfig) -> Result<Self, OAuth2IntegrationError> {
        // Load and parse the private key
        let encoding_key = EncodingKey::from_rsa_pem(config.private_key_pem.as_bytes())
            .map_err(|e| OAuth2IntegrationError::Configuration {
                message: format!("Failed to parse RSA private key: {}", e),
            })?;

        // Generate JWKS response
        let jwks_response = Self::generate_jwks_response(&config.private_key_pem)?;

        // Set up default registered clients
        let mut registered_clients = HashMap::new();
        
        // Default test client
        registered_clients.insert(
            "test-client".to_string(),
            RegisteredClient {
                client_id: "test-client".to_string(),
                redirect_uris: vec![
                    "http://localhost:8080/callback".to_string(),
                    "http://127.0.0.1:8080/callback".to_string(),
                ],
                allowed_scopes: vec![
                    "mcp:read".to_string(),
                    "mcp:write".to_string(),
                    "openid".to_string(),
                ],
                name: "Test MCP Client".to_string(),
            },
        );

        Ok(Self {
            config,
            encoding_key,
            jwks_response,
            authorization_codes: Arc::new(RwLock::new(HashMap::new())),
            active_tokens: Arc::new(RwLock::new(HashMap::new())),
            registered_clients,
        })
    }

    /// Generate JWKS response from private key
    fn generate_jwks_response(private_key_pem: &str) -> Result<Value, OAuth2IntegrationError> {
        // For demo purposes, return a mock JWKS response
        // In production, you would parse the RSA key and extract public key components
        let _key_info = private_key_pem; // Acknowledge the parameter
        
        let jwks_response = serde_json::json!({
            "keys": [
                {
                    "kty": "RSA",
                    "use": "sig",
                    "kid": "oauth2-mock-server-key",
                    "alg": "RS256",
                    "n": "demo_modulus_placeholder_for_testing_only",
                    "e": "AQAB"
                }
            ]
        });

        Ok(jwks_response)
    }

    /// Store an authorization code
    pub async fn store_authorization_code(&self, code: String, auth_data: AuthorizationCode) {
        let mut codes = self.authorization_codes.write().await;
        codes.insert(code, auth_data);
    }

    /// Get and remove an authorization code
    pub async fn consume_authorization_code(&self, code: &str) -> Option<AuthorizationCode> {
        let mut codes = self.authorization_codes.write().await;
        codes.remove(code)
    }

    /// Store an active token
    pub async fn store_active_token(&self, token: String, claims: TokenClaims) {
        let mut tokens = self.active_tokens.write().await;
        tokens.insert(token, claims);
    }

    /// Get token claims
    #[allow(dead_code)]
    pub async fn get_token_claims(&self, token: &str) -> Option<TokenClaims> {
        let tokens = self.active_tokens.read().await;
        tokens.get(token).cloned()
    }

    /// Validate client and redirect URI
    pub fn validate_client(&self, client_id: &str, redirect_uri: Option<&str>) -> Result<&RegisteredClient, OAuth2IntegrationError> {
        let client = self.registered_clients.get(client_id)
            .ok_or_else(|| OAuth2IntegrationError::OAuth2Flow {
                message: format!("Invalid client_id: {}", client_id),
            })?;

        // Verify the client_id matches (defensive programming)
        if client.client_id != client_id {
            return Err(OAuth2IntegrationError::OAuth2Flow {
                message: "Client ID mismatch".to_string(),
            });
        }

        if let Some(redirect_uri) = redirect_uri {
            if !client.redirect_uris.contains(&redirect_uri.to_string()) {
                return Err(OAuth2IntegrationError::OAuth2Flow {
                    message: "Invalid redirect_uri".to_string(),
                });
            }
        }

        Ok(client)
    }

    /// Validate scopes for a client
    pub fn validate_scopes(&self, client_id: &str, requested_scopes: &str) -> Result<Vec<String>, OAuth2IntegrationError> {
        let client = self.registered_clients.get(client_id)
            .ok_or_else(|| OAuth2IntegrationError::OAuth2Flow {
                message: "Invalid client_id".to_string(),
            })?;

        let requested: Vec<String> = requested_scopes
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Check if all requested scopes are allowed
        for scope in &requested {
            if !client.allowed_scopes.contains(scope) {
                return Err(OAuth2IntegrationError::OAuth2Flow {
                    message: format!("Scope '{}' not allowed for client", scope),
                });
            }
        }

        Ok(requested)
    }

    /// Clean up expired authorization codes
    #[allow(dead_code)]
    pub async fn cleanup_expired_codes(&self) {
        let mut codes = self.authorization_codes.write().await;
        let now = Utc::now();
        codes.retain(|_, auth_code| auth_code.expires_at > now);
    }

    /// Clean up expired tokens
    #[allow(dead_code)]
    pub async fn cleanup_expired_tokens(&self) {
        let mut tokens = self.active_tokens.write().await;
        let now = Utc::now().timestamp() as usize;  // Convert to usize to match exp field
        tokens.retain(|_, claims| claims.exp > now);
    }

    /// Get server statistics
    pub async fn get_stats(&self) -> ServerStats {
        let codes = self.authorization_codes.read().await;
        let tokens = self.active_tokens.read().await;

        ServerStats {
            active_authorization_codes: codes.len(),
            active_tokens: tokens.len(),
            registered_clients: self.registered_clients.len(),
        }
    }
}

/// OAuth2 server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub active_authorization_codes: usize,
    pub active_tokens: usize,
    pub registered_clients: usize,
}
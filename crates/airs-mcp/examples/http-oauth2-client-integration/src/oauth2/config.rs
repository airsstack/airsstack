//! OAuth2 configuration structures

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

/// Configuration for OAuth2 client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2ClientConfig {
    pub client_id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub redirect_uri: String,
    pub scope: String,
}

/// Configuration for OAuth2 authorization server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2ServerConfig {
    pub server_host: String,
    pub server_port: u16,
    pub issuer: String,
    pub jwks_endpoint: String,
    /// RSA private key for JWT signing (PEM format)
    pub private_key_pem: String,
}

/// Configuration for OAuth2-protected MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub server_host: String,
    pub server_port: u16,
    pub oauth2_jwks_url: String,
    pub required_scope: String,
}

/// Default configurations for development/testing
impl Default for OAuth2ClientConfig {
    fn default() -> Self {
        Self {
            client_id: "test-client".to_string(),
            authorization_endpoint: "http://localhost:3002/authorize".to_string(),
            token_endpoint: "http://localhost:3002/token".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scope: "mcp:read mcp:write".to_string(),
        }
    }
}

impl Default for OAuth2ServerConfig {
    fn default() -> Self {
        Self {
            server_host: "localhost".to_string(),
            server_port: 3002,
            issuer: "http://localhost:3002".to_string(),
            jwks_endpoint: "http://localhost:3002/jwks".to_string(),
            private_key_pem: include_str!("../../test_keys/private_key.pem").to_string(),
        }
    }
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            server_host: "localhost".to_string(),
            server_port: 3003,
            oauth2_jwks_url: "http://localhost:3002/jwks".to_string(),
            required_scope: "mcp:read".to_string(),
        }
    }
}
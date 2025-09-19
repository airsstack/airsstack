// Standard library imports
use std::collections::HashMap;

// Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Internal module imports
pub mod oauth2;

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

/// OAuth2 token response from authorization server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// OAuth2 error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Error {
    pub error: String,
    pub error_description: Option<String>,
    pub error_uri: Option<String>,
}

/// JWT token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub exp: i64,
    pub iat: i64,
    pub scope: String,
    pub client_id: String,
}

/// PKCE code challenge and verifier
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

/// OAuth2 authorization code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub scope: String,
    pub expires_at: DateTime<Utc>,
}

/// Stored OAuth2 tokens
#[derive(Debug, Clone)]
pub struct TokenStore {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
}

/// Common errors for OAuth2 integration
#[derive(Error, Debug)]
pub enum OAuth2IntegrationError {
    #[error("OAuth2 flow error: {message}")]
    OAuth2Flow { message: String },

    #[error("Token validation error: {message}")]
    TokenValidation { message: String },

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("MCP protocol error: {message}")]
    McpProtocol { message: String },

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid scope: required {required}, got {actual}")]
    InvalidScope { required: String, actual: String },
}

/// Helper function to generate a random string for PKCE
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Helper function to create SHA256 hash for PKCE code challenge
pub fn create_code_challenge(code_verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(code_verifier.as_bytes());
    base64::encode_config(digest, base64::URL_SAFE_NO_PAD)
}

/// Helper function to validate JWT token format
pub fn is_valid_jwt_format(token: &str) -> bool {
    token.split('.').count() == 3
}

/// Helper function to extract claims from JWT without verification
pub fn extract_jwt_claims_unverified(token: &str) -> Result<HashMap<String, serde_json::Value>, OAuth2IntegrationError> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(OAuth2IntegrationError::TokenValidation {
            message: "Invalid JWT format".to_string(),
        });
    }

    let payload = parts[1];
    let decoded = base64::decode_config(payload, base64::URL_SAFE_NO_PAD)
        .map_err(|_| OAuth2IntegrationError::TokenValidation {
            message: "Invalid base64 encoding in JWT payload".to_string(),
        })?;

    let claims: HashMap<String, serde_json::Value> = serde_json::from_slice(&decoded)?;
    Ok(claims)
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
            private_key_pem: include_str!("../test_keys/private_key.pem").to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(32);
        let s2 = generate_random_string(32);
        assert_eq!(s1.len(), 32);
        assert_eq!(s2.len(), 32);
        assert_ne!(s1, s2); // Should be different
    }

    #[test]
    fn test_code_challenge_generation() {
        let verifier = "test-verifier-12345";
        let challenge = create_code_challenge(verifier);
        assert!(!challenge.is_empty());
        assert!(!challenge.contains('=')); // URL-safe base64 without padding
    }

    #[test]
    fn test_jwt_format_validation() {
        assert!(is_valid_jwt_format("header.payload.signature"));
        assert!(!is_valid_jwt_format("invalid"));
        assert!(!is_valid_jwt_format("header.payload"));
        assert!(!is_valid_jwt_format("header.payload.signature.extra"));
    }
}
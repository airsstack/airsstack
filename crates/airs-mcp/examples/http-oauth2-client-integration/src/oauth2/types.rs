//! OAuth2 type definitions

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub aud: Vec<String>,  // Changed to Vec<String> to match usage
    pub iss: String,
    pub exp: usize,        // Changed to usize to match jsonwebtoken
    pub iat: usize,        // Changed to usize to match jsonwebtoken
    pub scope: Option<String>,  // Made optional to match usage
    pub client_id: Option<String>,  // Made optional to match usage
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
    pub state: Option<String>,  // Added state field
}

/// Stored OAuth2 tokens
#[derive(Debug, Clone)]
pub struct TokenStore {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
}
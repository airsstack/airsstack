//! JWT Test Token Generation for OAuth2 MCP Testing
//!
//! This module generates test JWT tokens with different scopes for testing
//! OAuth2 authentication with AirsStack MCP components.

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use uuid::Uuid;

// AirsStack OAuth2 types
use airs_mcp::oauth2::types::JwtClaims;

/// Test token configuration for different MCP access scenarios
#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub name: String,
    pub description: String,
    pub subject: String,
    pub scopes: Vec<String>,
    pub expires_minutes: i64,
}

impl TokenConfig {
    /// Full access token configuration - can access all MCP operations
    pub fn full_access() -> Self {
        Self {
            name: "Full Access".to_string(),
            description: "Complete access to all MCP operations".to_string(),
            subject: "admin@test.local".to_string(),
            scopes: vec!["mcp:*".to_string()],
            expires_minutes: 60,
        }
    }

    /// Tools-only access token configuration
    pub fn tools_only() -> Self {
        Self {
            name: "Tools Only".to_string(),
            description: "Access to tools operations only".to_string(),
            subject: "tools-user@test.local".to_string(),
            scopes: vec![
                "mcp:tools:list".to_string(),
                "mcp:tools:execute".to_string(),
            ],
            expires_minutes: 30,
        }
    }

    /// Resources-only access token configuration
    pub fn resources_only() -> Self {
        Self {
            name: "Resources Only".to_string(),
            description: "Access to resources operations only".to_string(),
            subject: "resources-user@test.local".to_string(),
            scopes: vec![
                "mcp:resources:list".to_string(),
                "mcp:resources:read".to_string(),
            ],
            expires_minutes: 30,
        }
    }

    /// Read-only access token configuration
    pub fn read_only() -> Self {
        Self {
            name: "Read Only".to_string(),
            description: "Read-only access to resources and tools listing".to_string(),
            subject: "readonly@test.local".to_string(),
            scopes: vec![
                "mcp:resources:list".to_string(),
                "mcp:tools:list".to_string(),
                "mcp:prompts:list".to_string(),
            ],
            expires_minutes: 15,
        }
    }
}

/// Generate a test JWT token with specified claims
///
/// This creates JWT tokens that will be validated by AirsStack's OAuth2Strategy
/// using the JWKS endpoint for signature verification.
pub fn generate_test_token(
    subject: &str,
    scopes: &[&str],
    audience: &str,
    issuer: &str,
    expires_in_minutes: i64,
    encoding_key: &EncodingKey,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = now + Duration::minutes(expires_in_minutes);
    
    // Create JWT claims using AirsStack's JwtClaims type
    let claims = JwtClaims {
        sub: subject.to_string(),
        aud: Some(audience.to_string()),
        iss: Some(issuer.to_string()),
        exp: Some(exp.timestamp()),
        nbf: Some(now.timestamp()),
        iat: Some(now.timestamp()),
        jti: Some(Uuid::new_v4().to_string()),
        scope: Some(scopes.join(" ")),
        scopes: Some(scopes.iter().map(|s| s.to_string()).collect()),
    };

    // Create JWT header with key ID for JWKS matching
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-oauth2-mcp".to_string());

    // Sign the token - this will be validated by AirsStack's OAuth2Strategy
    encode(&header, &claims, encoding_key)
}

/// Generate MCP Inspector command for testing with a given token
pub fn generate_inspector_command(token: &str) -> String {
    format!(
        "npx @modelcontextprotocol/inspector-cli --transport http --server-url http://localhost:3001/mcp --header \"Authorization: Bearer {}\"",
        token
    )
}

/// Generate curl test command for manual testing
pub fn generate_curl_test(token: &str) -> String {
    format!(
        "curl -H 'Authorization: Bearer {}' http://localhost:3001/mcp -X POST -H 'Content-Type: application/json' -d '{{\"jsonrpc\":\"2.0\",\"id\":\"test\",\"method\":\"initialize\",\"params\":{{}}}}'",
        token
    )
}

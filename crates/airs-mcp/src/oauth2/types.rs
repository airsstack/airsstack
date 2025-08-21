//! OAuth 2.1 Type Definitions
//!
//! Core type definitions used across the OAuth 2.1 implementation
//! following workspace standards for consistent data structures.

// Layer 1: Standard library imports
// (none for this module)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (none for this module)

/// JWT Claims structure for OAuth 2.1 tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    /// Token subject (user identifier)
    pub sub: String,

    /// Token audience (intended recipient)
    pub aud: Option<String>,

    /// Token issuer
    pub iss: Option<String>,

    /// Token expiration time (Unix timestamp)
    pub exp: Option<i64>,

    /// Token not-before time (Unix timestamp)
    pub nbf: Option<i64>,

    /// Token issued-at time (Unix timestamp)
    pub iat: Option<i64>,

    /// JWT ID (unique identifier for this token)
    pub jti: Option<String>,

    /// Token scopes (space-separated string or array)
    #[serde(default)]
    pub scope: Option<String>,

    /// Token scopes as array (alternative to scope string)
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
}

/// JWKS (JSON Web Key Set) response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

/// JSON Web Key structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwk {
    /// Key type (e.g., "RSA")
    pub kty: String,

    /// Key use (e.g., "sig" for signature)
    #[serde(default)]
    pub r#use: Option<String>,

    /// Key operations
    #[serde(default)]
    pub key_ops: Option<Vec<String>>,

    /// Algorithm intended for use with this key
    #[serde(default)]
    pub alg: Option<String>,

    /// Key ID
    #[serde(default)]
    pub kid: Option<String>,

    /// X.509 certificate chain
    #[serde(default)]
    pub x5c: Option<Vec<String>>,

    /// X.509 certificate SHA-1 thumbprint
    #[serde(default)]
    pub x5t: Option<String>,

    /// X.509 certificate SHA-256 thumbprint
    #[serde(default, rename = "x5t#S256")]
    pub x5t_s256: Option<String>,

    // RSA key parameters
    /// RSA public key modulus
    #[serde(default)]
    pub n: Option<String>,

    /// RSA public key exponent
    #[serde(default)]
    pub e: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_claims_deserialization() {
        let json = r#"{
            "sub": "user123",
            "aud": "mcp-server",
            "iss": "https://auth.example.com",
            "exp": 1640995200,
            "scope": "mcp:tools:execute mcp:resources:read"
        }"#;

        let claims: JwtClaims = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.aud, Some("mcp-server".to_string()));
        assert_eq!(
            claims.scope,
            Some("mcp:tools:execute mcp:resources:read".to_string())
        );
    }

    #[test]
    fn test_jwks_response_deserialization() {
        let json = r#"{
            "keys": [{
                "kty": "RSA",
                "e": "AQAB",
                "kid": "key1"
            }]
        }"#;

        let jwks: JwksResponse = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(jwks.keys.len(), 1);
        assert_eq!(jwks.keys[0].kty, "RSA");
        assert_eq!(jwks.keys[0].kid, Some("key1".to_string()));
    }
}

//! Token Management Module
//!
//! This module handles JWT token generation and test key management
//! for the OAuth2 MCP integration.

use std::collections::HashMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rsa::{pkcs8::DecodePrivateKey, traits::PublicKeyParts, RsaPrivateKey};
use serde_json::{json, Value};
use tracing::{debug, info};
use uuid::Uuid;

use airs_mcp::oauth2::types::JwtClaims;

/// Test JWT signing keys and JWKS data
#[derive(Clone)]
pub struct TestKeys {
    /// Private key for signing JWTs
    pub encoding_key: EncodingKey,
    /// Public key in JWK format for JWKS endpoint
    pub jwks_response: Value,
}

impl std::fmt::Debug for TestKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestKeys")
            .field("encoding_key", &"<EncodingKey>")
            .field("jwks_response", &self.jwks_response)
            .finish()
    }
}

impl TestKeys {
    /// Generate test RSA keys for JWT signing and validation
    pub fn generate() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔑 Generating test RSA keys for JWT operations");
        
        // Use a fixed RSA key for consistent testing
        // In production, keys would be rotated and managed securely
        let private_key_pem = include_str!("../keys/test_rsa_key.pem");
        debug!("📄 Loading RSA private key from test_rsa_key.pem");
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;

        // Extract the correct public key components from the actual private key
        debug!("🔧 Extracting public key components from actual RSA private key");
        let rsa_private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)?;
        let public_key = rsa_private_key.to_public_key();
        
        // Extract n (modulus) and e (public exponent) as big-endian bytes
        let n_bytes = public_key.n().to_bytes_be();
        let e_bytes = public_key.e().to_bytes_be();
        
        // Encode to base64url (no padding)
        let n_b64 = URL_SAFE_NO_PAD.encode(&n_bytes);
        let e_b64 = URL_SAFE_NO_PAD.encode(&e_bytes);
        
        debug!("🔍 Extracted public key components:");
        debug!("   - Modulus (n) length: {} bytes", n_bytes.len());
        debug!("   - Exponent (e) length: {} bytes", e_bytes.len());
        debug!("   - Modulus (n) base64: {}...", &n_b64[..20.min(n_b64.len())]);
        debug!("   - Exponent (e) base64: {}", e_b64);

        // Create JWKS response with the extracted public key components
        let jwks_response = json!({
            "keys": [
                {
                    "kty": "RSA",
                    "use": "sig",
                    "kid": "test-key-oauth2-mcp",
                    "alg": "RS256",
                    "n": n_b64,
                    "e": e_b64
                }
            ]
        });

        info!("✅ Test keys generated with correct public key components");
        Ok(Self {
            encoding_key,
            jwks_response,
        })
    }
}

/// Generate a test JWT token with specified claims
pub fn generate_test_token(
    subject: &str,
    scopes: &[&str],
    audience: &str,
    issuer: &str,
    expires_in_minutes: i64,
    encoding_key: &EncodingKey,
) -> Result<String, jsonwebtoken::errors::Error> {
    debug!("🎫 Generating JWT token for subject: {}", subject);
    debug!("🔐 Token scopes: {:?}", scopes);
    debug!("👥 Audience: {}", audience);
    debug!("🏢 Issuer: {}", issuer);
    debug!("⏰ Expires in: {} minutes", expires_in_minutes);
    
    let now = Utc::now();
    let exp = now + ChronoDuration::minutes(expires_in_minutes);

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

    debug!("📋 JWT claims: {:?}", claims);

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("test-key-oauth2-mcp".to_string());

    debug!("🔑 JWT header: {:?}", header);
    debug!("🔐 Encoding JWT with RSA private key...");

    let token = encode(&header, &claims, encoding_key)?;
    info!("✅ JWT token generated successfully (length: {})", token.len());
    debug!("🎫 Generated token: {}", token);
    
    Ok(token)
}

/// Test token configurations for different scenarios
#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub name: String,
    pub description: String,
    pub subject: String,
    pub scopes: Vec<String>,
    pub expires_minutes: i64,
}

impl TokenConfig {
    pub fn full_access() -> Self {
        Self {
            name: "Full Access".to_string(),
            description: "Complete access to all MCP operations".to_string(),
            subject: "admin@test.local".to_string(),
            scopes: vec!["mcp:*".to_string()],
            expires_minutes: 60,
        }
    }

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

    /// Get all default token configurations
    pub fn all_configs() -> HashMap<String, TokenConfig> {
        let mut configs = HashMap::new();
        configs.insert("full".to_string(), Self::full_access());
        configs.insert("tools".to_string(), Self::tools_only());
        configs.insert("resources".to_string(), Self::resources_only());
        configs.insert("readonly".to_string(), Self::read_only());
        configs
    }
}
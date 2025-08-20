//! JWT Token Validator with JWKS Client Support
//!
//! This module provides JWT token validation with automatic JWKS key retrieval,
//! caching, and RS256 signature validation for OAuth 2.1 compliance.

// Layer 1: Standard library imports
use std::{
    collections::HashSet,
    sync::Arc,
    time::Duration,
};

// Layer 2: Third-party crate imports
use dashmap::DashMap;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use tracing::debug;

// Layer 3: Internal module imports
use crate::oauth2::{config::OAuth2Config, error::OAuth2Error, error::OAuth2Result};

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

/// Cached JWKS key with expiration
#[derive(Clone)]
struct CachedKey {
    key: DecodingKey,
    expires_at: Instant,
}

impl std::fmt::Debug for CachedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedKey")
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive() // Don't expose the DecodingKey for security
    }
}

/// JWT Token Validator with JWKS client support and caching
pub struct JwtValidator {
    /// HTTP client for JWKS requests
    client: Client,
    
    /// OAuth configuration
    config: OAuth2Config,
    
    /// Cached JWKS keys (kid -> CachedKey)
    key_cache: Arc<DashMap<String, CachedKey>>,
    
    /// JWT validation configuration
    validation: Validation,
}

impl JwtValidator {
    /// Create a new JWT validator with the given configuration
    pub fn new(config: OAuth2Config) -> OAuth2Result<Self> {
        let mut validation = Validation::new(Algorithm::RS256);
        
        // Configure validation based on OAuth config
        validation.leeway = config.validation.leeway.as_secs();
        validation.validate_exp = config.validation.require_exp;
        validation.validate_nbf = config.validation.validate_nbf;
        validation.required_spec_claims = HashSet::new();
        
        if config.validation.require_aud {
            validation.required_spec_claims.insert("aud".to_string());
            validation.aud = Some(HashSet::from([config.audience.clone()]));
        }
        
        if config.validation.require_iss {
            validation.required_spec_claims.insert("iss".to_string());
            validation.iss = Some(HashSet::from([config.issuer.clone()]));
        }
        
        // Set allowed algorithms
        validation.algorithms = config.validation.algorithms
            .iter()
            .map(|alg| {
                match alg.as_str() {
                    "RS256" => Algorithm::RS256,
                    "RS384" => Algorithm::RS384,
                    "RS512" => Algorithm::RS512,
                    "ES256" => Algorithm::ES256,
                    "ES384" => Algorithm::ES384,
                    _ => Algorithm::RS256, // Default fallback
                }
            })
            .collect();

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| OAuth2Error::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config,
            key_cache: Arc::new(DashMap::new()),
            validation,
        })
    }

    /// Validate a JWT token and return the claims
    pub async fn validate_token(&self, token: &str) -> OAuth2Result<JwtClaims> {
        // Decode the token header to get the key ID
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| OAuth2Error::TokenValidation(format!("Invalid token header: {}", e)))?;

        let kid = header.kid.ok_or_else(|| {
            OAuth2Error::TokenValidation("Token missing key ID (kid) in header".to_string())
        })?;

        // Get the decoding key
        let decoding_key = self.get_decoding_key(&kid).await?;

        // Decode and validate the token
        let token_data = decode::<JwtClaims>(token, &decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    OAuth2Error::TokenExpired {
                        expired_at: "Token has expired".to_string(),
                    }
                }
                jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                    OAuth2Error::InvalidAudience {
                        expected: self.config.audience.clone(),
                        actual: "Invalid audience in token".to_string(),
                    }
                }
                jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                    OAuth2Error::InvalidIssuer {
                        expected: self.config.issuer.clone(),
                        actual: "Invalid issuer in token".to_string(),
                    }
                }
                _ => OAuth2Error::TokenValidation(format!("Token validation failed: {}", e)),
            })?;

        debug!("Successfully validated JWT token for subject: {}", token_data.claims.sub);
        Ok(token_data.claims)
    }

    /// Get a decoding key for the given key ID, fetching from JWKS if needed
    async fn get_decoding_key(&self, kid: &str) -> OAuth2Result<DecodingKey> {
        // Check cache first
        if let Some(cached) = self.key_cache.get(kid) {
            if cached.expires_at > Instant::now() {
                debug!("Using cached JWKS key for kid: {}", kid);
                return Ok(cached.key.clone());
            } else {
                debug!("Cached JWKS key expired for kid: {}", kid);
                self.key_cache.remove(kid);
            }
        }

        // Fetch from JWKS endpoint
        debug!("Fetching JWKS key for kid: {}", kid);
        self.fetch_and_cache_key(kid).await
    }

    /// Fetch JWKS and cache the requested key
    async fn fetch_and_cache_key(&self, kid: &str) -> OAuth2Result<DecodingKey> {
        let jwks = self.fetch_jwks().await?;
        
        // Find the key with matching kid
        let jwk = jwks.keys
            .iter()
            .find(|key| key.kid.as_deref() == Some(kid))
            .ok_or_else(|| {
                OAuth2Error::JwksError(format!("Key with id '{}' not found in JWKS", kid))
            })?;

        // Convert JWK to DecodingKey
        let decoding_key = self.jwk_to_decoding_key(jwk)?;
        
        // Cache the key
        let cached_key = CachedKey {
            key: decoding_key.clone(),
            expires_at: Instant::now() + self.config.cache.jwks_cache_ttl,
        };
        
        self.key_cache.insert(kid.to_string(), cached_key);
        debug!("Cached JWKS key for kid: {}", kid);

        // Clean up expired cache entries if we're at capacity
        if self.key_cache.len() > self.config.cache.jwks_cache_max_size {
            self.cleanup_expired_keys();
        }

        Ok(decoding_key)
    }

    /// Fetch JWKS from the configured endpoint
    async fn fetch_jwks(&self) -> OAuth2Result<JwksResponse> {
        debug!("Fetching JWKS from: {}", self.config.jwks_url);
        
        let response = self
            .client
            .get(self.config.jwks_url.clone())
            .send()
            .await
            .map_err(|e| OAuth2Error::JwksError(format!("Failed to fetch JWKS: {}", e)))?;

        if !response.status().is_success() {
            return Err(OAuth2Error::JwksError(format!(
                "JWKS endpoint returned status: {}",
                response.status()
            )));
        }

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| OAuth2Error::JwksError(format!("Failed to parse JWKS response: {}", e)))?;

        debug!("Successfully fetched JWKS with {} keys", jwks.keys.len());
        Ok(jwks)
    }

    /// Convert a JWK to a DecodingKey for token validation
    fn jwk_to_decoding_key(&self, jwk: &Jwk) -> OAuth2Result<DecodingKey> {
        match jwk.kty.as_str() {
            "RSA" => {
                let n = jwk.n.as_ref().ok_or_else(|| {
                    OAuth2Error::JwksError("RSA key missing modulus (n)".to_string())
                })?;
                let e = jwk.e.as_ref().ok_or_else(|| {
                    OAuth2Error::JwksError("RSA key missing exponent (e)".to_string())
                })?;

                // Decode base64url encoded values
                DecodingKey::from_rsa_components(n, e)
                    .map_err(|e| OAuth2Error::JwksError(format!("Failed to create RSA key: {}", e)))
            }
            _ => Err(OAuth2Error::JwksError(format!(
                "Unsupported key type: {}",
                jwk.kty
            ))),
        }
    }

    /// Clean up expired keys from the cache
    fn cleanup_expired_keys(&self) {
        let now = Instant::now();
        let expired_keys: Vec<String> = self
            .key_cache
            .iter()
            .filter_map(|entry| {
                if entry.value().expires_at <= now {
                    Some(entry.key().clone())
                } else {
                    None
                }
            })
            .collect();

        for kid in expired_keys {
            self.key_cache.remove(&kid);
            debug!("Removed expired JWKS key: {}", kid);
        }
    }

    /// Get the token scopes as a vector
    pub fn extract_scopes(&self, claims: &JwtClaims) -> Vec<String> {
        // Try scopes array first, then scope string
        if let Some(scopes) = &claims.scopes {
            scopes.clone()
        } else if let Some(scope_str) = &claims.scope {
            scope_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

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
        assert_eq!(claims.scope, Some("mcp:tools:execute mcp:resources:read".to_string()));
    }

    #[test]
    fn test_extract_scopes() {
        let config = OAuth2Config::builder()
            .jwks_url(Url::parse("https://example.com/jwks").unwrap())
            .audience("test".to_string())
            .issuer("test".to_string())
            .build()
            .unwrap();

        let validator = JwtValidator::new(config).expect("Should create validator");

        // Test scope string
        let claims1 = JwtClaims {
            sub: "user1".to_string(),
            scope: Some("scope1 scope2 scope3".to_string()),
            scopes: None,
            aud: None,
            iss: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };

        let scopes1 = validator.extract_scopes(&claims1);
        assert_eq!(scopes1, vec!["scope1", "scope2", "scope3"]);

        // Test scopes array
        let claims2 = JwtClaims {
            sub: "user2".to_string(),
            scope: None,
            scopes: Some(vec!["scope1".to_string(), "scope2".to_string()]),
            aud: None,
            iss: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };

        let scopes2 = validator.extract_scopes(&claims2);
        assert_eq!(scopes2, vec!["scope1", "scope2"]);

        // Test no scopes
        let claims3 = JwtClaims {
            sub: "user3".to_string(),
            scope: None,
            scopes: None,
            aud: None,
            iss: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };

        let scopes3 = validator.extract_scopes(&claims3);
        assert!(scopes3.is_empty());
    }

    #[test]
    fn test_jwks_response_deserialization() {
        let json = r#"{
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "kid": "key1",
                "n": "example_modulus",
                "e": "AQAB"
            }]
        }"#;

        let jwks: JwksResponse = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(jwks.keys.len(), 1);
        assert_eq!(jwks.keys[0].kty, "RSA");
        assert_eq!(jwks.keys[0].kid, Some("key1".to_string()));
    }
}

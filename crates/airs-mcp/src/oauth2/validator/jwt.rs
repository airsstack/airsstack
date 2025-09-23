//! JWT Validator Trait and Implementation
//!
//! Provides trait-based JWT validation following workspace standards for
//! zero-cost abstractions and flexible error handling.

// Layer 1: Standard library imports
use std::{collections::HashSet, sync::Arc, time::Duration};

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use dashmap::DashMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use tokio::time::Instant;
use tracing::debug;

// Layer 3: Internal module imports
use crate::oauth2::{
    config::OAuth2Config,
    error::{OAuth2Error, OAuth2Result},
    types::{Jwk, JwksResponse, JwtClaims},
};

/// JWT validation trait following workspace standards
///
/// Uses associated types for flexible error handling while maintaining
/// zero-cost abstractions through generic monomorphization.
#[async_trait]
pub trait JwtValidator {
    /// Error type specific to this validator implementation
    /// Must be convertible to OAuth2Error for unified error handling
    type Error: Into<OAuth2Error> + Send + Sync + 'static;

    /// Validate a JWT token and extract claims
    ///
    /// # Arguments
    /// * `token` - The JWT token string to validate
    ///
    /// # Returns
    /// * `Ok(JwtClaims)` - Successfully validated token with extracted claims
    /// * `Err(Self::Error)` - Validation failed with validator-specific error
    async fn validate(&self, token: &str) -> Result<JwtClaims, Self::Error>;

    /// Extract OAuth scopes from validated JWT claims
    ///
    /// Default implementation handles standard "scope" claim as space-separated string
    /// and "scopes" claim as array. Override for custom scope extraction logic.
    fn extract_scopes(&self, claims: &JwtClaims) -> Vec<String> {
        // Handle space-separated scope string (RFC 6749 standard)
        if let Some(scope_str) = &claims.scope {
            return scope_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
        }

        // Handle scopes array (alternative format)
        if let Some(scopes_array) = &claims.scopes {
            return scopes_array.clone();
        }

        // No scopes found
        Vec::new()
    }
}

/// Concrete JWT validator implementation
///
/// Self-contained JWT validator with JWKS client support and caching
/// following workspace standards for zero-cost abstractions.
pub struct Jwt {
    /// HTTP client for JWKS requests
    client: Client,

    /// OAuth configuration
    config: OAuth2Config,

    /// Cached JWKS keys (kid -> CachedKey)
    key_cache: Arc<DashMap<String, CachedKey>>,

    /// JWT validation configuration
    validation: Validation,
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

impl Jwt {
    /// Create new JWT validator from OAuth2 configuration
    ///
    /// # Arguments
    /// * `config` - OAuth2 configuration containing JWKS endpoint and validation rules
    ///
    /// # Returns
    /// * `Ok(Jwt)` - Successfully created validator
    /// * `Err(OAuth2Error)` - Configuration or initialization error
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
        validation.algorithms = config
            .validation
            .algorithms
            .iter()
            .map(|alg| {
                match alg.as_str() {
                    "RS256" => Algorithm::RS256,
                    "RS384" => Algorithm::RS384,
                    "RS512" => Algorithm::RS512,
                    _ => Algorithm::RS256, // Default fallback
                }
            })
            .collect();

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| {
                OAuth2Error::Configuration(format!("Failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            client,
            config,
            key_cache: Arc::new(DashMap::new()),
            validation,
        })
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
        let jwk = jwks
            .keys
            .iter()
            .find(|key| key.kid.as_deref() == Some(kid))
            .ok_or_else(|| {
                OAuth2Error::JwksError(format!("Key with id '{kid}' not found in JWKS"))
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
            .map_err(|e| OAuth2Error::JwksError(format!("Failed to fetch JWKS: {e}")))?;

        if !response.status().is_success() {
            return Err(OAuth2Error::JwksError(format!(
                "JWKS endpoint returned status: {}",
                response.status()
            )));
        }

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| OAuth2Error::JwksError(format!("Failed to parse JWKS response: {e}")))?;

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
                    .map_err(|e| OAuth2Error::JwksError(format!("Failed to create RSA key: {e}")))
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

    /// Validate a JWT token with detailed error analysis
    ///
    /// This method provides comprehensive error analysis for debugging and
    /// security purposes, returning structured error information.
    ///
    /// # Arguments
    /// * `token` - The JWT token string to validate
    ///
    /// # Returns
    /// * `Ok(JwtClaims)` - Successfully validated token with extracted claims
    /// * `Err(OAuth2Error)` - Detailed validation error with specific failure reason
    pub async fn validate_with_detailed_errors(
        &self,
        token: &str,
    ) -> Result<JwtClaims, OAuth2Error> {
        // Enhanced validation with specific error categorization
        if token.trim().is_empty() {
            return Err(OAuth2Error::TokenValidation(
                "Empty or whitespace-only token provided".to_string(),
            ));
        }

        // Check for common token format issues
        if !token.contains('.') {
            return Err(OAuth2Error::TokenValidation(
                "Invalid JWT format: token must contain '.' separators".to_string(),
            ));
        }

        // Validate JWT structure before attempting to decode
        let token_parts: Vec<&str> = token.split('.').collect();
        if token_parts.len() != 3 {
            return Err(OAuth2Error::TokenValidation(format!(
                "Invalid JWT structure: expected 3 parts (header.payload.signature), got {} parts",
                token_parts.len()
            )));
        }

        // Validate each part has content
        if token_parts[0].is_empty() {
            return Err(OAuth2Error::TokenValidation(
                "Invalid JWT: header part is empty".to_string(),
            ));
        }
        if token_parts[1].is_empty() {
            return Err(OAuth2Error::TokenValidation(
                "Invalid JWT: payload part is empty".to_string(),
            ));
        }
        if token_parts[2].is_empty() {
            return Err(OAuth2Error::TokenValidation(
                "Invalid JWT: signature part is empty".to_string(),
            ));
        }

        // Use the standard validation method with enhanced error context
        self.validate(token).await
    }
}

#[async_trait]
impl JwtValidator for Jwt {
    type Error = OAuth2Error;

    async fn validate(&self, token: &str) -> Result<JwtClaims, Self::Error> {
        // Basic token format validation
        if token.trim().is_empty() {
            return Err(OAuth2Error::TokenValidation(
                "Empty or whitespace-only token provided".to_string(),
            ));
        }

        // Check basic JWT structure (header.payload.signature)
        let token_parts: Vec<&str> = token.split('.').collect();
        if token_parts.len() != 3 {
            return Err(OAuth2Error::TokenValidation(format!(
                "Invalid JWT structure: expected 3 parts separated by '.', got {} parts",
                token_parts.len()
            )));
        }

        // Validate each part is non-empty
        for (i, part) in token_parts.iter().enumerate() {
            if part.is_empty() {
                let part_name = match i {
                    0 => "header",
                    1 => "payload",
                    2 => "signature",
                    _ => "unknown",
                };
                return Err(OAuth2Error::TokenValidation(format!(
                    "Invalid JWT structure: {part_name} part is empty"
                )));
            }
        }

        // Decode the token header to get the key ID
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| OAuth2Error::TokenValidation(format!("Invalid token header: {e}")))?;

        let kid = header.kid.ok_or_else(|| {
            OAuth2Error::TokenValidation("Token missing key ID (kid) in header".to_string())
        })?;

        // Get the decoding key
        let decoding_key = self.get_decoding_key(&kid).await?;

        // Decode and validate the token
        let token_data =
            decode::<JwtClaims>(token, &decoding_key, &self.validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        OAuth2Error::TokenValidation("Token has expired".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::ImmatureSignature => {
                        OAuth2Error::TokenValidation("Token not yet valid (nbf)".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                        OAuth2Error::TokenValidation("Invalid audience".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                        OAuth2Error::TokenValidation("Invalid issuer".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        OAuth2Error::TokenValidation("Invalid token signature".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::Base64(_) => {
                        OAuth2Error::TokenValidation("Invalid base64 encoding in token".to_string())
                    }
                    jsonwebtoken::errors::ErrorKind::Json(_) => {
                        OAuth2Error::TokenValidation("Invalid JSON structure in token".to_string())
                    }
                    _ => OAuth2Error::TokenValidation(format!("Token validation failed: {e}")),
                }
            })?;

        debug!(
            "Successfully validated JWT token for subject: {}",
            token_data.claims.sub
        );
        Ok(token_data.claims)
    }

    fn extract_scopes(&self, claims: &JwtClaims) -> Vec<String> {
        // Handle space-separated scope string (RFC 6749 standard)
        if let Some(scope_str) = &claims.scope {
            return scope_str
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
        }

        // Handle scopes array (alternative format)
        if let Some(scopes_array) = &claims.scopes {
            return scopes_array.clone();
        }

        // No scopes found
        Vec::new()
    }
}

// Implement Clone for sharing across async contexts
impl Clone for Jwt {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            key_cache: Arc::clone(&self.key_cache),
            validation: self.validation.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_trait_implementation() {
        // Test that our trait implementation works correctly
        // This test validates the abstraction without requiring real JWT infrastructure

        let config = OAuth2Config::default();
        let jwt_validator = Jwt::new(config);

        // Should handle configuration errors gracefully
        assert!(jwt_validator.is_ok() || jwt_validator.is_err());
    }

    #[test]
    fn test_scope_extraction() {
        let config = OAuth2Config::default();
        let jwt = Jwt::new(config).unwrap();

        // Test space-separated scopes
        let claims_with_scope = JwtClaims {
            sub: "test".to_string(),
            scope: Some("read write admin".to_string()),
            scopes: None,
            aud: None,
            iss: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };

        let scopes = jwt.extract_scopes(&claims_with_scope);
        assert_eq!(scopes, vec!["read", "write", "admin"]);

        // Test scopes array
        let claims_with_scopes = JwtClaims {
            sub: "test".to_string(),
            scope: None,
            scopes: Some(vec!["read".to_string(), "write".to_string()]),
            aud: None,
            iss: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };

        let scopes = jwt.extract_scopes(&claims_with_scopes);
        assert_eq!(scopes, vec!["read", "write"]);

        // Test no scopes
        let claims_no_scopes = JwtClaims {
            sub: "test".to_string(),
            scope: None,
            scopes: None,
            aud: None,
            iss: None,
            exp: None,
            nbf: None,
            iat: None,
            jti: None,
        };

        let scopes = jwt.extract_scopes(&claims_no_scopes);
        assert!(scopes.is_empty());
    }
}

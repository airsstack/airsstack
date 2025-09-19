//! OAuth2 utility functions

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

// Layer 3: Internal module imports
use crate::oauth2::errors::OAuth2IntegrationError;

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
    let digest = Sha256::digest(code_verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
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
    let decoded = URL_SAFE_NO_PAD.decode(payload)
        .map_err(|_| OAuth2IntegrationError::TokenValidation {
            message: "Invalid base64 encoding in JWT payload".to_string(),
        })?;

    let claims: HashMap<String, serde_json::Value> = serde_json::from_slice(&decoded)?;
    Ok(claims)
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
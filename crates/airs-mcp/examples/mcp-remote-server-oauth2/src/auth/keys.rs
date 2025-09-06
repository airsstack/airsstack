//! RSA Key Management for OAuth2 JWT Operations
//!
//! This module handles RSA key generation and JWKS creation for OAuth2 JWT
//! token signing and validation in the AirsStack MCP context.

use std::fmt;
use jsonwebtoken::EncodingKey;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rsa::{RsaPrivateKey, pkcs1::DecodeRsaPrivateKey, traits::PublicKeyParts};

/// RSA keys for JWT operations with JWKS support
#[derive(Clone)]
pub struct TestKeys {
    /// Private key for signing JWTs
    pub encoding_key: EncodingKey,
    /// Public key in JWK format for JWKS endpoint
    pub jwks_response: Value,
}

impl fmt::Debug for TestKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestKeys")
            .field("encoding_key", &"<EncodingKey>")
            .field("jwks_response", &self.jwks_response)
            .finish()
    }
}

/// Generate RSA keys for JWT signing and create matching JWKS response
///
/// This function loads the RSA private key from the embedded PEM file,
/// extracts the public key components, and creates a proper JWKS response
/// that can be used by the AirsStack OAuth2 validation components.
pub fn generate_test_keys() -> Result<TestKeys, Box<dyn std::error::Error>> {
    // Load RSA key from the keys directory
    let private_key_pem = include_str!("../../keys/test_rsa_key.pem");
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
    
    // Parse the private key to extract public key components for JWKS
    let private_key = RsaPrivateKey::from_pkcs1_pem(private_key_pem)?;
    let public_key = private_key.to_public_key();
    
    // Extract the modulus (n) and exponent (e) for the JWKS
    let n = public_key.n();
    let e = public_key.e();
    
    // Convert to base64url encoding (without padding) for JWKS
    let n_bytes = n.to_bytes_be();
    let e_bytes = e.to_bytes_be();
    let n_b64 = URL_SAFE_NO_PAD.encode(&n_bytes);
    let e_b64 = URL_SAFE_NO_PAD.encode(&e_bytes);
    
    // Create JWKS response with the actual public key components
    // This will be used by the mock JWKS server to provide the public key
    // for AirsStack's OAuth2Strategy JWT validation
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

    Ok(TestKeys {
        encoding_key,
        jwks_response,
    })
}

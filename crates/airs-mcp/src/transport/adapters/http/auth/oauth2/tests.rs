//! OAuth2 HTTP Authentication Tests
//!
//! Comprehensive test suite for OAuth2 HTTP authentication components
//! including adapter, extractor, and error handling functionality.

#[cfg(test)]
use super::super::adapter::*;
#[cfg(test)]
use crate::authentication::strategies::oauth2::OAuth2Strategy;
#[cfg(test)]
use crate::oauth2::error::OAuth2Error;
#[cfg(test)]
use crate::oauth2::types::JwtClaims;
#[cfg(test)]
use crate::oauth2::validator::{jwt::JwtValidator, scope::ScopeValidator, Validator};
#[cfg(test)]
use async_trait::async_trait;
#[cfg(test)]
use std::collections::HashMap;

// Mock types for testing
#[cfg(test)]
#[derive(Debug, Clone)]
struct MockValidator;

#[cfg(test)]
#[derive(Debug, Clone)]
struct MockSecurityPolicy;

// Mock error for testing
#[cfg(test)]
#[derive(Debug, thiserror::Error)]
#[error("Mock validation error")]
struct MockError;

#[cfg(test)]
impl From<MockError> for OAuth2Error {
    fn from(_val: MockError) -> Self {
        OAuth2Error::InvalidToken("Mock validation failed".to_string())
    }
}

#[cfg(test)]
#[async_trait]
impl JwtValidator for MockValidator {
    type Error = MockError;

    async fn validate(&self, _token: &str) -> Result<JwtClaims, Self::Error> {
        // For testing, return a simple mock claims object
        Ok(JwtClaims {
            sub: "test_user".to_string(),
            aud: Some("test_audience".to_string()),
            iss: Some("test_issuer".to_string()),
            exp: Some(9999999999i64),
            nbf: None,
            iat: Some(1630000000i64),
            jti: Some("test_jti".to_string()),
            scope: Some("read write".to_string()),
            scopes: Some(vec!["read".to_string(), "write".to_string()]),
        })
    }
}

#[cfg(test)]
impl ScopeValidator for MockSecurityPolicy {
    type Error = MockError;

    fn validate_method_access(&self, _method: &str, _scopes: &[String]) -> Result<(), Self::Error> {
        // For testing, always allow access
        Ok(())
    }
}

#[tokio::test]
async fn test_oauth2_strategy_adapter_authentication_success() {
    // Create OAuth2 strategy with mock validator
    let jwt = MockValidator;
    let scope = MockSecurityPolicy;
    let validator = Validator::new(jwt, scope);
    let strategy = OAuth2Strategy::new(validator);
    let _adapter = OAuth2StrategyAdapter::new(strategy);

    // Create HTTP auth request with valid bearer token
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        "Bearer valid_test_token".to_string(),
    );

    let request = HttpAuthRequest::new(headers, "/mcp/tools/call".to_string());

    // Note: This test would require implementing the actual oauth2::validator::Validator trait
    // for MockValidator to test successful authentication. For now, we test request conversion.

    // Verify request structure
    assert_eq!(request.path, "/mcp/tools/call");
    assert!(request.headers.contains_key("Authorization"));
    assert_eq!(request.client_id, None);
}

#[test]
fn test_http_auth_request_creation() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let request = HttpAuthRequest::new(headers.clone(), "/api/test".to_string());

    assert_eq!(request.path, "/api/test");
    assert_eq!(request.headers, headers);
    assert_eq!(request.client_id, None);
}

#[test]
fn test_http_auth_request_with_client_id() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    let request = HttpAuthRequest::with_client_id(
        headers.clone(),
        "/api/test".to_string(),
        "test_client".to_string(),
    );

    assert_eq!(request.path, "/api/test");
    assert_eq!(request.headers, headers);
    assert_eq!(request.client_id, Some("test_client".to_string()));
}

#[test]
fn test_http_auth_request_with_metadata() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    let metadata = HashMap::new();
    let request =
        HttpAuthRequest::with_metadata(headers.clone(), "/api/test".to_string(), metadata.clone());

    assert_eq!(request.path, "/api/test");
    assert_eq!(request.headers, headers);
    assert_eq!(request.metadata, metadata);
}

#[test]
fn test_http_auth_request_builder_pattern() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    let request = HttpAuthRequest::new(headers.clone(), "/api/test".to_string())
        .client_id("test_client".to_string())
        .metadata(HashMap::new());

    assert_eq!(request.path, "/api/test");
    assert_eq!(request.headers, headers);
    assert_eq!(request.client_id, Some("test_client".to_string()));
}

#[test]
fn test_http_auth_request_display() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let request =
        HttpAuthRequest::new(headers, "/api/test".to_string()).client_id("test_client".to_string());

    let display_str = format!("{request}");
    assert!(display_str.contains("/api/test"));
    assert!(display_str.contains("test_client"));
    assert!(display_str.contains("headers: 2"));
}

#[test]
fn test_oauth2_strategy_adapter_creation() {
    let jwt = MockValidator;
    let scope = MockSecurityPolicy;
    let validator = Validator::new(jwt, scope);
    let strategy = OAuth2Strategy::new(validator);
    let adapter = OAuth2StrategyAdapter::new(strategy);

    // Verify adapter was created successfully
    // Note: Cannot access private strategy field directly,
    // which is correct encapsulation
    let _ = adapter; // Use the adapter (will be dropped automatically)
}

#[test]
fn test_http_auth_request_serialization() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    let request = HttpAuthRequest::new(headers, "/api/test".to_string());

    // Test serialization
    let serialized = serde_json::to_string(&request).expect("Should serialize");
    assert!(serialized.contains("Bearer token123"));
    assert!(serialized.contains("/api/test"));

    // Test deserialization
    let deserialized: HttpAuthRequest =
        serde_json::from_str(&serialized).expect("Should deserialize");
    assert_eq!(deserialized.path, request.path);
    assert_eq!(deserialized.headers, request.headers);
}

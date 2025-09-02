//! OAuth2 Request Types
//!
//! OAuth2-specific request types that bridge HTTP transport data
//! to the authentication strategy without HTTP dependencies.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use crate::authentication::request::AuthRequest;

/// OAuth2 authentication request data
///
/// Contains the essential data needed for OAuth2 authentication
/// extracted from transport layer without HTTP framework dependencies.
#[derive(Debug, Clone)]
pub struct OAuth2Request {
    /// Bearer token from Authorization header
    pub bearer_token: String,
    
    /// Optional MCP method for scope validation
    pub method: Option<String>,
    
    /// Additional metadata for audit logging and context
    pub metadata: HashMap<String, String>,
}

impl OAuth2Request {
    /// Create new OAuth2 request with bearer token
    pub fn new(bearer_token: String) -> Self {
        Self {
            bearer_token,
            method: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set MCP method for scope validation
    pub fn with_method(mut self, method: String) -> Self {
        self.method = Some(method);
        self
    }
    
    /// Add metadata entry
    pub fn with_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple metadata entries
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }
}

/// OAuth2 authentication request wrapper
///
/// Implements AuthRequest trait to bridge OAuth2Request with the
/// authentication strategy interface.
#[derive(Debug)]
pub struct OAuth2AuthRequest {
    oauth2_request: OAuth2Request,
}

impl OAuth2AuthRequest {
    /// Create new OAuth2 auth request wrapper
    pub fn new(oauth2_request: OAuth2Request) -> Self {
        Self { oauth2_request }
    }
}

impl AuthRequest<OAuth2Request> for OAuth2AuthRequest {
    fn custom_attribute(&self, key: &str) -> Option<String> {
        self.oauth2_request.metadata.get(key).cloned()
    }
    
    fn custom_attributes(&self) -> HashMap<String, String> {
        self.oauth2_request.metadata.clone()
    }
    
    fn inner(&self) -> &OAuth2Request {
        &self.oauth2_request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_request_creation() {
        let token = "test_bearer_token";
        let request = OAuth2Request::new(token.to_string());
        
        assert_eq!(request.bearer_token, token);
        assert_eq!(request.method, None);
        assert!(request.metadata.is_empty());
    }

    #[test]
    fn test_oauth2_request_builder_pattern() {
        let token = "test_token";
        let method = "tools/call";
        let request = OAuth2Request::new(token.to_string())
            .with_method(method.to_string())
            .with_metadata("client_ip", "192.168.1.1")
            .with_metadata("user_agent", "test-client/1.0");
        
        assert_eq!(request.bearer_token, token);
        assert_eq!(request.method, Some(method.to_string()));
        assert_eq!(request.metadata.get("client_ip"), Some(&"192.168.1.1".to_string()));
        assert_eq!(request.metadata.get("user_agent"), Some(&"test-client/1.0".to_string()));
    }

    #[test]
    fn test_oauth2_auth_request_trait_implementation() {
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        metadata.insert("another_key".to_string(), "another_value".to_string());
        
        let oauth2_request = OAuth2Request::new("token".to_string())
            .with_metadata_map(metadata.clone());
        let auth_request = OAuth2AuthRequest::new(oauth2_request);
        
        // Test custom_attribute
        assert_eq!(auth_request.custom_attribute("test_key"), Some("test_value".to_string()));
        assert_eq!(auth_request.custom_attribute("nonexistent"), None);
        
        // Test custom_attributes
        let returned_metadata = auth_request.custom_attributes();
        assert_eq!(returned_metadata, metadata);
        
        // Test inner
        assert_eq!(auth_request.inner().bearer_token, "token");
    }
}

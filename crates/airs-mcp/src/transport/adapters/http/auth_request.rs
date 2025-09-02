//! HTTP Authentication Request
//!
//! HTTP-specific authentication request implementation.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use crate::authentication::request::AuthRequest;

/// HTTP authentication request implementation
#[derive(Debug)]
pub struct HttpAuthRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub client_ip: Option<String>,
    pub custom_attributes: HashMap<String, String>,
}

impl HttpAuthRequest {
    /// Create new HTTP auth request
    pub fn new(
        method: String,
        path: String,
        headers: HashMap<String, String>,
        query_params: HashMap<String, String>,
    ) -> Self {
        let mut custom_attributes = HashMap::new();
        
        // Pre-populate common attributes that strategies might need
        custom_attributes.insert("http_method".to_string(), method.clone());
        custom_attributes.insert("http_path".to_string(), path.clone());
        
        // Add all headers with "header:" prefix (case-insensitive)
        for (key, value) in &headers {
            custom_attributes.insert(format!("header:{}", key.to_lowercase()), value.clone());
        }
        
        // Add all query params with "query:" prefix
        for (key, value) in &query_params {
            custom_attributes.insert(format!("query:{}", key), value.clone());
        }
        
        Self {
            method,
            path,
            headers,
            query_params,
            body: None,
            client_ip: None,
            custom_attributes,
        }
    }
    
    /// Set client IP
    pub fn with_client_ip(mut self, client_ip: String) -> Self {
        self.custom_attributes.insert("client_ip".to_string(), client_ip.clone());
        self.client_ip = Some(client_ip);
        self
    }
    
    /// Set request body
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
    
    /// Add custom attribute
    pub fn add_custom_attribute<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.custom_attributes.insert(key.into(), value.into());
        self
    }
}

impl AuthRequest<HttpAuthRequest> for HttpAuthRequest {
    fn custom_attribute(&self, key: &str) -> Option<String> {
        self.custom_attributes.get(key).cloned()
    }
    
    fn custom_attributes(&self) -> HashMap<String, String> {
        self.custom_attributes.clone()
    }
    
    fn inner(&self) -> &HttpAuthRequest {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_auth_request() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        headers.insert("User-Agent".to_string(), "test-agent".to_string());
        
        let mut query_params = HashMap::new();
        query_params.insert("api_key".to_string(), "key123".to_string());
        
        let request = HttpAuthRequest::new(
            "GET".to_string(),
            "/api/test".to_string(),
            headers,
            query_params,
        ).with_client_ip("192.168.1.1".to_string());
        
        // Test predefined attributes
        assert_eq!(request.custom_attribute("http_method"), Some("GET".to_string()));
        assert_eq!(request.custom_attribute("http_path"), Some("/api/test".to_string()));
        assert_eq!(request.custom_attribute("client_ip"), Some("192.168.1.1".to_string()));
        
        // Test header access (case-insensitive)
        assert_eq!(
            request.custom_attribute("header:authorization"),
            Some("Bearer token123".to_string())
        );
        assert_eq!(
            request.custom_attribute("header:user-agent"),
            Some("test-agent".to_string())
        );
        
        // Test query param access
        assert_eq!(
            request.custom_attribute("query:api_key"),
            Some("key123".to_string())
        );
    }
}

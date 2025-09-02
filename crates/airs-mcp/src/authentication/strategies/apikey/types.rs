//! API Key Authentication Request Types
//!
//! Core types for API key authentication requests and sources.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports

/// API Key authentication request containing key location and validation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRequest {
    /// The API key value to validate
    pub api_key: String,
    /// Where the API key was found (header name, query parameter, etc.)
    pub source: ApiKeySource,
    /// Optional metadata for additional validation context
    pub metadata: HashMap<String, String>,
}

/// Enum defining different API key sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeySource {
    /// Authorization header with Bearer scheme: "Authorization: Bearer <key>"
    AuthorizationBearer,
    /// Custom header: "X-API-Key: <key>", "API-Key: <key>", etc.
    Header(String),
    /// Query parameter: "?api_key=<key>", "?apikey=<key>", etc.
    QueryParameter(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_sources() {
        // Test different API key source types
        let sources = vec![
            ApiKeySource::AuthorizationBearer,
            ApiKeySource::Header("X-API-Key".to_string()),
            ApiKeySource::QueryParameter("api_key".to_string()),
        ];

        for source in sources {
            let request = ApiKeyRequest {
                api_key: "test_key".to_string(),
                source,
                metadata: HashMap::new(),
            };
            // Just verify the structure compiles and works
            assert_eq!(request.api_key, "test_key");
        }
    }
}

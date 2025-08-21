//! OAuth 2.0 Protected Resource Metadata (RFC 9728)
//!
//! This module provides the protected resource metadata endpoint that publishes
//! OAuth 2.0 server configuration and scope requirements for MCP servers.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use axum::{response::Json, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::oauth2::{config::OAuth2Config, error::OAuth2Result};

/// OAuth 2.0 Protected Resource Metadata as defined in RFC 9728
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtectedResourceMetadata {
    /// The protected resource identifier  
    pub resource: String,
    
    /// Authorization server identifier
    pub authorization_server: String,
    
    /// JSON Web Key Set endpoint for token validation
    pub jwks_uri: String,
    
    /// Supported token types (Bearer)
    pub bearer_methods_supported: Vec<String>,
    
    /// Resource indicator values (RFC 8707)
    pub resource_indicators_supported: Vec<String>,
    
    /// Supported scopes for this resource
    pub scopes_supported: Vec<String>,
    
    /// Resource documentation URL
    pub resource_documentation: Option<String>,
    
    /// Supported signing algorithms
    pub signed_metadata_supported: Option<Vec<String>>,
    
    /// Timestamp when metadata was generated
    pub metadata_generated_at: DateTime<Utc>,
    
    /// Additional metadata extensions
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl ProtectedResourceMetadata {
    /// Create new protected resource metadata from OAuth configuration
    pub fn new(config: &OAuth2Config) -> OAuth2Result<Self> {
        let metadata = Self {
            resource: config.audience.clone(),
            authorization_server: config.issuer.clone(),
            jwks_uri: config.jwks_url.to_string(),
            bearer_methods_supported: vec!["header".to_string()],
            resource_indicators_supported: vec![config.audience.clone()],
            scopes_supported: Self::extract_supported_scopes(config),
            resource_documentation: config.documentation_url.clone(),
            signed_metadata_supported: Some(vec!["RS256".to_string()]),
            metadata_generated_at: Utc::now(),
            extensions: HashMap::new(),
        };
        
        Ok(metadata)
    }
    
    /// Extract supported scopes from configuration
    fn extract_supported_scopes(config: &OAuth2Config) -> Vec<String> {
        let mut scopes = vec![
            "mcp:*".to_string(),
            "mcp:tools:*".to_string(),
            "mcp:resources:*".to_string(),
            "mcp:prompts:*".to_string(),
        ];
        
        // Add scopes from default mappings
        for mapping in &config.scope_mappings {
            if !scopes.contains(&mapping.scope) {
                scopes.push(mapping.scope.clone());
            }
        }
        
        scopes.sort();
        scopes
    }
    
    /// Add custom extension to metadata
    pub fn with_extension(mut self, key: String, value: serde_json::Value) -> Self {
        self.extensions.insert(key, value);
        self
    }
}

/// Axum handler for the protected resource metadata endpoint
/// 
/// Serves the /.well-known/oauth-protected-resource endpoint as defined in RFC 9728
pub async fn oauth_metadata_handler(config: OAuth2Config) -> Result<Json<ProtectedResourceMetadata>, StatusCode> {
    match ProtectedResourceMetadata::new(&config) {
        Ok(metadata) => Ok(Json(metadata)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn create_test_config() -> OAuth2Config {
        OAuth2Config::builder()
            .audience("mcp-server".to_string())
            .issuer("https://auth.example.com".to_string())
            .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json").unwrap())
            .build()
            .unwrap()
    }

    #[test]
    fn test_metadata_creation() {
        let config = create_test_config();
        let metadata = ProtectedResourceMetadata::new(&config).unwrap();
        
        assert_eq!(metadata.resource, "mcp-server");
        assert_eq!(metadata.authorization_server, "https://auth.example.com");
        assert_eq!(metadata.jwks_uri, "https://auth.example.com/.well-known/jwks.json");
        assert!(metadata.bearer_methods_supported.contains(&"header".to_string()));
        assert!(metadata.scopes_supported.contains(&"mcp:*".to_string()));
    }

    #[test]
    fn test_metadata_serialization() {
        let config = create_test_config();
        let metadata = ProtectedResourceMetadata::new(&config).unwrap();
        
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"resource\":\"mcp-server\""));
        assert!(json.contains("\"bearer_methods_supported\""));
    }

    #[test]
    fn test_metadata_with_extensions() {
        let config = create_test_config();
        let metadata = ProtectedResourceMetadata::new(&config)
            .unwrap()
            .with_extension("custom_field".to_string(), serde_json::json!("custom_value"));
        
        assert_eq!(metadata.extensions.get("custom_field"), Some(&serde_json::json!("custom_value")));
    }

    #[test]
    fn test_scope_extraction() {
        let config = create_test_config();
        let scopes = ProtectedResourceMetadata::extract_supported_scopes(&config);
        
        assert!(scopes.contains(&"mcp:*".to_string()));
        assert!(scopes.contains(&"mcp:tools:*".to_string()));
        assert!(scopes.contains(&"mcp:resources:*".to_string()));
        assert!(scopes.contains(&"mcp:prompts:*".to_string()));
    }

    #[tokio::test]
    async fn test_metadata_handler() {
        let config = create_test_config();
        let result = oauth_metadata_handler(config).await;
        
        assert!(result.is_ok());
        let metadata = result.unwrap().0;
        assert_eq!(metadata.resource, "mcp-server");
    }
}

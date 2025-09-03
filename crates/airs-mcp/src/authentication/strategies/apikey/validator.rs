//! API Key Validator Implementations
//!
//! This module contains different validator implementations for API key authentication.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::types::ApiKeyRequest;
use crate::authentication::{AuthContext, AuthError};

/// API key authentication data
#[derive(Debug, Clone)]
pub struct ApiKeyAuthData {
    pub key_id: String,
    pub source: super::types::ApiKeySource,
}

/// Validator for API keys - can be customized for different validation logic
#[async_trait]
pub trait ApiKeyValidator: Send + Sync {
    /// Validate an API key and return user context information
    async fn validate_api_key(&self, request: &ApiKeyRequest) -> Result<AuthContext<ApiKeyAuthData>, AuthError>;
}

/// Simple in-memory API key validator for testing and simple use cases
#[derive(Debug, Clone)]
pub struct InMemoryApiKeyValidator {
    valid_keys: HashMap<String, AuthContext<ApiKeyAuthData>>,
}

impl InMemoryApiKeyValidator {
    /// Create a new in-memory validator with the given valid keys
    pub fn new(valid_keys: HashMap<String, AuthContext<ApiKeyAuthData>>) -> Self {
        Self { valid_keys }
    }

    /// Add a valid API key with associated auth context
    pub fn add_key(&mut self, key: String, context: AuthContext<ApiKeyAuthData>) {
        self.valid_keys.insert(key, context);
    }
}

#[async_trait]
impl ApiKeyValidator for InMemoryApiKeyValidator {
    async fn validate_api_key(&self, request: &ApiKeyRequest) -> Result<AuthContext<ApiKeyAuthData>, AuthError> {
        self.valid_keys
            .get(&request.api_key)
            .cloned()
            .ok_or_else(|| AuthError::InvalidCredentials("Invalid API key".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::strategies::apikey::types::{ApiKeyRequest, ApiKeySource};

    #[tokio::test]
    async fn test_in_memory_validator_valid_key() {
        use crate::authentication::AuthMethod;
        
        let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
        let auth_context = AuthContext::new(
            AuthMethod::new("apikey"), 
            ApiKeyAuthData {
                key_id: "test_user".to_string(),
                source: ApiKeySource::AuthorizationBearer,
            }
        );
        validator.add_key("valid_key_123".to_string(), auth_context);

        let request = ApiKeyRequest {
            api_key: "valid_key_123".to_string(),
            source: ApiKeySource::AuthorizationBearer,
            metadata: HashMap::new(),
        };

        let result = validator.validate_api_key(&request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().auth_data.key_id, "test_user");
    }

    #[tokio::test]
    async fn test_in_memory_validator_invalid_key() {
        let validator = InMemoryApiKeyValidator::new(HashMap::new());
        let request = ApiKeyRequest {
            api_key: "invalid_key".to_string(),
            source: ApiKeySource::Header("X-API-Key".to_string()),
            metadata: HashMap::new(),
        };

        let result = validator.validate_api_key(&request).await;
        assert!(result.is_err());
    }
}

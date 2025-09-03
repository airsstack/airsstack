//! API Key Authentication Strategy Implementation
//!
//! This module contains the core ApiKeyStrategy implementation that
//! validates API keys using pluggable validators.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::types::ApiKeyRequest;
use super::validator::{ApiKeyAuthData, ApiKeyValidator};
use crate::authentication::{
    AuthContext, AuthMethod, AuthRequest, AuthResult, AuthenticationStrategy,
};

/// Simple API key strategy that validates keys using a provided validator
#[derive(Clone)]
pub struct ApiKeyStrategy<V>
where
    V: Clone,
{
    validator: V,
}

impl<V> ApiKeyStrategy<V>
where
    V: ApiKeyValidator + Clone + 'static,
{
    /// Create a new API key strategy with the given validator
    pub fn new(validator: V) -> Self {
        Self { validator }
    }
}

#[async_trait]
impl<V> AuthenticationStrategy<ApiKeyRequest, ApiKeyAuthData> for ApiKeyStrategy<V>
where
    V: ApiKeyValidator + Clone + 'static,
{
    fn method(&self) -> AuthMethod {
        AuthMethod::new("apikey")
    }

    async fn authenticate(
        &self,
        request: &impl AuthRequest<ApiKeyRequest>,
    ) -> AuthResult<AuthContext<ApiKeyAuthData>> {
        // Extract the inner ApiKeyRequest from the wrapper
        let api_key_request = request.inner();

        // Use the validator to validate the API key request
        self.validator.validate_api_key(api_key_request).await
    }

    async fn validate(&self, _context: &AuthContext<ApiKeyAuthData>) -> AuthResult<bool> {
        // For API keys, validation is typically just checking if the context is still valid
        // In a real implementation, you might check expiration, revocation lists, etc.
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::strategies::apikey::types::ApiKeySource;
    use crate::authentication::strategies::apikey::validator::InMemoryApiKeyValidator;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_api_key_strategy_valid_key() {
        use crate::authentication::AuthMethod;
        use std::collections::HashMap;

        let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
        let auth_context = AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "test_user".to_string(),
                source: ApiKeySource::AuthorizationBearer,
            },
        );
        validator.add_key("valid_key_123".to_string(), auth_context);

        let strategy = ApiKeyStrategy::new(validator);

        // Create a mock request
        struct MockRequest {
            attributes: HashMap<String, String>,
            inner_request: ApiKeyRequest,
        }

        impl AuthRequest<ApiKeyRequest> for MockRequest {
            fn custom_attribute(&self, key: &str) -> Option<String> {
                self.attributes.get(key).cloned()
            }

            fn custom_attributes(&self) -> HashMap<String, String> {
                self.attributes.clone()
            }

            fn inner(&self) -> &ApiKeyRequest {
                &self.inner_request
            }
        }

        let mut attributes = HashMap::new();
        attributes.insert("api_key".to_string(), "valid_key_123".to_string());
        attributes.insert("api_key_source".to_string(), "bearer".to_string());

        let inner_request = ApiKeyRequest {
            api_key: "valid_key_123".to_string(),
            source: ApiKeySource::AuthorizationBearer,
            metadata: HashMap::new(),
        };

        let request = MockRequest {
            attributes,
            inner_request,
        };

        let result = strategy.authenticate(&request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().auth_data.key_id, "test_user");
    }

    #[tokio::test]
    async fn test_api_key_strategy_invalid_key() {
        let validator = InMemoryApiKeyValidator::new(HashMap::new());
        let strategy = ApiKeyStrategy::new(validator);

        // Create a mock request with invalid key
        struct MockRequest {
            attributes: HashMap<String, String>,
            inner_request: ApiKeyRequest,
        }

        impl AuthRequest<ApiKeyRequest> for MockRequest {
            fn custom_attribute(&self, key: &str) -> Option<String> {
                self.attributes.get(key).cloned()
            }

            fn custom_attributes(&self) -> HashMap<String, String> {
                self.attributes.clone()
            }

            fn inner(&self) -> &ApiKeyRequest {
                &self.inner_request
            }
        }

        let mut attributes = HashMap::new();
        attributes.insert("api_key".to_string(), "invalid_key".to_string());
        attributes.insert("api_key_source".to_string(), "header".to_string());

        let inner_request = ApiKeyRequest {
            api_key: "invalid_key".to_string(),
            source: ApiKeySource::Header("X-API-Key".to_string()),
            metadata: HashMap::new(),
        };

        let request = MockRequest {
            attributes,
            inner_request,
        };

        let result = strategy.authenticate(&request).await;
        assert!(result.is_err());
    }
}

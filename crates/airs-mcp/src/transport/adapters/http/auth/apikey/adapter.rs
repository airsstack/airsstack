//! API Key HTTP Authentication Adapter
//!
//! This module provides the core ApiKeyStrategyAdapter that bridges
//! HTTP requests to API key authentication strategies.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;

// Layer 2: Third-party crate imports
use axum::http::{HeaderMap, Uri};

// Layer 3: Internal module imports
use super::super::oauth2::error::HttpAuthError;
use crate::authentication::{
    strategies::apikey::{
        ApiKeyAuthData, ApiKeyRequest, ApiKeySource, ApiKeyStrategy, ApiKeyValidator,
    },
    AuthContext, AuthRequest, AuthenticationStrategy,
};

/// Wrapper for ApiKeyRequest that implements AuthRequest trait
#[derive(Debug)]
struct ApiKeyRequestWrapper {
    request: ApiKeyRequest,
    attributes: HashMap<String, String>,
}

impl ApiKeyRequestWrapper {
    fn new(request: ApiKeyRequest) -> Self {
        // Convert metadata to custom attributes
        let attributes = request.metadata.clone();
        Self {
            request,
            attributes,
        }
    }
}

impl AuthRequest<ApiKeyRequest> for ApiKeyRequestWrapper {
    fn custom_attribute(&self, key: &str) -> Option<String> {
        self.attributes.get(key).cloned()
    }

    fn custom_attributes(&self) -> HashMap<String, String> {
        self.attributes.clone()
    }

    fn inner(&self) -> &ApiKeyRequest {
        &self.request
    }
}

/// Configuration for API key extraction from HTTP requests
#[derive(Debug, Clone)]
pub struct ApiKeyConfig {
    /// Custom header names to check for API keys (e.g., "X-API-Key", "API-Key")
    pub custom_headers: Vec<String>,
    /// Query parameter names to check for API keys (e.g., "api_key", "apikey")
    pub query_parameters: Vec<String>,
    /// Whether to check Authorization header for Bearer tokens
    pub check_bearer_token: bool,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            custom_headers: vec![
                "X-API-Key".to_string(),
                "API-Key".to_string(),
                "X-Api-Key".to_string(),
            ],
            query_parameters: vec![
                "api_key".to_string(),
                "apikey".to_string(),
                "key".to_string(),
            ],
            check_bearer_token: true,
        }
    }
}

/// HTTP adapter for API key authentication strategies
pub struct ApiKeyStrategyAdapter<V> {
    strategy: ApiKeyStrategy<V>,
    config: ApiKeyConfig,
}

impl<V> ApiKeyStrategyAdapter<V>
where
    V: ApiKeyValidator + 'static,
{
    /// Create a new API key strategy adapter
    pub fn new(strategy: ApiKeyStrategy<V>, config: ApiKeyConfig) -> Self {
        Self { strategy, config }
    }

    /// Create a new API key strategy adapter with default configuration
    pub fn with_default_config(strategy: ApiKeyStrategy<V>) -> Self {
        Self::new(strategy, ApiKeyConfig::default())
    }

    /// Extract API key from HTTP request headers and query parameters
    fn extract_api_key(
        &self,
        headers: &HeaderMap,
        uri: &Uri,
    ) -> Result<ApiKeyRequest, HttpAuthError> {
        // First, try Authorization header with Bearer scheme
        if self.config.check_bearer_token {
            if let Some(auth_header) = headers.get("authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        return Ok(ApiKeyRequest {
                            api_key: token.to_string(),
                            source: ApiKeySource::AuthorizationBearer,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        // Try custom headers
        for header_name in &self.config.custom_headers {
            if let Some(header_value) = headers.get(header_name.as_str()) {
                if let Ok(key) = header_value.to_str() {
                    return Ok(ApiKeyRequest {
                        api_key: key.to_string(),
                        source: ApiKeySource::Header(header_name.clone()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        // Try query parameters
        if let Some(query) = uri.query() {
            for param_name in &self.config.query_parameters {
                if let Some(key) = self.parse_query_parameter(query, param_name) {
                    return Ok(ApiKeyRequest {
                        api_key: key,
                        source: ApiKeySource::QueryParameter(param_name.clone()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        Err(HttpAuthError::MissingApiKey)
    }

    /// Parse a specific query parameter from the query string
    fn parse_query_parameter(&self, query: &str, param_name: &str) -> Option<String> {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == param_name {
                    return Some(urlencoding::decode(value).ok()?.into_owned());
                }
            }
        }
        None
    }

    /// Authenticate an HTTP request using API key extraction and validation
    pub async fn authenticate_http(
        &self,
        headers: &HeaderMap,
        uri: &Uri,
    ) -> Result<AuthContext<ApiKeyAuthData>, HttpAuthError> {
        // Extract API key from HTTP request
        let api_key_request = self.extract_api_key(headers, uri)?;

        // Wrap in AuthRequest trait
        let wrapped_request = ApiKeyRequestWrapper::new(api_key_request);

        // Authenticate using the strategy
        self.strategy
            .authenticate(&wrapped_request)
            .await
            .map_err(HttpAuthError::from)
    }
}

impl<V> fmt::Display for ApiKeyStrategyAdapter<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKeyStrategyAdapter")
    }
}

impl<V> fmt::Debug for ApiKeyStrategyAdapter<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKeyStrategyAdapter")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::strategies::apikey::InMemoryApiKeyValidator;
    use axum::http::{HeaderMap, HeaderValue, Uri};

    #[tokio::test]
    async fn test_bearer_token_extraction() {
        use crate::authentication::AuthMethod;

        let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
        let auth_context = AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "test_user".to_string(),
                source: ApiKeySource::AuthorizationBearer,
            },
        );
        validator.add_key("test_token_123".to_string(), auth_context);

        let strategy = ApiKeyStrategy::new(validator);
        let adapter = ApiKeyStrategyAdapter::with_default_config(strategy);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test_token_123"),
        );

        let uri: Uri = "http://example.com/api".parse().unwrap();
        let result = adapter.authenticate_http(&headers, &uri).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().auth_data.key_id, "test_user");
    }

    #[tokio::test]
    async fn test_custom_header_extraction() {
        use crate::authentication::AuthMethod;

        let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
        let auth_context = AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "api_user".to_string(),
                source: ApiKeySource::Header("X-API-Key".to_string()),
            },
        );
        validator.add_key("custom_key_456".to_string(), auth_context);

        let strategy = ApiKeyStrategy::new(validator);
        let adapter = ApiKeyStrategyAdapter::with_default_config(strategy);

        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_static("custom_key_456"));

        let uri: Uri = "http://example.com/api".parse().unwrap();
        let result = adapter.authenticate_http(&headers, &uri).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().auth_data.key_id, "api_user");
    }

    #[tokio::test]
    async fn test_query_parameter_extraction() {
        use crate::authentication::AuthMethod;

        let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
        let auth_context = AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "query_user".to_string(),
                source: ApiKeySource::QueryParameter("api_key".to_string()),
            },
        );
        validator.add_key("query_key_789".to_string(), auth_context);

        let strategy = ApiKeyStrategy::new(validator);
        let adapter = ApiKeyStrategyAdapter::with_default_config(strategy);

        let headers = HeaderMap::new();
        let uri: Uri = "http://example.com/api?api_key=query_key_789"
            .parse()
            .unwrap();
        let result = adapter.authenticate_http(&headers, &uri).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().auth_data.key_id, "query_user");
    }

    #[tokio::test]
    async fn test_missing_api_key() {
        let validator = InMemoryApiKeyValidator::new(HashMap::new());
        let strategy = ApiKeyStrategy::new(validator);
        let adapter = ApiKeyStrategyAdapter::with_default_config(strategy);

        let headers = HeaderMap::new();
        let uri: Uri = "http://example.com/api".parse().unwrap();
        let result = adapter.authenticate_http(&headers, &uri).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HttpAuthError::MissingApiKey => {}
            _ => panic!("Expected MissingApiKey error"),
        }
    }

    #[tokio::test]
    async fn test_invalid_api_key() {
        let validator = InMemoryApiKeyValidator::new(HashMap::new());
        let strategy = ApiKeyStrategy::new(validator);
        let adapter = ApiKeyStrategyAdapter::with_default_config(strategy);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer invalid_key"),
        );

        let uri: Uri = "http://example.com/api".parse().unwrap();
        let result = adapter.authenticate_http(&headers, &uri).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            HttpAuthError::AuthError(_) => {}
            _ => panic!("Expected AuthError variant"),
        }
    }

    #[tokio::test]
    async fn test_custom_config() {
        use crate::authentication::AuthMethod;

        let mut validator = InMemoryApiKeyValidator::new(HashMap::new());
        let auth_context = AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "custom_user".to_string(),
                source: ApiKeySource::Header("Custom-Auth-Header".to_string()),
            },
        );
        validator.add_key("custom_key".to_string(), auth_context);

        let strategy = ApiKeyStrategy::new(validator);

        let config = ApiKeyConfig {
            custom_headers: vec!["Custom-Auth-Header".to_string()],
            query_parameters: vec!["custom_key".to_string()],
            check_bearer_token: false,
        };

        let adapter = ApiKeyStrategyAdapter::new(strategy, config);

        let mut headers = HeaderMap::new();
        headers.insert("custom-auth-header", HeaderValue::from_static("custom_key"));

        let uri: Uri = "http://example.com/api".parse().unwrap();
        let result = adapter.authenticate_http(&headers, &uri).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().auth_data.key_id, "custom_user");
    }
}

//! Token Refresh Implementation
//!
//! This module provides token refresh capabilities including automatic token
//! renewal, refresh token validation, and integration with OAuth 2.1 authorization servers.

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, error, info};
use url::Url;

// Layer 3: Internal module imports
use super::traits::TokenRefreshProvider;
use super::types::{RefreshTokenRequest, RefreshTokenResponse};
use crate::oauth2::{AuthContext, OAuth2Config, OAuth2Error, OAuth2Result};

/// Refresh token strategy configuration
#[derive(Debug, Clone)]
pub enum RefreshTokenStrategy {
    /// Automatic refresh when token is close to expiration
    Automatic {
        /// Threshold before expiration to trigger refresh
        threshold: Duration,
        /// Maximum number of retry attempts
        max_retries: u32,
    },

    /// Manual refresh only when explicitly requested
    Manual,

    /// Proactive refresh at regular intervals
    Proactive {
        /// Interval between refresh attempts
        refresh_interval: Duration,
        /// Minimum time before expiration to allow refresh
        min_lifetime_remaining: Duration,
    },
}

impl Default for RefreshTokenStrategy {
    fn default() -> Self {
        Self::Automatic {
            threshold: Duration::from_secs(300), // 5 minutes
            max_retries: 3,
        }
    }
}

/// Configuration for refresh token handler
#[derive(Debug, Clone)]
pub struct RefreshTokenConfig {
    /// OAuth 2.1 configuration
    pub oauth_config: OAuth2Config,

    /// Token endpoint URL for refresh requests
    pub token_endpoint: Url,

    /// Client ID for refresh requests
    pub client_id: String,

    /// Client secret (if using confidential client)
    pub client_secret: Option<String>,

    /// Refresh strategy
    pub strategy: RefreshTokenStrategy,

    /// HTTP client timeout
    pub request_timeout: Duration,

    /// Enable refresh token rotation
    pub enable_token_rotation: bool,

    /// Additional parameters to include in refresh requests
    pub additional_params: std::collections::HashMap<String, String>,
}

impl RefreshTokenConfig {
    /// Create a new refresh token configuration
    pub fn new(oauth_config: OAuth2Config, token_endpoint: Url, client_id: String) -> Self {
        Self {
            oauth_config,
            token_endpoint,
            client_id,
            client_secret: None,
            strategy: RefreshTokenStrategy::default(),
            request_timeout: Duration::from_secs(30),
            enable_token_rotation: true,
            additional_params: std::collections::HashMap::new(),
        }
    }

    /// Set client secret for confidential clients
    pub fn with_client_secret(mut self, secret: String) -> Self {
        self.client_secret = Some(secret);
        self
    }

    /// Set refresh strategy
    pub fn with_strategy(mut self, strategy: RefreshTokenStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Enable or disable token rotation
    pub fn with_token_rotation(mut self, enabled: bool) -> Self {
        self.enable_token_rotation = enabled;
        self
    }

    /// Add additional parameter to refresh requests
    pub fn with_param(mut self, key: String, value: String) -> Self {
        self.additional_params.insert(key, value);
        self
    }
}

/// Refresh token handler implementation
#[derive(Debug)]
pub struct RefreshTokenHandler {
    /// Configuration
    config: RefreshTokenConfig,

    /// HTTP client for making refresh requests
    http_client: Client,

    /// Metrics for monitoring refresh operations
    metrics: Arc<tokio::sync::RwLock<RefreshMetrics>>,
}

/// Metrics for refresh token operations
#[derive(Debug, Default, Clone)]
pub struct RefreshMetrics {
    /// Total number of refresh attempts
    pub total_attempts: u64,

    /// Number of successful refreshes
    pub successful_refreshes: u64,

    /// Number of failed refreshes
    pub failed_refreshes: u64,

    /// Number of expired refresh tokens encountered
    pub expired_refresh_tokens: u64,

    /// Number of invalid refresh tokens encountered
    pub invalid_refresh_tokens: u64,

    /// Average refresh duration in milliseconds
    pub average_refresh_duration_ms: f64,

    /// Last refresh timestamp
    pub last_refresh_at: Option<DateTime<Utc>>,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

impl RefreshMetrics {
    /// Update success rate calculation
    pub fn calculate_success_rate(&mut self) {
        if self.total_attempts > 0 {
            self.success_rate = self.successful_refreshes as f64 / self.total_attempts as f64;
        } else {
            self.success_rate = 0.0;
        }
    }
}

impl RefreshTokenHandler {
    /// Create a new refresh token handler
    pub fn new(config: RefreshTokenConfig) -> OAuth2Result<Self> {
        let http_client = Client::builder()
            .timeout(config.request_timeout)
            .build()
            .map_err(|e| OAuth2Error::NetworkError(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
            metrics: Arc::new(tokio::sync::RwLock::new(RefreshMetrics::default())),
        })
    }

    /// Get refresh metrics
    pub async fn get_metrics(&self) -> RefreshMetrics {
        self.metrics.read().await.clone()
    }

    /// Update metrics for a refresh attempt
    async fn update_metrics(&self, success: bool, duration: Duration) {
        let mut metrics = self.metrics.write().await;

        metrics.total_attempts += 1;
        if success {
            metrics.successful_refreshes += 1;
        } else {
            metrics.failed_refreshes += 1;
        }

        metrics.last_refresh_at = Some(Utc::now());

        // Update average duration (exponential moving average)
        let duration_ms = duration.as_millis() as f64;
        if metrics.total_attempts == 1 {
            metrics.average_refresh_duration_ms = duration_ms;
        } else {
            let alpha = 0.1; // Smoothing factor
            metrics.average_refresh_duration_ms =
                alpha * duration_ms + (1.0 - alpha) * metrics.average_refresh_duration_ms;
        }

        metrics.calculate_success_rate();
    }

    /// Build refresh request parameters
    fn build_refresh_params(
        &self,
        request: &RefreshTokenRequest,
    ) -> std::collections::HashMap<String, String> {
        let mut params = std::collections::HashMap::new();

        // Required OAuth 2.1 parameters
        params.insert("grant_type".to_string(), "refresh_token".to_string());
        params.insert("refresh_token".to_string(), request.refresh_token.clone());
        params.insert("client_id".to_string(), request.client_id.clone());

        // Client secret if configured
        if let Some(ref secret) = self.config.client_secret {
            params.insert("client_secret".to_string(), secret.clone());
        }

        // Scope if specified
        if let Some(ref scope) = request.scope {
            params.insert("scope".to_string(), scope.clone());
        }

        // Additional configured parameters
        params.extend(self.config.additional_params.clone());

        // Additional request parameters
        params.extend(request.additional_params.clone());

        params
    }

    /// Parse refresh response from JSON
    fn parse_refresh_response(&self, json: Value) -> OAuth2Result<RefreshTokenResponse> {
        let access_token = json
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OAuth2Error::InvalidTokenResponse("Missing access_token".to_string()))?
            .to_string();

        let token_type = json
            .get("token_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Bearer")
            .to_string();

        let expires_in = json.get("expires_in").and_then(|v| v.as_u64());

        let refresh_token = json
            .get("refresh_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let scope = json
            .get("scope")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut additional_data = std::collections::HashMap::new();
        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if ![
                    "access_token",
                    "token_type",
                    "expires_in",
                    "refresh_token",
                    "scope",
                ]
                .contains(&key.as_str())
                {
                    additional_data.insert(key, value);
                }
            }
        }

        Ok(RefreshTokenResponse {
            access_token,
            token_type,
            expires_in,
            refresh_token,
            scope,
            additional_data,
        })
    }

    /// Handle OAuth error response
    fn handle_error_response(&self, json: Value) -> OAuth2Error {
        let error = json
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_error");

        let error_description = json
            .get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match error {
            "invalid_grant" => OAuth2Error::InvalidRefreshToken,
            "invalid_client" => OAuth2Error::InvalidClient,
            "invalid_request" => OAuth2Error::InvalidRequest(error_description.to_string()),
            "unsupported_grant_type" => OAuth2Error::UnsupportedGrantType,
            _ => OAuth2Error::RefreshFailed(format!("{error}: {error_description}")),
        }
    }
}

#[async_trait]
impl TokenRefreshProvider for RefreshTokenHandler {
    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> OAuth2Result<RefreshTokenResponse> {
        let start_time = std::time::Instant::now();

        debug!("Starting token refresh for client: {}", request.client_id);

        let params = self.build_refresh_params(&request);

        // Make the refresh request
        let response = self
            .http_client
            .post(self.config.token_endpoint.clone())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| OAuth2Error::NetworkError(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| OAuth2Error::NetworkError(e.to_string()))?;

        let duration = start_time.elapsed();

        // Parse JSON response
        let json: Value = serde_json::from_str(&response_text)
            .map_err(|e| OAuth2Error::InvalidTokenResponse(e.to_string()))?;

        if status.is_success() {
            // Successful refresh
            let refresh_response = self.parse_refresh_response(json)?;
            self.update_metrics(true, duration).await;

            info!(
                "Token refresh successful for client: {} (took: {:?})",
                request.client_id, duration
            );

            Ok(refresh_response)
        } else {
            // Error response
            let oauth_error = self.handle_error_response(json);
            self.update_metrics(false, duration).await;

            // Update specific error metrics
            {
                let mut metrics = self.metrics.write().await;
                if oauth_error == OAuth2Error::InvalidRefreshToken {
                    metrics.invalid_refresh_tokens += 1;
                }
            }

            error!(
                "Token refresh failed for client: {} with status: {} (took: {:?})",
                request.client_id, status, duration
            );

            Err(oauth_error)
        }
    }

    async fn should_refresh(&self, auth_context: &AuthContext) -> OAuth2Result<bool> {
        if auth_context.expires_at.is_none() {
            return Ok(false); // No expiration, no need to refresh
        }

        let expires_at = auth_context.expires_at.unwrap();
        let now = Utc::now();

        // Check if already expired
        if now >= expires_at {
            return Ok(true);
        }

        // Check strategy-specific conditions
        match &self.config.strategy {
            RefreshTokenStrategy::Automatic { threshold, .. } => {
                let time_until_expiry = expires_at.signed_duration_since(now);
                let threshold_duration = chrono::Duration::from_std(*threshold).map_err(|_| {
                    OAuth2Error::Configuration("Invalid threshold duration".to_string())
                })?;

                Ok(time_until_expiry <= threshold_duration)
            }
            RefreshTokenStrategy::Manual => Ok(false), // Never auto-refresh
            RefreshTokenStrategy::Proactive {
                min_lifetime_remaining,
                ..
            } => {
                let time_until_expiry = expires_at.signed_duration_since(now);
                let min_duration =
                    chrono::Duration::from_std(*min_lifetime_remaining).map_err(|_| {
                        OAuth2Error::Configuration("Invalid min lifetime duration".to_string())
                    })?;

                Ok(time_until_expiry <= min_duration)
            }
        }
    }

    fn get_refresh_threshold(&self) -> Duration {
        match &self.config.strategy {
            RefreshTokenStrategy::Automatic { threshold, .. } => *threshold,
            RefreshTokenStrategy::Manual => Duration::from_secs(0),
            RefreshTokenStrategy::Proactive {
                min_lifetime_remaining,
                ..
            } => *min_lifetime_remaining,
        }
    }

    async fn validate_refresh_token(&self, refresh_token: &str) -> OAuth2Result<bool> {
        // Basic validation - check if token is not empty and has reasonable format
        if refresh_token.is_empty() {
            return Ok(false);
        }

        // Additional validation could include:
        // - JWT parsing and validation (if refresh tokens are JWTs)
        // - Database lookup for token validity
        // - Introspection endpoint call

        // For now, just check basic format
        Ok(refresh_token.len() >= 10) // Minimum reasonable length
    }

    async fn revoke_refresh_token(&self, refresh_token: &str) -> OAuth2Result<()> {
        // This would typically involve calling a revocation endpoint
        // For now, just log the revocation
        info!(
            "Refresh token revoked: {}",
            &refresh_token[..8.min(refresh_token.len())]
        );

        // In a real implementation, you would make an HTTP request to the revocation endpoint:
        // POST /revoke
        // Content-Type: application/x-www-form-urlencoded
        // token=<refresh_token>&token_type_hint=refresh_token&client_id=<client_id>

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::OAuth2Config;
    use url::Url;

    fn create_test_config() -> RefreshTokenConfig {
        let oauth_config = OAuth2Config::default();
        let token_endpoint = Url::parse("https://auth.example.com/token").unwrap();

        RefreshTokenConfig::new(oauth_config, token_endpoint, "test_client".to_string())
            .with_client_secret("test_secret".to_string())
            .with_timeout(Duration::from_secs(10))
    }

    fn create_test_auth_context(expires_in_seconds: i64) -> AuthContext {
        let jwt_claims = crate::oauth2::types::JwtClaims {
            sub: "user123".to_string(),
            aud: Some("client456".to_string()),
            iss: Some("test-issuer".to_string()),
            exp: Some(chrono::Utc::now().timestamp() + expires_in_seconds),
            nbf: None,
            iat: Some(chrono::Utc::now().timestamp()),
            jti: Some("token123".to_string()),
            scope: Some("read write".to_string()),
            scopes: None,
        };

        AuthContext::new(jwt_claims, vec!["read".to_string(), "write".to_string()])
    }

    #[test]
    fn test_refresh_config_builder() {
        let config = create_test_config();

        assert_eq!(config.client_id, "test_client");
        assert_eq!(config.client_secret, Some("test_secret".to_string()));
        assert_eq!(config.request_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_refresh_strategy_default() {
        let strategy = RefreshTokenStrategy::default();

        match strategy {
            RefreshTokenStrategy::Automatic {
                threshold,
                max_retries,
            } => {
                assert_eq!(threshold, Duration::from_secs(300));
                assert_eq!(max_retries, 3);
            }
            _ => panic!("Expected Automatic strategy"),
        }
    }

    #[tokio::test]
    async fn test_should_refresh_expired_token() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        // Create expired token
        let auth_context = create_test_auth_context(-3600); // Expired 1 hour ago

        let should_refresh = handler.should_refresh(&auth_context).await.unwrap();
        assert!(should_refresh);
    }

    #[tokio::test]
    async fn test_should_refresh_near_expiration() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        // Create token expiring in 2 minutes (less than 5-minute threshold)
        let auth_context = create_test_auth_context(120);

        let should_refresh = handler.should_refresh(&auth_context).await.unwrap();
        assert!(should_refresh);
    }

    #[tokio::test]
    async fn test_should_not_refresh_valid_token() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        // Create token expiring in 1 hour (more than 5-minute threshold)
        let auth_context = create_test_auth_context(3600);

        let should_refresh = handler.should_refresh(&auth_context).await.unwrap();
        assert!(!should_refresh);
    }

    #[tokio::test]
    async fn test_validate_refresh_token() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        // Valid token
        assert!(handler
            .validate_refresh_token("valid_refresh_token_123")
            .await
            .unwrap());

        // Invalid tokens
        assert!(!handler.validate_refresh_token("").await.unwrap());
        assert!(!handler.validate_refresh_token("short").await.unwrap());
    }

    #[test]
    fn test_build_refresh_params() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        let request = RefreshTokenRequest::new("refresh123".to_string(), "client456".to_string())
            .with_scope("read:tools".to_string())
            .with_param("custom".to_string(), "value".to_string());

        let params = handler.build_refresh_params(&request);

        assert_eq!(params.get("grant_type"), Some(&"refresh_token".to_string()));
        assert_eq!(params.get("refresh_token"), Some(&"refresh123".to_string()));
        assert_eq!(params.get("client_id"), Some(&"client456".to_string()));
        assert_eq!(
            params.get("client_secret"),
            Some(&"test_secret".to_string())
        );
        assert_eq!(params.get("scope"), Some(&"read:tools".to_string()));
        assert_eq!(params.get("custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_refresh_response() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        let json = serde_json::json!({
            "access_token": "new_access_token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "new_refresh_token",
            "scope": "read:tools write:tools",
            "custom_field": "custom_value"
        });

        let response = handler.parse_refresh_response(json).unwrap();

        assert_eq!(response.access_token, "new_access_token");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, Some(3600));
        assert_eq!(
            response.refresh_token,
            Some("new_refresh_token".to_string())
        );
        assert_eq!(response.scope, Some("read:tools write:tools".to_string()));
        assert!(response.additional_data.contains_key("custom_field"));
    }

    #[test]
    fn test_handle_error_response() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        let json = serde_json::json!({
            "error": "invalid_grant",
            "error_description": "The refresh token is invalid"
        });

        let error = handler.handle_error_response(json);

        match error {
            OAuth2Error::InvalidRefreshToken => {} // Expected
            _ => panic!("Expected InvalidRefreshToken error"),
        }
    }

    #[tokio::test]
    async fn test_metrics_calculation() {
        let config = create_test_config();
        let handler = RefreshTokenHandler::new(config).unwrap();

        // Simulate some refresh attempts
        handler
            .update_metrics(true, Duration::from_millis(100))
            .await;
        handler
            .update_metrics(false, Duration::from_millis(200))
            .await;
        handler
            .update_metrics(true, Duration::from_millis(150))
            .await;

        let metrics = handler.get_metrics().await;

        assert_eq!(metrics.total_attempts, 3);
        assert_eq!(metrics.successful_refreshes, 2);
        assert_eq!(metrics.failed_refreshes, 1);
        assert_eq!(metrics.success_rate, 2.0 / 3.0);
        assert!(metrics.average_refresh_duration_ms > 0.0);
        assert!(metrics.last_refresh_at.is_some());
    }
}

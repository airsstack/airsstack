//! OAuth 2.1 Middleware Utilities
//!
//! This module provides utility functions and builder patterns for OAuth middleware
//! configuration and setup.

// Layer 1: Standard library imports
use std::result::Result;

// Layer 2: Third-party crate imports
// (None needed for this module)

// Layer 3: Internal module imports
use super::{axum, types, OAuth2MiddlewareCore};
use crate::oauth2::OAuth2Config;

/// Convenience function to create a default OAuth middleware core
///
/// This function provides a quick way to create a middleware core with
/// sensible defaults for common use cases.
///
/// # Arguments
/// * `config` - OAuth 2.1 configuration
///
/// # Returns
/// * `Result<OAuth2MiddlewareCore, types::MiddlewareError>` - Configured middleware core
///
/// # Example
/// ```rust,no_run
/// use airs_mcp::oauth2::{OAuth2Config, middleware::utils};
/// use url::Url;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OAuth2Config::builder()
///     .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json")?)
///     .audience("mcp-server".to_string())
///     .issuer("https://auth.example.com".to_string())
///     .build()?;
///
/// let core = utils::create_default_core(config)?;
/// # Ok(())
/// # }
/// ```
pub fn create_default_core(
    config: OAuth2Config,
) -> Result<OAuth2MiddlewareCore, types::MiddlewareError> {
    OAuth2MiddlewareCore::new(config).map_err(types::MiddlewareError::OAuth)
}

/// Middleware builder for fluent configuration
///
/// Provides a builder pattern for configuring OAuth middleware with
/// various options and framework-specific adapters.
#[derive(Debug)]
pub struct MiddlewareBuilder {
    oauth_config: OAuth2Config,
    middleware_config: types::MiddlewareConfig,
}

impl MiddlewareBuilder {
    /// Create a new middleware builder
    pub fn new(oauth_config: OAuth2Config) -> Self {
        Self {
            oauth_config,
            middleware_config: types::MiddlewareConfig::default(),
        }
    }

    /// Set middleware configuration
    pub fn with_middleware_config(mut self, config: types::MiddlewareConfig) -> Self {
        self.middleware_config = config;
        self
    }

    /// Set authentication realm
    pub fn with_auth_realm(mut self, realm: String) -> Self {
        self.middleware_config.auth_realm = Some(realm);
        self
    }

    /// Add paths to skip OAuth validation
    pub fn with_skip_paths(mut self, paths: Vec<String>) -> Self {
        self.middleware_config.skip_paths = paths;
        self
    }

    /// Enable or disable error details in responses
    pub fn with_error_details(mut self, include: bool) -> Self {
        self.middleware_config.include_error_details = include;
        self
    }

    /// Set CORS configuration
    pub fn with_cors_config(mut self, cors: types::CorsConfig) -> Self {
        self.middleware_config.cors_config = Some(cors);
        self
    }

    /// Build the OAuth middleware core
    pub fn build_core(self) -> Result<OAuth2MiddlewareCore, types::MiddlewareError> {
        OAuth2MiddlewareCore::new(self.oauth_config).map_err(types::MiddlewareError::OAuth)
    }

    /// Build Axum-specific middleware
    pub fn build_axum(self) -> Result<axum::AxumOAuth2Middleware, types::MiddlewareError> {
        axum::AxumOAuth2Middleware::new(self.oauth_config).map_err(types::MiddlewareError::OAuth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    fn create_test_config() -> OAuth2Config {
        OAuth2Config::builder()
            .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json").unwrap())
            .audience("test-audience".to_string())
            .issuer("https://auth.example.com".to_string())
            .build()
            .unwrap()
    }

    #[test]
    fn test_create_default_core() {
        let config = create_test_config();
        let result = create_default_core(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_middleware_builder() {
        let config = create_test_config();
        let builder = MiddlewareBuilder::new(config)
            .with_auth_realm("test-realm".to_string())
            .with_error_details(false)
            .with_skip_paths(vec!["/custom-health".to_string()]);

        let core = builder.build_core();
        assert!(core.is_ok());
    }

    #[test]
    fn test_builder_fluent_api() {
        let config = create_test_config();
        let cors = types::CorsConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            ..Default::default()
        };

        let builder = MiddlewareBuilder::new(config)
            .with_cors_config(cors)
            .with_auth_realm("custom-realm".to_string());

        let result = builder.build_core();
        assert!(result.is_ok());
    }
}

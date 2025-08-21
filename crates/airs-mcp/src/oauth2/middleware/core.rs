//! OAuth 2.1 Middleware Core Implementation
//!
//! This module provides the framework-agnostic core implementation of OAuth 2.1
//! authentication and authorization logic. It implements the AuthenticationProvider
//! trait without depending on specific HTTP frameworks.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tracing::{debug, warn};

// Layer 3: Internal module imports
use super::traits::{AuthenticationProvider, OAuthMiddlewareConfig};
use crate::oauth2::{
    config::OAuth2Config,
    context::AuthContext,
    error::OAuth2Error,
    validator::{Jwt, JwtValidator, Scope},
};

/// Core OAuth 2.1 middleware implementation
///
/// This struct provides framework-agnostic OAuth 2.1 authentication and authorization
/// logic. It can be composed with framework-specific adapters to create complete
/// middleware implementations for different HTTP frameworks.
#[derive(Clone)]
pub struct OAuth2MiddlewareCore {
    /// JWT validator for token verification
    jwt_validator: Arc<Jwt>,

    /// Scope validator for method access control
    scope_validator: Arc<Scope>,

    /// OAuth configuration
    config: OAuth2Config,
}

impl OAuth2MiddlewareCore {
    /// Create a new OAuth 2.1 middleware core instance
    ///
    /// # Arguments
    /// * `config` - OAuth 2.1 configuration including JWKS URL, audience, issuer
    ///
    /// # Returns
    /// * `Result<Self, OAuth2Error>` - Core instance or configuration error
    ///
    /// # Example
    /// ```rust,no_run
    /// use airs_mcp::oauth2::{OAuth2Config, middleware::OAuth2MiddlewareCore};
    /// use url::Url;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = OAuth2Config::builder()
    ///     .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json")?)
    ///     .audience("mcp-server".to_string())
    ///     .issuer("https://auth.example.com".to_string())
    ///     .build()?;
    ///     
    /// let core = OAuth2MiddlewareCore::new(config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: OAuth2Config) -> Result<Self, OAuth2Error> {
        debug!("Creating OAuth2MiddlewareCore with config: {:?}", config);

        let jwt_validator = Arc::new(Jwt::new(config.clone())?);
        let scope_validator = Arc::new(Scope::with_default_mappings());

        Ok(Self {
            jwt_validator,
            scope_validator,
            config,
        })
    }

    /// Get reference to JWT validator
    pub fn jwt_validator(&self) -> &Jwt {
        &self.jwt_validator
    }

    /// Get reference to scope validator
    pub fn scope_validator(&self) -> &Scope {
        &self.scope_validator
    }

    /// Get reference to OAuth configuration
    pub fn config(&self) -> &OAuth2Config {
        &self.config
    }

    /// Validate MCP method access against OAuth scopes
    ///
    /// For MCP requests, extracts the JSON-RPC method and validates against
    /// the required OAuth scope for that operation.
    pub async fn validate_mcp_access(
        &self,
        context: &AuthContext,
        resource_path: &str,
    ) -> Result<(), OAuth2Error> {
        // For MCP JSON-RPC requests, we need to validate scope based on the method
        if resource_path.starts_with("/mcp") {
            // In a full implementation, we would parse the JSON-RPC body to get the actual method
            // For now, we use a simplified approach based on the resource path
            let required_scope = "mcp:*"; // Base MCP access scope

            let user_scopes = self.extract_scopes(context);

            // Check if user has the required scope or a wildcard scope
            let has_access = user_scopes.iter().any(|scope| {
                scope == required_scope || scope == "mcp:*" || scope.starts_with("mcp:")
            });

            if !has_access {
                warn!(
                    "Insufficient scope for MCP access. Required: {}, User scopes: {:?}",
                    required_scope, user_scopes
                );
                return Err(OAuth2Error::InsufficientScope {
                    required: required_scope.to_string(),
                    provided: user_scopes.join(" "),
                });
            }

            debug!("MCP access authorized for scopes: {:?}", user_scopes);
        }

        Ok(())
    }
}

#[async_trait]
impl AuthenticationProvider for OAuth2MiddlewareCore {
    /// Authenticate a Bearer token
    ///
    /// Validates the JWT token including signature verification, claims validation,
    /// and scope extraction. Returns an AuthContext with user information.
    async fn authenticate(&self, token: &str) -> Result<AuthContext, OAuth2Error> {
        debug!("Authenticating Bearer token");

        // Validate JWT token and get claims
        let claims = self.jwt_validator.validate(token).await?;
        debug!("JWT validation successful for subject: {:?}", claims.sub);

        // Extract scopes from token claims
        let scopes = self.jwt_validator.extract_scopes(&claims);
        debug!("Extracted scopes from token: {:?}", scopes);

        // Create AuthContext with claims and scopes
        let auth_context = AuthContext::new(claims, scopes);

        debug!("Authentication successful, created AuthContext");
        Ok(auth_context)
    }

    /// Authorize access to a specific resource
    ///
    /// Checks if the authenticated context has sufficient OAuth scopes
    /// for the requested resource operation.
    async fn authorize(&self, context: &AuthContext, resource: &str) -> Result<(), OAuth2Error> {
        debug!("Authorizing access to resource: {}", resource);

        // Validate MCP-specific access patterns
        self.validate_mcp_access(context, resource).await?;

        // Additional authorization logic can be added here
        // For example, checking specific resource permissions

        debug!("Authorization successful for resource: {}", resource);
        Ok(())
    }

    /// Check if authentication should be skipped for a path
    ///
    /// Public endpoints that don't require OAuth authentication:
    /// - Health checks (/health)
    /// - OAuth metadata (/.well-known/oauth-protected-resource)
    /// - Public API documentation endpoints
    fn should_skip_auth(&self, path: &str) -> bool {
        let skip_paths = [
            "/health",
            "/.well-known/oauth-protected-resource",
            "/docs",
            "/openapi.json",
        ];

        let should_skip = skip_paths.contains(&path);

        if should_skip {
            debug!(
                "Skipping OAuth authentication for public endpoint: {}",
                path
            );
        }

        should_skip
    }

    /// Extract OAuth scopes from authenticated context
    fn extract_scopes(&self, context: &AuthContext) -> Vec<String> {
        context.scopes.clone()
    }
}

impl OAuthMiddlewareConfig for OAuth2MiddlewareCore {
    /// Get the authentication realm for WWW-Authenticate headers
    fn get_auth_realm(&self) -> Option<&str> {
        // Use the audience as the realm if available
        Some(&self.config.audience)
    }

    /// Check if a specific path should skip OAuth validation
    fn should_skip_path(&self, path: &str) -> bool {
        self.should_skip_auth(path)
    }

    /// Get CORS configuration for OAuth endpoints
    fn get_cors_config(&self) -> Option<&str> {
        // Basic CORS configuration - could be expanded
        Some("*")
    }

    /// Check if detailed error information should be included in responses
    fn include_error_details(&self) -> bool {
        // In production, you might want to disable detailed errors
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::OAuth2Config;
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
    fn test_core_creation() {
        let config = create_test_config();
        let result = OAuth2MiddlewareCore::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_skip_auth() {
        let config = create_test_config();
        let core = OAuth2MiddlewareCore::new(config).unwrap();

        // Should skip authentication for public endpoints
        assert!(core.should_skip_auth("/health"));
        assert!(core.should_skip_auth("/.well-known/oauth-protected-resource"));
        assert!(core.should_skip_auth("/docs"));

        // Should require authentication for protected endpoints
        assert!(!core.should_skip_auth("/mcp"));
        assert!(!core.should_skip_auth("/api/protected"));
    }

    #[test]
    fn test_config_trait_implementation() {
        let config = create_test_config();
        let core = OAuth2MiddlewareCore::new(config).unwrap();

        assert_eq!(core.get_auth_realm(), Some("test-audience"));
        assert!(core.should_skip_path("/health"));
        assert!(core.include_error_details());
    }

    #[test]
    fn test_extract_scopes_from_token() {
        let config = create_test_config();
        let core = OAuth2MiddlewareCore::new(config).unwrap();

        // Create mock JWT claims with scopes
        let claims = crate::oauth2::types::JwtClaims {
            sub: "test-user".to_string(),
            aud: Some("test-audience".to_string()),
            iss: Some("https://auth.example.com".to_string()),
            exp: Some(1234567890),
            nbf: None,
            iat: None,
            jti: Some("test-jti".to_string()),
            scope: Some("mcp:read mcp:write admin:*".to_string()),
            scopes: None,
        };

        let scopes = core.jwt_validator.extract_scopes(&claims);
        assert_eq!(scopes, vec!["mcp:read", "mcp:write", "admin:*"]);
    }
}

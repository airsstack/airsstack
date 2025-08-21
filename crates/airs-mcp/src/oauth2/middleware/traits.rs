//! OAuth 2.1 Middleware Traits
//!
//! This module defines the core traits for OAuth 2.1 middleware implementation,
//! providing clean abstractions that separate OAuth logic from HTTP framework specifics.

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::oauth2::{AuthContext, OAuth2Error};

/// Core OAuth 2.1 middleware trait for framework integration
///
/// This trait defines the main interface for OAuth middleware implementations
/// across different HTTP frameworks (Axum, Warp, Tide, etc.).
#[async_trait]
pub trait OAuthMiddleware {
    /// HTTP request type (framework-specific)
    type Request: Send;
    /// HTTP response type (framework-specific)  
    type Response: Send;
    /// Next middleware handler type (framework-specific)
    type Next: Send;
    /// Error type for middleware operations
    type Error: Send + Sync + std::error::Error;

    /// Handle OAuth 2.1 authentication and authorization
    ///
    /// This is the main entry point for OAuth middleware processing:
    /// - Extracts Bearer tokens from requests
    /// - Validates JWT tokens and claims
    /// - Checks OAuth scopes against required permissions
    /// - Injects AuthContext into request extensions
    /// - Handles error responses according to RFC 6750
    async fn handle_oauth(
        &self,
        request: Self::Request,
        next: Self::Next,
    ) -> Result<Self::Response, Self::Error>;
}

/// OAuth-specific request processing
///
/// This trait handles framework-specific request operations for OAuth middleware.
/// Implementations provide request parsing and context injection capabilities.
pub trait OAuthRequestProcessor<Request> {
    /// Extract Bearer token from Authorization header
    ///
    /// Parses the Authorization header according to RFC 6750:
    /// `Authorization: Bearer <token>`
    fn extract_bearer_token(&self, request: &Request) -> Result<String, OAuth2Error>;

    /// Extract request path for scope validation
    ///
    /// Returns the request path/resource identifier used for scope checking.
    /// For MCP requests, this typically includes the JSON-RPC method.
    fn extract_resource_path(&self, request: &Request) -> String;

    /// Extract HTTP method for comprehensive scope validation
    fn extract_http_method(&self, request: &Request) -> String;

    /// Check if request should skip OAuth validation
    ///
    /// Certain endpoints (health checks, metadata) should bypass OAuth.
    /// Returns true for paths that don't require authentication.
    fn should_skip_oauth(&self, request: &Request) -> bool;

    /// Inject OAuth context into request
    ///
    /// Adds the authenticated user context to request extensions
    /// for downstream handlers to access.
    fn inject_oauth_context(&self, request: &mut Request, context: AuthContext);
}

/// OAuth-specific response building
///
/// This trait handles framework-specific response creation for OAuth errors
/// and challenges according to RFC 6750 Bearer Token specification.
pub trait OAuthResponseBuilder<Response> {
    /// Create RFC 6750 compliant error response
    ///
    /// Generates appropriate HTTP error responses for OAuth failures:
    /// - 401 Unauthorized for invalid/missing tokens
    /// - 403 Forbidden for insufficient scope
    /// - 500 Internal Server Error for server-side issues
    fn create_oauth_error_response(&self, error: OAuth2Error) -> Response;

    /// Create WWW-Authenticate challenge response
    ///
    /// Generates RFC 6750 compliant challenge responses with optional realm.
    /// Used for 401 responses to prompt client authentication.
    fn create_oauth_challenge_response(&self, realm: Option<&str>) -> Response;

    /// Create scope-specific error response
    ///
    /// Generates 403 responses with scope information for insufficient_scope errors.
    fn create_scope_error_response(&self, required_scope: &str) -> Response;
}

/// Core authentication logic provider (framework-agnostic)
///
/// This trait encapsulates the core OAuth 2.1 authentication and authorization
/// logic independent of any HTTP framework. It provides the business logic
/// for token validation and scope checking.
#[async_trait]
pub trait AuthenticationProvider {
    /// Authenticate a Bearer token
    ///
    /// Validates the JWT token including:
    /// - Signature verification using JWKS
    /// - Claims validation (aud, iss, exp, nbf)
    /// - Scope extraction from token claims
    async fn authenticate(&self, token: &str) -> Result<AuthContext, OAuth2Error>;

    /// Authorize access to a specific resource
    ///
    /// Checks if the authenticated context has sufficient scope/permissions
    /// for the requested resource operation.
    async fn authorize(&self, context: &AuthContext, resource: &str) -> Result<(), OAuth2Error>;

    /// Check if authentication should be skipped for a path
    ///
    /// Determines if a request path should bypass OAuth validation.
    /// Used for public endpoints like health checks and metadata.
    fn should_skip_auth(&self, path: &str) -> bool;

    /// Extract OAuth scopes from authenticated context
    ///
    /// Helper method to get the list of OAuth scopes associated with
    /// the current authentication context.
    fn extract_scopes(&self, context: &AuthContext) -> Vec<String>;
}

/// OAuth middleware configuration trait
///
/// Provides configuration and policy decisions for OAuth middleware behavior.
pub trait OAuthMiddlewareConfig {
    /// Get the authentication realm for WWW-Authenticate headers
    fn get_auth_realm(&self) -> Option<&str>;

    /// Check if a specific path should skip OAuth validation
    fn should_skip_path(&self, path: &str) -> bool;

    /// Get CORS configuration for OAuth endpoints
    fn get_cors_config(&self) -> Option<&str>; // Could be expanded to a proper CORS type

    /// Check if detailed error information should be included in responses
    fn include_error_details(&self) -> bool;
}

/// Composite trait for complete OAuth middleware functionality
///
/// This trait combines all OAuth middleware capabilities into a single interface
/// for convenient implementation and dependency injection.
#[async_trait]
pub trait FullOAuthMiddleware<Request, Response, Next>:
    OAuthMiddleware<Request = Request, Response = Response, Next = Next>
    + AuthenticationProvider
    + OAuthMiddlewareConfig
    + Send
    + Sync
where
    Request: Send + 'static,
    Response: Send + 'static,
    Next: Send + 'static,
{
    /// Process complete OAuth request cycle
    async fn process_request(&self, request: Request, next: Next) -> Result<Response, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations for testing trait design
    struct MockAuthProvider;

    #[async_trait]
    impl AuthenticationProvider for MockAuthProvider {
        async fn authenticate(&self, _token: &str) -> Result<AuthContext, OAuth2Error> {
            // Mock implementation for testing
            let claims = crate::oauth2::jwt_validator::JwtClaims {
                sub: "test-user".to_string(),
                aud: Some("test-audience".to_string()),
                iss: Some("https://auth.example.com".to_string()),
                exp: Some(1234567890),
                nbf: None,
                iat: None,
                jti: Some("test-jti".to_string()),
                scope: Some("mcp:read mcp:write".to_string()),
                scopes: None,
            };
            let scopes = vec!["mcp:read".to_string(), "mcp:write".to_string()];
            Ok(AuthContext::new(claims, scopes))
        }

        async fn authorize(
            &self,
            _context: &AuthContext,
            _resource: &str,
        ) -> Result<(), OAuth2Error> {
            Ok(())
        }

        fn should_skip_auth(&self, path: &str) -> bool {
            matches!(path, "/health" | "/.well-known/oauth-protected-resource")
        }

        fn extract_scopes(&self, _context: &AuthContext) -> Vec<String> {
            vec!["mcp:read".to_string(), "mcp:write".to_string()]
        }
    }

    #[test]
    fn test_auth_provider_trait_design() {
        let provider = MockAuthProvider;
        assert!(provider.should_skip_auth("/health"));
        assert!(!provider.should_skip_auth("/mcp"));
    }
}

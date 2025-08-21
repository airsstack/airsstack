//! OAuth 2.1 Middleware Implementation - Simplified Version
//!
//! This module provides the OAuth 2.1 middleware layer that integrates with
//! Axum HTTP transport for request authentication and authorization.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use axum::{
    extract::Request,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::{debug, error};

// Layer 3: Internal module imports
use crate::oauth2::{
    config::OAuth2Config, context::AuthContext, error::OAuth2Error, jwt_validator::JwtValidator,
    scope_validator::ScopeValidator,
};

/// OAuth 2.1 middleware for Axum HTTP server
///
/// This middleware validates Bearer tokens, checks scopes, and injects
/// AuthContext into request extensions for downstream handlers.
#[derive(Clone)]
pub struct OAuth2Middleware {
    /// JWT token validator with JWKS support
    jwt_validator: Arc<JwtValidator>,

    /// Scope validator for MCP method authorization
    scope_validator: Arc<ScopeValidator>,

    /// OAuth configuration
    config: OAuth2Config,
}

impl OAuth2Middleware {
    /// Create a new OAuth 2.1 middleware instance
    pub fn new(config: OAuth2Config) -> Result<Self, OAuth2Error> {
        let jwt_validator = Arc::new(JwtValidator::new(config.clone())?);
        let scope_validator = Arc::new(ScopeValidator::with_default_mappings());

        Ok(Self {
            jwt_validator,
            scope_validator,
            config,
        })
    }

    /// Create RFC 6750 compliant error response
    fn create_error_response(error: OAuth2Error, include_challenge: bool) -> Response {
        let (status, error_code, description) = match &error {
            OAuth2Error::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_request",
                "Missing authorization header",
            ),
            OAuth2Error::InvalidTokenFormat => (
                StatusCode::UNAUTHORIZED,
                "invalid_request",
                "Invalid authorization header format",
            ),
            OAuth2Error::InvalidToken(_) => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "The access token is invalid or expired",
            ),
            OAuth2Error::InsufficientScope {
                required: _,
                provided: _,
            } => (
                StatusCode::FORBIDDEN,
                "insufficient_scope",
                "Insufficient scope for this operation",
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                "Internal server error",
            ),
        };

        let mut headers = HeaderMap::new();

        if include_challenge {
            let challenge = match &error {
                OAuth2Error::InsufficientScope { required, .. } => {
                    format!("Bearer scope=\"{}\"", required)
                }
                _ => "Bearer".to_string(),
            };

            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_str(&challenge)
                    .unwrap_or_else(|_| HeaderValue::from_static("Bearer")),
            );
        }

        let error_response = json!({
            "error": error_code,
            "error_description": description
        });

        (status, headers, Json(error_response)).into_response()
    }
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, OAuth2Error> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .ok_or(OAuth2Error::MissingToken)?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| OAuth2Error::InvalidTokenFormat)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(OAuth2Error::InvalidTokenFormat);
    }

    let token = auth_str.strip_prefix("Bearer ").unwrap().trim();

    if token.is_empty() {
        return Err(OAuth2Error::InvalidTokenFormat);
    }

    Ok(token.to_string())
}

/// OAuth 2.1 middleware handler implementation
///
/// Performs the actual OAuth 2.1 authentication and authorization:
/// - Extracts Bearer tokens from Authorization headers
/// - Validates JWT tokens using JWKS
/// - Checks token scopes against required MCP operation scopes
/// - Injects AuthContext into request extensions
pub async fn oauth2_middleware_handler(
    mut request: Request,
    next: Next,
    middleware: OAuth2Middleware,
) -> Result<Response, OAuth2Error> {
    let path = request.uri().path();

    // Skip OAuth for health checks and metadata endpoints
    if path == "/health" || path == "/.well-known/oauth-protected-resource" {
        debug!("Skipping OAuth for public endpoint: {}", path);
        return Ok(next.run(request).await);
    }

    debug!("Processing OAuth request for path: {}", path);

    // Extract Bearer token from Authorization header
    let token = extract_bearer_token(request.headers())?;

    // Validate JWT token and get claims
    let claims = middleware.jwt_validator.validate_token(&token).await?;

    // Extract scopes from token (OAuth 2.1 scopes are space-separated)
    let scopes: Vec<String> = claims
        .scope
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // For MCP requests, extract the method and validate scope
    // This is a simplified approach - in production, you might parse the JSON-RPC body
    if path.starts_with("/mcp") {
        // For now, we'll use a basic scope validation
        // In a full implementation, you'd parse the JSON-RPC request to get the actual method
        let required_scope = "mcp:*"; // Base MCP access scope

        if !scopes
            .iter()
            .any(|s| s == required_scope || s == "mcp:*" || s.starts_with("mcp:"))
        {
            debug!(
                "Insufficient scope for MCP access. Required: {}, Provided: {:?}",
                required_scope, scopes
            );
            return Err(OAuth2Error::InsufficientScope {
                required: required_scope.to_string(),
                provided: scopes.join(" "),
            });
        }
    }

    // Create AuthContext and inject into request extensions
    let auth_context = AuthContext::new(claims, scopes);
    request.extensions_mut().insert(auth_context);

    debug!("OAuth authentication successful, proceeding to next handler");

    // Continue to next middleware
    Ok(next.run(request).await)
}

/// OAuth 2.1 middleware layer factory function
///
/// Creates a simple middleware function that can be used with Axum
pub fn oauth2_middleware_layer(
    middleware: OAuth2Middleware,
) -> impl Clone
       + Fn(
    Request,
    Next,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, std::convert::Infallible>> + Send>,
> {
    move |req: Request, next: Next| {
        let middleware = middleware.clone();
        Box::pin(async move {
            match oauth2_middleware_handler(req, next, middleware).await {
                Ok(response) => Ok(response),
                Err(oauth_error) => {
                    error!("OAuth middleware error: {:?}", oauth_error);
                    Ok(OAuth2Middleware::create_error_response(oauth_error, true))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_bearer_token_extraction() {
        let mut headers = HeaderMap::new();

        // Test valid Bearer token
        headers.insert(
            header::AUTHORIZATION,
            "Bearer test-token-123".parse().unwrap(),
        );
        let result = extract_bearer_token(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-token-123");

        // Test missing authorization header
        let empty_headers = HeaderMap::new();
        let result = extract_bearer_token(&empty_headers);
        assert!(matches!(result, Err(OAuth2Error::MissingToken)));

        // Test invalid format
        let mut invalid_headers = HeaderMap::new();
        invalid_headers.insert(header::AUTHORIZATION, "Basic test".parse().unwrap());
        let result = extract_bearer_token(&invalid_headers);
        assert!(matches!(result, Err(OAuth2Error::InvalidTokenFormat)));
    }

    #[test]
    fn test_middleware_creation() {
        // Test that middleware can be created with default config
        let config = OAuth2Config::default();
        let result = OAuth2Middleware::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_response_creation() {
        let error = OAuth2Error::MissingToken;
        let response = OAuth2Middleware::create_error_response(error, true);
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

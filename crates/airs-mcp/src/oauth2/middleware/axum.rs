//! Axum-specific OAuth 2.1 middleware implementation
//!
//! This module provides the Axum-specific implementation of OAuth 2.1 middleware

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use axum::{
    extract::Request,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tower::Layer;
use tracing::{debug, error};

// Layer 3: Internal module imports
use crate::oauth2::{config::OAuth2Config, context::AuthContext, error::OAuth2Error};

use super::{
    core::OAuth2MiddlewareCore,
    traits::{AuthenticationProvider, OAuthMiddleware},
    types::MiddlewareRequest,
};

/// Axum-specific OAuth 2.1 middleware implementation
///
/// This struct adapts the framework-agnostic OAuth middleware core to work
/// specifically with Axum's middleware system and types.
#[derive(Clone)]
pub struct AxumOAuth2Middleware {
    /// Framework-agnostic OAuth core implementation
    core: OAuth2MiddlewareCore,
}

impl AxumOAuth2Middleware {
    /// Create a new Axum OAuth 2.1 middleware instance
    pub fn new(config: OAuth2Config) -> Result<Self, OAuth2Error> {
        let core = OAuth2MiddlewareCore::new(config)?;
        Ok(Self { core })
    }

    /// Process OAuth request and return either the processed request or None if handled
    async fn process_oauth_request(
        &self,
        mut request: Request,
    ) -> Result<Option<Request>, OAuth2Error> {
        let path = request.uri().path();

        // Skip OAuth for health checks and metadata endpoints
        if path == "/health" || path == "/.well-known/oauth-protected-resource" {
            debug!("Skipping OAuth for public endpoint: {}", path);
            return Ok(Some(request));
        }

        debug!("Processing OAuth request for path: {}", path);

        // Convert Axum request to middleware request
        let middleware_request = self.convert_request(&request)?;

        // Extract the authorization token
        let token = middleware_request
            .authorization_header()
            .ok_or(OAuth2Error::MissingToken)?
            .strip_prefix("Bearer ")
            .ok_or(OAuth2Error::InvalidTokenFormat)?;

        // Authenticate using the core middleware
        let auth_result = self.core.authenticate(token).await?;

        // For MCP requests, validate scope requirements
        if path.starts_with("/mcp") {
            // Use the scope validator from core to check scope requirements
            self.core.validate_mcp_access(&auth_result, "mcp").await?;
        }

        // Create AuthContext and inject into request extensions
        let auth_context = AuthContext::new(auth_result.claims, auth_result.scopes);
        request.extensions_mut().insert(auth_context);

        debug!("OAuth authentication successful, proceeding to next handler");

        Ok(Some(request))
    }
}

/// Implementation of Tower Layer trait for direct usage
impl<S> Layer<S> for AxumOAuth2Middleware {
    type Service = AxumOAuth2Service<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AxumOAuth2Service {
            inner,
            middleware: self.clone(),
        }
    }
}

/// Service wrapper for OAuth 2.1 middleware
#[derive(Clone)]
pub struct AxumOAuth2Service<S> {
    inner: S,
    middleware: AxumOAuth2Middleware,
}

impl<S> tower::Service<Request> for AxumOAuth2Service<S>
where
    S: tower::Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let inner = self.inner.clone();
        let middleware = self.middleware.clone();

        Box::pin(async move {
            let mut inner = inner;

            // Process OAuth middleware logic
            match middleware.process_oauth_request(req).await {
                Ok(Some(req)) => {
                    // Request passed OAuth, call inner service
                    inner.call(req).await
                }
                Ok(None) => {
                    // OAuth handled the request (like returning an error response)
                    // This case shouldn't happen with our current design
                    Ok(Response::new(axum::body::Body::empty()))
                }
                Err(oauth_error) => {
                    // OAuth error, return error response
                    Ok(middleware.build_error_response(oauth_error, true))
                }
            }
        })
    }
}

impl AxumOAuth2Middleware {
    /// Process OAuth 2.1 middleware request
    async fn process_request(
        &self,
        mut request: Request,
        next: Next,
    ) -> Result<Response, OAuth2Error> {
        let path = request.uri().path();

        // Skip OAuth for health checks and metadata endpoints
        if path == "/health" || path == "/.well-known/oauth-protected-resource" {
            debug!("Skipping OAuth for public endpoint: {}", path);
            return Ok(next.run(request).await);
        }

        debug!("Processing OAuth request for path: {}", path);

        // Convert Axum request to middleware request
        let middleware_request = self.convert_request(&request)?;

        // Extract the authorization token
        let token = middleware_request
            .authorization_header()
            .ok_or(OAuth2Error::MissingToken)?
            .strip_prefix("Bearer ")
            .ok_or(OAuth2Error::InvalidTokenFormat)?;

        // Authenticate using the core middleware
        let auth_result = self.core.authenticate(token).await?;

        // For MCP requests, validate scope requirements
        if path.starts_with("/mcp") {
            // Use the scope validator from core to check scope requirements
            self.core.validate_mcp_access(&auth_result, "mcp").await?;
        }

        // Create AuthContext and inject into request extensions
        let auth_context = AuthContext::new(auth_result.claims, auth_result.scopes);
        request.extensions_mut().insert(auth_context);

        debug!("OAuth authentication successful, proceeding to next handler");

        // Continue to next middleware
        Ok(next.run(request).await)
    }

    /// Convert Axum request to middleware request format
    fn convert_request(&self, request: &Request) -> Result<MiddlewareRequest, OAuth2Error> {
        let path = request.uri().path().to_string();
        let method = request.method().to_string();
        let headers = self.convert_headers(request.headers());

        Ok(MiddlewareRequest {
            path,
            method,
            headers,
            query_params: std::collections::HashMap::new(), // TODO: Extract query params
            body: None,        // For now, we don't need the body for JWT validation
            remote_addr: None, // TODO: Extract remote address
        })
    }

    /// Convert Axum HeaderMap to our middleware header format
    fn convert_headers(&self, headers: &HeaderMap) -> std::collections::HashMap<String, String> {
        headers
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|v| (name.to_string(), v.to_string()))
            })
            .collect()
    }

    /// Build RFC 6750 compliant error response
    fn build_error_response(&self, error: OAuth2Error, include_challenge: bool) -> Response {
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
            OAuth2Error::InsufficientScope { .. } => (
                StatusCode::FORBIDDEN,
                "insufficient_scope",
                "Insufficient scope for the requested resource",
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
                    format!("Bearer scope=\"{required}\"")
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

/// Implementation of OAuthMiddleware trait for Axum
#[async_trait]
impl OAuthMiddleware for AxumOAuth2Middleware {
    type Request = Request;
    type Response = Response;
    type Next = Next;
    type Error = OAuth2Error;

    async fn handle_oauth(
        &self,
        request: Self::Request,
        next: Self::Next,
    ) -> Result<Self::Response, Self::Error> {
        match self.process_request(request, next).await {
            Ok(response) => Ok(response),
            Err(oauth_error) => {
                error!("OAuth middleware error: {:?}", oauth_error);
                Ok(self.build_error_response(oauth_error, true))
            }
        }
    }
}

/// Public convenience function for creating Axum OAuth 2.1 middleware
///
/// This function provides a simple way to create OAuth middleware for Axum applications.
///
/// # Example
///
/// ```rust,no_run
/// use airs_mcp::oauth2::{config::OAuth2Config, middleware::axum::oauth2_middleware_layer};
/// use axum::{Router, routing::post};
/// use tower::Layer;
///
/// let config = OAuth2Config::default();
/// let oauth_middleware = oauth2_middleware_layer(config).unwrap();
///
/// let app = Router::<()>::new()
///     .route("/mcp", post(mcp_handler))
///     .layer(oauth_middleware);
///
/// async fn mcp_handler() -> &'static str {
///     "MCP endpoint"
/// }
/// ```
pub fn oauth2_middleware_layer(config: OAuth2Config) -> Result<AxumOAuth2Middleware, OAuth2Error> {
    AxumOAuth2Middleware::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_axum_middleware_creation() {
        let config = OAuth2Config::default();
        let result = AxumOAuth2Middleware::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_header_conversion() {
        let middleware = AxumOAuth2Middleware::new(OAuth2Config::default()).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test-token".parse().unwrap());
        headers.insert("content-type", "application/json".parse().unwrap());

        let converted = middleware.convert_headers(&headers);
        assert_eq!(
            converted.get("authorization"),
            Some(&"Bearer test-token".to_string())
        );
        assert_eq!(
            converted.get("content-type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_error_response_creation() {
        let middleware = AxumOAuth2Middleware::new(OAuth2Config::default()).unwrap();

        let error = OAuth2Error::MissingToken;
        let response = middleware.build_error_response(error, true);
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_public_layer_function() {
        let config = OAuth2Config::default();
        let result = oauth2_middleware_layer(config);
        assert!(result.is_ok());
    }
}

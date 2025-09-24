//! Axum HTTP Authentication Middleware Integration
//!
//! This module provides the Tower-compatible middleware wrapper that integrates
//! the generic HttpAuthMiddleware with Axum's middleware system for zero-cost
//! authentication in HTTP requests.

// Layer 1: Standard library imports
use std::task::{Context, Poll};

// Layer 2: Third-party crate imports
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::future::BoxFuture;
use tower::{Layer, Service};

// Layer 3: Internal module imports
use super::middleware::{
    HttpAuthConfig, HttpAuthMiddleware, HttpAuthRequest, HttpAuthStrategyAdapter,
};
use super::oauth2::error::HttpAuthError;

/// Axum-compatible HTTP authentication middleware wrapper
///
/// This middleware integrates HttpAuthMiddleware\<A\> with Tower's Layer and Service
/// traits for seamless integration with Axum routers. It provides zero-cost
/// authentication by using compile-time generics and avoiding dynamic dispatch.
///
/// # Type Parameters
/// * `A` - Authentication strategy adapter implementing HttpAuthStrategyAdapter
///
/// # Performance Characteristics
/// * Zero dynamic dispatch - all authentication calls are monomorphized
/// * Stack allocation - middleware state is stack-allocated
/// * Compile-time optimization - authentication logic is inlined
/// * Tower compatibility - works with all Tower middleware
///
/// # Examples
///
/// ```rust
/// # use std::sync::Arc;
/// # use airs_mcp::transport::adapters::http::auth::{
/// #     axum_middleware::AxumHttpAuthLayer,
/// #     middleware::{HttpAuthConfig, HttpAuthMiddleware, HttpAuthStrategyAdapter},
/// # };
/// # use tower::ServiceBuilder;
/// # use airs_mcp::authentication::{AuthContext, AuthMethod};
/// # use airs_mcp::transport::adapters::http::auth::oauth2::error::HttpAuthError;
/// # use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthRequest;
/// #
/// # // Mock adapter for the example
/// # #[derive(Clone, Debug)]
/// # struct MockAdapter;
/// #
/// # #[async_trait::async_trait]
/// # impl HttpAuthStrategyAdapter for MockAdapter {
/// #     type RequestType = ();
/// #     type AuthData = ();
/// #     fn auth_method(&self) -> &'static str { "mock" }
/// #     async fn authenticate_http_request(&self, _request: &HttpAuthRequest)
/// #         -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
/// #         Ok(AuthContext::new(AuthMethod::new("mock"), ()))
/// #     }
/// # }
/// #
/// let adapter = MockAdapter;
/// let config = HttpAuthConfig::default();
/// let middleware = HttpAuthMiddleware::new(adapter, config);
/// let layer = AxumHttpAuthLayer::from_middleware(middleware);
///
/// // Use with ServiceBuilder
/// let service_builder = ServiceBuilder::new().layer(layer);
/// ```
#[derive(Clone)]
pub struct AxumHttpAuthMiddleware<A>
where
    A: HttpAuthStrategyAdapter,
{
    /// Core authentication middleware (zero-cost generic)
    middleware: HttpAuthMiddleware<A>,
}

impl<A> AxumHttpAuthMiddleware<A>
where
    A: HttpAuthStrategyAdapter,
{
    /// Create a new Axum HTTP authentication middleware
    ///
    /// # Arguments
    /// * `middleware` - Core HTTP authentication middleware
    ///
    /// # Returns
    /// * New Axum-compatible middleware wrapper
    pub fn new(middleware: HttpAuthMiddleware<A>) -> Self {
        Self { middleware }
    }

    /// Create middleware with default configuration
    ///
    /// # Arguments
    /// * `adapter` - Authentication strategy adapter
    ///
    /// # Returns
    /// * New middleware with default configuration
    pub fn with_adapter(adapter: A) -> Self {
        let middleware = HttpAuthMiddleware::with_default_config(adapter);
        Self::new(middleware)
    }

    /// Get the authentication method name
    pub fn auth_method(&self) -> &'static str {
        self.middleware.auth_method()
    }

    /// Get the middleware configuration
    pub fn config(&self) -> &HttpAuthConfig {
        self.middleware.config()
    }
}

/// Tower Layer implementation for HTTP authentication middleware
///
/// This layer creates the authentication service for each request.
/// Uses zero-cost generics for maximum performance.
#[derive(Clone)]
pub struct AxumHttpAuthLayer<A>
where
    A: HttpAuthStrategyAdapter,
{
    middleware: AxumHttpAuthMiddleware<A>,
}

impl<A> AxumHttpAuthLayer<A>
where
    A: HttpAuthStrategyAdapter,
{
    /// Create a new authentication layer
    pub fn new(middleware: AxumHttpAuthMiddleware<A>) -> Self {
        Self { middleware }
    }

    /// Create a layer from core middleware
    pub fn from_middleware(middleware: HttpAuthMiddleware<A>) -> Self {
        Self::new(AxumHttpAuthMiddleware::new(middleware))
    }
}

impl<S, A> Layer<S> for AxumHttpAuthLayer<A>
where
    A: HttpAuthStrategyAdapter + Clone,
{
    type Service = AxumHttpAuthService<S, A>;

    fn layer(&self, inner: S) -> Self::Service {
        AxumHttpAuthService {
            inner,
            middleware: self.middleware.clone(),
        }
    }
}

/// Tower Service implementation for HTTP authentication
///
/// This service handles the actual authentication logic for each request.
/// Uses zero-cost generics and direct method calls for maximum performance.
#[derive(Clone)]
pub struct AxumHttpAuthService<S, A>
where
    A: HttpAuthStrategyAdapter,
{
    inner: S,
    middleware: AxumHttpAuthMiddleware<A>,
}

impl<S, A> Service<Request> for AxumHttpAuthService<S, A>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
    A: HttpAuthStrategyAdapter + Clone + Send + Sync + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let middleware = self.middleware.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract request information for authentication
            let auth_request = match extract_auth_request(&request) {
                Ok(req) => req,
                Err(response) => return Ok(*response),
            };

            // Perform authentication using zero-cost generic middleware
            match middleware.middleware.authenticate(&auth_request).await {
                Ok(Some(auth_context)) => {
                    // Authentication successful - add auth context to request extensions
                    request.extensions_mut().insert(auth_context);
                }
                Ok(None) => {
                    // Authentication skipped (e.g., for health check paths)
                    // Continue without adding auth context
                }
                Err(auth_error) => {
                    // Authentication failed - return error response
                    return Ok(create_auth_error_response(auth_error, middleware.config()));
                }
            }

            // Continue to the next middleware/handler
            inner.call(request).await
        })
    }
}

/// Extract authentication request from Axum request
///
/// Converts Axum's request format to the generic HttpAuthRequest format
/// used by the authentication middleware.
fn extract_auth_request(request: &Request) -> Result<HttpAuthRequest, Box<Response>> {
    // Extract headers
    let headers = request
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                value.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    // Extract path
    let path = request.uri().path().to_string();

    // Extract query parameters
    let query_params = request
        .uri()
        .query()
        .map(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(HttpAuthRequest::new(headers, path, query_params))
}

/// Create authentication error response
///
/// Converts HttpAuthError to appropriate HTTP error response with proper
/// status codes and error messages.
fn create_auth_error_response(error: HttpAuthError, config: &HttpAuthConfig) -> Response {
    let (status_code, message) = match error {
        HttpAuthError::MissingApiKey => (StatusCode::UNAUTHORIZED, "Missing API key".to_string()),
        HttpAuthError::AuthenticationFailed { message } => {
            if config.include_error_details {
                (StatusCode::UNAUTHORIZED, message)
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication failed".to_string(),
                )
            }
        }
        HttpAuthError::InvalidRequest { message } => {
            if config.include_error_details {
                (StatusCode::BAD_REQUEST, message)
            } else {
                (StatusCode::BAD_REQUEST, "Invalid request".to_string())
            }
        }
        HttpAuthError::MissingHeader { header } => {
            if config.include_error_details {
                (
                    StatusCode::UNAUTHORIZED,
                    format!("Missing header: {header}"),
                )
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing required header".to_string(),
                )
            }
        }
        HttpAuthError::MalformedAuth { message } => {
            if config.include_error_details {
                (StatusCode::UNAUTHORIZED, message)
            } else {
                (
                    StatusCode::UNAUTHORIZED,
                    "Malformed authorization".to_string(),
                )
            }
        }
        _ => (StatusCode::UNAUTHORIZED, "Authentication error".to_string()),
    };

    // Create WWW-Authenticate header for 401 responses
    let mut response = (status_code, message).into_response();
    if status_code == StatusCode::UNAUTHORIZED {
        response.headers_mut().insert(
            "WWW-Authenticate",
            format!("Bearer realm=\"{}\"", config.auth_realm)
                .parse()
                .unwrap_or_else(|_| "Bearer".parse().unwrap()),
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::{AuthContext, AuthMethod};
    use futures::Future;
    use std::pin::Pin;

    // Mock adapter for testing
    #[derive(Debug, Clone)]
    struct MockAuthAdapter {
        should_authenticate: bool,
    }

    impl MockAuthAdapter {
        fn new(should_authenticate: bool) -> Self {
            Self {
                should_authenticate,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpAuthStrategyAdapter for MockAuthAdapter {
        type RequestType = ();
        type AuthData = String;

        fn auth_method(&self) -> &'static str {
            "mock"
        }

        async fn authenticate_http_request(
            &self,
            _request: &HttpAuthRequest,
        ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
            if self.should_authenticate {
                Ok(AuthContext::new(
                    AuthMethod::new("mock"),
                    "test_user".to_string(),
                ))
            } else {
                Err(HttpAuthError::AuthenticationFailed {
                    message: "Mock authentication failed".to_string(),
                })
            }
        }
    }

    // Mock service for testing
    #[derive(Clone)]
    struct MockService;

    impl Service<Request> for MockService {
        type Response = Response;
        type Error = std::convert::Infallible;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _request: Request) -> Self::Future {
            Box::pin(async { Ok((StatusCode::OK, "Success").into_response()) })
        }
    }

    #[tokio::test]
    async fn test_axum_middleware_successful_auth() {
        let adapter = MockAuthAdapter::new(true);
        let config = HttpAuthConfig::default();
        let middleware = HttpAuthMiddleware::new(adapter, config);
        let layer = AxumHttpAuthLayer::from_middleware(middleware);

        let mut service = layer.layer(MockService);

        let request = Request::builder()
            .uri("/api/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_axum_middleware_failed_auth() {
        let adapter = MockAuthAdapter::new(false);
        let config = HttpAuthConfig::default();
        let middleware = HttpAuthMiddleware::new(adapter, config);
        let layer = AxumHttpAuthLayer::from_middleware(middleware);

        let mut service = layer.layer(MockService);

        let request = Request::builder()
            .uri("/api/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = service.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_extract_auth_request() {
        let request = Request::builder()
            .uri("/api/test?param=value")
            .header("Authorization", "Bearer token123")
            .header("X-Custom", "custom-value")
            .body(axum::body::Body::empty())
            .unwrap();

        let auth_request = extract_auth_request(&request).unwrap();

        assert_eq!(auth_request.path, "/api/test");
        assert_eq!(
            auth_request.headers.get("authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            auth_request.headers.get("x-custom"),
            Some(&"custom-value".to_string())
        );
        assert_eq!(
            auth_request.query_params.get("param"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_create_auth_error_response() {
        let config = HttpAuthConfig {
            include_error_details: true,
            auth_realm: "Test Realm".to_string(),
            ..HttpAuthConfig::default()
        };

        let error = HttpAuthError::AuthenticationFailed {
            message: "Invalid token".to_string(),
        };

        let response = create_auth_error_response(error, &config);
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let auth_header = response.headers().get("WWW-Authenticate");
        assert!(auth_header.is_some());
        assert!(auth_header
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Test Realm"));
    }
}

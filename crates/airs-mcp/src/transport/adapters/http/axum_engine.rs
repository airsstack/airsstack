//! Axum HTTP Engine Implementation
//!
//! This module provides the default HTTP engine implementation using the Axum
//! web framework. It implements the HttpEngine trait and provides MCP protocol
//! support with OAuth2 authentication middleware.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use axum::{
    extract::{rejection::BytesRejection, Bytes, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

// Layer 3: Internal module imports
use super::engine::{
    AuthenticationConfig, AuthenticationContext, HttpEngine, HttpEngineError, HttpMiddleware,
    HttpResponse, McpRequestHandler, ResponseMode,
};
use super::session::{extract_session_id, SessionId};
use crate::oauth2::middleware::oauth2_middleware_layer;
use crate::oauth2::{AuthContext, OAuth2Config};

/// Axum-specific configuration
#[derive(Debug, Clone)]
pub struct AxumEngineConfig {
    /// Enable CORS middleware
    pub cors_enabled: bool,
    /// Maximum request body size in bytes
    pub max_request_size: usize,
    /// Request timeout duration
    pub timeout: Duration,
    /// Graceful shutdown timeout
    pub graceful_shutdown_timeout: Duration,
}

impl Default for AxumEngineConfig {
    fn default() -> Self {
        Self {
            cors_enabled: true,
            max_request_size: 1024 * 1024, // 1MB
            timeout: Duration::from_secs(30),
            graceful_shutdown_timeout: Duration::from_secs(10),
        }
    }
}

/// Axum HTTP engine error types
#[derive(Debug, thiserror::Error)]
pub enum AxumEngineError {
    /// Server binding failed
    #[error("Failed to bind server: {0}")]
    BindFailed(#[from] std::io::Error),

    /// OAuth2 configuration error
    #[error("OAuth2 configuration error: {0}")]
    OAuth2(String),

    /// Router building failed
    #[error("Failed to build router: {message}")]
    RouterBuild { message: String },
}

/// Axum HTTP engine implementation
pub struct AxumHttpEngine {
    /// Axum router (built when starting)
    router: Option<Router>,

    /// Server handle for shutdown management
    server_handle: Option<tokio::task::JoinHandle<Result<(), std::io::Error>>>,

    /// MCP request handler
    mcp_handler: Option<Arc<dyn McpRequestHandler>>,

    /// OAuth2 configuration (will be replaced with AuthenticationManager in Phase 5)
    oauth_config: Option<OAuth2Config>,

    /// Custom middleware stack
    middleware_stack: Vec<Box<dyn HttpMiddleware>>,

    /// Engine configuration
    config: AxumEngineConfig,

    /// Local address the server is bound to
    local_addr: Option<SocketAddr>,

    /// TCP listener for the server
    listener: Option<TcpListener>,

    /// Running state
    is_running: bool,
}

impl AxumHttpEngine {
    /// Build the Axum router with all middleware and routes
    fn build_router(&self) -> Result<Router, AxumEngineError> {
        let mut router = Router::new()
            .route("/mcp", post(handle_mcp_post))
            .route("/mcp", get(handle_mcp_sse))
            .route("/health", get(handle_health_check));

        // Add OAuth middleware if configured
        if let Some(oauth_config) = &self.oauth_config {
            let oauth_layer = oauth2_middleware_layer(oauth_config.clone());
            router = router.layer(oauth_layer);
        }

        // Add CORS if enabled
        if self.config.cors_enabled {
            router = router.layer(CorsLayer::very_permissive());
        }

        // Add custom middleware
        for middleware in &self.middleware_stack {
            // Note: In a real implementation, each middleware would need to
            // provide a method to convert itself to an Axum layer
            // For now, we'll log that custom middleware is registered
            tracing::info!("Registered custom middleware: {}", middleware.name());
        }

        // Add the MCP handler as application state
        router = router.with_state(self.mcp_handler.clone());

        Ok(router)
    }

    /// Extract authentication context from request headers
    fn extract_auth_context(headers: &HeaderMap) -> Option<AuthenticationContext> {
        // Check for OAuth2 AuthContext in extensions (set by OAuth middleware)
        if let Some(oauth_context) = headers.extensions().get::<AuthContext>() {
            let mut metadata = HashMap::new();
            metadata.insert("method".to_string(), "oauth2".to_string());
            metadata.insert("user_id".to_string(), oauth_context.claims.sub.clone());

            return Some(AuthenticationContext {
                method: "oauth2".to_string(),
                identity: oauth_context.claims.sub.clone(),
                metadata,
            });
        }

        // Future: Add support for API keys, basic auth, etc. in Phase 5
        None
    }
}

#[async_trait]
impl HttpEngine for AxumHttpEngine {
    type Error = AxumEngineError;
    type Config = AxumEngineConfig;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            router: None,
            server_handle: None,
            mcp_handler: None,
            oauth_config: None,
            middleware_stack: Vec::new(),
            config,
            local_addr: None,
            listener: None,
            is_running: false,
        })
    }

    async fn bind(&mut self, addr: SocketAddr) -> Result<(), HttpEngineError> {
        if self.is_bound() {
            return Err(HttpEngineError::AlreadyBound {
                addr: self.local_addr.unwrap(),
            });
        }

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| HttpEngineError::Io(e))?;

        let actual_addr = listener.local_addr().map_err(|e| HttpEngineError::Io(e))?;

        self.listener = Some(listener);
        self.local_addr = Some(actual_addr);

        Ok(())
    }

    async fn start(&mut self) -> Result<(), HttpEngineError> {
        if !self.is_bound() {
            return Err(HttpEngineError::NotBound);
        }

        if self.is_running {
            return Err(HttpEngineError::AlreadyRunning);
        }

        // Build the router
        let router = self.build_router().map_err(|e| HttpEngineError::Engine {
            message: e.to_string(),
        })?;

        // Take ownership of the listener
        let listener = self.listener.take().ok_or(HttpEngineError::NotBound)?;

        // Start the server
        let server_handle = tokio::spawn(async move { axum::serve(listener, router).await });

        self.server_handle = Some(server_handle);
        self.is_running = true;

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), HttpEngineError> {
        if let Some(handle) = self.server_handle.take() {
            // For graceful shutdown, we would need to modify the server startup
            // to use axum::serve().with_graceful_shutdown()
            // For now, we'll abort the task
            handle.abort();

            // Wait for the task to complete or timeout
            tokio::time::timeout(self.config.graceful_shutdown_timeout, async {
                let _ = handle.await;
            })
            .await
            .ok();
        }

        self.is_running = false;
        Ok(())
    }

    fn register_mcp_handler(&mut self, handler: Arc<dyn McpRequestHandler>) {
        self.mcp_handler = Some(handler);
    }

    fn register_authentication(
        &mut self,
        auth_config: AuthenticationConfig,
    ) -> Result<(), HttpEngineError> {
        match auth_config.method.as_str() {
            "oauth2" => {
                // Convert generic auth config to OAuth2Config
                let jwks_url = auth_config.config.get("jwks_url").ok_or_else(|| {
                    HttpEngineError::Authentication {
                        message: "OAuth2 requires jwks_url".to_string(),
                    }
                })?;
                let audience = auth_config.config.get("audience").ok_or_else(|| {
                    HttpEngineError::Authentication {
                        message: "OAuth2 requires audience".to_string(),
                    }
                })?;
                let issuer = auth_config.config.get("issuer").ok_or_else(|| {
                    HttpEngineError::Authentication {
                        message: "OAuth2 requires issuer".to_string(),
                    }
                })?;

                let oauth_config =
                    OAuth2Config::builder()
                        .jwks_url(jwks_url.parse().map_err(|e| {
                            HttpEngineError::Authentication {
                                message: format!("Invalid JWKS URL: {}", e),
                            }
                        })?)
                        .audience(audience.clone())
                        .issuer(issuer.clone())
                        .build()
                        .map_err(|e| HttpEngineError::Authentication {
                            message: e.to_string(),
                        })?;

                self.oauth_config = Some(oauth_config);
                Ok(())
            }
            method => Err(HttpEngineError::Authentication {
                message: format!("Unsupported authentication method: {}", method),
            }),
        }
    }

    fn register_middleware(&mut self, middleware: Box<dyn HttpMiddleware>) {
        self.middleware_stack.push(middleware);
    }

    fn is_bound(&self) -> bool {
        self.local_addr.is_some()
    }

    fn is_running(&self) -> bool {
        self.is_running
    }

    fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    fn engine_type(&self) -> &'static str {
        "axum"
    }
}

/// Axum route handler for MCP POST requests
async fn handle_mcp_post(
    State(handler): State<Option<Arc<dyn McpRequestHandler>>>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, StatusCode> {
    let handler = handler.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract session ID
    let session_id =
        extract_session_id(&headers).unwrap_or_else(|| format!("session-{}", uuid::Uuid::new_v4()));

    // Extract authentication context
    let auth_context = AxumHttpEngine::extract_auth_context(&headers);

    // Get request body
    let body_bytes = body.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();

    // Handle the MCP request
    match handler
        .handle_mcp_request(session_id, body_bytes, ResponseMode::Json, auth_context)
        .await
    {
        Ok(http_response) => {
            let mut response = Response::builder().status(http_response.status);

            // Add headers
            for (key, value) in http_response.headers {
                response = response.header(key, value);
            }

            response
                .body(http_response.body.into())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Axum route handler for MCP Server-Sent Events
async fn handle_mcp_sse(
    State(handler): State<Option<Arc<dyn McpRequestHandler>>>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let handler = handler.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract session ID
    let session_id = extract_session_id(&headers)
        .unwrap_or_else(|| format!("sse-session-{}", uuid::Uuid::new_v4()));

    // Extract authentication context
    let auth_context = AxumHttpEngine::extract_auth_context(&headers);

    // Handle SSE initialization
    match handler
        .handle_mcp_request(
            session_id,
            Vec::new(), // Empty body for SSE initialization
            ResponseMode::ServerSentEvents,
            auth_context,
        )
        .await
    {
        Ok(http_response) => {
            let mut response = Response::builder().status(http_response.status);

            // Add headers
            for (key, value) in http_response.headers {
                response = response.header(key, value);
            }

            response
                .body(http_response.body.into())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Health check endpoint
async fn handle_health_check() -> Result<Response, StatusCode> {
    Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(r#"{"status":"healthy"}"#.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_axum_engine_creation() {
        let config = AxumEngineConfig::default();
        let engine = AxumHttpEngine::new(config);
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert_eq!(engine.engine_type(), "axum");
        assert!(!engine.is_bound());
        assert!(!engine.is_running());
        assert!(engine.local_addr().is_none());
    }

    #[tokio::test]
    async fn test_axum_engine_bind() {
        let config = AxumEngineConfig::default();
        let mut engine = AxumHttpEngine::new(config).unwrap();

        // Test binding to available port
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let result = engine.bind(addr).await;
        assert!(result.is_ok());
        assert!(engine.is_bound());
        assert!(engine.local_addr().is_some());

        // Test double binding fails
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
        let result2 = engine.bind(addr2).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_authentication_config() {
        let config = AxumEngineConfig::default();
        let mut engine = AxumHttpEngine::new(config).unwrap();

        // Test OAuth2 configuration
        let auth_config = AuthenticationConfig::oauth2(
            "https://example.com/.well-known/jwks.json".to_string(),
            "test-audience".to_string(),
            "https://example.com".to_string(),
        );

        let result = engine.register_authentication(auth_config);
        assert!(result.is_ok());

        // Test unsupported authentication method
        let invalid_config = AuthenticationConfig {
            method: "unsupported".to_string(),
            config: HashMap::new(),
        };

        let result2 = engine.register_authentication(invalid_config);
        assert!(result2.is_err());
    }

    #[test]
    fn test_http_response_creation() {
        let body = b"test response".to_vec();

        // Test JSON response
        let json_response = HttpResponse::json(body.clone());
        assert_eq!(json_response.status, 200);
        assert_eq!(json_response.mode, ResponseMode::Json);
        assert_eq!(
            json_response.headers.get("content-type"),
            Some(&"application/json".to_string())
        );

        // Test SSE response
        let sse_response = HttpResponse::sse(body.clone());
        assert_eq!(sse_response.status, 200);
        assert_eq!(sse_response.mode, ResponseMode::ServerSentEvents);
        assert_eq!(
            sse_response.headers.get("content-type"),
            Some(&"text/event-stream".to_string())
        );

        // Test error response
        let error_response = HttpResponse::error(400, "Bad Request");
        assert_eq!(error_response.status, 400);
        assert!(error_response.body.len() > 0);
    }
}

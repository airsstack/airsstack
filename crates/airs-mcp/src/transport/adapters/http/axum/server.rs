//! Axum HTTP Server Implementation
//!
//! This module provides the main HTTP server implementation using Axum framework
//! with clean separation of concerns and proper dependency injection.
//!
//! The AxumHttpServer implements the HttpEngine trait to provide pluggable
//! HTTP framework abstraction while maintaining full MCP protocol support.

// Layer 1: Standard library imports
use std::net::SocketAddr;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::net::TcpListener;
use tokio::sync::broadcast;

// Layer 3: Internal module imports
use crate::authentication::manager::AuthenticationManager;
use crate::authentication::strategy::AuthenticationStrategy;
use crate::authentication::AuthContext;
use crate::authorization::{
    context::{AuthzContext, BinaryAuthContext, NoAuthContext, ScopeAuthContext},
    middleware::AuthorizationRequest,
    policy::{
        AuthorizationPolicy, BinaryAuthorizationPolicy, NoAuthorizationPolicy, ScopeBasedPolicy,
    },
};
// TODO: Migrate concurrent features to new protocol module
// use crate::base::jsonrpc::concurrent::ConcurrentProcessor;
use crate::transport::adapters::http::auth::jsonrpc_authorization::{
    JsonRpcAuthorizationLayer, JsonRpcHttpRequest,
};
use crate::transport::adapters::http::auth::middleware::{
    HttpAuthConfig, HttpAuthMiddleware, HttpAuthRequest, HttpAuthStrategyAdapter,
};
use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::adapters::http::connection_manager::HttpConnectionManager;
use crate::transport::adapters::http::engine::{
    HttpEngine, HttpEngineError, HttpMiddleware, McpRequestHandler,
};
use crate::transport::error::TransportError;

use super::handlers::{create_router, ServerState};
use super::mcp_handlers::{McpHandlers, McpHandlersBuilder};

/// Zero-cost no authentication adapter for default server behavior
///
/// This type serves as the default authentication strategy for AxumHttpServer,
/// providing a zero-cost abstraction when no authentication is required.
/// It implements HttpAuthStrategyAdapter with no-op behavior.
#[derive(Debug, Clone, Default)]
pub struct NoAuth;

#[async_trait]
impl HttpAuthStrategyAdapter for NoAuth {
    type RequestType = ();
    type AuthData = ();

    fn auth_method(&self) -> &'static str {
        "none"
    }

    async fn authenticate_http_request(
        &self,
        _request: &HttpAuthRequest,
    ) -> Result<
        AuthContext<Self::AuthData>,
        crate::transport::adapters::http::auth::oauth2::error::HttpAuthError,
    > {
        // NoAuth always returns a successful authentication with empty data
        use crate::authentication::AuthMethod;
        Ok(AuthContext::new(AuthMethod::new("none"), ()))
    }

    fn should_skip_path(&self, _path: &str) -> bool {
        true // NoAuth skips authentication for all paths
    }
}

/// HTTP server implementation using Axum framework
///
/// This server provides a clean, modular architecture for handling MCP JSON-RPC
/// requests over HTTP with proper session management and connection tracking.
/// It implements the HttpEngine trait for pluggable HTTP framework abstraction.
///
/// # Type Parameters
/// * `A` - Authentication strategy adapter (defaults to NoAuth for backward compatibility)
/// * `P` - Authorization policy (defaults to NoAuthorizationPolicy for backward compatibility)
/// * `C` - Authorization context (defaults to NoAuthContext for backward compatibility)
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
/// // Default server with no authentication or authorization
/// // let server = AxumHttpServer::new(...).await?;
///
/// // OAuth2 server with scope-based authorization
/// // let oauth2_server = server.with_oauth2_authorization(oauth2_adapter, config);
/// ```
pub struct AxumHttpServer<A = NoAuth, P = NoAuthorizationPolicy<NoAuthContext>, C = NoAuthContext>
where
    A: HttpAuthStrategyAdapter,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
    /// Server state shared across handlers
    state: ServerState<A, P, C>,
    /// TCP listener for accepting connections
    listener: Option<TcpListener>,
    /// Local address the server is bound to
    local_addr: Option<SocketAddr>,
    /// Whether the server is currently running
    is_running: bool,
    /// Registered MCP request handler
    mcp_handler: Option<Arc<dyn McpRequestHandler>>,
    /// Custom middleware
    middleware: Vec<Box<dyn HttpMiddleware>>,
}

impl AxumHttpServer<NoAuth> {
    /// Create a new Axum HTTP server with the specified configuration and handlers
    ///
    /// Creates a server with NoAuth (no authentication) as the default.
    /// Use with_authentication() to add authentication to the server.
    pub async fn new(
        connection_manager: Arc<HttpConnectionManager>,
        mcp_handlers: Arc<McpHandlers>,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        // Create SSE broadcast channel for HTTP Streamable support
        let (sse_broadcaster, _receiver) = broadcast::channel(1000);

        let state = ServerState {
            connection_manager,
            mcp_handlers,
            config: config.clone(),
            sse_broadcaster,
            auth_middleware: None,     // NoAuth has no middleware
            authorization_layer: None, // NoAuth has no authorization layer
        };

        Ok(Self {
            state,
            listener: None,
            local_addr: None,
            is_running: false,
            mcp_handler: None,
            middleware: Vec::new(),
        })
    }

    /// Create a new Axum HTTP server with empty MCP handlers (for testing/development)
    pub async fn new_with_empty_handlers(
        connection_manager: Arc<HttpConnectionManager>,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let mcp_handlers = Arc::new(McpHandlersBuilder::new().build());

        Self::new(connection_manager, mcp_handlers, config).await
    }

    /// Create a new Axum HTTP server using a handlers builder
    pub async fn with_handlers(
        connection_manager: Arc<HttpConnectionManager>,
        handlers_builder: McpHandlersBuilder,
        config: HttpTransportConfig,
    ) -> Result<Self, TransportError> {
        let mcp_handlers = Arc::new(handlers_builder.build());

        Self::new(connection_manager, mcp_handlers, config).await
    }

    /// Add authentication to the server (zero-cost type conversion)
    ///
    /// Converts the server from NoAuth to a specific authentication strategy.
    /// This is a zero-cost conversion that happens at compile time.
    ///
    /// # Type Parameters
    /// * `A` - Authentication strategy adapter
    ///
    /// # Arguments
    /// * `adapter` - Authentication strategy adapter
    /// * `config` - Authentication middleware configuration
    ///
    /// # Returns
    /// * AxumHttpServer with the specified authentication strategy
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthConfig;
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    ///
    /// // Example usage (requires proper setup)
    /// // let server = AxumHttpServer::new(
    /// //     connection_manager,
    /// //     session_manager,
    /// //     jsonrpc_processor,
    /// //     mcp_handlers,
    /// //     config,
    /// // ).await?
    /// // .with_authentication(oauth_adapter, HttpAuthConfig::default());
    /// ```
    pub fn with_authentication<A>(
        self,
        adapter: A,
        auth_config: HttpAuthConfig,
    ) -> AxumHttpServer<A>
    where
        A: HttpAuthStrategyAdapter,
    {
        let auth_middleware = HttpAuthMiddleware::new(adapter, auth_config);

        let new_state = ServerState {
            connection_manager: self.state.connection_manager,
            mcp_handlers: self.state.mcp_handlers,
            config: self.state.config,
            sse_broadcaster: self.state.sse_broadcaster,
            auth_middleware: Some(auth_middleware),
            authorization_layer: None, // Will be set by authorization builder methods
        };

        AxumHttpServer {
            state: new_state,
            listener: self.listener,
            local_addr: self.local_addr,
            is_running: self.is_running,
            mcp_handler: self.mcp_handler,
            middleware: self.middleware,
        }
    }

    /// Add OAuth2 authentication with scope-based authorization (zero-cost type conversion)
    ///
    /// This method creates a server with OAuth2 authentication and scope-based authorization
    /// in a single step. This is the most common pattern for OAuth2 integration.
    ///
    /// # Type Parameters
    /// * `O` - OAuth2 authentication strategy adapter
    ///
    /// # Arguments
    /// * `oauth2_adapter` - OAuth2 authentication strategy adapter
    /// * `auth_config` - HTTP authentication middleware configuration
    ///
    /// # Returns
    /// * AxumHttpServer with OAuth2 authentication and scope-based authorization
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthConfig;
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    ///
    /// // Example usage (requires proper OAuth2 setup)
    /// // let server = AxumHttpServer::new(deps).await?
    /// //     .with_oauth2_authorization(oauth2_adapter, HttpAuthConfig::default());
    /// ```
    pub fn with_oauth2_authorization<O>(
        self,
        oauth2_adapter: O,
        auth_config: HttpAuthConfig,
    ) -> AxumHttpServer<O, ScopeBasedPolicy, ScopeAuthContext>
    where
        O: HttpAuthStrategyAdapter,
    {
        let auth_middleware = HttpAuthMiddleware::new(oauth2_adapter, auth_config);
        let authorization_layer = JsonRpcAuthorizationLayer::new(ScopeBasedPolicy::mcp());

        let new_state = ServerState {
            connection_manager: self.state.connection_manager,
            mcp_handlers: self.state.mcp_handlers,
            config: self.state.config,
            sse_broadcaster: self.state.sse_broadcaster,
            auth_middleware: Some(auth_middleware),
            authorization_layer: Some(authorization_layer),
        };

        AxumHttpServer {
            state: new_state,
            listener: self.listener,
            local_addr: self.local_addr,
            is_running: self.is_running,
            mcp_handler: self.mcp_handler,
            middleware: self.middleware,
        }
    }

    /// Add custom authentication and authorization (zero-cost type conversion)
    ///
    /// This method allows full customization of both authentication and authorization.
    /// Use this when you need non-standard combinations.
    ///
    /// # Type Parameters
    /// * `A` - Authentication strategy adapter
    /// * `P` - Authorization policy
    /// * `C` - Authorization context
    ///
    /// # Arguments
    /// * `adapter` - Authentication strategy adapter
    /// * `auth_config` - HTTP authentication middleware configuration
    /// * `policy` - Authorization policy to apply
    ///
    /// # Returns
    /// * AxumHttpServer with the specified authentication and authorization
    pub fn with_auth_and_authz<A, P, C>(
        self,
        adapter: A,
        auth_config: HttpAuthConfig,
        policy: P,
    ) -> AxumHttpServer<A, P, C>
    where
        A: HttpAuthStrategyAdapter,
        P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
        C: AuthzContext + Clone,
    {
        let auth_middleware = HttpAuthMiddleware::new(adapter, auth_config);
        let authorization_layer = JsonRpcAuthorizationLayer::new(policy);

        let new_state = ServerState {
            connection_manager: self.state.connection_manager,
            mcp_handlers: self.state.mcp_handlers,
            config: self.state.config,
            sse_broadcaster: self.state.sse_broadcaster,
            auth_middleware: Some(auth_middleware),
            authorization_layer: Some(authorization_layer),
        };

        AxumHttpServer {
            state: new_state,
            listener: self.listener,
            local_addr: self.local_addr,
            is_running: self.is_running,
            mcp_handler: self.mcp_handler,
            middleware: self.middleware,
        }
    }
}

/// Implementation for servers with authentication but no authorization
impl<A> AxumHttpServer<A, NoAuthorizationPolicy<NoAuthContext>, NoAuthContext>
where
    A: HttpAuthStrategyAdapter,
{
    /// Add scope-based authorization to an authenticated server (zero-cost type conversion)
    ///
    /// This method is typically used after adding authentication to enable OAuth2
    /// scope-based authorization.
    ///
    /// # Returns
    /// * AxumHttpServer with scope-based authorization policy
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
    ///
    /// // Chain authentication and authorization
    /// // let server = AxumHttpServer::new(deps).await?
    /// //     .with_authentication(oauth2_adapter, HttpAuthConfig::default())
    /// //     .with_scope_authorization(ScopeBasedPolicy::new());
    /// ```
    pub fn with_scope_authorization(
        self,
        policy: ScopeBasedPolicy,
    ) -> AxumHttpServer<A, ScopeBasedPolicy, ScopeAuthContext> {
        let authorization_layer = JsonRpcAuthorizationLayer::new(policy);

        let new_state = ServerState {
            connection_manager: self.state.connection_manager,
            mcp_handlers: self.state.mcp_handlers,
            config: self.state.config,
            sse_broadcaster: self.state.sse_broadcaster,
            auth_middleware: self.state.auth_middleware, // Preserve authentication
            authorization_layer: Some(authorization_layer),
        };

        AxumHttpServer {
            state: new_state,
            listener: self.listener,
            local_addr: self.local_addr,
            is_running: self.is_running,
            mcp_handler: self.mcp_handler,
            middleware: self.middleware,
        }
    }

    /// Add binary authorization to an authenticated server (zero-cost type conversion)
    ///
    /// This method enables simple allow/deny authorization policies.
    ///
    /// # Arguments
    /// * `policy` - Binary authorization policy to apply
    ///
    /// # Returns
    /// * AxumHttpServer with binary authorization policy
    pub fn with_binary_authorization(
        self,
        policy: BinaryAuthorizationPolicy,
    ) -> AxumHttpServer<A, BinaryAuthorizationPolicy, BinaryAuthContext> {
        let authorization_layer = JsonRpcAuthorizationLayer::new(policy);

        let new_state = ServerState {
            connection_manager: self.state.connection_manager,
            mcp_handlers: self.state.mcp_handlers,
            config: self.state.config,
            sse_broadcaster: self.state.sse_broadcaster,
            auth_middleware: self.state.auth_middleware, // Preserve authentication
            authorization_layer: Some(authorization_layer),
        };

        AxumHttpServer {
            state: new_state,
            listener: self.listener,
            local_addr: self.local_addr,
            is_running: self.is_running,
            mcp_handler: self.mcp_handler,
            middleware: self.middleware,
        }
    }

    /// Add custom authorization to an authenticated server (zero-cost type conversion)
    ///
    /// This method allows full customization of authorization policy and context.
    ///
    /// # Type Parameters
    /// * `P` - Authorization policy
    /// * `C` - Authorization context
    ///
    /// # Arguments
    /// * `policy` - Authorization policy to apply
    ///
    /// # Returns
    /// * AxumHttpServer with the specified authorization policy
    pub fn with_authorization<P, C>(self, policy: P) -> AxumHttpServer<A, P, C>
    where
        P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
        C: AuthzContext + Clone,
    {
        let authorization_layer = JsonRpcAuthorizationLayer::new(policy);

        let new_state = ServerState {
            connection_manager: self.state.connection_manager,
            mcp_handlers: self.state.mcp_handlers,
            config: self.state.config,
            sse_broadcaster: self.state.sse_broadcaster,
            auth_middleware: self.state.auth_middleware, // Preserve authentication
            authorization_layer: Some(authorization_layer),
        };

        AxumHttpServer {
            state: new_state,
            listener: self.listener,
            local_addr: self.local_addr,
            is_running: self.is_running,
            mcp_handler: self.mcp_handler,
            middleware: self.middleware,
        }
    }
}

/// Generic implementation for AxumHttpServer with any authentication strategy
impl<A, P, C> AxumHttpServer<A, P, C>
where
    A: HttpAuthStrategyAdapter,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
    /// Bind the server to the specified address
    pub async fn bind(&mut self, addr: SocketAddr) -> Result<(), TransportError> {
        let listener = TcpListener::bind(addr).await.map_err(TransportError::Io)?;
        self.local_addr = Some(listener.local_addr().map_err(TransportError::Io)?);
        self.listener = Some(listener);
        Ok(())
    }

    /// Start the HTTP server and begin accepting connections
    pub async fn serve(mut self) -> Result<(), TransportError> {
        let app = create_router(self.state);

        let listener = self.listener.ok_or_else(|| TransportError::Format {
            message: "Server not bound to address".into(),
        })?;

        tracing::info!(
            "Starting Axum HTTP server on {}",
            listener.local_addr().unwrap()
        );

        self.is_running = true;

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(TransportError::Io)?;

        self.is_running = false;
        Ok(())
    }

    /// Start the HTTP server using mutable reference (for HttpEngine trait)
    pub async fn start(&mut self) -> Result<(), TransportError> {
        if self.is_running {
            return Err(TransportError::Format {
                message: "Server is already running".into(),
            });
        }

        let app = create_router(self.state.clone());

        let listener = self.listener.take().ok_or_else(|| TransportError::Format {
            message: "Server not bound to address".into(),
        })?;

        tracing::info!(
            "Starting Axum HTTP server on {}",
            listener.local_addr().unwrap()
        );

        self.is_running = true;

        // Start server in the background - this is a simplified approach
        // for the HttpEngine trait interface
        tokio::spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });

        Ok(())
    }

    /// Check if the server is bound to an address
    pub fn is_bound(&self) -> bool {
        self.listener.is_some()
    }

    /// Check if the server is currently running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the local address the server is bound to
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }
}

// ================================================================================================
// HttpEngine Trait Implementation
// ================================================================================================

#[async_trait]
impl<A, P, C> HttpEngine for AxumHttpServer<A, P, C>
where
    A: HttpAuthStrategyAdapter,
    P: AuthorizationPolicy<C, AuthorizationRequest<JsonRpcHttpRequest>> + Clone,
    C: AuthzContext + Clone,
{
    type Error = TransportError;
    type Config = HttpTransportConfig;
    type Handler = super::super::DefaultAxumMcpRequestHandler;

    /// Create a new HTTP engine with the given configuration
    fn new(_config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // Note: This simplified constructor is for HttpEngine trait compatibility.
        // However, AxumHttpServer requires complex async dependencies that cannot
        // be created in a synchronous constructor.
        //
        // For actual usage, use the `new()` method with proper dependency injection.
        // This limitation will be addressed in Phase 5 with proper async factory patterns.

        Err(TransportError::Format {
            message: "AxumHttpServer requires dependency injection - use new() with proper dependencies or async factory pattern".into(),
        })
    }

    /// Bind the engine to a network address
    async fn bind(&mut self, addr: SocketAddr) -> Result<(), HttpEngineError> {
        // Use the internal bind method from AxumHttpServer
        let listener = TcpListener::bind(addr).await.map_err(HttpEngineError::Io)?;
        self.local_addr = Some(listener.local_addr().map_err(HttpEngineError::Io)?);
        self.listener = Some(listener);
        Ok(())
    }

    /// Start the HTTP server
    async fn start(&mut self) -> Result<(), HttpEngineError> {
        if self.is_running {
            return Err(HttpEngineError::AlreadyRunning);
        }

        let app = create_router(self.state.clone());

        let listener = self
            .listener
            .take()
            .ok_or_else(|| HttpEngineError::Engine {
                message: "Server not bound to address".to_string(),
            })?;

        tracing::info!(
            "Starting Axum HTTP server on {}",
            listener.local_addr().unwrap()
        );

        self.is_running = true;

        // Start server in the background - this is a simplified approach
        // for the HttpEngine trait interface
        tokio::spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });

        Ok(())
    }

    /// Gracefully shutdown the HTTP server
    async fn shutdown(&mut self) -> Result<(), HttpEngineError> {
        if !self.is_running {
            return Ok(());
        }

        self.is_running = false;
        // Note: For proper shutdown, we'd need to store the server handle
        // This will be improved in Phase 5
        Ok(())
    }

    /// Register the MCP request handler
    fn register_mcp_handler(&mut self, handler: Self::Handler) {
        // For now, we'll need to store the handler in a way that's compatible 
        // with the existing server state. This is a transitional implementation.
        // TODO: Update ServerState to use concrete handler type in Phase 3
        self.mcp_handler = Some(std::sync::Arc::new(handler));
    }

    /// Register authentication middleware
    fn register_authentication<S, T, D>(
        &mut self,
        _auth_manager: AuthenticationManager<S, T, D>,
    ) -> Result<(), HttpEngineError>
    where
        S: AuthenticationStrategy<T, D>,
        T: Send + Sync,
        D: Send + Sync + 'static,
    {
        // TODO: Implement authentication integration
        // For now, this is a placeholder that accepts the authentication manager
        // but doesn't store it. Full integration will be implemented in Phase 5.
        Ok(())
    }

    /// Register custom HTTP middleware
    fn register_middleware(&mut self, middleware: Box<dyn HttpMiddleware>) {
        self.middleware.push(middleware);
    }

    /// Check if the engine is bound to an address
    fn is_bound(&self) -> bool {
        self.listener.is_some()
    }

    /// Check if the engine is currently running
    fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the local address the engine is bound to
    fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Get the engine type identifier
    fn engine_type(&self) -> &'static str {
        "axum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::adapters::http::connection_manager::HealthCheckConfig;

    async fn create_test_server() -> AxumHttpServer<NoAuth> {
        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));

        // Create HTTP transport configuration
        let config = HttpTransportConfig::new();

        AxumHttpServer::new_with_empty_handlers(connection_manager, config)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_axum_server_creation() {
        let server = create_test_server().await;
        assert!(!server.is_bound());
    }

    #[tokio::test]
    async fn test_axum_server_bind() {
        let mut server = create_test_server().await;
        let addr = "127.0.0.1:0".parse().unwrap();

        server.bind(addr).await.unwrap();
        assert!(server.is_bound());
    }
}

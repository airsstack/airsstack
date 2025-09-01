# KNOWLEDGE-004: HTTP Engine Abstraction Architecture

**Category**: Architecture  
**Created**: 2025-09-01  
**Updated**: 2025-09-01  
**Status**: Active  
**Priority**: High

## Overview

Comprehensive design for pluggable HTTP engine architecture that allows teams to choose their preferred HTTP framework (Axum, Rocket, Warp, etc.) while maintaining a consistent Transport interface and MCP protocol compliance.

## Key Design Principles

### Pluggable HTTP Engine Pattern

**Core Philosophy**: Separate HTTP framework specifics from MCP transport logic through clean abstraction layers.

**Benefits**:
- **Framework Choice**: Teams can use preferred HTTP framework (performance, familiarity, ecosystem)
- **Migration Flexibility**: Can switch HTTP engines without changing MCP protocol logic
- **Performance Optimization**: Choose engine based on specific performance requirements
- **Consistent Interface**: Same Transport trait regardless of underlying HTTP framework

### Architecture Layers

```text
┌─────────────────────────────────────────────────────────────┐
│                    MCP Protocol Layer                      │
│             (McpServer, MessageHandler)                    │
└─────────────────────┬───────────────────────────────────────┘
                      │ JsonRpcMessage, MessageContext
┌─────────────────────▼───────────────────────────────────────┐
│                 Transport Interface                        │
│              (HttpServerTransport<E>)                      │
└─────────────────────┬───────────────────────────────────────┘
                      │ HttpEngine trait
┌─────────────────────▼───────────────────────────────────────┐
│               HTTP Engine Layer                            │
│      (AxumHttpEngine, RocketHttpEngine, WarpHttpEngine)   │
└─────────────────────┬───────────────────────────────────────┘
                      │ Framework-specific implementation
┌─────────────────────▼───────────────────────────────────────┐
│              HTTP Framework                                │
│           (Axum, Rocket, Warp, etc.)                      │
└─────────────────────────────────────────────────────────────┘
```

## HttpEngine Trait Design

### Core Interface

```rust
#[async_trait]
pub trait HttpEngine: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    type Config: Clone + Send + Sync;
    
    /// Create new HTTP engine with configuration
    fn new(config: Self::Config) -> Result<Self, Self::Error> where Self: Sized;
    
    /// Lifecycle management
    async fn bind(&mut self, addr: SocketAddr) -> Result<(), Self::Error>;
    async fn start(&mut self) -> Result<(), Self::Error>;
    async fn shutdown(&mut self) -> Result<(), Self::Error>;
    
    /// MCP integration
    fn register_mcp_handler(&mut self, handler: Arc<dyn McpRequestHandler>);
    fn register_oauth_middleware(&mut self, oauth_config: OAuth2Config) -> Result<(), Self::Error>;
    fn register_middleware(&mut self, middleware: Box<dyn HttpMiddleware>);
    
    /// Server state
    fn is_bound(&self) -> bool;
    fn local_addr(&self) -> Option<SocketAddr>;
    fn engine_type(&self) -> &'static str;
}
```

### MCP Request Handler Interface

```rust
#[async_trait]
pub trait McpRequestHandler: Send + Sync {
    async fn handle_mcp_request(
        &self,
        session_id: String,
        request_data: Vec<u8>,
        response_mode: ResponseMode,
        auth_context: Option<AuthContext>,
    ) -> Result<HttpResponse, McpError>;
}

#[derive(Debug, Clone)]
pub enum ResponseMode {
    Json,           // Standard JSON response
    ServerSentEvents, // SSE streaming response  
    Streaming,      // Custom streaming response
}
```

## Engine Implementations

### Axum Engine (Default)

```rust
pub struct AxumHttpEngine {
    router: Option<Router>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    mcp_handler: Option<Arc<dyn McpRequestHandler>>,
    oauth_config: Option<OAuth2Config>,
    middleware_stack: Vec<Box<dyn HttpMiddleware>>,
    config: AxumEngineConfig,
    local_addr: Option<SocketAddr>,
}

#[derive(Debug, Clone)]
pub struct AxumEngineConfig {
    pub cors_enabled: bool,
    pub max_request_size: usize,
    pub timeout: Duration,
    pub graceful_shutdown_timeout: Duration,
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
        })
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        let router = self.build_router()?;
        let listener = tokio::net::TcpListener::bind(
            self.local_addr.ok_or(AxumEngineError::NotBound)?
        ).await?;
        
        self.server_handle = Some(tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown_signal())
                .await
        }));
        
        Ok(())
    }
    
    fn register_oauth_middleware(&mut self, oauth_config: OAuth2Config) -> Result<(), Self::Error> {
        self.oauth_config = Some(oauth_config);
        Ok(())
    }
    
    fn engine_type(&self) -> &'static str {
        "axum"
    }
}

impl AxumHttpEngine {
    fn build_router(&self) -> Result<Router, AxumEngineError> {
        let mut router = Router::new()
            .route("/mcp", axum::routing::post(handle_mcp_post))
            .route("/mcp", axum::routing::get(handle_mcp_sse));
        
        // Add OAuth middleware if configured
        if let Some(oauth_config) = &self.oauth_config {
            let oauth_layer = OAuth2MiddlewareCore::new(oauth_config.clone()).into_axum_layer();
            router = router.layer(oauth_layer);
        }
        
        // Add CORS if enabled
        if self.config.cors_enabled {
            router = router.layer(CorsLayer::very_permissive());
        }
        
        // Add custom middleware
        for middleware in &self.middleware_stack {
            router = middleware.apply_to_axum_router(router);
        }
        
        router = router.with_state(self.mcp_handler.clone());
        Ok(router)
    }
}

// Axum-specific request handlers
async fn handle_mcp_post(
    State(handler): State<Option<Arc<dyn McpRequestHandler>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, StatusCode> {
    let session_id = extract_session_id(&headers)?;
    let auth_context = headers.extensions().get::<AuthContext>().cloned();
    
    if let Some(h) = handler {
        let response = h.handle_mcp_request(
            session_id,
            body.to_vec(),
            ResponseMode::Json,
            auth_context,
        ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(response.into_axum_response())
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn handle_mcp_sse(
    State(handler): State<Option<Arc<dyn McpRequestHandler>>>,
    headers: HeaderMap,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let session_id = extract_session_id(&headers)?;
    let auth_context = headers.extensions().get::<AuthContext>().cloned();
    
    if let Some(h) = handler {
        let response = h.handle_mcp_request(
            session_id,
            Vec::new(),
            ResponseMode::ServerSentEvents,
            auth_context,
        ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(response.into_sse_stream())
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
```

### Rocket Engine (Alternative)

```rust
pub struct RocketHttpEngine {
    rocket: Option<Rocket<rocket::Build>>,
    mcp_handler: Option<Arc<dyn McpRequestHandler>>,
    oauth_config: Option<OAuth2Config>,
    config: RocketEngineConfig,
    local_addr: Option<SocketAddr>,
}

#[derive(Debug, Clone)]
pub struct RocketEngineConfig {
    pub port: u16,
    pub workers: usize,
    pub max_request_size: usize,
    pub keep_alive: Duration,
}

#[async_trait]
impl HttpEngine for RocketHttpEngine {
    type Error = RocketEngineError;
    type Config = RocketEngineConfig;
    
    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        let rocket_config = rocket::Config {
            port: config.port,
            workers: config.workers,
            max_request_size: config.max_request_size,
            keep_alive: config.keep_alive.as_secs() as u32,
            ..Default::default()
        };
        
        let rocket = rocket::build()
            .configure(rocket_config)
            .mount("/", routes![handle_mcp_post, handle_mcp_sse]);
            
        Ok(Self {
            rocket: Some(rocket),
            mcp_handler: None,
            oauth_config: None,
            config,
            local_addr: None,
        })
    }
    
    fn register_oauth_middleware(&mut self, oauth_config: OAuth2Config) -> Result<(), Self::Error> {
        self.oauth_config = Some(oauth_config);
        
        if let Some(rocket) = self.rocket.take() {
            let oauth_fairing = OAuth2MiddlewareCore::new(oauth_config).into_rocket_fairing();
            self.rocket = Some(rocket.attach(oauth_fairing));
        }
        
        Ok(())
    }
    
    fn engine_type(&self) -> &'static str {
        "rocket"
    }
}

#[rocket::post("/mcp", data = "<body>")]
async fn handle_mcp_post(
    body: Vec<u8>,
    headers: &rocket::http::HeaderMap,
    handler: &rocket::State<Arc<dyn McpRequestHandler>>,
) -> Result<String, rocket::http::Status> {
    let session_id = extract_session_id_rocket(headers)?;
    let auth_context = get_auth_context_rocket(headers);
    
    let response = handler.handle_mcp_request(
        session_id,
        body,
        ResponseMode::Json,
        auth_context,
    ).await.map_err(|_| rocket::http::Status::InternalServerError)?;
    
    Ok(response.into_rocket_response())
}

#[rocket::get("/mcp")]
async fn handle_mcp_sse(
    headers: &rocket::http::HeaderMap,
    handler: &rocket::State<Arc<dyn McpRequestHandler>>,
) -> Result<rocket::response::stream::EventStream![rocket::response::stream::Event], rocket::http::Status> {
    let session_id = extract_session_id_rocket(headers)?;
    let auth_context = get_auth_context_rocket(headers);
    
    let response = handler.handle_mcp_request(
        session_id,
        Vec::new(),
        ResponseMode::ServerSentEvents,
        auth_context,
    ).await.map_err(|_| rocket::http::Status::InternalServerError)?;
    
    Ok(response.into_rocket_sse_stream())
}
```

### Warp Engine (Alternative)

```rust
pub struct WarpHttpEngine {
    filters: Option<warp::filters::BoxedFilter<(impl warp::Reply,)>>,
    server_handle: Option<warp::serve::Server>,
    mcp_handler: Option<Arc<dyn McpRequestHandler>>,
    oauth_config: Option<OAuth2Config>,
    config: WarpEngineConfig,
    local_addr: Option<SocketAddr>,
}

#[derive(Debug, Clone)]
pub struct WarpEngineConfig {
    pub max_request_size: u64,
    pub cors_origins: Vec<String>,
    pub timeout: Duration,
}

#[async_trait]
impl HttpEngine for WarpHttpEngine {
    type Error = WarpEngineError;
    type Config = WarpEngineConfig;
    
    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            filters: None,
            server_handle: None,
            mcp_handler: None,
            oauth_config: None,
            config,
            local_addr: None,
        })
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        let filters = self.build_filters()?;
        let addr = self.local_addr.ok_or(WarpEngineError::NotBound)?;
        
        self.server_handle = Some(warp::serve(filters).bind(addr));
        Ok(())
    }
    
    fn register_oauth_middleware(&mut self, oauth_config: OAuth2Config) -> Result<(), Self::Error> {
        self.oauth_config = Some(oauth_config);
        Ok(())
    }
    
    fn engine_type(&self) -> &'static str {
        "warp"
    }
}

impl WarpHttpEngine {
    fn build_filters(&self) -> Result<warp::filters::BoxedFilter<(impl warp::Reply,)>, WarpEngineError> {
        let mcp_handler = self.mcp_handler.clone();
        
        // POST /mcp - JSON responses
        let mcp_post = warp::path("mcp")
            .and(warp::post())
            .and(warp::body::bytes())
            .and(warp::header::headers_cloned())
            .and_then(move |body: bytes::Bytes, headers: HeaderMap| {
                let handler = mcp_handler.clone();
                async move {
                    let session_id = extract_session_id_warp(&headers)?;
                    let auth_context = get_auth_context_warp(&headers);
                    
                    if let Some(h) = handler {
                        let response = h.handle_mcp_request(
                            session_id,
                            body.to_vec(),
                            ResponseMode::Json,
                            auth_context,
                        ).await.map_err(|_| warp::reject::custom(McpError::InternalError))?;
                        
                        Ok(response.into_warp_reply())
                    } else {
                        Err(warp::reject::custom(McpError::InternalError))
                    }
                }
            });
        
        // GET /mcp - SSE responses
        let mcp_sse = warp::path("mcp")
            .and(warp::get())
            .and(warp::header::headers_cloned())
            .and_then(move |headers: HeaderMap| {
                let handler = mcp_handler.clone();
                async move {
                    let session_id = extract_session_id_warp(&headers)?;
                    let auth_context = get_auth_context_warp(&headers);
                    
                    if let Some(h) = handler {
                        let response = h.handle_mcp_request(
                            session_id,
                            Vec::new(),
                            ResponseMode::ServerSentEvents,
                            auth_context,
                        ).await.map_err(|_| warp::reject::custom(McpError::InternalError))?;
                        
                        Ok(response.into_warp_sse_reply())
                    } else {
                        Err(warp::reject::custom(McpError::InternalError))
                    }
                }
            });
        
        let routes = mcp_post.or(mcp_sse);
        
        // Add OAuth middleware if configured
        let routes = if let Some(oauth_config) = &self.oauth_config {
            let oauth_filter = OAuth2MiddlewareCore::new(oauth_config.clone()).into_warp_filter();
            routes.and(oauth_filter).boxed()
        } else {
            routes.boxed()
        };
        
        // Add CORS if configured
        let routes = if !self.config.cors_origins.is_empty() {
            let cors = warp::cors()
                .allow_origins(self.config.cors_origins.iter().map(|s| s.as_str()))
                .allow_headers(vec!["authorization", "content-type"])
                .allow_methods(vec!["GET", "POST"]);
            routes.with(cors).boxed()
        } else {
            routes
        };
        
        Ok(routes)
    }
}
```

## Transport Integration

### Generic HTTP Transport

```rust
pub struct HttpServerTransport<E: HttpEngine> {
    engine: E,
    message_handler: Option<Arc<dyn MessageHandler>>,
    current_session: Option<String>,
    session_manager: Arc<SessionManager>,
    session_channels: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<JsonRpcMessage>>>>,
    is_connected: bool,
}

impl<E: HttpEngine> HttpServerTransport<E> {
    pub fn new(mut engine: E, session_manager: Arc<SessionManager>) -> Self {
        // Register transport as MCP handler with the engine
        let transport_handler = Arc::new(TransportMcpHandler::new());
        engine.register_mcp_handler(transport_handler);
        
        Self {
            engine,
            message_handler: None,
            current_session: None,
            session_manager,
            session_channels: Arc::new(Mutex::new(HashMap::new())),
            is_connected: false,
        }
    }
}

// Convenience constructors for different engines
impl HttpServerTransport<AxumHttpEngine> {
    pub fn with_axum(
        config: AxumEngineConfig,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self, HttpError> {
        let engine = AxumHttpEngine::new(config)?;
        Ok(Self::new(engine, session_manager))
    }
    
    pub fn with_axum_and_oauth(
        config: AxumEngineConfig,
        oauth_config: OAuth2Config,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self, HttpError> {
        let mut engine = AxumHttpEngine::new(config)?;
        engine.register_oauth_middleware(oauth_config)?;
        Ok(Self::new(engine, session_manager))
    }
}

impl HttpServerTransport<RocketHttpEngine> {
    pub fn with_rocket(
        config: RocketEngineConfig,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self, HttpError> {
        let engine = RocketHttpEngine::new(config)?;
        Ok(Self::new(engine, session_manager))
    }
    
    pub fn with_rocket_and_oauth(
        config: RocketEngineConfig,
        oauth_config: OAuth2Config,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self, HttpError> {
        let mut engine = RocketHttpEngine::new(config)?;
        engine.register_oauth_middleware(oauth_config)?;
        Ok(Self::new(engine, session_manager))
    }
}

impl HttpServerTransport<WarpHttpEngine> {
    pub fn with_warp(
        config: WarpEngineConfig,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self, HttpError> {
        let engine = WarpHttpEngine::new(config)?;
        Ok(Self::new(engine, session_manager))
    }
    
    pub fn with_warp_and_oauth(
        config: WarpEngineConfig,
        oauth_config: OAuth2Config,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self, HttpError> {
        let mut engine = WarpHttpEngine::new(config)?;
        engine.register_oauth_middleware(oauth_config)?;
        Ok(Self::new(engine, session_manager))
    }
}

#[async_trait]
impl<E: HttpEngine> Transport for HttpServerTransport<E> {
    type Error = HttpTransportError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        self.engine.start().await?;
        self.is_connected = true;
        Ok(())
    }
    
    async fn close(&mut self) -> Result<(), Self::Error> {
        self.engine.shutdown().await?;
        self.is_connected = false;
        Ok(())
    }
    
    async fn send(&mut self, message: JsonRpcMessage) -> Result<(), Self::Error> {
        if let Some(session_id) = &self.current_session {
            let channels = self.session_channels.lock().await;
            if let Some(sender) = channels.get(session_id) {
                sender.send(message)?;
            }
        }
        Ok(())
    }
    
    fn set_message_handler(&mut self, handler: Arc<dyn MessageHandler>) {
        self.message_handler = Some(handler);
    }
    
    fn transport_type(&self) -> &'static str {
        self.engine.engine_type()
    }
    
    fn is_connected(&self) -> bool {
        self.is_connected && self.engine.is_bound()
    }
}
```

## Usage Examples

### Framework Selection Based on Requirements

```rust
// High-performance async workload - use Axum
let axum_config = AxumEngineConfig {
    cors_enabled: true,
    max_request_size: 1024 * 1024,
    timeout: Duration::from_secs(30),
    graceful_shutdown_timeout: Duration::from_secs(10),
};
let transport = HttpServerTransport::with_axum_and_oauth(
    axum_config,
    oauth_config,
    session_manager,
)?;

// Type-safe API requirements - use Rocket  
let rocket_config = RocketEngineConfig {
    port: 3000,
    workers: 4,
    max_request_size: 2 * 1024 * 1024,
    keep_alive: Duration::from_secs(60),
};
let transport = HttpServerTransport::with_rocket_and_oauth(
    rocket_config,
    oauth_config,
    session_manager,
)?;

// Functional programming style - use Warp
let warp_config = WarpEngineConfig {
    max_request_size: 1024 * 1024,
    cors_origins: vec!["https://example.com".to_string()],
    timeout: Duration::from_secs(30),
};
let transport = HttpServerTransport::with_warp_and_oauth(
    warp_config,
    oauth_config,
    session_manager,
)?;

// All work with same MCP server interface!
let mcp_server = McpServerBuilder::new()
    .server_info("Multi-Engine Server", "1.0.0")
    .build(transport)  // ← Same regardless of engine choice
    .await?;
```

### OAuth Integration Across All Engines

```rust
// OAuth works consistently across all HTTP engines
let oauth_config = OAuth2Config::builder()
    .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json")?)
    .audience("mcp-server".to_string())
    .issuer("https://auth.example.com".to_string())
    .build()?;

// Engine choice doesn't affect OAuth functionality
match preferred_framework {
    "axum" => HttpServerTransport::with_axum_and_oauth(axum_config, oauth_config, session_manager)?,
    "rocket" => HttpServerTransport::with_rocket_and_oauth(rocket_config, oauth_config, session_manager)?,
    "warp" => HttpServerTransport::with_warp_and_oauth(warp_config, oauth_config, session_manager)?,
    _ => return Err("Unsupported framework"),
}
```

## Implementation Benefits

### Development Flexibility
- **Framework Migration**: Switch HTTP frameworks without changing MCP protocol logic
- **Team Preferences**: Different teams can use familiar HTTP frameworks
- **Performance Optimization**: Choose engine based on specific performance requirements
- **Ecosystem Integration**: Leverage framework-specific middleware and extensions

### Architectural Cleanness
- **Separation of Concerns**: HTTP framework details isolated from MCP transport logic
- **Consistent Interface**: Same Transport trait regardless of underlying HTTP engine
- **OAuth Integration**: Authentication works consistently across all engines
- **Response Mode Support**: JSON, SSE, and streaming responses supported by all engines

### Operational Benefits
- **Testing**: Can test MCP logic independently of HTTP framework choice
- **Deployment**: Framework choice becomes deployment-time decision
- **Monitoring**: Consistent metrics and logging across all HTTP engines
- **Maintenance**: Bug fixes and features benefit all engine implementations

## Related Documentation
- [ADR-001: MCP-Compliant Transport Redesign](../adr/ADR-001-mcp-compliant-transport-redesign.md)
- [KNOWLEDGE-003: MCP Transport Architecture Patterns](./KNOWLEDGE-003-mcp-transport-architecture-patterns.md)
- [OAuth2 Integration Documentation](../../oauth2/README.md)

## Implementation Notes

### Performance Considerations
- **Engine Selection**: Axum for async performance, Rocket for type safety, Warp for functional style
- **Memory Usage**: Shared session management across all engines
- **Request Parsing**: Framework-specific optimizations while maintaining consistent interface
- **Connection Handling**: Each engine handles connections according to its strengths

### Security Implementation
- **OAuth Integration**: Consistent security model across all HTTP engines
- **Session Isolation**: Framework-agnostic session management
- **Input Validation**: Engine-specific request validation with common security policies
- **Rate Limiting**: Framework-specific rate limiting with consistent configuration

### Testing Strategy
- **Engine-Agnostic Tests**: Test MCP protocol logic independently of HTTP engine
- **Engine-Specific Tests**: Test HTTP engine integration and framework-specific features
- **Performance Benchmarks**: Compare performance characteristics across different engines
- **Security Testing**: Validate OAuth and session security across all engines

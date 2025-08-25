# OAuth 2.1 Middleware Integration Technical Specification

**Document Version**: 1.0  
**Created**: August 13, 2025  
**Status**: Technical Specification - Ready for Implementation  
**Implementation Method**: Axum Middleware Stack with HTTP Streamable Integration  
**Target Timeline**: 3-week implementation cycle (Phase 2 of remote server plan)  

---

## Executive Summary

OAuth 2.1 + PKCE authentication implementation for airs-mcp using **Axum middleware architecture** that integrates seamlessly with HTTP Streamable transport. This specification provides a complete technical plan for enterprise-grade authentication with clean separation of concerns, reusable security components, and production-ready performance.

**Key Innovation**: OAuth middleware layer that wraps HTTP Streamable transport, providing authentication without modifying core transport logic.

## Core Architecture

### Middleware Stack Design

```rust
// Complete OAuth-enabled HTTP Streamable server
pub async fn create_oauth_enabled_server(
    transport_config: HttpTransportConfig,
    oauth_config: OAuth2Config,
) -> Result<Router, ServerError> {
    
    let transport = Arc::new(HttpStreamableTransport::new(transport_config).await?);
    let oauth_layer = Arc::new(OAuth2Middleware::new(oauth_config).await?);
    
    Router::new()
        .route("/mcp", post(handle_mcp_post))
        .route("/mcp", get(handle_mcp_get))
        .route("/health", get(health_check))
        .route("/.well-known/oauth-protected-resource", get(oauth_metadata))
        // Middleware stack (applied in LIFO order)
        .layer(TraceLayer::new_for_http())                  // 1. Logging/tracing
        .layer(CorsLayer::permissive())                     // 2. CORS handling
        .layer(rate_limiting_middleware())                  // 3. Rate limiting
        .layer(oauth_middleware_layer(oauth_layer))         // 4. OAuth authentication
        .layer(session_middleware_layer(transport.clone())) // 5. Session management
        .with_state(transport)
}
```

### OAuth 2.1 Middleware Core

```rust
// OAuth 2.1 authentication middleware
pub struct OAuth2Middleware {
    token_validator: Arc<dyn TokenValidator + Send + Sync>,
    protected_resource_metadata: ProtectedResourceMetadata,
    config: OAuth2Config,
    metrics: OAuth2Metrics,
}

#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub authorization_server_url: String,
    pub resource_server_id: String,
    pub jwks_endpoint: String,
    pub required_scopes: Vec<String>,
    pub human_approval_operations: Vec<String>,
    pub token_cache_ttl: Duration,
    pub enforce_resource_binding: bool,
}

// Authentication context passed through middleware chain
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub token_exp: SystemTime,
    pub audience: Vec<String>,
    pub custom_claims: HashMap<String, serde_json::Value>,
}
```

## Implementation Phases

### Phase 1: OAuth Foundation & Token Validation (Week 1)

#### 1.1 JWT Token Validator Implementation

```rust
// JWT token validation with JWKS support
pub struct JwtTokenValidator {
    jwks_client: Arc<JwksClient>,
    issuer: String,
    audience: String,
    algorithm: Algorithm,
    cache: Arc<RwLock<TokenCache>>,
}

#[async_trait]
pub trait TokenValidator: Send + Sync {
    async fn validate_token(&self, token: &str) -> Result<AuthContext, OAuth2Error>;
    async fn refresh_jwks(&self) -> Result<(), OAuth2Error>;
}

impl JwtTokenValidator {
    pub async fn new(config: &OAuth2Config) -> Result<Self, OAuth2Error> {
        let jwks_client = JwksClient::builder()
            .jwks_uri(config.jwks_endpoint.parse()?)
            .cache_duration(Duration::from_secs(300))
            .request_timeout(Duration::from_secs(10))
            .build()?;
            
        Ok(Self {
            jwks_client: Arc::new(jwks_client),
            issuer: config.authorization_server_url.clone(),
            audience: config.resource_server_id.clone(),
            algorithm: Algorithm::RS256,
            cache: Arc::new(RwLock::new(TokenCache::new())),
        })
    }
    
    async fn validate_jwt(&self, token: &str) -> Result<AuthContext, OAuth2Error> {
        // Check cache first
        if let Some(cached) = self.cache.read().await.get(token) {
            if cached.expires_at > SystemTime::now() {
                return Ok(cached.auth_context.clone());
            }
        }
        
        // Validate with JWKS
        let header = decode_header(token)?;
        let kid = header.kid.ok_or(OAuth2Error::MissingKeyId)?;
        
        let jwk = self.jwks_client.get(&kid).await?;
        let decoding_key = DecodingKey::from_jwk(&jwk)?;
        
        let validation = Validation::new(self.algorithm);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        
        let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
        
        // Validate resource binding (RFC 8707)
        if !token_data.claims.aud.contains(&self.audience) {
            return Err(OAuth2Error::InvalidAudience);
        }
        
        let auth_context = AuthContext {
            user_id: token_data.claims.sub,
            client_id: token_data.claims.client_id,
            scopes: token_data.claims.scope.split_whitespace().map(String::from).collect(),
            token_exp: UNIX_EPOCH + Duration::from_secs(token_data.claims.exp as u64),
            audience: token_data.claims.aud,
            custom_claims: token_data.claims.extra,
        };
        
        // Cache validated token
        self.cache.write().await.insert(
            token.to_string(),
            CachedToken {
                auth_context: auth_context.clone(),
                expires_at: auth_context.token_exp,
            }
        );
        
        Ok(auth_context)
    }
}
```

#### 1.2 OAuth Middleware Handler

```rust
// OAuth middleware implementation
pub fn oauth_middleware_layer(
    oauth: Arc<OAuth2Middleware>
) -> tower::middleware::FromFnLayer<
    impl Fn(Request<Body>, Next<Body>) -> impl Future<Output = Result<Response<Body>, Infallible>> + Clone,
    Request<Body>,
    Response<Body>,
> {
    tower::middleware::from_fn(move |req, next| {
        let oauth = oauth.clone();
        async move {
            oauth_middleware_handler(req, next, oauth).await
        }
    })
}

async fn oauth_middleware_handler(
    mut req: Request<Body>,
    next: Next<Body>,
    oauth: Arc<OAuth2Middleware>,
) -> Result<Response<Body>, Infallible> {
    
    // Skip OAuth for health checks and metadata endpoints
    if req.uri().path() == "/health" || 
       req.uri().path() == "/.well-known/oauth-protected-resource" {
        return next.run(req).await;
    }
    
    // Extract Authorization header
    let auth_header = req.headers().get("authorization");
    let token = match extract_bearer_token(auth_header) {
        Ok(token) => token,
        Err(e) => return Ok(oauth.create_unauthorized_response(e).await),
    };
    
    // Validate OAuth token
    match oauth.token_validator.validate_token(&token).await {
        Ok(auth_context) => {
            // Record successful authentication
            oauth.metrics.successful_authentications.inc();
            
            // Attach OAuth context to request
            req.extensions_mut().insert(auth_context);
            
            // Continue to next middleware
            next.run(req).await
        },
        Err(oauth_error) => {
            // Record failed authentication
            oauth.metrics.failed_authentications.inc();
            
            // Return RFC 6750 compliant error response
            Ok(oauth.create_error_response(oauth_error).await)
        }
    }
}

impl OAuth2Middleware {
    async fn create_error_response(&self, error: OAuth2Error) -> Response<Body> {
        let (status, error_code, description) = match error {
            OAuth2Error::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_request",
                "Missing authorization header"
            ),
            OAuth2Error::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "The access token is invalid or expired"
            ),
            OAuth2Error::InsufficientScope => (
                StatusCode::FORBIDDEN,
                "insufficient_scope",
                "The request requires higher privileges"
            ),
            OAuth2Error::InvalidAudience => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "Token audience does not match this resource server"
            ),
        };
        
        let www_authenticate = format!(
            r#"Bearer realm="mcp-server", resource_metadata="{}/.well-known/oauth-protected-resource", error="{}", error_description="{}""#,
            self.config.resource_server_id,
            error_code,
            description
        );
        
        Response::builder()
            .status(status)
            .header("WWW-Authenticate", www_authenticate)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::json!({
                "error": error_code,
                "error_description": description
            }).to_string()))
            .unwrap()
    }
}
```

### Phase 2: Session Integration & Scope Management (Week 2)

#### 2.1 Enhanced Session Middleware

```rust
// Session middleware with OAuth context integration
async fn session_middleware_handler(
    mut req: Request<Body>,
    next: Next<Body>,
    transport: Arc<HttpStreamableTransport>,
) -> Result<Response<Body>, Infallible> {
    
    // Extract or create session ID
    let session_id = extract_or_create_session_id(&req.headers());
    
    // Get or create session
    let mut session = transport.session_manager
        .get_or_create_session(session_id).await;
    
    // Integrate OAuth context from previous middleware
    if let Some(auth_context) = req.extensions().get::<AuthContext>() {
        session.auth_context = Some(auth_context.clone());
        session.last_activity = Instant::now();
        session.authenticated = true;
        
        // Update session scopes
        session.authorized_scopes = auth_context.scopes.clone();
        session.user_id = Some(auth_context.user_id.clone());
    }
    
    // Attach session to request
    req.extensions_mut().insert(session.clone());
    
    // Process request
    let mut response = next.run(req).await;
    
    // Add session header to response
    response.headers_mut().insert(
        "Mcp-Session-Id",
        session_id.to_string().parse().unwrap()
    );
    
    Ok(response)
}

// Enhanced session context with OAuth integration
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub id: SessionId,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub last_event_id: Option<EventId>,
    
    // OAuth integration fields
    pub authenticated: bool,
    pub auth_context: Option<AuthContext>,
    pub authorized_scopes: Vec<String>,
    pub user_id: Option<String>,
    
    // Connection state
    pub connection_state: ConnectionState,
    pub capabilities: Option<NegotiatedCapabilities>,
}
```

#### 2.2 Operation-Specific Scope Validation

```rust
// Scope validation utilities
pub struct ScopeValidator;

impl ScopeValidator {
    pub fn require_scope(scopes: &[String], required: &str) -> Result<(), OAuth2Error> {
        if scopes.contains(&required.to_string()) {
            Ok(())
        } else {
            Err(OAuth2Error::InsufficientScope)
        }
    }
    
    pub fn require_any_scope(scopes: &[String], required: &[&str]) -> Result<(), OAuth2Error> {
        for scope in required {
            if scopes.contains(&scope.to_string()) {
                return Ok(());
            }
        }
        Err(OAuth2Error::InsufficientScope)
    }
    
    pub fn validate_mcp_operation(
        scopes: &[String],
        method: &str,
    ) -> Result<(), OAuth2Error> {
        match method {
            "initialize" => Self::require_scope(scopes, "mcp:connect"),
            "tools/list" => Self::require_scope(scopes, "mcp:tools:read"),
            "tools/call" => Self::require_scope(scopes, "mcp:tools:execute"),
            "resources/list" => Self::require_scope(scopes, "mcp:resources:read"),
            "resources/read" => Self::require_scope(scopes, "mcp:resources:read"),
            "resources/subscribe" => Self::require_scope(scopes, "mcp:resources:subscribe"),
            "prompts/list" => Self::require_scope(scopes, "mcp:prompts:read"),
            "prompts/get" => Self::require_scope(scopes, "mcp:prompts:read"),
            "logging/setLevel" => Self::require_scope(scopes, "mcp:logging:write"),
            _ => Err(OAuth2Error::InsufficientScope),
        }
    }
}
```

### Phase 3: Token Lifecycle & Rate Limiting (Week 3)

#### 3.1 Token Lifecycle Management System

```rust
// Token lifecycle management with refresh and caching
#[derive(Debug)]
pub struct TokenManager {
    cache: Arc<RwLock<TokenCache>>,
    refresh_client: RefreshTokenClient,
    config: TokenConfig,
}

#[derive(Debug, Clone)]
pub struct TokenCache {
    entries: HashMap<String, CachedToken>,
    expiration_queue: BinaryHeap<ExpirationEntry>,
}

#[derive(Debug, Clone)]
pub struct CachedToken {
    pub auth_context: AuthContext,
    pub expires_at: SystemTime,
    pub refresh_token: Option<String>,
    pub cached_at: SystemTime,
}

impl TokenManager {
    pub async fn new(config: TokenConfig) -> Result<Self, TokenError> {
        let refresh_client = RefreshTokenClient::new(
            config.token_endpoint.clone(),
            config.client_id.clone(),
            config.client_secret.clone(),
        )?;
        
        Ok(Self {
            cache: Arc::new(RwLock::new(TokenCache::new())),
            refresh_client,
            config,
        })
    }
    
    pub async fn get_valid_token(&self, token: &str) -> Result<AuthContext, TokenError> {
        // Check cache first
        if let Some(cached) = self.cache.read().await.get(token) {
            // If token is still valid, return cached context
            if cached.expires_at > SystemTime::now() + Duration::from_secs(30) {
                return Ok(cached.auth_context.clone());
            }
            
            // If token is near expiry and has refresh token, attempt refresh
            if let Some(refresh_token) = &cached.refresh_token {
                if let Ok(new_tokens) = self.refresh_token(refresh_token).await {
                    self.cache_token(&new_tokens.access_token, new_tokens.auth_context.clone(), 
                                   new_tokens.refresh_token).await?;
                    return Ok(new_tokens.auth_context);
                }
            }
        }
        
        // Token not in cache or expired without refresh - validation required
        Err(TokenError::ValidationRequired)
    }
    
    async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshResult, TokenError> {
        let response = self.refresh_client.refresh_access_token(refresh_token).await?;
        
        let auth_context = AuthContext::from_token_response(&response)?;
        
        Ok(RefreshResult {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            auth_context,
            expires_in: response.expires_in,
        })
    }
    
    async fn cache_token(
        &self,
        token: &str,
        auth_context: AuthContext,
        refresh_token: Option<String>
    ) -> Result<(), TokenError> {
        let expires_at = SystemTime::now() + Duration::from_secs(auth_context.expires_in);
        
        let cached_token = CachedToken {
            auth_context,
            expires_at,
            refresh_token,
            cached_at: SystemTime::now(),
        };
        
        self.cache.write().await.insert(token.to_string(), cached_token);
        Ok(())
    }
    
    // Background task for cleaning expired tokens
    pub async fn cleanup_expired_tokens(&self) {
        let mut cache = self.cache.write().await;
        let now = SystemTime::now();
        
        cache.entries.retain(|_, token| token.expires_at > now);
        
        // Update expiration queue
        while let Some(entry) = cache.expiration_queue.peek() {
            if entry.expires_at <= now {
                cache.expiration_queue.pop();
            } else {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct RefreshResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub auth_context: AuthContext,
    pub expires_in: u64,
}
```

#### 3.2 Rate Limiting Middleware

```rust
// Rate limiting middleware for OAuth-protected endpoints
#[derive(Debug, Clone)]
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
    config: RateLimitConfig,
}

#[derive(Debug)]
pub struct RateLimiter {
    client_limits: Arc<RwLock<HashMap<String, ClientRateLimit>>>,
    global_limit: Arc<Mutex<GlobalRateLimit>>,
    config: RateLimitConfig,
}

#[derive(Debug)]
pub struct ClientRateLimit {
    pub client_id: String,
    pub requests: VecDeque<SystemTime>,
    pub window_start: SystemTime,
    pub current_count: u32,
    pub blocked_until: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
    pub block_duration: Duration,
    pub global_requests_per_second: u32,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(config.clone())),
            config,
        }
    }
}

#[async_trait]
impl<S> Layer<S> for RateLimitMiddleware {
    type Service = RateLimitService<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<RateLimiter>,
}

#[async_trait]
impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    
    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let limiter = self.limiter.clone();
        let mut inner = self.inner.clone();
        
        Box::pin(async move {
            // Extract client info from OAuth context
            let client_id = req.extensions()
                .get::<AuthContext>()
                .map(|ctx| &ctx.client_id)
                .unwrap_or("anonymous");
            
            // Check rate limits
            match limiter.check_rate_limit(client_id).await {
                RateLimitResult::Allowed => {
                    // Continue with request
                    inner.call(req).await
                },
                RateLimitResult::RateLimited { retry_after } => {
                    // Return 429 Too Many Requests
                    let response = Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .header("Retry-After", retry_after.as_secs().to_string())
                        .header("X-RateLimit-Limit", limiter.config.requests_per_minute.to_string())
                        .header("X-RateLimit-Remaining", "0")
                        .body(Body::from("Rate limit exceeded"))
                        .unwrap();
                    
                    Ok(response)
                },
                RateLimitResult::Blocked { until } => {
                    // Return 429 with longer block time
                    let retry_after = until.duration_since(SystemTime::now())
                        .unwrap_or(Duration::from_secs(60));
                    
                    let response = Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .header("Retry-After", retry_after.as_secs().to_string())
                        .body(Body::from("Client temporarily blocked due to abuse"))
                        .unwrap();
                    
                    Ok(response)
                }
            }
        })
    }
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            client_limits: Arc::new(RwLock::new(HashMap::new())),
            global_limit: Arc::new(Mutex::new(GlobalRateLimit::new(config.global_requests_per_second))),
            config,
        }
    }
    
    pub async fn check_rate_limit(&self, client_id: &str) -> RateLimitResult {
        // Check global rate limit first
        if !self.global_limit.lock().await.allow_request() {
            return RateLimitResult::RateLimited { 
                retry_after: Duration::from_secs(1) 
            };
        }
        
        let mut client_limits = self.client_limits.write().await;
        let now = SystemTime::now();
        
        let client_limit = client_limits.entry(client_id.to_string())
            .or_insert_with(|| ClientRateLimit::new(client_id.to_string()));
        
        // Check if client is currently blocked
        if let Some(blocked_until) = client_limit.blocked_until {
            if now < blocked_until {
                return RateLimitResult::Blocked { until: blocked_until };
            } else {
                client_limit.blocked_until = None;
            }
        }
        
        // Clean old requests outside the window
        let window_start = now - Duration::from_secs(60); // 1 minute window
        client_limit.requests.retain(|&req_time| req_time >= window_start);
        
        // Check rate limits
        if client_limit.requests.len() >= self.config.requests_per_minute as usize {
            // Block client for abuse
            client_limit.blocked_until = Some(now + self.config.block_duration);
            return RateLimitResult::Blocked { until: client_limit.blocked_until.unwrap() };
        }
        
        // Allow request and record it
        client_limit.requests.push_back(now);
        RateLimitResult::Allowed
    }
}

#[derive(Debug)]
pub enum RateLimitResult {
    Allowed,
    RateLimited { retry_after: Duration },
    Blocked { until: SystemTime },
}
```

#### 3.3 Production Integration

```rust
// Integration with OAuth middleware stack
Router::new()
    .route("/mcp", post(handle_mcp_post))
    .route("/mcp", get(handle_mcp_get))
    .layer(rate_limit_middleware_layer(rate_config))   // Rate limiting
    .layer(oauth_middleware_layer(oauth_config))       // OAuth authentication  
    .layer(session_middleware_layer(session_config))   // Session management
```
                    &request.id,
                    &approval_id,
                ));
            },
            ApprovalDecision::Approved { .. } => {
                // Continue with operation
            }
        }
    }
    
    // Process authenticated request
    let response = transport.process_authenticated_request(
        request,
        &auth_context,
        &session,
    ).await?;
    
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .header("X-Auth-User", &auth_context.user_id)
        .header("X-Auth-Scopes", auth_context.scopes.join(","))
        .body(Body::from(serde_json::to_string(&response)?))?)
}

pub async fn handle_mcp_get(
    Extension(auth_context): Extension<AuthContext>,
    Extension(session): Extension<SessionContext>,
    State(transport): State<Arc<HttpStreamableTransport>>,
    headers: HeaderMap,
) -> Result<Response, HttpError> {
    
    // Validate streaming permissions
    ScopeValidator::require_scope(&auth_context.scopes, "mcp:stream")?;
    
    // Create authenticated SSE stream
    let last_event_id = extract_last_event_id(&headers);
    let stream = transport.create_authenticated_stream(
        session.id,
        auth_context,
        last_event_id,
    ).await?;
    
    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response())
}
```

## Configuration & Integration

### Complete Configuration Structure

```rust
// Unified OAuth + HTTP Streamable configuration
#[derive(Debug, Clone)]
pub struct OAuthHttpConfig {
    // HTTP Streamable base configuration
    pub transport: HttpTransportConfig,
    
    // OAuth 2.1 configuration
    pub oauth: OAuth2Config,
    
    // Approval workflow configuration
    pub approval: ApprovalConfig,
    
    // Security configuration
    pub security: SecurityConfig,
}

impl OAuthHttpConfig {
    pub fn new() -> Self {
        Self {
            transport: HttpTransportConfig::new(),
            oauth: OAuth2Config::default(),
            approval: ApprovalConfig::default(),
            security: SecurityConfig::default(),
        }
    }
    
    // Builder pattern for easy configuration
    pub fn with_authorization_server(mut self, url: &str) -> Self {
        self.oauth.authorization_server_url = url.to_string();
        self
    }
    
    pub fn with_jwks_endpoint(mut self, url: &str) -> Self {
        self.oauth.jwks_endpoint = url.to_string();
        self
    }
    
    pub fn require_human_approval(mut self, operations: Vec<&str>) -> Self {
        self.oauth.human_approval_operations = operations.into_iter()
            .map(String::from)
            .collect();
        self
    }
}
```

### Usage Examples

```rust
// Example 1: Simple OAuth integration
let config = OAuthHttpConfig::new()
    .with_authorization_server("https://auth.example.com")
    .with_jwks_endpoint("https://auth.example.com/.well-known/jwks.json")
    .require_human_approval(vec!["tools/call"]);

let server = create_oauth_enabled_server(
    config.transport,
    config.oauth,
).await?;

// Example 2: Enterprise deployment with external IdP
let config = OAuthHttpConfig::new()
    .with_authorization_server("https://login.microsoftonline.com/tenant-id/v2.0")
    .with_jwks_endpoint("https://login.microsoftonline.com/tenant-id/discovery/v2.0/keys")
    .require_human_approval(vec!["tools/call", "resources/read"])
    .with_resource_id("api://mcp-server.company.com");

// Example 3: AWS Cognito integration
let config = OAuthHttpConfig::new()
    .with_authorization_server("https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123DEF")
    .with_jwks_endpoint("https://cognito-idp.us-east-1.amazonaws.com/us-east-1_ABC123DEF/.well-known/jwks.json")
    .with_required_scopes(vec!["mcp:tools:execute", "mcp:resources:read"]);
```

## Dependencies & Integration

### Required Cargo.toml Dependencies

```toml
[dependencies]
# OAuth 2.1 and JWT handling
jsonwebtoken = "9.0"
oauth2 = "4.4"
reqwest = { version = "0.11", features = ["json"] }

# Middleware and HTTP server
axum = { version = "0.7", features = ["middleware", "tokio"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Async and concurrency
tokio = { version = "1.0", features = ["full"] }
dashmap = "5.5"

# Serialization and time
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
time = "0.3"

# Caching and performance
moka = { version = "0.12", features = ["future"] }
```

### Integration with Existing Infrastructure

```rust
// Seamless integration with existing HTTP Streamable transport
impl HttpStreamableTransport {
    pub async fn with_oauth(
        mut self,
        oauth_config: OAuth2Config,
    ) -> Result<Self, OAuth2Error> {
        self.oauth_middleware = Some(OAuth2Middleware::new(oauth_config).await?);
        Ok(self)
    }
    
    pub async fn process_authenticated_request(
        &self,
        request: JsonRpcRequest,
        auth_context: &AuthContext,
        session: &SessionContext,
    ) -> Result<JsonRpcResponse, MccError> {
        
        // Audit logging
        self.audit_logger.log_request(
            &request.method,
            &auth_context.user_id,
            &auth_context.client_id,
            &session.id,
        ).await;
        
        // Process with existing MCP infrastructure
        self.mcp_processor
            .process_request_with_context(request, auth_context, session)
            .await
    }
}
```

## Testing Strategy

### OAuth Middleware Testing

```rust
#[cfg(test)]
mod oauth_middleware_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_valid_token_processing() {
        let oauth_middleware = create_test_oauth_middleware().await;
        let valid_token = create_test_jwt_token();
        
        let req = Request::builder()
            .header("Authorization", format!("Bearer {}", valid_token))
            .body(Body::empty())
            .unwrap();
            
        let response = oauth_middleware_handler(req, test_next_handler(), oauth_middleware).await;
        
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_invalid_token_rejection() {
        let oauth_middleware = create_test_oauth_middleware().await;
        
        let req = Request::builder()
            .header("Authorization", "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();
            
        let response = oauth_middleware_handler(req, test_next_handler(), oauth_middleware).await;
        
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().get("WWW-Authenticate").is_some());
    }
    
    #[tokio::test]
    async fn test_scope_validation() {
        let auth_context = AuthContext {
            scopes: vec!["mcp:tools:read".to_string()],
            ..Default::default()
        };
        
        // Should succeed for tools/list
        assert!(ScopeValidator::validate_mcp_operation(&auth_context.scopes, "tools/list").is_ok());
        
        // Should fail for tools/call (requires mcp:tools:execute)
        assert!(ScopeValidator::validate_mcp_operation(&auth_context.scopes, "tools/call").is_err());
    }
}
```

## Performance & Security Characteristics

### Performance Targets
- **OAuth Validation Latency**: <5ms (with token caching)
- **Middleware Overhead**: <2ms per request
- **JWKS Cache Hit Rate**: >95%
- **Session Creation**: <1ms

### Security Features
- **RFC 6750 Compliance**: Proper WWW-Authenticate headers
- **RFC 8707 Resource Indicators**: Mandatory token-to-resource binding
- **Token Caching**: Secure in-memory cache with TTL
- **Audit Logging**: Comprehensive authentication event tracking
- **Rate Limiting**: Per-client and per-user request limits

This specification provides a complete, production-ready OAuth 2.1 middleware integration that preserves the HTTP Streamable transport's performance while adding enterprise-grade authentication capabilities.

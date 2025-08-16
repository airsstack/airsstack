# OAuth 2.1 Module Structure & Integration Architecture

This document defines the complete code organization, module structure, and integration patterns for OAuth 2.1 authentication in airs-mcp, following our Single Responsibility Principle and standards compliance requirements.

## Module Structure Overview

```
crates/airs-mcp/src/
├── lib.rs                          # Existing - add oauth2 module export
├── oauth2/                         # NEW MODULE - OAuth 2.1 implementation
│   ├── mod.rs                      # Module organization & public API
│   ├── middleware.rs               # Axum middleware for OAuth validation
│   ├── jwt_validator.rs            # JWT token validation & JWKS client
│   ├── scope_validator.rs          # MCP method to OAuth scope mapping
│   ├── metadata.rs                 # RFC 9728 Protected Resource Metadata
│   ├── error.rs                    # OAuth-specific error types
│   ├── config.rs                   # OAuth configuration structures
│   └── context.rs                  # AuthContext for authenticated requests
├── transport/
│   ├── http/
│   │   ├── client.rs               # Existing - no changes needed
│   │   ├── server.rs               # Existing - integrate oauth middleware
│   │   └── mod.rs                  # Existing - no changes needed
│   └── mod.rs                      # Existing
└── ...                             # Other existing modules
```

## Core Module Specifications

### OAuth Module Organization (`oauth2/mod.rs`)

**Purpose**: Public API coordination following Single Responsibility Principle

```rust
//! OAuth 2.1 Authentication for MCP Protocol
//! 
//! Provides enterprise-grade OAuth 2.1 authentication with:
//! - RFC 9728: Protected Resource Metadata
//! - RFC 7636: PKCE (Proof Key for Code Exchange)  
//! - RFC 8707: Resource Indicators
//! - MCP 2025-06-18: Protocol integration

pub mod middleware;
pub mod jwt_validator;
pub mod scope_validator;
pub mod metadata;
pub mod error;
pub mod config;
pub mod context;

// Public API - only expose what's needed for integration
pub use middleware::{OAuth2Middleware, oauth2_middleware_layer};
pub use jwt_validator::JwtValidator;
pub use scope_validator::ScopeValidator;
pub use metadata::ProtectedResourceMetadata;
pub use error::{OAuth2Error, OAuth2Result};
pub use config::OAuth2Config;
pub use context::AuthContext;

// Re-export common types for convenience
pub use jsonwebtoken::{DecodingKey, Validation};
```

### 1. Middleware Integration (`middleware.rs`)

**Single Responsibility**: Axum middleware for OAuth 2.1 token validation and request authentication

**Core Functions**:
- Extract Bearer token from Authorization header
- Validate JWT signature and claims via JwtValidator
- Extract and validate scopes for MCP methods via ScopeValidator
- Inject AuthContext into request extensions for MCP handlers
- Generate RFC 6750 compliant error responses

**Key Structures**:
```rust
pub struct OAuth2Middleware {
    jwt_validator: Arc<JwtValidator>,
    scope_validator: Arc<ScopeValidator>,
}

pub fn oauth2_middleware_layer(config: OAuth2Config) -> OAuth2MiddlewareLayer
```

**Integration Pattern**:
- Axum middleware layer that wraps existing HTTP transport
- Zero modifications required to existing transport code
- Clean separation between OAuth authentication and MCP transport

### 2. JWT Token Validation (`jwt_validator.rs`)

**Single Responsibility**: JWT token validation with JWKS client support and caching

**Core Functions**:
- JWKS endpoint client with automatic key retrieval and caching
- RS256 signature validation using retrieved public keys
- Audience and issuer verification per OAuth 2.1 requirements
- Token expiration and not-before claims validation
- Key rotation support with cache invalidation

**Key Structures**:
```rust
pub struct JwtValidator {
    jwks_client: JwksClient,
    validation_config: Validation,
    cache: Arc<DashMap<String, DecodingKey>>,
}
```

**Performance Considerations**:
- DashMap for concurrent key caching
- Configurable cache TTL for JWKS keys
- Async JWKS client with connection pooling

### 3. MCP Scope Validation (`scope_validator.rs`)

**Single Responsibility**: MCP method to OAuth scope mapping and validation

**Core Functions**:
- Map MCP protocol methods to required OAuth scopes
- Validate user token scopes against MCP method requirements
- Provide scope requirement lookup for MCP operations
- Support for custom scope mappings via configuration

**MCP Scope Mappings**:
```rust
const MCP_SCOPE_MAPPINGS: &[(&str, &str)] = &[
    ("tools/call", "mcp:tools:execute"),
    ("tools/list", "mcp:tools:read"),
    ("resources/read", "mcp:resources:read"),
    ("resources/list", "mcp:resources:list"),
    ("resources/subscribe", "mcp:resources:subscribe"),
    ("prompts/get", "mcp:prompts:read"),
    ("prompts/list", "mcp:prompts:list"),
    ("logging/setLevel", "mcp:logging:configure"),
];
```

**Key Structures**:
```rust
pub struct ScopeValidator {
    scope_mappings: HashMap<String, String>,
}
```

### 4. Protected Resource Metadata (`metadata.rs`)

**Single Responsibility**: RFC 9728 Protected Resource Metadata endpoint implementation

**Core Functions**:
- Serve `/.well-known/oauth-protected-resource` endpoint
- Provide OAuth authorization server discovery information
- Advertise supported MCP scopes and bearer token methods
- Generate RFC 9728 compliant metadata responses

**Key Structures**:
```rust
pub struct ProtectedResourceMetadata {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub bearer_methods_supported: Vec<String>,
}

pub async fn handle_metadata() -> Json<ProtectedResourceMetadata>
```

**RFC 9728 Compliance**:
- Exact metadata format per specification
- Proper Content-Type headers (application/json)
- Support for multiple authorization servers
- MCP-specific scope advertisement

### 5. OAuth Error Handling (`error.rs`)

**Single Responsibility**: OAuth 2.1 error types and RFC 6750 compliant error responses

**Core Functions**:
- Structured OAuth error types with detailed context
- RFC 6750 compliant WWW-Authenticate header generation
- HTTP status code mapping for OAuth errors
- Integration with Axum error handling patterns

**Error Types**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum OAuth2Error {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Insufficient scope: required {required}, got {provided}")]
    InsufficientScope { required: String, provided: String },
    
    #[error("JWKS error: {0}")]
    JwksError(String),
    
    #[error("Token expired: {0}")]
    TokenExpired(String),
    
    #[error("Invalid audience: expected {expected}, got {actual}")]
    InvalidAudience { expected: String, actual: String },
}
```

**HTTP Response Integration**:
- Automatic WWW-Authenticate header injection
- Proper HTTP status codes (401, 403, 400)
- RFC 6749 error response format compliance

### 6. Configuration Management (`config.rs`)

**Single Responsibility**: OAuth 2.1 configuration structures and environment loading

**Core Functions**:
- Type-safe OAuth configuration structures
- Environment variable loading with validation
- Default configuration values for development
- Configuration validation and error handling

**Key Structures**:
```rust
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    pub jwks_uri: String,
    pub issuer: String,
    pub audience: String,
    pub cache_duration: Duration,
    pub scope_mappings: HashMap<String, String>,
}

impl OAuth2Config {
    pub fn from_env() -> Result<Self, ConfigError>
    pub fn with_defaults() -> Self
    pub fn validate(&self) -> Result<(), ConfigError>
}
```

**Environment Integration**:
- Standard OAuth environment variables (OAUTH2_JWKS_URI, etc.)
- Sensible defaults for local development
- Configuration validation on startup

### 7. Authentication Context (`context.rs`)

**Single Responsibility**: Authenticated request context for MCP operations

**Core Functions**:
- Store validated user identity and permissions
- Provide convenient scope checking methods
- Extract token claims for MCP handler use
- Session correlation for audit logging

**Key Structures**:
```rust
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub scopes: HashSet<String>,
    pub token_claims: TokenClaims,
    pub session_id: Option<String>,
}

impl AuthContext {
    pub fn has_scope(&self, scope: &str) -> bool
    pub fn can_execute_mcp_method(&self, method: &str) -> bool
    pub fn user_id(&self) -> &str
    pub fn session_id(&self) -> Option<&str>
}
```

**Integration Pattern**:
- Injected into Axum request extensions by OAuth middleware
- Available to all MCP handlers for authorization decisions
- Immutable context ensuring security boundary integrity

## HTTP Transport Integration

### Server Integration Pattern (`transport/http/server.rs`)

**Integration Philosophy**: OAuth middleware wraps existing transport without modifications

```rust
impl HttpServerTransport {
    fn create_router_with_oauth(&self, oauth_config: OAuth2Config) -> Router {
        Router::new()
            // Core MCP endpoints - UNCHANGED
            .route("/mcp", post(Self::handle_mcp_request))
            .route("/health", get(Self::handle_health))
            
            // OAuth 2.1 metadata endpoint (RFC 9728)
            .route("/.well-known/oauth-protected-resource", 
                   get(oauth2::metadata::handle_metadata))
            
            // Middleware stack - CLEAN SEPARATION
            .layer(oauth2::middleware::oauth2_middleware_layer(oauth_config))
            .layer(session_middleware_layer(self.transport.clone()))
            .layer(rate_limiting_middleware())
    }
    
    // OAuth-aware MCP request handler
    async fn handle_mcp_request(
        State(transport): State<Arc<HttpServerTransport>>,
        auth_context: AuthContext,  // Injected by OAuth middleware
        Json(request): Json<JsonRpcRequest>,
    ) -> Result<Json<JsonRpcResponse>, HttpError> {
        // Validate MCP method scope before processing
        if !auth_context.can_execute_mcp_method(&request.method) {
            return Err(HttpError::InsufficientScope);
        }
        
        // Process MCP request with authenticated context
        transport.process_authenticated_request(request, auth_context).await
    }
}
```

**Key Integration Points**:
- OAuth middleware layer added to existing middleware stack
- AuthContext injection via Axum request extensions
- Scope validation before MCP method execution
- RFC 9728 metadata endpoint alongside existing routes

### Client Integration Considerations

**Future Client Integration** (when implementing OAuth client):
- OAuth 2.1 Authorization Code flow with PKCE
- Resource indicators for MCP server identification
- Token management and refresh handling
- Integration with existing HttpClientTransport

## Dependencies Specification

### Required Crate Dependencies

```toml
[dependencies]
# OAuth 2.1 core functionality
oauth2 = "4.4"              # OAuth 2.1 + PKCE + Resource Indicators
jsonwebtoken = "9.3"        # JWT validation with RS256
reqwest = { version = "0.11", features = ["json"] }  # JWKS client

# Supporting dependencies
uuid = { version = "1.0", features = ["v4"] }        # State parameters
dashmap = "5.5"            # Concurrent hashmap for JWKS caching
tokio = { version = "1.0", features = ["time"] }     # Cache expiration

# Integration with existing stack (already present)
axum = "0.7"               # HTTP server framework
tower = "0.4"              # Middleware support
serde = { version = "1.0", features = ["derive"] }   # Serialization
thiserror = "1.0"          # Error handling
```

### Feature Flags

```toml
[features]
default = ["oauth2"]
oauth2 = ["dep:oauth2", "dep:jsonwebtoken", "dep:reqwest"]
oauth2-dev = ["oauth2", "test-utils"]  # Development utilities
```

## Implementation Sequence

### Phase 1: Core OAuth Infrastructure
1. **`error.rs`**: OAuth error types and HTTP response integration
2. **`config.rs`**: Configuration structures and environment loading
3. **`context.rs`**: AuthContext for authenticated requests
4. **`jwt_validator.rs`**: JWT validation with JWKS client

### Phase 2: MCP Integration
5. **`scope_validator.rs`**: MCP method to OAuth scope mapping
6. **`metadata.rs`**: RFC 9728 Protected Resource Metadata endpoint
7. **`middleware.rs`**: Axum middleware integration

### Phase 3: Transport Integration
8. **HTTP Server Integration**: OAuth middleware layer in HttpServerTransport
9. **Integration Testing**: OAuth + HTTP transport validation
10. **Documentation**: Complete implementation documentation

## Testing Strategy

### Unit Testing Pattern
Each module follows Single Responsibility testing:

```rust
// In each module file (e.g., jwt_validator.rs)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_jwt_validation_success() {
        // Test successful JWT validation
    }
    
    #[tokio::test]
    async fn test_jwt_validation_expired_token() {
        // Test expired token handling
    }
    
    // ... comprehensive module-specific tests
}
```

### Integration Testing
- OAuth middleware + HTTP transport integration
- End-to-end authentication flow testing
- MCP method authorization validation
- RFC compliance verification

## Compliance Validation

### Standards Adherence Checklist
- ✅ **RFC 9728**: Protected Resource Metadata endpoint implementation
- ✅ **RFC 7636**: PKCE support for authorization code protection
- ✅ **RFC 8707**: Resource indicators for server identification
- ✅ **RFC 6749**: OAuth 2.0 core framework compliance
- ✅ **MCP 2025-06-18**: Complete protocol integration requirements

### Security Validation
- ✅ **Token Audience Validation**: Prevent token passthrough attacks
- ✅ **Scope Enforcement**: MCP method authorization before execution
- ✅ **JWKS Key Validation**: Proper cryptographic signature verification
- ✅ **Error Information Disclosure**: No sensitive information in error responses

This architecture provides a **production-ready, standards-compliant OAuth 2.1 implementation** that integrates cleanly with existing airs-mcp transport infrastructure while maintaining our established code quality and architectural principles.

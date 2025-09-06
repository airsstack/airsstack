# OAuth2 Migration Guide: Zero-Cost Authorization Framework

This guide provides step-by-step migration instructions for updating OAuth2 implementations from the legacy patterns to the new zero-cost generic authorization framework introduced in AIRS MCP v2.0.

## Overview of Changes

The AIRS MCP OAuth2 implementation has been redesigned around three core improvements:

1. **Zero-Cost Generic Authorization**: Compile-time optimization eliminating runtime dispatch
2. **Proper Layer Separation**: HTTP authentication vs JSON-RPC method extraction vs MCP authorization
3. **JsonRpc Method Extraction**: Fixes critical bug where methods were extracted from URL paths instead of JSON payloads

## Migration Scenarios

### Scenario 1: Basic OAuth2 Server Migration

#### Before: Legacy Dynamic Dispatch Pattern

```rust
// Old pattern with runtime dispatch and incorrect method extraction
use airs_mcp::transport::http::axum::AxumHttpServer;
use airs_mcp::oauth2::middleware::OAuth2Middleware;

async fn create_oauth2_server_old() -> Result<AxumHttpServer, ServerError> {
    let oauth2_config = OAuth2Config::new()
        .client_id(env::var("OAUTH2_CLIENT_ID")?)
        .client_secret(env::var("OAUTH2_CLIENT_SECRET")?)
        .auth_url("https://auth.example.com/oauth/authorize")
        .token_url("https://auth.example.com/oauth/token")
        .scopes(vec!["mcp:*".to_string()]);

    // This used dynamic dispatch (dyn trait objects)
    let oauth2_middleware: Arc<dyn AuthenticationMiddleware> = 
        Arc::new(OAuth2Middleware::new(oauth2_config));

    let server = AxumHttpServer::new(deps).await?
        .with_middleware(oauth2_middleware);  // Runtime dispatch overhead

    Ok(server)
}
```

#### After: Zero-Cost Generic Pattern

```rust
// New pattern with compile-time optimization and correct method extraction
use airs_mcp::authorization::{AuthorizationMiddleware, JsonRpcMethodExtractor, ScopeBasedPolicy};
use airs_mcp::transport::adapters::http::auth::oauth2::OAuth2StrategyAdapter;
use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
use airs_mcp::oauth2::validator::{JwtValidator, ScopeValidator};

async fn create_oauth2_server_new() -> Result<AxumHttpServer<OAuth2StrategyAdapter<JwtValidator, ScopeValidator>>, ServerError> {
    // Create validators for zero-cost generic architecture
    let jwt_validator = JwtValidator::new(
        "https://auth.example.com/.well-known/jwks".to_string(),
        "https://auth.example.com".to_string(), // issuer
        "https://mcp.example.com".to_string(),  // audience
    ).await?;

    let scope_validator = ScopeValidator::new(vec![
        "mcp:*".to_string(),
        "mcp:tools:*".to_string(),
        "mcp:resources:*".to_string(),
    ]);

    // Create OAuth2 configuration
    let oauth2_config = OAuth2Config::new()
        .client_id(env::var("OAUTH2_CLIENT_ID")?)
        .auth_url("https://auth.example.com/oauth/authorize")
        .token_url("https://auth.example.com/oauth/token");

    // Create zero-cost OAuth2 adapter
    let oauth2_adapter = OAuth2StrategyAdapter::with_validators(
        oauth2_config,
        jwt_validator,
        scope_validator,
    );

    // Authentication configuration
    let auth_config = HttpAuthConfig {
        include_error_details: false,
        auth_realm: "MCP API".to_string(),
        request_timeout_secs: 30,
        skip_paths: vec!["/health".to_string()],
    };

    // Create server with zero-cost authentication
    // Type: AxumHttpServer<OAuth2StrategyAdapter<JwtValidator, ScopeValidator>>
    let server = AxumHttpServer::new(deps).await?
        .with_authentication(oauth2_adapter, auth_config)
        .with_scope_authorization(ScopeBasedPolicy::mcp());

    Ok(server)
}
```

### Scenario 2: Custom Authorization Policy Migration

#### Before: Custom Authorization Logic

```rust
// Old custom authorization with runtime dispatch
struct CustomOAuth2Validator {
    // Custom validation logic
}

#[async_trait]
impl AuthenticationMiddleware for CustomOAuth2Validator {
    async fn validate(&self, request: &HttpRequest) -> Result<AuthContext, AuthError> {
        // Custom OAuth2 validation with incorrect method extraction from URL
        let method = extract_method_from_url(&request.uri().path()); // BUG!
        
        // Validate token and scopes
        let token = extract_bearer_token(request)?;
        let claims = self.validate_jwt(token).await?;
        
        // Check scopes against incorrectly extracted method
        self.validate_scopes(&claims, &method)?;
        
        Ok(AuthContext::OAuth2(claims))
    }
}
```

#### After: Zero-Cost Custom Authorization

```rust
// New zero-cost custom authorization with proper method extraction
use airs_mcp::authorization::{
    AuthorizationPolicy, AuthContext, MethodExtractor, ScopeAuthContext
};
use airs_mcp::shared::jsonrpc::JsonRpcRequest;

// Custom policy implementing AuthorizationPolicy trait
pub struct CustomMcpPolicy {
    admin_users: HashSet<String>,
    restricted_methods: HashMap<String, Vec<String>>,
}

impl AuthorizationPolicy<ScopeAuthContext> for CustomMcpPolicy {
    type Error = AuthorizationError;

    async fn authorize<R>(
        &self,
        context: &ScopeAuthContext,
        request: &R,
        method: Option<&str>,
    ) -> Result<(), Self::Error>
    where
        R: Send + Sync,
    {
        // Method correctly extracted from JSON-RPC payload by JsonRpcMethodExtractor
        let method = method.ok_or(AuthorizationError::MissingMethod)?;
        
        // Custom authorization logic
        match method {
            "tools/call" => {
                if self.is_restricted_tool(method, context) {
                    return Err(AuthorizationError::InsufficientPermissions(
                        format!("Restricted tool access denied for user {}", context.subject())
                    ));
                }
            }
            "admin/shutdown" => {
                if !self.admin_users.contains(context.subject()) {
                    return Err(AuthorizationError::AdminRequired);
                }
            }
            _ => {
                // Default scope validation
                if !context.has_scope("mcp:*") && !context.has_scope(&format!("mcp:{}:*", method)) {
                    return Err(AuthorizationError::InsufficientScope(
                        format!("Required scope for {}: mcp:* or mcp:{}:*", method, method)
                    ));
                }
            }
        }
        
        Ok(())
    }
}

// Usage with zero-cost architecture
async fn create_custom_oauth2_server() -> Result<impl ServerTrait, ServerError> {
    let oauth2_adapter = create_oauth2_adapter().await?;
    
    let custom_policy = CustomMcpPolicy {
        admin_users: ["admin@example.com".to_string()].iter().cloned().collect(),
        restricted_methods: [
            ("dangerous_tool".to_string(), vec!["admin".to_string()]),
        ].into_iter().collect(),
    };

    let server = AxumHttpServer::new(deps).await?
        .with_authentication(oauth2_adapter, HttpAuthConfig::default())
        .with_authorization(custom_policy, JsonRpcMethodExtractor::new());

    Ok(server)
}
```

## Common Migration Issues and Solutions

### Issue 1: Method Extraction Bug

**Problem**: Scope validation fails with "mcp:mcp:*" required instead of "mcp:*"

**Cause**: Old implementation extracted method from URL path (`/mcp`) instead of JSON-RPC payload

**Solution**: Use `JsonRpcMethodExtractor` in authorization middleware

```rust
// ❌ Wrong: Method extracted from URL path
let method = request.uri().path().strip_prefix('/').unwrap_or("unknown");

// ✅ Correct: Method extracted from JSON-RPC payload
let auth_middleware = AuthorizationMiddleware::new(
    oauth2_adapter,
    ScopeBasedPolicy::mcp(),
    JsonRpcMethodExtractor::new(), // Correctly extracts from JSON payload
);
```

### Issue 2: Runtime Dispatch Overhead

**Problem**: Authentication performance degradation due to `dyn` trait objects

**Cause**: Dynamic dispatch through trait objects instead of compile-time generics

**Solution**: Use zero-cost generic server types

```rust
// ❌ Wrong: Dynamic dispatch with runtime overhead
let server: AxumHttpServer<Arc<dyn AuthenticationStrategy>> = ...;

// ✅ Correct: Zero-cost generics with compile-time specialization
let server: AxumHttpServer<OAuth2StrategyAdapter<JwtValidator, ScopeValidator>> = ...;
```

### Issue 3: Incorrect Type Safety

**Problem**: Authentication configuration errors only discovered at runtime

**Cause**: Loss of type information through dynamic dispatch

**Solution**: Leverage compile-time type checking with generics

```rust
// ❌ Wrong: Type errors only discovered at runtime
fn configure_server(middleware: Arc<dyn AuthenticationStrategy>) -> Result<(), Error> {
    // Errors only appear when requests are processed
}

// ✅ Correct: Type errors caught at compile time
fn configure_server<A, P, C>(
    adapter: A,
    policy: P,
    extractor: C,
) -> Result<AxumHttpServer<A>, Error>
where
    A: AuthenticationStrategy + Send + Sync + 'static,
    P: AuthorizationPolicy<ScopeAuthContext> + Send + Sync + 'static,
    C: MethodExtractor + Send + Sync + 'static,
{
    // Compile-time verification of compatibility
}
```

### Issue 4: Missing Authorization Layer Separation

**Problem**: Authentication and authorization mixed in single middleware

**Cause**: Lack of proper layer separation in architecture

**Solution**: Separate authentication (who) from authorization (what)

```rust
// ❌ Wrong: Mixed authentication and authorization
impl OAuth2Middleware {
    async fn process(&self, request: &HttpRequest) -> Result<(), AuthError> {
        let token = self.validate_token(request).await?; // Authentication
        let method = extract_method_from_url(request);   // Wrong layer!
        self.check_scope(&token, &method).await?;       // Authorization
        Ok(())
    }
}

// ✅ Correct: Layered architecture
// 1. HTTP Layer: OAuth2StrategyAdapter (authentication only)
let oauth2_adapter = OAuth2StrategyAdapter::new(config);

// 2. Authorization Layer: AuthorizationMiddleware (authorization only)
let auth_middleware = AuthorizationMiddleware::new(
    oauth2_adapter,                // Who are you? (authentication)
    ScopeBasedPolicy::mcp(),       // What can you do? (authorization)
    JsonRpcMethodExtractor::new(), // Method extraction from correct layer
);
```

## Migration Checklist

### Pre-Migration Assessment

- [ ] Identify current OAuth2 implementation pattern
- [ ] Document existing authentication flows and configurations
- [ ] Review custom authorization logic and scope requirements
- [ ] Plan testing approach for validating migration

### Code Migration Steps

1. **Update Dependencies**
   - [ ] Update to latest AIRS MCP version with zero-cost authorization
   - [ ] Review new trait definitions and interfaces

2. **Replace Authentication Middleware**
   - [ ] Replace `OAuth2Middleware` with `OAuth2StrategyAdapter`
   - [ ] Configure `JwtValidator` and `ScopeValidator` generics
   - [ ] Update server creation to use builder pattern

3. **Implement Proper Method Extraction**
   - [ ] Replace URL path extraction with `JsonRpcMethodExtractor`
   - [ ] Verify JSON-RPC payload parsing works correctly
   - [ ] Update scope mappings to use correct method names

4. **Update Authorization Logic**
   - [ ] Implement `AuthorizationPolicy` trait for custom logic
   - [ ] Separate authentication concerns from authorization concerns
   - [ ] Test scope validation with real MCP methods

### Post-Migration Testing

- [ ] **Unit Tests**: Verify individual components work with new architecture
- [ ] **Integration Tests**: Test complete OAuth2 flow with MCP Inspector
- [ ] **Performance Tests**: Confirm zero-cost abstraction benefits
- [ ] **Scope Validation**: Verify correct scopes required for each method
- [ ] **Error Handling**: Test authentication failure scenarios

## Benefits After Migration

### Performance Improvements

- **Zero Runtime Dispatch**: All authentication calls inlined by compiler
- **Compile-Time Optimization**: Dead code elimination for unused auth paths
- **Memory Efficiency**: Stack allocation instead of heap allocation for auth contexts
- **CPU Cache Friendly**: Direct method calls improve cache locality

### Architecture Improvements

- **Type Safety**: Authentication configuration errors caught at compile time
- **Layer Separation**: Clear boundaries between HTTP, JSON-RPC, and MCP layers
- **Correct Method Extraction**: OAuth2 scopes validated against actual MCP methods
- **Framework Agnostic**: Authorization logic works with any authentication strategy

### Developer Experience

- **Builder Pattern**: Ergonomic server configuration with method chaining
- **Clear Error Messages**: Compile-time errors provide better debugging information
- **Better Testing**: Mockable interfaces with proper separation of concerns
- **Documentation**: Comprehensive guides and examples for all patterns

## Need Help?

If you encounter issues during migration:

1. **Check the troubleshooting section** in the OAuth2 authentication guide
2. **Review integration tests** for working examples of new patterns
3. **Examine the example servers** for complete implementation references
4. **Consult the authorization module documentation** for detailed API information

The migration to zero-cost OAuth2 authorization provides significant performance and architecture benefits while maintaining full compatibility with the MCP protocol specification.

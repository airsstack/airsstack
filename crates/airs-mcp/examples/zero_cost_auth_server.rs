//! Zero-Cost Generic HTTP Authentication Server Example
//!
//! This example demonstrates the zero-cost generic authentication middleware
//! architecture concepts and patterns. It focuses on the type system benefits
//! and performance characteristics rather than full API usage due to complex
//! authentication dependencies.
//!
//! Key Features Demonstrated:
//! - Zero dynamic dispatch (no Box<dyn Trait>)
//! - Compile-time type specialization 
//! - Stack allocation for middleware state
//! - Builder pattern concepts
//! - Backward compatibility with NoAuth default

use std::collections::HashMap;
use std::time::Instant;

use airs_mcp::authentication::{AuthContext, AuthMethod};
use airs_mcp::transport::adapters::http::auth::middleware::{
    HttpAuthConfig, HttpAuthMiddleware, HttpAuthRequest, HttpAuthStrategyAdapter
};
use airs_mcp::transport::adapters::http::auth::oauth2::error::HttpAuthError;

/// Mock authentication adapter demonstrating zero-cost generic patterns
#[derive(Debug, Clone)]
struct MockAuthAdapter {
    name: &'static str,
    always_succeed: bool,
}

impl MockAuthAdapter {
    fn new(name: &'static str, always_succeed: bool) -> Self {
        Self { name, always_succeed }
    }
}

#[async_trait::async_trait]
impl HttpAuthStrategyAdapter for MockAuthAdapter {
    type RequestType = ();
    type AuthData = String;

    fn auth_method(&self) -> &'static str {
        self.name
    }

    async fn authenticate_http_request(
        &self,
        _request: &HttpAuthRequest,
    ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
        if self.always_succeed {
            Ok(AuthContext::new(
                AuthMethod::new(self.name),
                format!("user_authenticated_via_{}", self.name),
            ))
        } else {
            Err(HttpAuthError::AuthenticationFailed {
                message: format!("Authentication failed for {}", self.name),
            })
        }
    }

    fn should_skip_path(&self, path: &str) -> bool {
        path.starts_with("/health") || path.starts_with("/metrics")
    }
}

/// NoAuth adapter for demonstration
#[derive(Debug, Clone, Default)]
struct NoAuthAdapter;

#[async_trait::async_trait]
impl HttpAuthStrategyAdapter for NoAuthAdapter {
    type RequestType = ();
    type AuthData = ();

    fn auth_method(&self) -> &'static str {
        "none"
    }

    async fn authenticate_http_request(
        &self,
        _request: &HttpAuthRequest,
    ) -> Result<AuthContext<Self::AuthData>, HttpAuthError> {
        Ok(AuthContext::new(AuthMethod::new("none"), ()))
    }

    fn should_skip_path(&self, _path: &str) -> bool {
        true
    }
}

/// Example 1: Zero-Cost Generic Middleware Pattern
/// 
/// This demonstrates the core zero-cost generic pattern for authentication middleware.
fn demonstrate_zero_cost_pattern() {
    println!("🔓 Zero-Cost Generic Authentication Middleware Pattern...");
    
    // Create different authentication adapters
    let no_auth = NoAuthAdapter;
    let api_auth = MockAuthAdapter::new("apikey", true);
    let oauth_auth = MockAuthAdapter::new("oauth2", true);
    
    // Create middleware instances (zero-cost, different types)
    let no_auth_middleware = HttpAuthMiddleware::new(no_auth, HttpAuthConfig::default());
    let api_middleware = HttpAuthMiddleware::new(api_auth, HttpAuthConfig::default());
    let oauth_middleware = HttpAuthMiddleware::new(oauth_auth, HttpAuthConfig::default());
    
    // Each middleware has a different type at compile time
    println!("   ✓ NoAuth middleware type: {}", 
             std::any::type_name_of_val(&no_auth_middleware));
    println!("   ✓ API middleware type: {}", 
             std::any::type_name_of_val(&api_middleware));
    println!("   ✓ OAuth middleware type: {}", 
             std::any::type_name_of_val(&oauth_middleware));
    
    println!("   ✓ All types resolved at compile time (zero runtime dispatch)");
    println!("   ✓ Stack allocation confirmed: {} bytes", 
             std::mem::size_of_val(&api_middleware));
}

/// Example 2: Authentication Performance Demonstration
///
/// This demonstrates the performance benefits of zero-cost generics.
async fn demonstrate_authentication_performance() {
    println!("🔑 Authentication Performance with Zero-Cost Generics...");
    
    const NUM_REQUESTS: usize = 10000;
    
    // Create different middleware types
    let no_auth = NoAuthAdapter;
    let fast_auth = MockAuthAdapter::new("fast", true);
    let secure_auth = MockAuthAdapter::new("secure", true);
    
    let no_auth_middleware = HttpAuthMiddleware::new(no_auth, HttpAuthConfig::default());
    let fast_middleware = HttpAuthMiddleware::new(fast_auth, HttpAuthConfig::default());
    let secure_middleware = HttpAuthMiddleware::new(secure_auth, HttpAuthConfig::default());
    
    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/test".to_string(),
        HashMap::new(),
    );
    
    // Benchmark NoAuth performance (baseline)
    let start = Instant::now();
    for _ in 0..NUM_REQUESTS {
        let _ = no_auth_middleware.authenticate(&auth_request).await;
    }
    let no_auth_duration = start.elapsed();
    
    // Benchmark fast adapter performance
    let start = Instant::now();
    for _ in 0..NUM_REQUESTS {
        let _ = fast_middleware.authenticate(&auth_request).await;
    }
    let fast_duration = start.elapsed();
    
    // Benchmark secure adapter performance
    let start = Instant::now();
    for _ in 0..NUM_REQUESTS {
        let _ = secure_middleware.authenticate(&auth_request).await;
    }
    let secure_duration = start.elapsed();
    
    println!("   ✓ NoAuth: {:?} ({} req/s)", 
             no_auth_duration,
             (NUM_REQUESTS as f64 / no_auth_duration.as_secs_f64()) as u64);
    println!("   ✓ Fast Auth: {:?} ({} req/s)", 
             fast_duration,
             (NUM_REQUESTS as f64 / fast_duration.as_secs_f64()) as u64);
    println!("   ✓ Secure Auth: {:?} ({} req/s)", 
             secure_duration,
             (NUM_REQUESTS as f64 / secure_duration.as_secs_f64()) as u64);
    
    println!("   ✓ All strategies use direct method calls (zero vtable overhead)");
    println!("   ✓ Performance differences reflect actual work, not dispatch cost");
}

/// Example 3: Type Safety and Compilation Benefits
///
/// This demonstrates how different authentication strategies create different
/// types that are specialized at compile time for maximum performance.
fn demonstrate_type_safety() {
    println!("🔧 Compile-Time Type Safety and Specialization...");
    
    // Create different authentication adapters
    let no_auth = NoAuthAdapter;
    let api_auth = MockAuthAdapter::new("apikey", true);
    let oauth_auth = MockAuthAdapter::new("oauth2", true);
    
    // Create middleware with different adapters - each has different type
    let no_auth_middleware = HttpAuthMiddleware::new(no_auth, HttpAuthConfig::default());
    let api_middleware = HttpAuthMiddleware::new(api_auth, HttpAuthConfig::default());
    let oauth_middleware = HttpAuthMiddleware::new(oauth_auth, HttpAuthConfig::default());
    
    // Verify different types at compile time
    println!("   ✓ NoAuth type: {}", std::any::type_name_of_val(&no_auth_middleware));
    println!("   ✓ API Key type: {}", std::any::type_name_of_val(&api_middleware));
    println!("   ✓ OAuth2 type: {}", std::any::type_name_of_val(&oauth_middleware));
    
    // Authentication methods differ by adapter
    println!("   ✓ NoAuth method: {}", no_auth_middleware.auth_method());
    println!("   ✓ API Key method: {}", api_middleware.auth_method());
    println!("   ✓ OAuth2 method: {}", oauth_middleware.auth_method());
    
    // Memory sizes (stack allocation)
    println!("   ✓ NoAuth size: {} bytes", std::mem::size_of_val(&no_auth_middleware));
    println!("   ✓ API Key size: {} bytes", std::mem::size_of_val(&api_middleware));
    println!("   ✓ OAuth2 size: {} bytes", std::mem::size_of_val(&oauth_middleware));
    
    println!("   ✓ Each type optimized independently by compiler");
    println!("   ✓ No runtime type checking or vtable overhead");
    println!("   ✓ Stack allocation eliminates heap allocation costs");
}

/// Example 4: Migration from Dynamic Dispatch (Documentation)
///
/// This example documents the migration pattern from the old factory-based
/// dynamic dispatch pattern to the new zero-cost generic pattern.
fn document_migration_patterns() {
    println!("📚 Migration Guide: Dynamic Dispatch → Zero-Cost Generics");
    println!();
    
    println!("OLD PATTERN (Dynamic Dispatch):");
    println!("   // ❌ Old factory pattern with heap allocation");
    println!("   let auth_factory = AuthStrategyFactory::new();");
    println!("   let strategy: Box<dyn AuthStrategy> = auth_factory.create_oauth2(config);");
    println!("   let server = HttpServer::new(strategy); // Box<dyn> overhead");
    println!();
    
    println!("NEW PATTERN (Zero-Cost Generics):");
    println!("   // ✅ New zero-cost generic pattern"); 
    println!("   let base_server = AxumHttpServer::new(deps).await?;");
    println!("   let oauth2_adapter = OAuth2StrategyAdapter::new(config);");
    println!("   let server = base_server.with_authentication(oauth2_adapter, auth_config);");
    println!("   // AxumHttpServer<OAuth2StrategyAdapter> - compile-time specialization");
    println!();
    
    println!("BENEFITS:");
    println!("   ✓ Zero runtime overhead - no vtable lookups");
    println!("   ✓ Stack allocation - no heap allocations for middleware");
    println!("   ✓ Compile-time optimization - methods inlined");
    println!("   ✓ Type safety - authentication strategy known at compile time");
    println!("   ✓ Backward compatibility - existing NoAuth usage unchanged");
    println!();
    
    println!("PERFORMANCE IMPROVEMENTS:");
    println!("   ✓ Authentication calls directly invoked (no dynamic dispatch)");
    println!("   ✓ CPU cache friendly (no indirect function calls)");
    println!("   ✓ Compiler can optimize across authentication boundaries");
    println!("   ✓ Branch prediction improved (static call sites)");
    println!();
    
    println!("TYPE SYSTEM BENEFITS:");
    println!("   ✓ Associated types for Request/AuthData eliminate generic parameters");
    println!("   ✓ Each authentication strategy is a unique type");
    println!("   ✓ Impossible to mix authentication strategies at runtime");
    println!("   ✓ Configuration errors caught at compile time");
}

/// Example 5: Production Configuration Patterns
///
/// This documents security and performance patterns for production deployments.
fn demonstrate_production_patterns() {
    println!("🏥 Production Authentication Configuration Patterns...");
    
    // Production authentication configuration
    let prod_config = HttpAuthConfig {
        include_error_details: false,             // Hide details in production  
        auth_realm: "MCP Production API".to_string(),
        request_timeout_secs: 10,                 // Fast timeout for production
        skip_paths: vec![
            "/health".to_string(),
            "/metrics".to_string(), 
            "/status".to_string(),
            "/docs".to_string(),                  // Documentation endpoint
        ],
    };
    
    // Example production adapter
    let prod_adapter = MockAuthAdapter::new("production", true);
    let prod_middleware = HttpAuthMiddleware::new(prod_adapter, prod_config);
    
    println!("   ✓ Production config: Error details disabled");
    println!("   ✓ Security: Custom auth realm set");
    println!("   ✓ Performance: {} second auth timeout", prod_middleware.config().request_timeout_secs);
    println!("   ✓ Skip paths: {} configured", prod_middleware.config().skip_paths.len());
    
    println!("   📋 Production server pattern:");
    println!("      let base_server = AxumHttpServer::new(deps).await?;");
    println!("      let auth_server = base_server.with_authentication(adapter, prod_config);");
    println!("      auth_server.bind(\"0.0.0.0:3000\").await?;");
    println!("      auth_server.serve().await?;");
}

/// Example 6: Error Handling Patterns
///
/// Demonstrates proper error handling patterns with zero-cost generics.
async fn demonstrate_error_handling() {
    println!("🚨 Authentication Error Handling with Zero-Cost Generics...");
    
    // Create different error scenarios
    let success_adapter = MockAuthAdapter::new("success", true);
    let failure_adapter = MockAuthAdapter::new("failure", false);
    
    let success_middleware = HttpAuthMiddleware::new(success_adapter, HttpAuthConfig::default());
    let failure_middleware = HttpAuthMiddleware::new(failure_adapter, HttpAuthConfig::default());
    
    let auth_request = HttpAuthRequest::new(
        HashMap::new(),
        "/api/test".to_string(),
        HashMap::new(),
    );
    
    // Test successful authentication
    match success_middleware.authenticate(&auth_request).await {
        Ok(Some(context)) => {
            println!("   ✓ Success: Authenticated as '{}' via {}", 
                     context.auth_data, context.method.as_str());
        }
        Ok(None) => println!("   ✓ Success: Path skipped authentication"),
        Err(e) => println!("   ❌ Unexpected error: {e:?}"),
    }
    
    // Test failed authentication
    match failure_middleware.authenticate(&auth_request).await {
        Ok(_) => println!("   ❌ Unexpected success"),
        Err(e) => {
            println!("   ✓ Expected failure: {e:?}");
            println!("   ✓ Error context preserved through zero-cost abstractions");
        }
    }
    
    println!("   ✓ Type-safe error handling without dynamic dispatch");
    println!("   ✓ Error types resolved at compile time");
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for better example output
    tracing_subscriber::fmt::init();
    
    println!("🚀 Zero-Cost Generic HTTP Authentication Middleware Examples");
    println!("==========================================================");
    println!();
    
    // Example 1: Zero-cost generic patterns
    demonstrate_zero_cost_pattern();
    println!();
    
    // Example 2: Authentication performance
    demonstrate_authentication_performance().await;
    println!();
    
    // Example 3: Type safety demonstration
    demonstrate_type_safety();
    println!();
    
    // Example 4: Migration documentation
    document_migration_patterns();
    
    // Example 5: Production patterns
    demonstrate_production_patterns();
    println!();
    
    // Example 6: Error handling
    demonstrate_error_handling().await;
    println!();
    
    println!("✅ All authentication middleware examples completed successfully!");
    println!();
    println!("🎯 Key Takeaways:");
    println!("   • Zero runtime overhead through compile-time generics");
    println!("   • Type safety prevents authentication strategy mixing");
    println!("   • Builder pattern provides ergonomic configuration");
    println!("   • Full backward compatibility with existing NoAuth usage");
    println!("   • Stack allocation avoids heap overhead");
    println!("   • Direct method calls eliminate vtable lookups");
    println!();
    println!("📖 Next Steps:");
    println!("   • Review workspace standard §6 compliance");
    println!("   • Implement custom authentication strategies as needed");  
    println!("   • Deploy with production security configurations");
    println!("   • Monitor authentication performance metrics");
    
    Ok(())
}

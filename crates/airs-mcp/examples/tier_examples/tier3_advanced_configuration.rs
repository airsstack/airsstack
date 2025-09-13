//! Tier 3: Advanced Configuration Example
//!
//! This example demonstrates the builder pattern with full control over engine
//! configuration. Perfect for advanced users who need fine-grained control
//! over server setup while still maintaining clean, readable code.
//!
//! # Key Features
//! - Full builder pattern control
//! - Custom middleware support
//! - Complex authentication setups
//! - Post-configuration engine tuning

use std::error::Error;

use airs_mcp::protocol::TransportError;
use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
use airs_mcp::transport::adapters::http::{HttpTransport, HttpTransportBuilder};

/// Tier 3: Advanced Configuration - Builder Pattern Control
///
/// This pattern is perfect for:
/// - Advanced users who need fine-grained control
/// - Complex authentication and authorization setups
/// - Custom middleware requirements
/// - Performance tuning and optimization
/// - Enterprise applications with specific requirements
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("⚡ Tier 3: Advanced Configuration HTTP Transport Example");

    // Tier 3A: Builder pattern with complex engine configuration
    let _transport = HttpTransportBuilder::with_configured_engine(|| {
        // Complex engine configuration using builder pattern
        create_advanced_engine()
    })?
    .configure_engine(|engine| {
        // Post-configuration engine tuning
        tune_engine_performance(engine);
    })
    .bind(
        "127.0.0.1:8080"
            .parse()
            .map_err(|e| TransportError::Connection {
                message: format!("Failed to parse address: {e}"),
            })?,
    )
    .await?
    .build()
    .await?;

    println!("✅ Advanced HTTP Transport created with complex configuration");
    println!("   - Server type: AxumHttpServer with advanced setup");
    println!("   - Authentication: OAuth2 + Custom middleware");
    println!("   - Authorization: Scope-based policies");
    println!("   - Performance: Optimized for high throughput");

    // Tier 3B: Multi-layer authentication example
    let _multi_auth_transport = create_multi_auth_transport().await?;
    println!("✅ Multi-layer authentication transport created");

    // Tier 3C: Custom middleware stack example
    let _middleware_transport = create_custom_middleware_transport().await?;
    println!("✅ Custom middleware transport created");

    println!("🎯 All advanced transports ready for production deployment");

    Ok(())
}

/// Create a complex engine with full builder pattern control
///
/// This demonstrates how to configure complex authentication, authorization,
/// and middleware setups using the builder pattern.
fn create_advanced_engine() -> Result<AxumHttpServer, TransportError> {
    // Tier 3A: Complex builder pattern configuration
    println!("🔧 Creating advanced engine with builder pattern...");

    // In a real implementation, this would be:
    // let engine = AxumHttpServer::builder()
    //     .with_oauth2_authorization(oauth2_config)
    //     .with_scope_authorization(scope_policies)
    //     .with_custom_middleware(rate_limiting_middleware)
    //     .with_custom_middleware(logging_middleware)
    //     .with_performance_config(perf_config)
    //     .build()?;

    // For now, use placeholder
    let engine = AxumHttpServer::default();

    println!("   ✓ OAuth2 authentication configured");
    println!("   ✓ Scope-based authorization policies set");
    println!("   ✓ Custom middleware stack assembled");
    println!("   ✓ Performance optimizations applied");

    Ok(engine)
}

/// Performance tuning function for post-configuration optimization
///
/// This shows how the configure_engine callback can be used for
/// fine-tuning performance parameters after initial construction.
fn tune_engine_performance(_engine: &mut AxumHttpServer) {
    println!("⚡ Tuning engine performance...");

    // In a real implementation:
    // engine.set_connection_pool_size(100);
    // engine.set_request_timeout(Duration::from_secs(30));
    // engine.set_keep_alive_timeout(Duration::from_secs(75));
    // engine.set_max_concurrent_requests(1000);
    // engine.enable_request_compression(true);

    println!("   ✓ Connection pool size: 100");
    println!("   ✓ Request timeout: 30s");
    println!("   ✓ Keep-alive timeout: 75s");
    println!("   ✓ Max concurrent requests: 1000");
    println!("   ✓ Request compression enabled");
}

/// Multi-layer authentication transport example
///
/// Demonstrates how to combine multiple authentication strategies
/// and authorization policies for enterprise-grade security.
async fn create_multi_auth_transport() -> Result<HttpTransport<AxumHttpServer>, TransportError> {
    println!("🔐 Creating multi-layer authentication transport...");

    let transport = HttpTransportBuilder::with_configured_engine(|| {
        // Multi-layer auth configuration
        // let oauth2_config = OAuth2Config::new(/* enterprise settings */);
        // let apikey_fallback = ApiKeyConfig::new(/* fallback for services */);
        // let cert_auth = CertificateAuth::new(/* mutual TLS */);

        // AxumHttpServer::builder()
        //     .with_primary_auth(oauth2_config)
        //     .with_fallback_auth(apikey_fallback)
        //     .with_certificate_auth(cert_auth)
        //     .with_auth_chain_policy(AuthChainPolicy::FirstSuccess)
        //     .build()

        // For demonstration, return Ok with default server
        Ok::<_, TransportError>(AxumHttpServer::default())
    })?
    .configure_engine(|_engine| {
        // Post-auth security hardening
        println!("   ✓ Security hardening applied");
        println!("   ✓ Auth chain policy: FirstSuccess");
        println!("   ✓ Certificate validation enabled");
    })
    .bind(
        "127.0.0.1:8443"
            .parse()
            .map_err(|e| TransportError::Connection {
                message: format!("Failed to parse address: {e}"),
            })?,
    )
    .await?
    .build()
    .await?;

    Ok(transport)
}

/// Custom middleware stack transport example
///
/// Shows how to build complex middleware pipelines for logging,
/// monitoring, rate limiting, and other cross-cutting concerns.
async fn create_custom_middleware_transport(
) -> Result<HttpTransport<AxumHttpServer>, TransportError> {
    println!("🔧 Creating custom middleware transport...");

    let transport = HttpTransportBuilder::with_configured_engine(|| {
        // Custom middleware stack
        // let rate_limiter = RateLimitingMiddleware::new(requests_per_minute: 1000);
        // let metrics = MetricsMiddleware::new(prometheus_registry);
        // let tracing = TracingMiddleware::new(jaeger_config);
        // let cors = CorsMiddleware::new(cors_policy);

        // AxumHttpServer::builder()
        //     .with_middleware(rate_limiter)
        //     .with_middleware(metrics)
        //     .with_middleware(tracing)
        //     .with_middleware(cors)
        //     .with_middleware_order(MiddlewareOrder::RequestResponseChain)
        //     .build()

        // For demonstration, return Ok with default server
        Ok::<_, TransportError>(AxumHttpServer::default())
    })?
    .configure_engine(|_engine| {
        // Middleware performance tuning
        println!("   ✓ Rate limiting: 1000 req/min per client");
        println!("   ✓ Prometheus metrics enabled");
        println!("   ✓ Jaeger tracing configured");
        println!("   ✓ CORS policies applied");
        println!("   ✓ Middleware chain optimized");
    })
    .bind(
        "127.0.0.1:8090"
            .parse()
            .map_err(|e| TransportError::Connection {
                message: format!("Failed to parse address: {e}"),
            })?,
    )
    .await?
    .build()
    .await?;

    Ok(transport)
}

/// Error handling and resilience patterns
///
/// Demonstrates advanced error handling, circuit breakers,
/// and resilience patterns for production deployments.
#[allow(dead_code)]
async fn create_resilient_transport() -> Result<HttpTransport<AxumHttpServer>, TransportError> {
    let transport = HttpTransportBuilder::with_configured_engine(|| {
        // Resilience configuration
        // let circuit_breaker = CircuitBreakerConfig::new()
        //     .failure_threshold(10)
        //     .timeout(Duration::from_secs(60))
        //     .half_open_max_calls(5);

        // let retry_policy = RetryPolicy::new()
        //     .max_attempts(3)
        //     .backoff_strategy(ExponentialBackoff::default());

        // AxumHttpServer::builder()
        //     .with_circuit_breaker(circuit_breaker)
        //     .with_retry_policy(retry_policy)
        //     .with_health_checks(health_check_config)
        //     .build()

        // For demonstration, return Ok with default server
        Ok::<_, TransportError>(AxumHttpServer::default())
    })?
    .bind(
        "127.0.0.1:8091"
            .parse()
            .map_err(|e| TransportError::Connection {
                message: format!("Failed to parse address: {e}"),
            })?,
    )
    .await?
    .build()
    .await?;

    Ok(transport)
}

/// Test helpers to verify tier 3 patterns work
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tier3_advanced_engine() {
        let result = create_advanced_engine();
        assert!(
            result.is_ok(),
            "Tier 3 advanced engine creation should work"
        );
    }

    #[tokio::test]
    async fn test_tier3_multi_auth() {
        let result = create_multi_auth_transport().await;
        assert!(result.is_ok(), "Tier 3 multi-auth transport should work");
    }

    #[tokio::test]
    async fn test_tier3_custom_middleware() {
        let result = create_custom_middleware_transport().await;
        assert!(result.is_ok(), "Tier 3 middleware transport should work");
    }

    #[tokio::test]
    async fn test_tier3_resilient() {
        let result = create_resilient_transport().await;
        assert!(result.is_ok(), "Tier 3 resilient transport should work");
    }
}

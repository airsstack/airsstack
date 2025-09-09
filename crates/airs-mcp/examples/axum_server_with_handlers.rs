//! Complete Example: AxumHttpServer MCP Handler Configuration with Zero-Cost Authentication
//!
//! This example demonstrates the different approaches to configuring
//! MCP handlers with the AxumHttpServer architecture, including the new
//! zero-cost generic authentication middleware system.

use airs_mcp::authentication::strategies::apikey::types::ApiKeySource;
use airs_mcp::authentication::strategies::apikey::{
    ApiKeyAuthData, ApiKeyStrategy, InMemoryApiKeyValidator,
};
use airs_mcp::authentication::{AuthContext, AuthMethod};
use airs_mcp::correlation::manager::{CorrelationConfig, CorrelationManager};
use airs_mcp::integration::mcp::server::McpServerConfig;
use airs_mcp::protocol::{JsonRpcRequest, JsonRpcResponse, RequestId}; // Use new protocol module
use airs_mcp::transport::adapters::http::auth::apikey::ApiKeyStrategyAdapter;
use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthConfig;
use airs_mcp::transport::adapters::http::axum::{AxumHttpServer, McpHandlers, McpHandlersBuilder};
use airs_mcp::transport::adapters::http::config::HttpTransportConfig;
use airs_mcp::transport::adapters::http::connection_manager::{
    HealthCheckConfig, HttpConnectionManager,
};
use airs_mcp::transport::adapters::http::session::{SessionConfig, SessionManager};
use airs_mcp::transport::error::TransportError;
use std::collections::HashMap;

use std::sync::Arc;

/// Create shared infrastructure components
async fn create_infrastructure() -> (
    Arc<HttpConnectionManager>,
    Arc<SessionManager>,
    Arc<ConcurrentProcessor>,
    HttpTransportConfig,
) {
    let connection_manager = Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
    let correlation_manager = Arc::new(
        CorrelationManager::new(CorrelationConfig::default())
            .await
            .unwrap(),
    );
    let session_manager = Arc::new(SessionManager::new(
        correlation_manager,
        SessionConfig::default(),
    ));
    let processor_config = ProcessorConfig {
        worker_count: 4,
        queue_capacity: 1000,
        max_batch_size: 50,
        processing_timeout: chrono::Duration::seconds(30),
        enable_ordering: false,
        enable_backpressure: true,
    };
    let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
    let config = HttpTransportConfig::new();

    (
        connection_manager,
        session_manager,
        jsonrpc_processor,
        config,
    )
}

/// Approach 1: Direct handler configuration
async fn create_server_with_direct_handlers() -> Result<AxumHttpServer, TransportError> {
    let (connection_manager, session_manager, jsonrpc_processor, config) =
        create_infrastructure().await;

    // Create MCP handlers directly (with empty providers for this demo)
    let mcp_handlers = Arc::new(McpHandlers {
        resource_provider: None, // Would be Some(Arc::new(MyResourceProvider)) in real usage
        tool_provider: None,     // Would be Some(Arc::new(MyToolProvider)) in real usage
        prompt_provider: None,   // Would be Some(Arc::new(MyPromptProvider)) in real usage
        logging_handler: None,   // Would be Some(Arc::new(MyLoggingHandler)) in real usage
        config: McpServerConfig::default(),
    });

    AxumHttpServer::new(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        mcp_handlers,
        config,
    )
    .await
}

/// Approach 2: Builder pattern (recommended)
async fn create_server_with_builder() -> Result<AxumHttpServer, TransportError> {
    let (connection_manager, session_manager, jsonrpc_processor, config) =
        create_infrastructure().await;

    // Use builder pattern for clean, fluent configuration
    let handlers_builder = McpHandlersBuilder::new()
        // .with_resource_provider(Arc::new(MyResourceProvider))
        // .with_tool_provider(Arc::new(MyToolProvider))
        // .with_prompt_provider(Arc::new(MyPromptProvider))
        // .with_logging_handler(Arc::new(MyLoggingHandler))
        .with_config(McpServerConfig::default());

    AxumHttpServer::with_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        handlers_builder,
        config,
    )
    .await
}

/// Approach 3: Empty handlers for testing/development
async fn create_server_for_testing() -> Result<AxumHttpServer, TransportError> {
    let (connection_manager, session_manager, jsonrpc_processor, config) =
        create_infrastructure().await;

    // Create server with empty handlers (returns method not found errors for MCP requests)
    AxumHttpServer::new_with_empty_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        config,
    )
    .await
}

/// Approach 4: Server with zero-cost API key authentication
async fn create_server_with_api_key_auth(
) -> Result<AxumHttpServer<ApiKeyStrategyAdapter<InMemoryApiKeyValidator>>, TransportError> {
    let (connection_manager, session_manager, jsonrpc_processor, config) =
        create_infrastructure().await;

    // Create base server first
    let base_server = AxumHttpServer::new_with_empty_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        config,
    )
    .await?;

    // Create API key authentication
    let mut api_keys = HashMap::new();
    api_keys.insert(
        "demo-key-123".to_string(),
        AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "demo_user".to_string(),
                source: ApiKeySource::AuthorizationBearer,
            },
        ),
    );
    api_keys.insert(
        "production-key-456".to_string(),
        AuthContext::new(
            AuthMethod::new("apikey"),
            ApiKeyAuthData {
                key_id: "prod_user".to_string(),
                source: ApiKeySource::AuthorizationBearer,
            },
        ),
    );
    let validator = InMemoryApiKeyValidator::new(api_keys);
    let strategy = ApiKeyStrategy::new(validator);
    let adapter = ApiKeyStrategyAdapter::new(strategy, Default::default());

    let auth_config = HttpAuthConfig {
        include_error_details: false,
        auth_realm: "MCP Demo Server".to_string(),
        request_timeout_secs: 30,
        skip_paths: vec!["/health".to_string(), "/metrics".to_string()],
    };

    // Zero-cost type conversion with builder pattern
    Ok(base_server.with_authentication(adapter, auth_config))
}

/// Approach 5: Conditional authentication based on environment
async fn create_server_with_conditional_auth() -> Result<(), TransportError> {
    let (connection_manager, session_manager, jsonrpc_processor, config) =
        create_infrastructure().await;

    // Create base server
    let base_server = AxumHttpServer::new_with_empty_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        config,
    )
    .await?;

    // Conditional authentication based on environment
    let auth_enabled =
        std::env::var("MCP_AUTH_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true";

    if auth_enabled {
        let mut api_keys = HashMap::new();
        api_keys.insert(
            "production-key".to_string(),
            AuthContext::new(
                AuthMethod::new("apikey"),
                ApiKeyAuthData {
                    key_id: "production_user".to_string(),
                    source: ApiKeySource::AuthorizationBearer,
                },
            ),
        );
        let validator = InMemoryApiKeyValidator::new(api_keys);
        let strategy = ApiKeyStrategy::new(validator);
        let adapter = ApiKeyStrategyAdapter::new(strategy, Default::default());
        let auth_config = HttpAuthConfig {
            include_error_details: false,
            auth_realm: "MCP Production".to_string(),
            request_timeout_secs: 10,
            skip_paths: vec!["/health".to_string()],
        };

        let _auth_server = base_server.with_authentication(adapter, auth_config);
        println!("   ✓ Server created with API key authentication");
    } else {
        let _no_auth_server = base_server; // Default (no authentication)
        println!("   ✓ Server created without authentication (development mode)");
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AxumHttpServer MCP Handler & Authentication Configuration Examples ===\n");

    // Demonstrate all approaches
    println!("1. Creating server with direct handler configuration...");
    let _server1 = create_server_with_direct_handlers().await?;
    println!("   ✓ Server created successfully with direct handler configuration\n");

    println!("2. Creating server with builder pattern (recommended)...");
    let _server2 = create_server_with_builder().await?;
    println!("   ✓ Server created successfully using builder pattern\n");

    println!("3. Creating server with empty handlers for testing...");
    let _server3 = create_server_for_testing().await?;
    println!("   ✓ Server created successfully with no handlers (testing mode)\n");

    println!("4. Creating server with zero-cost API key authentication...");
    let _server4 = create_server_with_api_key_auth().await?;
    println!("   ✓ Server created successfully with API key authentication\n");

    println!("5. Creating server with conditional authentication...");
    create_server_with_conditional_auth().await?;
    println!();

    println!("All server configurations completed successfully!");
    println!("\n🎯 Architecture Summary:");
    println!("1. Direct Configuration:");
    println!("   - Create McpHandlers struct directly");
    println!("   - Pass to AxumHttpServer::new()");
    println!("   - More verbose but explicit");

    println!("\n2. Builder Pattern (Recommended):");
    println!("   - Use McpHandlersBuilder for fluent configuration");
    println!("   - Chain .with_* methods for clean setup");
    println!("   - Pass builder to AxumHttpServer::with_handlers()");

    println!("\n3. Empty Handlers (Testing):");
    println!("   - Use AxumHttpServer::new_with_empty_handlers()");
    println!("   - All MCP requests return 'method not found' errors");
    println!("   - Perfect for testing HTTP server functionality");

    println!("\n🔐 Zero-Cost Authentication Features:");
    println!("4. API Key Authentication:");
    println!("   - Zero runtime dispatch overhead");
    println!("   - Different server types at compile time");
    println!("   - Stack allocation for all middleware state");
    println!("   - Type: AxumHttpServer<ApiKeyStrategyAdapter<V>>");

    println!("\n5. Conditional Authentication:");
    println!("   - Environment-based authentication selection");
    println!("   - NoAuth default preserves existing APIs");
    println!("   - Zero-cost builder pattern for type conversion");
    println!("   - Production vs development configurations");

    println!("\n🚀 Performance Benefits:");
    println!("• Zero Dynamic Dispatch: All authentication calls inlined");
    println!("• Compile-Time Optimization: Methods specialized per strategy");
    println!("• Stack Allocation: 64-88 bytes per middleware (no heap)");
    println!("• Type Safety: Authentication errors caught at compile time");
    println!("• Backward Compatibility: Existing NoAuth usage unchanged");

    println!("\n📋 Next Steps:");
    println!("- Implement actual ResourceProvider, ToolProvider, etc.");
    println!("- Choose appropriate authentication strategy for your use case");
    println!("- Configure production security settings (auth_realm, timeouts)");
    println!("- Start the server with server.bind() and server.serve()");
    println!("- See docs/src/usages/zero_cost_authentication.md for complete guide");

    Ok(())
}

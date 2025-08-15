//! Simple Example: AxumHttpServer MCP Handler Configuration
//!
//! This example demonstrates the different approaches to configuring
//! MCP handlers with the AxumHttpServer architecture.

use airs_mcp::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};
use airs_mcp::correlation::manager::{CorrelationConfig, CorrelationManager};
use airs_mcp::integration::mcp::server::McpServerConfig;
use airs_mcp::transport::error::TransportError;
use airs_mcp::transport::http::axum::{AxumHttpServer, McpHandlers, McpHandlersBuilder};
use airs_mcp::transport::http::config::HttpTransportConfig;
use airs_mcp::transport::http::connection_manager::{HealthCheckConfig, HttpConnectionManager};
use airs_mcp::transport::http::session::{SessionConfig, SessionManager};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AxumHttpServer MCP Handler Configuration Examples ===\n");

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

    println!("All server configurations completed successfully!");
    println!("\nArchitecture Summary:");
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

    println!("\nNext Steps:");
    println!("- Implement actual ResourceProvider, ToolProvider, etc.");
    println!("- Use the builder pattern to configure them");
    println!("- Start the server with server.bind() and server.serve()");

    Ok(())
}

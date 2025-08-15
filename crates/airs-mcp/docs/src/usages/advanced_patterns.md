# Advanced Patterns

*High-performance implementations and sophisticated usage patterns*

# Advanced Patterns

*High-performance implementations and sophisticated usage patterns*

## Buffer Pooling and Performance Optimization

### HTTP Buffer Pool Implementation

AIRS MCP provides a production-ready HTTP buffer pool that reduces allocation overhead:

```rust
use airs_mcp::transport::http::{BufferPool, BufferPoolConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure HTTP buffer pool
    let config = BufferPoolConfig::new()
        .max_buffers(100)              // Pool up to 100 buffers
        .buffer_size(8 * 1024)         // 8KB buffers
        .adaptive_sizing(true);        // Enable adaptive sizing
    
    let pool = BufferPool::new(config);
    
    // Get buffers with automatic pooling
    let mut buffer = pool.get_buffer();
    buffer.extend_from_slice(b"Hello, World!");
    
    // Buffer automatically returns to pool when dropped
    
    Ok(())
}
```

### Buffer Pool Metrics and Monitoring

```rust
use airs_mcp::transport::http::BufferPool;

async fn monitor_buffer_performance(pool: &BufferPool) {
    let stats = pool.stats();
    
    println!("HTTP Buffer Pool Performance:");
    println!("  Available Buffers: {}", stats.available_buffers);
    println!("  Total Buffers: {}", stats.total_buffers);
    println!("  Max Buffers: {}", stats.max_buffers);
}
```

## Streaming JSON Parsing

### Memory-Efficient JSON Parsing

AIRS MCP provides a streaming JSON parser for processing large messages efficiently:

```rust
use airs_mcp::base::jsonrpc::streaming::{StreamingParser, StreamingConfig};

async fn streaming_parser_example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure streaming parser for large messages
    let config = StreamingConfig {
        max_message_size: 16 * 1024 * 1024, // 16MB messages
        read_buffer_size: 64 * 1024,        // 64KB buffer
        strict_validation: true,            // Comprehensive validation
    };
    
    let mut parser = StreamingParser::new(config);
    
    // Parse from streaming source (e.g., network connection)
    let json_data = br#"{"jsonrpc":"2.0","method":"large_data_operation","params":{"data":"..."},"id":1}"#;
    let message = parser.parse_from_bytes(json_data).await?;
    
    println!("Parsed message: {:?}", message);
    
    // Get buffer statistics for monitoring
    let stats = parser.buffer_stats();
    println!("Buffer utilization: {:.2}%", stats.utilization() * 100.0);
    
    Ok(())
}
```

### Multi-Message Batch Processing

```rust
use airs_mcp::base::jsonrpc::streaming::StreamingParser;

async fn batch_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = StreamingParser::new_default();
    
    // Process multiple JSON-RPC messages from a single buffer
    let batch_data = br#"{"jsonrpc":"2.0","method":"ping","id":1}{"jsonrpc":"2.0","method":"status","id":2}{"jsonrpc":"2.0","method":"info","id":3}"#;
    
    let messages = parser.parse_multiple_from_bytes(batch_data).await?;
    
    println!("Processed {} messages in batch", messages.len());
    for (i, message) in messages.iter().enumerate() {
        println!("  Message {}: {:?}", i + 1, message.message_type());
    }
    
    Ok(())
}
```

## Zero-Copy Transport Optimizations

### Zero-Copy Message Processing

```rust
use airs_mcp::transport::{ZeroCopyTransport, StdioTransport};
use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest};
use bytes::BytesMut;

async fn zero_copy_example() -> Result<(), Box<dyn std::error::Error>> {
    let transport = StdioTransport::new().await?;
    
    // Zero-copy serialization directly to buffer
    let request = JsonRpcRequest::new(1, "ping".to_string(), None);
    let mut buffer = BytesMut::with_capacity(1024);
    request.serialize_to_buffer(&mut buffer)?;
    
    // Zero-copy send operation
    transport.send_bytes(&buffer).await?;
    
    // Zero-copy receive into pooled buffer
    let mut recv_buffer = transport.acquire_buffer().await?;
    let bytes_read = transport.receive_into_buffer(&mut recv_buffer).await?;
    
    // Deserialize from buffer without copying
    let received_message = JsonRpcMessage::from_bytes(&recv_buffer[..bytes_read])?;
    println!("Received: {:?}", received_message);
    
    Ok(())
}
```

## Correlation Management

### Request Correlation Implementation

AIRS MCP provides correlation management for tracking request-response pairs:

```rust
use airs_mcp::correlation::{CorrelationManager, CorrelationConfig};
use airs_mcp::base::jsonrpc::{JsonRpcRequest, RequestId};
use std::time::Duration;

async fn correlation_management_example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure correlation tracking with timeouts
    let config = CorrelationConfig {
        default_timeout: Duration::from_secs(30),
        max_pending_requests: 1000,
        cleanup_interval: Duration::from_secs(60),
    };
    
    let correlation_manager = CorrelationManager::new(config).await?;
    
    // Register outbound request
    let request = JsonRpcRequest::new(
        RequestId::Number(42), 
        "long_operation".to_string(), 
        None
    );
    let request_future = correlation_manager.register_request(request.clone()).await?;
    
    // Send request through transport (implementation specific)
    // transport.send(&request.to_json()?).await?;
    
    // Wait for correlated response with timeout
    let response = tokio::time::timeout(Duration::from_secs(10), request_future).await??;
    println!("Received correlated response: {:?}", response);
    
    Ok(())
}
```

### Correlation Cleanup and Monitoring

```rust
use airs_mcp::correlation::CorrelationManager;
use tokio::time::{interval, Duration};

async fn correlation_monitoring_task(manager: CorrelationManager) {
    let mut cleanup_interval = interval(Duration::from_secs(30));
    
    loop {
        cleanup_interval.tick().await;
        
        // Clean up expired requests
        let cleaned_count = manager.cleanup_expired().await;
        if cleaned_count > 0 {
            println!("Cleaned up {} expired correlation entries", cleaned_count);
        }
        
        // Monitor correlation health
        let stats = manager.statistics().await;
        if stats.pending_count > 500 {
            eprintln!("Warning: High number of pending correlations: {}", stats.pending_count);
        }
        
        if stats.success_rate < 0.95 {
            eprintln!("Warning: Low correlation success rate: {:.2}%", stats.success_rate * 100.0);
        }
    }
}
```

## Concurrent Request Handling

### Production-Ready Concurrent Processor

```rust
use airs_mcp::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig, MessageHandler};
use airs_mcp::base::jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, streaming::ParsedMessage};
use std::time::Duration;
use async_trait::async_trait;

#[derive(Debug)]
struct HighPerformanceHandler;

#[async_trait]
impl MessageHandler for HighPerformanceHandler {
    async fn handle_request(
        &self,
        request: &JsonRpcRequest,
    ) -> Result<JsonRpcResponse, String> {
        // Simulate CPU-intensive work
        tokio::task::yield_now().await;
        
        Ok(JsonRpcResponse::success(
            request.id.clone(),
            serde_json::json!({
                "method": request.method,
                "processed": true,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })
        ))
    }

    async fn handle_notification(
        &self,
        _notification: &JsonRpcNotification,
    ) -> Result<(), String> {
        // Handle notification
        Ok(())
    }
}

async fn concurrent_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure concurrent processor for high throughput
    let config = ProcessorConfig {
        worker_count: 8,                              // 8 worker threads
        queue_capacity: 1000,                         // Buffer 1000 requests
        processing_timeout: chrono::Duration::seconds(30),
        enable_ordering: false,
        enable_backpressure: true,
        max_batch_size: 50,
    };
    
    let mut processor = ConcurrentProcessor::new(config);
    processor.start().await?;
    processor.register_handler(HighPerformanceHandler).await?;
    
    // Process requests concurrently
    let request = JsonRpcRequest::new(
        airs_mcp::base::jsonrpc::RequestId::Number(1), 
        "compute_intensive".to_string(), 
        None
    );
    let message = ParsedMessage::Request(request);
    let result = processor.submit_message(message).await?;
    
    // Get processing statistics
    let stats = processor.stats();
    println!("Worker Pool Statistics:");
    println!("  Successful Operations: {}", stats.successful_operations);
    println!("  Failed Operations: {}", stats.failed_operations);
    
    println!("Response: {:?}", result);
    
    // Graceful shutdown
    processor.shutdown().await?;
    
    Ok(())
}
```

### Load Balancing and Backpressure

```rust
use airs_mcp::base::jsonrpc::concurrent::{ConcurrentJsonRpcProcessor, LoadBalancingStrategy};

async fn load_balancing_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = ProcessorConfig {
        worker_count: 12,
        queue_capacity: 2000,
        request_timeout: Duration::from_secs(60),
        shutdown_timeout: Duration::from_secs(10),
        backpressure_threshold: 1600,
        load_balancing: LoadBalancingStrategy::LeastLoaded, // Intelligent load balancing
    };
    
    let processor = ConcurrentJsonRpcProcessor::new(config, Box::new(HighPerformanceHandler)).await?;
    
    // Handle backpressure gracefully
    for i in 0..5000 {
        let request = JsonRpcRequest::new(i, "batch_operation".to_string(), None);
        
        match processor.try_process_request(request).await {
            Ok(future) => {
                // Request accepted, spawn task to handle response
                tokio::spawn(async move {
                    if let Ok(response) = future.await {
                        println!("Processed request {}: {:?}", i, response);
                    }
                });
            }
            Err(airs_mcp::base::jsonrpc::concurrent::BackpressureError) => {
                // Backpressure applied, wait and retry
                println!("Backpressure detected, waiting...");
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
        }
        
        // Brief pause to avoid overwhelming
        if i % 100 == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    
    Ok(())
}
```

## Custom Handler Implementations

### Real MessageRouter Implementation

```rust
use airs_mcp::integration::router::{MessageRouter, RouteConfig};
use airs_mcp::integration::handler::{RequestHandler, NotificationHandler};
use airs_mcp::base::jsonrpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, RequestId};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug)]
struct DatabaseHandler {
    connections: tokio::sync::RwLock<HashMap<String, DatabaseConnection>>,
}

#[async_trait]
impl RequestHandler for DatabaseHandler {
    async fn handle_request(
        &self,
        request: &JsonRpcRequest,
    ) -> Result<serde_json::Value, crate::integration::error::IntegrationError> {
        match request.method.as_str() {
            "db/query" => self.handle_query(&request.params).await,
            "db/transaction" => self.handle_transaction(&request.params).await,
            "db/migrate" => self.handle_migration(&request.params).await,
            _ => Err(crate::integration::error::IntegrationError::routing(format!(
                "Method '{}' not supported by DatabaseHandler", request.method
            )))
        }
    }
}

#[async_trait]
impl NotificationHandler for DatabaseHandler {
    async fn handle_notification(
        &self,
        notification: &JsonRpcNotification,
    ) -> Result<(), crate::integration::error::IntegrationError> {
        match notification.method.as_str() {
            "db/status" => {
                println!("Database status update: {:?}", notification.params);
                Ok(())
            }
            _ => Ok(()) // Ignore unknown notifications
        }
    }
}

impl DatabaseHandler {
    async fn handle_query(&self, params: &Option<serde_json::Value>) -> Result<serde_json::Value, crate::integration::error::IntegrationError> {
        // Complex query processing with connection pooling
        let query = params
            .as_ref()
            .and_then(|p| p.get("query"))
            .and_then(|q| q.as_str())
            .ok_or_else(|| crate::integration::error::IntegrationError::invalid_params(
                "Query parameter required".to_string(),
            ))?;
        
        // Execute query with proper connection management
        Ok(serde_json::json!({
            "query": query,
            "results": [],
            "execution_time_ms": 42
        }))
    }
    
    async fn handle_transaction(&self, _params: &Option<serde_json::Value>) -> Result<serde_json::Value, crate::integration::error::IntegrationError> {
        // Transaction handling with ACID guarantees
        Ok(serde_json::json!({
            "transaction_id": "txn_123456",
            "status": "committed"
        }))
    }
    
    async fn handle_migration(&self, _params: &Option<serde_json::Value>) -> Result<serde_json::Value, crate::integration::error::IntegrationError> {
        // Database migration with rollback support
        Ok(serde_json::json!({
            "migration": "v1.0_to_v1.1",
            "status": "completed",
            "affected_tables": ["users", "sessions"]
        }))
    }
}

// Placeholder for database connection
struct DatabaseConnection;

async fn message_routing_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = RouteConfig::default();
    let mut router = MessageRouter::new(config);
    
    // Register database handler for specific methods
    let db_handler = Arc::new(DatabaseHandler {
        connections: tokio::sync::RwLock::new(HashMap::new()),
    });
    
    router.register_request_handler("db/query", db_handler.clone())?;
    router.register_request_handler("db/transaction", db_handler.clone())?;
    router.register_request_handler("db/migrate", db_handler.clone())?;
    router.register_notification_handler("db/status", db_handler)?;
    
    // Route a request
    let request = JsonRpcRequest::new(
        "db/query",
        Some(serde_json::json!({
            "query": "SELECT * FROM users WHERE active = true"
        })),
        RequestId::new_number(1)
    );
    
    let response = router.route_request(&request).await?;
    println!("Routed response: {:?}", response);
    
    // Route a notification
    let notification = JsonRpcNotification::new(
        "db/status",
        Some(serde_json::json!({"status": "healthy"}))
    );
    
    router.route_notification(&notification).await?;
    
    Ok(())
}
```

## Streaming Transport Integration

### Real Streaming Transport Implementation

```rust
use airs_mcp::transport::streaming::StreamingTransport;
use airs_mcp::transport::stdio::StdioTransport;
use airs_mcp::base::jsonrpc::streaming::{StreamingConfig, ParsedMessage};
use airs_mcp::base::jsonrpc::{JsonRpcRequest, RequestId};

async fn streaming_transport_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create base transport
    let base_transport = StdioTransport::new().await?;
    
    // Configure streaming capabilities
    let streaming_config = StreamingConfig {
        max_message_size: 32 * 1024 * 1024,  // 32MB messages
        read_buffer_size: 256 * 1024,        // 256KB streaming buffer
        strict_validation: true,
    };
    
    // Create streaming transport wrapper
    let streaming_transport = StreamingTransport::new(base_transport, streaming_config);
    
    // Parse JSON message from bytes using streaming parser
    let json_data = br#"{"jsonrpc":"2.0","method":"test","id":"stream-1"}"#;
    let parsed_message = streaming_transport.parse_message(json_data).await?;
    
    match parsed_message {
        ParsedMessage::Request(request) => {
            println!("Parsed request: method={}, id={:?}", request.method, request.id);
        }
        ParsedMessage::Response(response) => {
            println!("Parsed response: id={:?}", response.id);
        }
        ParsedMessage::Notification(notification) => {
            println!("Parsed notification: method={}", notification.method);
        }
    }
    
    // Parse multiple messages from a single buffer
    let multi_json = br#"{"jsonrpc":"2.0","method":"ping","id":1}{"jsonrpc":"2.0","method":"pong","id":2}"#;
    let messages = streaming_transport.parse_multiple_messages(multi_json).await?;
    
    println!("Parsed {} messages from buffer", messages.len());
    for (i, message) in messages.iter().enumerate() {
        if let Some(method) = message.method() {
            println!("  Message {}: {}", i + 1, method);
        }
    }
    
    // Use streaming receive with automatic parsing
    println!("Waiting for streaming message...");
    let received_message = streaming_transport.receive_parsed().await?;
    println!("Received and parsed: {:?}", received_message);
    
    Ok(())
}

// Create a large request demonstrating streaming capabilities
fn create_large_request() -> ParsedMessage {
    let large_payload = serde_json::json!({
        "data": vec![0u8; 1024 * 1024], // 1MB payload
        "metadata": {
            "size": 1048576,
            "encoding": "binary"
        }
    });
    
    let request = JsonRpcRequest::new(
        "process_large_data",
        Some(large_payload),
        RequestId::new_string("large-req-1")
    );
    
    ParsedMessage::Request(request)
}
```

## Performance Monitoring and Benchmarking

### Real-time Performance Metrics

```rust
use airs_mcp::transport::buffer::BufferManager;
use airs_mcp::correlation::CorrelationManager;
use tokio::time::{interval, Duration};

async fn performance_monitoring_dashboard(
    buffer_manager: &BufferManager,
    correlation_manager: &CorrelationManager,
) {
    let mut interval = interval(Duration::from_secs(5));
    
    loop {
        interval.tick().await;
        
        // Buffer performance metrics
        let buffer_metrics = buffer_manager.metrics();
        
        // Correlation performance metrics
        let correlation_stats = correlation_manager.statistics().await;
        
        println!("\n=== Performance Dashboard ===");
        
        // Buffer Pool Health
        println!("Buffer Pool:");
        println!("  Buffer Hits: {}", 
                 buffer_metrics.buffer_hits.load(std::sync::atomic::Ordering::Relaxed));
        println!("  Buffer Misses: {}", 
                 buffer_metrics.buffer_misses.load(std::sync::atomic::Ordering::Relaxed));
        println!("  Pool Size: {}", 
                 buffer_metrics.pool_size.load(std::sync::atomic::Ordering::Relaxed));
        
        // Correlation Health  
        println!("Request Correlation:");
        println!("  Active Requests: {}", correlation_stats.active_requests);
        println!("  Completed Requests: {}", correlation_stats.completed_requests);
        println!("  Failed Requests: {}", correlation_stats.failed_requests);
        println!("  Average Response Time: {:?}", correlation_stats.average_response_time);
        
        // Performance alerts
        let total_ops = buffer_metrics.buffer_hits.load(std::sync::atomic::Ordering::Relaxed) +
                        buffer_metrics.buffer_misses.load(std::sync::atomic::Ordering::Relaxed);
        if total_ops > 0 {
            let hit_ratio = buffer_metrics.buffer_hits.load(std::sync::atomic::Ordering::Relaxed) as f64 / total_ops as f64;
            if hit_ratio < 0.8 {
                eprintln!("⚠️  WARNING: Low buffer pool hit ratio: {:.2}%", hit_ratio * 100.0);
            }
        }
        
        if correlation_stats.failed_requests > correlation_stats.completed_requests / 20 {
            eprintln!("⚠️  WARNING: High failure rate in request correlation!");
        }
        
        if correlation_stats.active_requests > 1000 {
            eprintln!("⚠️  WARNING: High number of pending requests: {}", correlation_stats.active_requests);
        }
    }
}
```

### Benchmarking Integration

### Axum Handler Configuration Patterns

#### Real AxumHttpServer Implementation

```rust
use airs_mcp::transport::http::axum::{AxumHttpServer, McpHandlers, McpHandlersBuilder};
use airs_mcp::integration::mcp::server::McpServerConfig;
use airs_mcp::transport::error::TransportError;
use airs_mcp::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};
use airs_mcp::correlation::manager::{CorrelationConfig, CorrelationManager};
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

    // Create MCP handlers directly
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
```

## MCP Handler Configuration Patterns

### Production-Ready Handler Setup

AIRS MCP provides sophisticated handler configuration patterns for production deployments. See [Handler Configuration Architecture](../architecture/handler_configuration.md) for detailed architectural information.

```rust
use airs_mcp::transport::http::axum::{AxumHttpServer, McpHandlersBuilder};
use airs_mcp::integration::mcp::{ResourceProvider, ToolProvider, PromptProvider, LoggingHandler};
use std::sync::Arc;

async fn production_server_setup() -> Result<(), Box<dyn std::error::Error>> {
    // Create infrastructure components
    let (connection_manager, session_manager, jsonrpc_processor, config) = 
        create_infrastructure_components().await;

    // Use builder pattern for clean configuration
    let handlers_builder = McpHandlersBuilder::new()
        .with_resource_provider(Arc::new(FileSystemResourceProvider::new("/data")))
        .with_tool_provider(Arc::new(DatabaseToolProvider::new(db_config)))
        .with_prompt_provider(Arc::new(TemplatePromptProvider::new("./prompts")))
        .with_logging_handler(Arc::new(StructuredLoggingHandler::new()))
        .with_config(production_mcp_config());

    let server = AxumHttpServer::with_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        handlers_builder,
        config,
    ).await?;

    // Bind and serve
    server.bind("0.0.0.0:8080".parse()?).await?;
    server.serve().await?;

    Ok(())
}
```

### Development and Testing Patterns

```rust
// Pattern 1: Empty handlers for infrastructure testing
async fn create_test_server() -> Result<AxumHttpServer, TransportError> {
    let (connection_manager, session_manager, jsonrpc_processor, config) = 
        create_test_infrastructure().await;

    AxumHttpServer::new_with_empty_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        config,
    ).await
}

// Pattern 2: Partial configuration for incremental development
async fn create_development_server() -> Result<AxumHttpServer, TransportError> {
    let (connection_manager, session_manager, jsonrpc_processor, config) = 
        create_dev_infrastructure().await;

    let handlers_builder = McpHandlersBuilder::new()
        .with_resource_provider(Arc::new(MockResourceProvider::new()))
        // Note: Only resources configured for development
        .with_config(development_mcp_config());

    AxumHttpServer::with_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        handlers_builder,
        config,
    ).await
}
```

### Environment-Specific Configuration

```rust
// Pattern 3: Environment-aware handler composition
async fn create_environment_server(env: Environment) -> Result<AxumHttpServer, TransportError> {
    let handlers_builder = match env {
        Environment::Development => {
            McpHandlersBuilder::new()
                .with_resource_provider(Arc::new(MockResourceProvider::new()))
                .with_tool_provider(Arc::new(DebugToolProvider::new()))
                .with_config(development_config())
        },
        Environment::Production => {
            McpHandlersBuilder::new()
                .with_resource_provider(Arc::new(HighPerformanceResourceProvider::new()))
                .with_tool_provider(Arc::new(SecureToolProvider::new()))
                .with_prompt_provider(Arc::new(OptimizedPromptProvider::new()))
                .with_logging_handler(Arc::new(ProductionLoggingHandler::new()))
                .with_config(production_config())
        }
    };

    let (connection_manager, session_manager, jsonrpc_processor, config) = 
        create_infrastructure_for_env(env).await;

    AxumHttpServer::with_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        handlers_builder,
        config,
    ).await
}
```

### Resilient Handler Patterns

```rust
// Pattern 4: Fallback providers for graceful degradation
struct FallbackResourceProvider {
    primary: Arc<dyn ResourceProvider>,
    fallback: Arc<dyn ResourceProvider>,
}

#[async_trait]
impl ResourceProvider for FallbackResourceProvider {
    async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        match self.primary.list_resources().await {
            Ok(resources) => Ok(resources),
            Err(error) => {
                tracing::warn!("Primary provider failed, using fallback: {}", error);
                self.fallback.list_resources().await
            }
        }
    }

    async fn read_resource(&self, uri: &str) -> McpResult<Vec<Content>> {
        match self.primary.read_resource(uri).await {
            Ok(content) => Ok(content),
            Err(error) => {
                tracing::warn!("Primary read failed for {}, falling back: {}", uri, error);
                self.fallback.read_resource(uri).await
            }
        }
    }
}

async fn create_resilient_server() -> Result<AxumHttpServer, TransportError> {
    let resilient_provider = Arc::new(FallbackResourceProvider {
        primary: Arc::new(DatabaseResourceProvider::new()),
        fallback: Arc::new(CacheResourceProvider::new()),
    });

    let handlers_builder = McpHandlersBuilder::new()
        .with_resource_provider(resilient_provider)
        .with_tool_provider(Arc::new(CircuitBreakerToolProvider::new()))
        .with_config(resilient_config());

    let (connection_manager, session_manager, jsonrpc_processor, config) = 
        create_resilient_infrastructure().await;

    AxumHttpServer::with_handlers(
        connection_manager,
        session_manager,
        jsonrpc_processor,
        handlers_builder,
        config,
    ).await
}
```

For complete implementation examples and more advanced patterns, see:
- [AxumHttpServer Handler Configuration Example](../../examples/axum_server_with_handlers.rs)
- [Performance Optimization Guide](./performance_optimization.md)
- [Basic Examples](./basic_examples.md)

---

*Next: [Performance Optimization](./performance_optimization.md) | Return to [Usages Overview](../usages.md)*

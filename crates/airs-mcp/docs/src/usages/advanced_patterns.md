# Advanced Patterns

*High-performance implementations and sophisticated usage patterns*

## Buffer Pooling and Performance Optimization

### Advanced Buffer Management

AIRS MCP provides enterprise-grade buffer pooling that achieves **60-80% reduction in allocation overhead**:

```rust
use airs_mcp::transport::buffer::{BufferManager, BufferConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure high-performance buffer management
    let config = BufferConfig {
        max_message_size: 10 * 1024 * 1024,  // 10MB max messages
        read_buffer_capacity: 128 * 1024,    // 128KB buffers
        write_buffer_capacity: 128 * 1024,   // 128KB buffers
        buffer_pool_size: 200,               // Pool 200 buffers
        pool_timeout: Duration::from_secs(30),
        enable_zero_copy: true,              // Enable zero-copy optimizations
        backpressure_threshold: 1024 * 1024, // 1MB backpressure threshold
    };
    
    let buffer_manager = BufferManager::new(config);
    
    // Acquire buffers with automatic pooling
    let read_buffer = buffer_manager.acquire_read_buffer().await?;
    let write_buffer = buffer_manager.acquire_write_buffer().await?;
    
    // Buffers automatically return to pool when dropped
    // Zero-copy operations where possible
    
    Ok(())
}
```

### Buffer Pool Metrics and Monitoring

```rust
use airs_mcp::transport::buffer::BufferManager;

async fn monitor_buffer_performance(manager: &BufferManager) {
    let metrics = manager.metrics();
    
    println!("Buffer Pool Performance:");
    println!("  Success Rate: {:.2}%", metrics.acquisition_success_rate() * 100.0);
    println!("  Bytes Processed: {}", metrics.total_bytes_processed());
    
    // Zero-copy specific metrics
    let zero_copy_metrics = manager.get_zero_copy_metrics();
    println!("  Buffer Hit Ratio: {:.2}%", zero_copy_metrics.buffer_pool_hit_ratio() * 100.0);
    println!("  Pool Utilization: {:.2}%", zero_copy_metrics.pool_utilization());
    println!("  Zero-Copy Sends: {}", zero_copy_metrics.zero_copy_sends);
    println!("  Zero-Copy Receives: {}", zero_copy_metrics.zero_copy_receives);
}
```

## Streaming JSON Parsing

### Memory-Efficient Incremental Parsing

```rust
use airs_mcp::base::jsonrpc::streaming::{StreamingParser, StreamingConfig};
use bytes::Bytes;

async fn streaming_parser_example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure streaming parser for large messages
    let config = StreamingConfig {
        max_message_size: 16 * 1024 * 1024, // 16MB messages
        read_buffer_size: 64 * 1024,        // 64KB buffer
        strict_validation: true,            // Comprehensive validation
    };
    
    let mut parser = StreamingParser::new(config);
    
    // Parse from streaming source (e.g., network connection)
    let large_json_data = get_large_json_data().await;
    let message = parser.parse_from_bytes(&large_json_data).await?;
    
    println!("Parsed message: {:?}", message);
    
    // Get buffer statistics for monitoring
    let stats = parser.buffer_stats();
    println!("Buffer utilization: {:.2}%", stats.utilization() * 100.0);
    
    Ok(())
}

async fn get_large_json_data() -> Vec<u8> {
    // Simulated large JSON data
    let large_message = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "process_large_dataset",
        "params": {
            "data": vec![0u8; 1024 * 1024], // 1MB of data
        },
        "id": 1
    });
    
    serde_json::to_vec(&large_message).unwrap()
}
```

### Multi-Message Batch Processing

```rust
use airs_mcp::base::jsonrpc::streaming::StreamingParser;

async fn batch_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = StreamingParser::default();
    
    // Process multiple JSON-RPC messages from a single buffer
    let batch_data = br#"
        {"jsonrpc":"2.0","method":"ping","id":1}
        {"jsonrpc":"2.0","method":"status","id":2}
        {"jsonrpc":"2.0","method":"info","id":3}
    "#;
    
    let messages = parser.parse_multiple_from_bytes(batch_data).await?;
    
    println!("Processed {} messages in batch", messages.len());
    for (i, message) in messages.iter().enumerate() {
        println!("  Message {}: {:?}", i + 1, message);
    }
    
    Ok(())
}
```

## Zero-Copy Transport Optimizations

### Zero-Copy Message Processing

```rust
use airs_mcp::transport::{ZeroCopyTransport, StdioTransport};
use airs_mcp::base::jsonrpc::JsonRpcMessage;
use bytes::BytesMut;

async fn zero_copy_example() -> Result<(), Box<dyn std::error::Error>> {
    let transport = StdioTransport::new().await?;
    
    // Zero-copy serialization directly to buffer
    let message = JsonRpcMessage::request(1, "ping", None);
    let mut buffer = BytesMut::with_capacity(1024);
    message.serialize_to_buffer(&mut buffer)?;
    
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

### Advanced Request Correlation

```rust
use airs_mcp::correlation::{CorrelationManager, CorrelationConfig};
use airs_mcp::base::jsonrpc::{JsonRpcRequest, JsonRpcResponse};
use std::time::Duration;
use tokio::time::timeout;

async fn correlation_management_example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure correlation tracking with timeouts
    let config = CorrelationConfig {
        default_timeout: Duration::from_secs(30),
        max_pending_requests: 1000,
        cleanup_interval: Duration::from_secs(60),
    };
    
    let correlation_manager = CorrelationManager::new(config);
    
    // Register outbound request
    let request = JsonRpcRequest::new(42, "long_operation".to_string(), None);
    let request_future = correlation_manager.register_request(request.clone()).await?;
    
    // Send request through transport
    send_request_somewhere(request).await?;
    
    // Wait for correlated response with timeout
    let response = timeout(Duration::from_secs(10), request_future).await??;
    println!("Received correlated response: {:?}", response);
    
    // Get correlation statistics
    let stats = correlation_manager.statistics().await;
    println!("Pending requests: {}", stats.pending_count);
    println!("Success rate: {:.2}%", stats.success_rate * 100.0);
    
    Ok(())
}

async fn send_request_somewhere(_request: JsonRpcRequest) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation depends on your transport
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
use airs_mcp::base::jsonrpc::concurrent::{ConcurrentJsonRpcProcessor, ProcessorConfig};
use airs_mcp::integration::handler::RequestHandler;
use std::time::Duration;
use async_trait::async_trait;

#[derive(Debug)]
struct HighPerformanceHandler;

#[async_trait]
impl RequestHandler for HighPerformanceHandler {
    async fn handle_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, airs_mcp::integration::error::IntegrationError> {
        // Simulate CPU-intensive work
        tokio::task::yield_now().await;
        
        Ok(serde_json::json!({
            "method": method,
            "processed": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}

async fn concurrent_processing_example() -> Result<(), Box<dyn std::error::Error>> {
    // Configure concurrent processor for high throughput
    let config = ProcessorConfig {
        worker_count: 8,                              // 8 worker threads
        queue_capacity: 1000,                         // Buffer 1000 requests
        request_timeout: Duration::from_secs(30),
        shutdown_timeout: Duration::from_secs(5),
        backpressure_threshold: 800,                  // Apply backpressure at 80% capacity
    };
    
    let processor = ConcurrentJsonRpcProcessor::new(
        config,
        Box::new(HighPerformanceHandler),
    ).await?;
    
    // Process requests concurrently
    let request = JsonRpcRequest::new(1, "compute_intensive".to_string(), None);
    let response_future = processor.process_request(request).await?;
    
    // Get processing statistics
    let stats = processor.statistics().await;
    println!("Worker Pool Statistics:");
    println!("  Active Workers: {}", stats.active_workers);
    println!("  Queue Depth: {}", stats.queue_depth);
    println!("  Processed Count: {}", stats.total_processed);
    println!("  Average Processing Time: {:?}", stats.avg_processing_time);
    
    // Wait for response
    let response = response_future.await?;
    println!("Response: {:?}", response);
    
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

### Sophisticated Request Router

```rust
use airs_mcp::integration::router::{RequestRouter, RoutePattern};
use airs_mcp::integration::handler::RequestHandler;
use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Debug)]
struct DatabaseHandler {
    connections: tokio::sync::RwLock<HashMap<String, DatabaseConnection>>,
}

#[async_trait]
impl RequestHandler for DatabaseHandler {
    async fn handle_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, airs_mcp::integration::error::IntegrationError> {
        match method {
            "db/query" => self.handle_query(params).await,
            "db/transaction" => self.handle_transaction(params).await,
            "db/migrate" => self.handle_migration(params).await,
            _ => Err(airs_mcp::integration::error::IntegrationError::MethodNotFound {
                method: method.to_string(),
            })
        }
    }
}

impl DatabaseHandler {
    async fn handle_query(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, airs_mcp::integration::error::IntegrationError> {
        // Complex query processing with connection pooling
        let query = params
            .and_then(|p| p.get("query"))
            .and_then(|q| q.as_str())
            .ok_or_else(|| airs_mcp::integration::error::IntegrationError::InvalidParams {
                message: "Query parameter required".to_string(),
            })?;
        
        // Execute query with proper connection management
        Ok(serde_json::json!({
            "query": query,
            "results": [],
            "execution_time_ms": 42
        }))
    }
    
    async fn handle_transaction(&self, _params: Option<serde_json::Value>) -> Result<serde_json::Value, airs_mcp::integration::error::IntegrationError> {
        // Transaction handling with ACID guarantees
        Ok(serde_json::json!({
            "transaction_id": "txn_123456",
            "status": "committed"
        }))
    }
    
    async fn handle_migration(&self, _params: Option<serde_json::Value>) -> Result<serde_json::Value, airs_mcp::integration::error::IntegrationError> {
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

async fn advanced_routing_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = RequestRouter::new();
    
    // Register sophisticated handlers with pattern matching
    router.register_handler(
        RoutePattern::exact("db/*"),
        Box::new(DatabaseHandler {
            connections: tokio::sync::RwLock::new(HashMap::new()),
        })
    );
    
    router.register_handler(
        RoutePattern::prefix("auth/"),
        Box::new(AuthHandler::new())
    );
    
    router.register_handler(
        RoutePattern::regex(r"^api/v\d+/.*$"),
        Box::new(VersionedApiHandler::new())
    );
    
    // Route requests efficiently
    let request = JsonRpcRequest::new(1, "db/query".to_string(), Some(serde_json::json!({
        "query": "SELECT * FROM users WHERE active = true"
    })));
    
    let response = router.route_request(&request).await?;
    println!("Routed response: {:?}", response);
    
    Ok(())
}

// Placeholder handlers
#[derive(Debug)]
struct AuthHandler;

impl AuthHandler {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for AuthHandler {
    async fn handle_request(&self, _method: &str, _params: Option<serde_json::Value>) -> Result<serde_json::Value, airs_mcp::integration::error::IntegrationError> {
        Ok(serde_json::json!({"auth": "handled"}))
    }
}

#[derive(Debug)]
struct VersionedApiHandler;

impl VersionedApiHandler {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RequestHandler for VersionedApiHandler {
    async fn handle_request(&self, _method: &str, _params: Option<serde_json::Value>) -> Result<serde_json::Value, airs_mcp::integration::error::IntegrationError> {
        Ok(serde_json::json!({"api": "handled"}))
    }
}
```

## Streaming Transport Integration

### High-Performance Transport Wrapper

```rust
use airs_mcp::transport::streaming::{StreamingTransport, StreamingStats};
use airs_mcp::transport::stdio::StdioTransport;
use airs_mcp::base::jsonrpc::streaming::StreamingConfig;

async fn streaming_transport_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create base transport
    let base_transport = StdioTransport::new().await?;
    
    // Wrap with streaming capabilities
    let streaming_config = StreamingConfig {
        max_message_size: 32 * 1024 * 1024,  // 32MB messages
        read_buffer_size: 256 * 1024,        // 256KB streaming buffer
        strict_validation: true,
    };
    
    let streaming_transport = StreamingTransport::new(base_transport, streaming_config);
    
    // Send and receive with streaming optimizations
    let large_request = create_large_request();
    streaming_transport.send_parsed(&large_request).await?;
    
    let response = streaming_transport.receive_parsed().await?;
    println!("Received streaming response: {:?}", response);
    
    // Monitor streaming performance
    let stats = streaming_transport.buffer_stats().await;
    println!("Streaming Buffer Stats:");
    println!("  Capacity: {} bytes", stats.capacity);
    println!("  Used: {} bytes", stats.used);
    println!("  Utilization: {:.2}%", stats.utilization() * 100.0);
    
    Ok(())
}

fn create_large_request() -> airs_mcp::base::jsonrpc::streaming::ParsedMessage {
    // Create a large request for testing streaming
    use airs_mcp::base::jsonrpc::JsonRpcRequest;
    use airs_mcp::base::jsonrpc::streaming::ParsedMessage;
    
    let large_data = vec![0u8; 5 * 1024 * 1024]; // 5MB payload
    let request = JsonRpcRequest::new(
        1,
        "process_large_data".to_string(),
        Some(serde_json::json!({
            "data": large_data,
            "compression": "none"
        }))
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
    buffer_manager: BufferManager,
    correlation_manager: CorrelationManager,
) {
    let mut interval = interval(Duration::from_secs(5));
    
    loop {
        interval.tick().await;
        
        // Buffer performance metrics
        let buffer_metrics = buffer_manager.metrics();
        let zero_copy_metrics = buffer_manager.get_zero_copy_metrics();
        
        // Correlation performance metrics
        let correlation_stats = correlation_manager.statistics().await;
        
        println!("\n=== Performance Dashboard ===");
        
        // Buffer Pool Health
        println!("Buffer Pool:");
        println!("  Hit Ratio: {:.2}%", zero_copy_metrics.buffer_pool_hit_ratio() * 100.0);
        println!("  Utilization: {:.2}%", zero_copy_metrics.pool_utilization());
        println!("  Bytes Processed: {:.2} MB", 
                 zero_copy_metrics.total_bytes_processed as f64 / (1024.0 * 1024.0));
        println!("  Zero-Copy Operations: {} sends, {} receives",
                 zero_copy_metrics.zero_copy_sends,
                 zero_copy_metrics.zero_copy_receives);
        
        // Correlation Health
        println!("Request Correlation:");
        println!("  Pending Requests: {}", correlation_stats.pending_count);
        println!("  Success Rate: {:.2}%", correlation_stats.success_rate * 100.0);
        println!("  Average Response Time: {:?}", correlation_stats.avg_response_time);
        
        // Performance alerts
        if zero_copy_metrics.buffer_pool_hit_ratio() < 0.8 {
            eprintln!("⚠️  WARNING: Low buffer pool hit ratio!");
        }
        
        if correlation_stats.success_rate < 0.95 {
            eprintln!("⚠️  WARNING: Low correlation success rate!");
        }
        
        if correlation_stats.pending_count > 1000 {
            eprintln!("⚠️  WARNING: High number of pending requests!");
        }
    }
}
```

### Benchmarking Integration

```rust
use criterion::{black_box, Criterion};
use airs_mcp::transport::buffer::{BufferManager, BufferConfig};

fn benchmark_buffer_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("buffer_acquisition", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = BufferConfig::default();
                let manager = BufferManager::new(config);
                
                let buffer = manager.acquire_read_buffer().await.unwrap();
                black_box(buffer);
            });
        });
    });
    
    c.bench_function("zero_copy_serialization", |b| {
        b.iter(|| {
            rt.block_on(async {
                let message = airs_mcp::base::jsonrpc::JsonRpcRequest::new(
                    1,
                    "benchmark".to_string(),
                    None
                );
                
                let mut buffer = bytes::BytesMut::with_capacity(1024);
                message.serialize_to_buffer(&mut buffer).unwrap();
                black_box(buffer);
            });
        });
    });
}
```

## MCP Handler Configuration Patterns

### Production-Ready Handler Setup

AIRS MCP provides sophisticated handler configuration patterns for production deployments. See [Handler Configuration Architecture](../architecture/handler_configuration.md) for detailed architectural information.

```rust
use airs_mcp::transport::http::axum_server::{AxumHttpServer, McpHandlersBuilder};
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

For complete implementation examples, see the [Handler Configuration Example](../../examples/axum_server_with_handlers.rs).

---

*Next: [Performance Optimization](./performance_optimization.md) | Return to [Usages Overview](../usages.md)*

Check back soon for advanced implementation strategies.

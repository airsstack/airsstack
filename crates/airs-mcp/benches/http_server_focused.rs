//! Phase 3D HTTP Server Focused Benchmark (Ultra-Lightweight)
//!
//! This benchmark provides minimal performance validation for the airs-mcp
//! HTTP server implementation, optimized for laptop development environments.
//!
//! ## Benchmark Categories (Simplified)
//!
//! 1. **Configuration Creation**: Basic config instantiation only
//! 2. **Component Building**: Lightweight component creation  
//! 3. **Config Structs**: Simple allocation testing
//! 4. **Request/Response Lifecycle**: Message processing validation
//!
//! ## Performance Targets (Ultra Resource-Conscious)
//!
//! - **Runtime**: <60 seconds total execution time
//! - **Memory**: <150MB peak usage for laptop development
//! - **Iterations**: <50 per benchmark to minimize system load
//! - **Focus**: Configuration, builders, and lightweight message processing

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;

// Minimal imports - only what we need for lightweight testing
use airs_mcp::base::jsonrpc::concurrent::ProcessorConfig;
use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId};
use airs_mcp::correlation::manager::CorrelationConfig;
use airs_mcp::shared::protocol::messages::resources::{
    ListResourcesRequest, ListResourcesResponse, Resource,
};
use airs_mcp::shared::protocol::messages::tools::{ListToolsRequest, ListToolsResponse, Tool};
use airs_mcp::transport::http::axum::McpHandlersBuilder;
use airs_mcp::transport::http::config::HttpTransportConfig;
use airs_mcp::transport::http::connection_manager::HealthCheckConfig;
use airs_mcp::transport::http::session::SessionConfig;

/// Benchmark 1: Configuration creation only (no server instances)
fn benchmark_configuration_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_creation");

    // Limit sample size to reduce system load
    group.sample_size(20);

    // Test HttpTransportConfig creation (lightweight)
    group.bench_function("http_transport_config", |b| {
        b.iter(|| {
            let config = black_box(HttpTransportConfig::new());
            black_box(config);
        });
    });

    // Test HttpTransportConfig with builder pattern (lightweight)
    group.bench_function("http_transport_config_builder", |b| {
        b.iter(|| {
            let config = black_box(
                HttpTransportConfig::new()
                    .max_connections(100) // Very small for laptop
                    .session_timeout(std::time::Duration::from_secs(10)),
            );
            black_box(config);
        });
    });

    // Test basic config structs creation
    group.bench_function("health_check_config", |b| {
        b.iter(|| {
            let config = black_box(HealthCheckConfig::default());
            black_box(config);
        });
    });

    group.bench_function("session_config", |b| {
        b.iter(|| {
            let config = black_box(SessionConfig::default());
            black_box(config);
        });
    });

    group.finish();
}

/// Benchmark 2: Builder patterns only (no async components)
fn benchmark_builder_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder_patterns");

    // Very small sample size to minimize load
    group.sample_size(15);

    // Test McpHandlersBuilder creation
    group.bench_function("mcp_handlers_builder", |b| {
        b.iter(|| {
            let builder = black_box(McpHandlersBuilder::new());
            black_box(builder);
        });
    });

    // Test McpHandlersBuilder with configuration
    group.bench_function("mcp_handlers_builder_with_config", |b| {
        b.iter(|| {
            let builder = black_box(McpHandlersBuilder::new().with_config(Default::default()));
            black_box(builder);
        });
    });

    // Test handlers build process (lightweight)
    group.bench_function("mcp_handlers_build", |b| {
        b.iter(|| {
            let builder = McpHandlersBuilder::new();
            let handlers = black_box(builder.build());
            black_box(handlers);
        });
    });

    group.finish();
}

/// Benchmark 3: Basic configuration structs (ultra-minimal)
fn benchmark_config_structs(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_structs");

    // Minimal sample size
    group.sample_size(10);

    // Test ProcessorConfig creation
    group.bench_function("processor_config", |b| {
        b.iter(|| {
            let config = black_box(ProcessorConfig {
                worker_count: 1, // Minimal for laptop
                queue_capacity: 10,
                max_batch_size: 2,
                processing_timeout: chrono::Duration::seconds(5),
                enable_ordering: false,
                enable_backpressure: true,
            });
            black_box(config);
        });
    });

    // Test CorrelationConfig creation
    group.bench_function("correlation_config", |b| {
        b.iter(|| {
            let config = black_box(CorrelationConfig::default());
            black_box(config);
        });
    });

    group.finish();
}

/// Benchmark 4: Request/Response lifecycle validation (laptop-friendly)
fn benchmark_request_response_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_response_lifecycle");

    // Small sample size for laptop performance
    group.sample_size(15);

    // Test JSON-RPC request creation and serialization
    group.bench_function("jsonrpc_request_creation", |b| {
        b.iter(|| {
            let request = black_box(JsonRpcRequest::new(
                "resources/list",
                Some(json!({})),
                RequestId::new_string("bench_001".to_string()),
            ));
            black_box(request);
        });
    });

    // Test JSON-RPC request serialization
    group.bench_function("jsonrpc_request_serialization", |b| {
        let request = JsonRpcRequest::new(
            "resources/list",
            Some(json!({})),
            RequestId::new_string("bench_001".to_string()),
        );

        b.iter(|| {
            let json_str = black_box(request.to_json().unwrap());
            black_box(json_str);
        });
    });

    // Test JSON-RPC request deserialization
    group.bench_function("jsonrpc_request_deserialization", |b| {
        let request = JsonRpcRequest::new(
            "resources/list",
            Some(json!({})),
            RequestId::new_string("bench_001".to_string()),
        );
        let json_str = request.to_json().unwrap();

        b.iter(|| {
            let parsed: JsonRpcRequest = black_box(serde_json::from_str(&json_str).unwrap());
            black_box(parsed);
        });
    });

    // Test MCP ListResourcesRequest processing
    group.bench_function("mcp_list_resources_request", |b| {
        b.iter(|| {
            let request = black_box(ListResourcesRequest::new());
            let json_str = black_box(request.to_json().unwrap());
            black_box(json_str);
        });
    });

    // Test MCP Resource creation and serialization
    group.bench_function("mcp_resource_creation", |b| {
        b.iter(|| {
            let resource = black_box(
                Resource::new(
                    "file:///test.txt",
                    "Test Resource",
                    Some("Benchmark test resource"),
                    Some("text/plain"),
                )
                .unwrap(),
            );
            let json_str = black_box(serde_json::to_string(&resource).unwrap());
            black_box(json_str);
        });
    });

    // Test MCP ListResourcesResponse creation
    group.bench_function("mcp_list_resources_response", |b| {
        let resources = vec![
            Resource::new(
                "file:///test1.txt",
                "Test Resource 1",
                None::<String>,
                None::<String>,
            )
            .unwrap(),
            Resource::new(
                "file:///test2.txt",
                "Test Resource 2",
                None::<String>,
                None::<String>,
            )
            .unwrap(),
        ];

        b.iter(|| {
            let response = black_box(ListResourcesResponse::new(resources.clone()));
            let json_str = black_box(serde_json::to_string(&response).unwrap());
            black_box(json_str);
        });
    });

    // Test MCP Tool operations
    group.bench_function("mcp_tool_operations", |b| {
        b.iter(|| {
            let list_request = black_box(ListToolsRequest::new());
            let tool = black_box(Tool::new(
                "test_tool",
                Some("Test Tool"),
                Some("Test tool for benchmarking"),
                json!({
                    "type": "object",
                    "properties": {
                        "input": {"type": "string"}
                    }
                }),
            ));
            let tools = vec![tool];
            let response = black_box(ListToolsResponse::new(tools));
            black_box((list_request, response));
        });
    });

    // Test JSON-RPC response creation
    group.bench_function("jsonrpc_response_creation", |b| {
        let result = json!({
            "resources": [
                {"uri": "file:///test.txt", "name": "Test"}
            ]
        });

        b.iter(|| {
            let response = black_box(JsonRpcResponse::success(
                result.clone(),
                RequestId::new_string("bench_001".to_string()),
            ));
            let json_str = black_box(response.to_json().unwrap());
            black_box(json_str);
        });
    });

    group.finish();
}

criterion_group!(
    ultra_lightweight_benchmarks,
    benchmark_configuration_creation,
    benchmark_builder_patterns,
    benchmark_config_structs,
    benchmark_request_response_lifecycle
);

criterion_main!(ultra_lightweight_benchmarks);

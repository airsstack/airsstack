//! Phase 3D HTTP Server Focused Benchmark
//!
//! This benchmark provides focused performance validation for the airs-mcp
//! HTTP server implementation, specifically targeting our recent AxumHttpServer development.
//!
//! ## Benchmark Categories
//!
//! 1. **Axum Server Creation**: AxumHttpServer instantiation patterns
//! 2. **HTTP Server Configuration**: Configuration and binding performance  
//! 3. **MCP Handler Architecture**: Handler architecture validation
//! 4. **Memory Footprint**: Resource utilization validation
//!
//! ## Performance Targets (Resource-Conscious)
//!
//! - **Runtime**: <3 minutes total execution time
//! - **Memory**: 200-300MB peak usage for laptop development
//! - **Server Creation**: <100ms for full infrastructure setup
//! - **Configuration**: <10ms for typical config scenarios
//! - **Handler Setup**: <50ms for complete MCP handler configuration
//! - **Memory Footprint**: <50MB base server memory usage

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

// Use proven working imports from our AxumHttpServer implementation
use airs_mcp::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};
use airs_mcp::correlation::manager::{CorrelationConfig, CorrelationManager};
use airs_mcp::transport::http::axum::{AxumHttpServer, McpHandlersBuilder};
use airs_mcp::transport::http::config::HttpTransportConfig;
use airs_mcp::transport::http::connection_manager::{HealthCheckConfig, HttpConnectionManager};
use airs_mcp::transport::http::session::{SessionConfig, SessionManager};

/// Create shared infrastructure components (from axum_server_with_handlers.rs)
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
        worker_count: 2, // Reduced for laptop development
        queue_capacity: 100,
        max_batch_size: 10,
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

/// Benchmark 1: AxumHttpServer creation patterns
fn benchmark_axum_server_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("axum_server_creation");

    let rt = Runtime::new().unwrap();

    // Test server creation with empty handlers (most common development pattern)
    group.bench_function("empty_handlers_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (connection_manager, session_manager, processor, config) =
                    create_infrastructure().await;

                let server = AxumHttpServer::new_with_empty_handlers(
                    connection_manager,
                    session_manager,
                    processor,
                    config,
                )
                .await
                .unwrap();

                black_box(server);
            });
        });
    });

    // Test server creation with handler builder pattern
    group.bench_function("handler_builder_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (connection_manager, session_manager, processor, config) =
                    create_infrastructure().await;

                let handlers_builder = McpHandlersBuilder::new();

                let server = AxumHttpServer::with_handlers(
                    connection_manager,
                    session_manager,
                    processor,
                    handlers_builder,
                    config,
                )
                .await
                .unwrap();

                black_box(server);
            });
        });
    });

    group.finish();
}

/// Benchmark 2: HTTP server configuration performance
fn benchmark_http_server_configuration(c: &mut Criterion) {
    let mut group = c.benchmark_group("http_server_configuration");

    // Test HttpTransportConfig creation (basic configuration)
    group.bench_function("transport_config_creation", |b| {
        b.iter(|| {
            let config = black_box(HttpTransportConfig::new());
            black_box(config);
        });
    });

    // Test HttpTransportConfig builder pattern
    group.bench_function("transport_config_builder", |b| {
        b.iter(|| {
            let config = black_box(
                HttpTransportConfig::builder()
                    .max_message_size(5 * 1024 * 1024) // 5MB - laptop friendly
                    .timeout_duration(std::time::Duration::from_secs(30))
                    .build(),
            );
            black_box(config);
        });
    });

    // Test connection manager creation
    group.bench_function("connection_manager_creation", |b| {
        b.iter(|| {
            let manager = black_box(
                HttpConnectionManager::new(5, HealthCheckConfig::default()), // Reduced pool size
            );
            black_box(manager);
        });
    });

    // Test processor configuration for typical laptop development
    group.bench_function("processor_config_creation", |b| {
        b.iter(|| {
            let processor_config = black_box(ProcessorConfig {
                worker_count: 2,
                queue_capacity: 50, // Reduced for laptop
                max_batch_size: 5,
                processing_timeout: chrono::Duration::seconds(15),
                enable_ordering: false,
                enable_backpressure: true,
            });

            let processor = black_box(ConcurrentProcessor::new(processor_config));
            black_box(processor);
        });
    });

    group.finish();
}

/// Benchmark 3: MCP handler architecture validation
fn benchmark_mcp_handler_architecture(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcp_handler_architecture");

    // Test McpHandlersBuilder performance
    group.bench_function("handlers_builder_creation", |b| {
        b.iter(|| {
            let builder = black_box(McpHandlersBuilder::new());
            black_box(builder);
        });
    });

    // Test handlers builder configuration
    group.bench_function("handlers_builder_configuration", |b| {
        b.iter(|| {
            let builder = black_box(McpHandlersBuilder::new().with_config(Default::default()));
            black_box(builder);
        });
    });

    // Test handlers building process
    group.bench_function("handlers_build_process", |b| {
        b.iter(|| {
            let builder = McpHandlersBuilder::new();
            let handlers = black_box(builder.build());
            black_box(handlers);
        });
    });

    let rt = Runtime::new().unwrap();

    // Test session manager creation (session overhead validation)
    group.bench_function("session_manager_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let correlation_manager = Arc::new(
                    CorrelationManager::new(CorrelationConfig::default())
                        .await
                        .unwrap(),
                );

                let session_manager = black_box(SessionManager::new(
                    correlation_manager,
                    SessionConfig::default(),
                ));

                black_box(session_manager);
            });
        });
    });

    group.finish();
}

/// Benchmark 4: Memory footprint validation (laptop-friendly)
fn benchmark_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_footprint");

    let rt = Runtime::new().unwrap();

    // Test base server memory allocation
    group.bench_function("base_server_allocation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (connection_manager, session_manager, processor, config) =
                    create_infrastructure().await;

                // Simulate complete server setup to measure memory footprint
                let server = AxumHttpServer::new_with_empty_handlers(
                    connection_manager,
                    session_manager,
                    processor,
                    config,
                )
                .await
                .unwrap();

                black_box(server);
            });
        });
    });

    // Test memory patterns for multiple server instances (resource sharing validation)
    group.bench_function("multiple_server_instances", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut servers = Vec::new();

                // Create 3 servers to test memory sharing patterns (laptop-friendly count)
                for _ in 0..3 {
                    let (connection_manager, session_manager, processor, config) =
                        create_infrastructure().await;

                    let server = AxumHttpServer::new_with_empty_handlers(
                        connection_manager,
                        session_manager,
                        processor,
                        config,
                    )
                    .await
                    .unwrap();

                    servers.push(server);
                }

                black_box(servers);
            });
        });
    });

    // Test infrastructure component memory patterns
    group.bench_function("infrastructure_components_allocation", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Test individual component allocations
                let connection_manager =
                    Arc::new(HttpConnectionManager::new(5, HealthCheckConfig::default()));

                let correlation_manager = Arc::new(
                    CorrelationManager::new(CorrelationConfig::default())
                        .await
                        .unwrap(),
                );

                let session_manager = Arc::new(SessionManager::new(
                    correlation_manager,
                    SessionConfig::default(),
                ));

                let processor = Arc::new(ConcurrentProcessor::new(ProcessorConfig {
                    worker_count: 2,
                    queue_capacity: 50,
                    max_batch_size: 5,
                    processing_timeout: chrono::Duration::seconds(15),
                    enable_ordering: false,
                    enable_backpressure: true,
                }));

                let config = HttpTransportConfig::new();

                black_box((connection_manager, session_manager, processor, config));
            });
        });
    });

    group.finish();
}

criterion_group!(
    http_server_focused_benchmarks,
    benchmark_axum_server_creation,
    benchmark_http_server_configuration,
    benchmark_mcp_handler_architecture,
    benchmark_memory_footprint
);

criterion_main!(http_server_focused_benchmarks);

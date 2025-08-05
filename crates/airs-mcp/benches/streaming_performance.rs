//! Streaming JSON Parser Performance Benchmarks
//!
//! This module benchmarks the performance of the streaming JSON parser,
//! focusing on configuration and setup performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;
use std::io::Cursor;
use tokio::runtime::Runtime;

use airs_mcp::base::jsonrpc::streaming::{StreamingConfig, StreamingParser};
use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest, RequestId};

/// Create test JSON data of specified size in KB
fn create_test_json(size_kb: usize) -> String {
    let data = "x".repeat(size_kb * 1024);
    let request = JsonRpcRequest::new(
        "test_method",
        Some(json!({
            "data": data,
            "size_kb": size_kb,
            "timestamp": chrono::Utc::now()
        })),
        RequestId::new_string(format!("req_{}", size_kb)),
    );
    request.to_json().unwrap()
}

/// Benchmark streaming parser creation and configuration
fn benchmark_streaming_parser_setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_parser_setup");

    group.bench_function("default_config", |b| {
        b.iter(|| {
            black_box(StreamingConfig::default());
        });
    });

    group.bench_function("custom_config", |b| {
        b.iter(|| {
            let config = StreamingConfig {
                max_message_size: 32 * 1024 * 1024, // 32MB
                read_buffer_size: 16 * 1024,        // 16KB
                strict_validation: true,
            };
            black_box(config);
        });
    });

    group.bench_function("parser_creation", |b| {
        let config = StreamingConfig::default();
        b.iter(|| {
            black_box(StreamingParser::new(config.clone()));
        });
    });

    group.finish();
}

/// Benchmark streaming configuration variations
fn benchmark_streaming_configurations(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_configurations");

    // Test different buffer sizes
    for buffer_size in [1024, 4096, 8192, 16384, 32768].iter() {
        group.bench_with_input(
            BenchmarkId::new("buffer_size", buffer_size),
            buffer_size,
            |b, &size| {
                b.iter(|| {
                    let config = StreamingConfig {
                        max_message_size: 16 * 1024 * 1024,
                        read_buffer_size: size,
                        strict_validation: false,
                    };
                    black_box(StreamingParser::new(config));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark test message creation performance
fn benchmark_test_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("test_message_creation");

    // Test message creation for different sizes
    for size_kb in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Bytes((*size_kb * 1024) as u64));

        group.bench_with_input(
            BenchmarkId::new("create_test_json", size_kb),
            size_kb,
            |b, &size| {
                b.iter(|| {
                    black_box(create_test_json(size));
                });
            },
        );
    }

    group.finish();
}

/// Create a test runtime for async benchmarks
fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
}

/// Create test messages of different types for parsing benchmarks
fn create_test_messages() -> Vec<(String, &'static str)> {
    vec![
        // Request
        (
            json!({
                "jsonrpc": "2.0",
                "method": "test_method",
                "params": {"key": "value"},
                "id": "test_123"
            })
            .to_string(),
            "request",
        ),
        // Notification
        (
            json!({
                "jsonrpc": "2.0",
                "method": "notify_method",
                "params": {"data": "notification_data"}
            })
            .to_string(),
            "notification",
        ),
        // Response
        (
            json!({
                "jsonrpc": "2.0",
                "result": {"status": "success", "data": "response_data"},
                "id": "test_123"
            })
            .to_string(),
            "response",
        ),
        // Error response
        (
            json!({
                "jsonrpc": "2.0",
                "error": {"code": -32600, "message": "Invalid Request"},
                "id": "test_123"
            })
            .to_string(),
            "error_response",
        ),
    ]
}

/// Benchmark actual streaming JSON parsing operations
fn benchmark_streaming_parsing(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("streaming_parsing");

    let test_messages = create_test_messages();

    // Benchmark parsing different message types
    for (message, msg_type) in &test_messages {
        let message_bytes = message.as_bytes();
        group.throughput(Throughput::Bytes(message_bytes.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("parse_from_bytes", msg_type),
            message,
            |b, msg| {
                b.iter_batched(
                    || StreamingParser::new_default(),
                    |mut parser| {
                        rt.block_on(async {
                            let result = parser.parse_from_bytes(msg.as_bytes()).await.unwrap();
                            black_box(result);
                        });
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark streaming parser with different message sizes
fn benchmark_streaming_message_sizes(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("streaming_message_sizes");

    // Test parsing performance with different message sizes
    for size_kb in [1, 10, 100].iter() {
        let test_message = create_test_json(*size_kb);
        let message_bytes = test_message.as_bytes();

        group.throughput(Throughput::Bytes(message_bytes.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parse_large_message", size_kb),
            &test_message,
            |b, msg| {
                b.iter_batched(
                    || StreamingParser::new_default(),
                    |mut parser| {
                        rt.block_on(async {
                            let result = parser.parse_from_bytes(msg.as_bytes()).await.unwrap();
                            black_box(result);
                        });
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark streaming parser with reader interface
fn benchmark_streaming_from_reader(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("streaming_from_reader");

    let test_messages = create_test_messages();

    for (message, msg_type) in &test_messages {
        let message_bytes = message.as_bytes();
        group.throughput(Throughput::Bytes(message_bytes.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("parse_from_reader", msg_type),
            message,
            |b, msg| {
                b.iter_batched(
                    || {
                        let parser = StreamingParser::new_default();
                        let cursor = Cursor::new(msg.as_bytes());
                        let reader = tokio::io::BufReader::new(cursor);
                        (parser, reader)
                    },
                    |(mut parser, mut reader)| {
                        rt.block_on(async {
                            let result = parser.parse_from_reader(&mut reader).await.unwrap();
                            black_box(result);
                        });
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark batch parsing operations
fn benchmark_streaming_batch_parsing(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("streaming_batch_parsing");

    // Create a batch of different messages
    let test_messages = create_test_messages();
    let batch_messages: Vec<String> = (0..50)
        .map(|i| test_messages[i % test_messages.len()].0.clone())
        .collect();

    let total_bytes: usize = batch_messages.iter().map(|m| m.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("parse_message_batch", |b| {
        b.iter_batched(
            || StreamingParser::new_default(),
            |mut parser| {
                rt.block_on(async {
                    let mut results = Vec::new();
                    for message in &batch_messages {
                        let result = parser.parse_from_bytes(message.as_bytes()).await.unwrap();
                        results.push(result);
                    }
                    black_box(results);
                });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark streaming configuration variations with actual parsing
fn benchmark_streaming_config_performance(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("streaming_config_performance");

    let test_message = create_test_json(10); // 10KB message

    // Test different configurations
    let configs = vec![
        ("default", StreamingConfig::default()),
        (
            "large_buffer",
            StreamingConfig {
                max_message_size: 32 * 1024 * 1024,
                read_buffer_size: 64 * 1024,
                strict_validation: false,
            },
        ),
        (
            "strict_validation",
            StreamingConfig {
                max_message_size: 16 * 1024 * 1024,
                read_buffer_size: 8192,
                strict_validation: true,
            },
        ),
    ];

    for (config_name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("config_parsing", config_name),
            &config,
            |b, cfg| {
                b.iter_batched(
                    || StreamingParser::new(cfg.clone()),
                    |mut parser| {
                        rt.block_on(async {
                            let result = parser
                                .parse_from_bytes(test_message.as_bytes())
                                .await
                                .unwrap();
                            black_box(result);
                        });
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    streaming_benches,
    benchmark_streaming_parser_setup,
    benchmark_streaming_configurations,
    benchmark_test_message_creation,
    benchmark_streaming_parsing,
    benchmark_streaming_message_sizes,
    benchmark_streaming_from_reader,
    benchmark_streaming_batch_parsing,
    benchmark_streaming_config_performance
);

criterion_main!(streaming_benches);

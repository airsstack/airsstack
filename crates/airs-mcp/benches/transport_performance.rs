//! Transport Performance Benchmarks
//!
//! This module benchmarks transport layer performance including buffer management
//! and configuration setup.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Runtime;

use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest, RequestId};
use airs_mcp::transport::buffer::{BufferConfig, BufferManager};
use airs_mcp::transport::StdioTransport;

/// Create a test runtime for async benchmarks
fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
}

/// Create a test message of specified size in KB
fn create_test_message(size_kb: usize) -> String {
    let request = JsonRpcRequest::new(
        "test_method",
        Some(json!({
            "data": "x".repeat(size_kb * 1024),
            "size_kb": size_kb,
            "timestamp": chrono::Utc::now()
        })),
        RequestId::new_string(format!("msg_{}", size_kb)),
    );
    request.to_json().unwrap()
}

/// Benchmark buffer configuration setup
fn benchmark_buffer_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_config");

    group.bench_function("default_config", |b| {
        b.iter(|| {
            black_box(BufferConfig::default());
        });
    });

    group.bench_function("custom_config", |b| {
        b.iter(|| {
            let config = BufferConfig {
                max_message_size: 16 * 1024 * 1024,
                read_buffer_capacity: 64 * 1024,
                write_buffer_capacity: 64 * 1024,
                buffer_pool_size: 16,
                pool_timeout: Duration::from_secs(30),
                enable_zero_copy: true,
                backpressure_threshold: 1024 * 1024,
            };
            black_box(config);
        });
    });

    group.finish();
}

/// Benchmark buffer manager creation
fn benchmark_buffer_manager_setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_manager_setup");

    group.bench_function("manager_creation_default", |b| {
        b.iter(|| {
            let config = BufferConfig::default();
            black_box(BufferManager::new(config));
        });
    });

    group.bench_function("manager_creation_custom", |b| {
        b.iter(|| {
            let config = BufferConfig {
                max_message_size: 8 * 1024 * 1024,
                read_buffer_capacity: 32 * 1024,
                write_buffer_capacity: 32 * 1024,
                buffer_pool_size: 8,
                pool_timeout: Duration::from_secs(30),
                enable_zero_copy: false,
                backpressure_threshold: 512 * 1024,
            };
            black_box(BufferManager::new(config));
        });
    });

    group.finish();
}

/// Benchmark buffer size variations
fn benchmark_buffer_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_sizes");

    // Test different buffer sizes
    for buffer_size in [1024, 4096, 8192, 16384, 32768, 65536].iter() {
        group.bench_with_input(
            BenchmarkId::new("buffer_manager_creation", buffer_size),
            buffer_size,
            |b, &size| {
                b.iter(|| {
                    let config = BufferConfig {
                        max_message_size: 1024 * 1024,
                        read_buffer_capacity: size,
                        write_buffer_capacity: size,
                        buffer_pool_size: 10,
                        pool_timeout: Duration::from_secs(30),
                        enable_zero_copy: true,
                        backpressure_threshold: 512 * 1024,
                    };
                    black_box(BufferManager::new(config));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark message throughput with different buffer configurations
fn benchmark_message_throughput(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("message_throughput");

    // Test messages of different sizes
    for size_kb in [1, 10, 100].iter() {
        let test_message = create_test_message(*size_kb);

        group.throughput(Throughput::Bytes(test_message.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("buffer_with_message", size_kb),
            &test_message,
            |b, msg| {
                b.iter_batched(
                    || {
                        let config = BufferConfig {
                            max_message_size: 10 * 1024 * 1024,
                            read_buffer_capacity: 8192,
                            write_buffer_capacity: 8192,
                            buffer_pool_size: 10,
                            pool_timeout: Duration::from_secs(30),
                            enable_zero_copy: true,
                            backpressure_threshold: 512 * 1024,
                        };
                        BufferManager::new(config)
                    },
                    |manager| {
                        rt.block_on(async {
                            let buffer = manager.acquire_read_buffer().await.unwrap();
                            black_box((buffer, msg.len()));
                        });
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark buffer acquisition performance  
fn benchmark_buffer_acquisition(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("buffer_acquisition");

    group.bench_function("acquire_read_buffer", |b| {
        b.iter_batched(
            || {
                let config = BufferConfig {
                    max_message_size: 1024 * 1024,
                    read_buffer_capacity: 8192,
                    write_buffer_capacity: 8192,
                    buffer_pool_size: 10,
                    pool_timeout: Duration::from_secs(30),
                    enable_zero_copy: true,
                    backpressure_threshold: 512 * 1024,
                };
                BufferManager::new(config)
            },
            |manager| {
                rt.block_on(async {
                    let buffer = manager.acquire_read_buffer().await.unwrap();
                    black_box(buffer);
                });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark pool size variations
fn benchmark_pool_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_sizes");

    // Test different pool sizes
    for pool_size in [1, 5, 10, 25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("buffer_manager_creation", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| {
                    let config = BufferConfig {
                        max_message_size: 1024 * 1024,
                        read_buffer_capacity: 8192,
                        write_buffer_capacity: 8192,
                        buffer_pool_size: size,
                        pool_timeout: Duration::from_secs(30),
                        enable_zero_copy: true,
                        backpressure_threshold: 512 * 1024,
                    };
                    black_box(BufferManager::new(config));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark STDIO transport creation
fn benchmark_stdio_transport_creation(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("stdio_transport_creation");

    group.bench_function("transport_new", |b| {
        b.iter_batched(
            || (),
            |_| {
                rt.block_on(async {
                    // Note: This will create a transport, but we won't actually use STDIO
                    // since benchmarks can't interact with real stdin/stdout
                    let result = StdioTransport::new().await;
                    // Explicitly handle the Result to satisfy must_use
                    let _ = black_box(result);
                });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("transport_with_custom_size", |b| {
        b.iter_batched(
            || (),
            |_| {
                rt.block_on(async {
                    let result = StdioTransport::with_max_message_size(1024 * 1024).await;
                    // Explicitly handle the Result to satisfy must_use
                    let _ = black_box(result);
                });
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark message serialization for transport
fn benchmark_transport_message_preparation(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport_message_prep");

    // Test message preparation for different sizes
    for size_kb in [1, 10, 100].iter() {
        let test_message = create_test_message(*size_kb);
        let message_bytes = test_message.as_bytes();

        group.throughput(Throughput::Bytes(message_bytes.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("message_to_bytes", size_kb),
            &test_message,
            |b, msg| {
                b.iter(|| {
                    // Simulate message preparation for transport
                    let bytes = msg.as_bytes();
                    // Add newline for STDIO transport framing
                    let mut framed = Vec::with_capacity(bytes.len() + 1);
                    framed.extend_from_slice(bytes);
                    framed.push(b'\n');
                    black_box(framed);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark transport buffer operations
fn benchmark_transport_buffer_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("transport_buffer_ops");

    // Test buffer operations that would be used in transport
    let test_messages: Vec<String> = (0..100).map(|_| create_test_message(1)).collect();

    group.bench_function("batch_message_framing", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            for msg in &test_messages {
                buffer.extend_from_slice(msg.as_bytes());
                buffer.push(b'\n');
            }
            black_box(buffer);
        });
    });

    group.bench_function("message_parsing", |b| {
        // Create a buffer with multiple newline-delimited messages
        let mut test_buffer = Vec::new();
        for msg in &test_messages {
            test_buffer.extend_from_slice(msg.as_bytes());
            test_buffer.push(b'\n');
        }

        b.iter(|| {
            let mut messages = Vec::new();
            let mut start = 0;

            for (pos, &byte) in test_buffer.iter().enumerate() {
                if byte == b'\n' {
                    if pos > start {
                        let message = &test_buffer[start..pos];
                        messages.push(message.to_vec());
                    }
                    start = pos + 1;
                }
            }
            black_box(messages);
        });
    });

    group.finish();
}

criterion_group!(
    transport_benches,
    benchmark_buffer_config,
    benchmark_buffer_manager_setup,
    benchmark_buffer_sizes,
    benchmark_message_throughput,
    benchmark_buffer_acquisition,
    benchmark_pool_sizes,
    benchmark_stdio_transport_creation,
    benchmark_transport_message_preparation,
    benchmark_transport_buffer_ops
);

criterion_main!(transport_benches);

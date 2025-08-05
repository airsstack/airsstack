//! Simple correlation performance benchmarks focused on core operations
//!
//! This benchmark suite measures the performance of correlation manager operations
//! with conservative memory usage to prevent system exhaustion.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Runtime;

use airs_mcp::correlation::manager::{CorrelationConfig, CorrelationManager};
use chrono::TimeDelta;

/// Create a simple runtime for async benchmarks
fn create_runtime() -> Runtime {
    Runtime::new().expect("Failed to create Tokio runtime")
}

/// Benchmark basic configuration creation with minimal overhead
fn benchmark_correlation_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("correlation_config");

    group.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = CorrelationConfig {
                default_timeout: TimeDelta::seconds(30),
                cleanup_interval: Duration::from_secs(10),
                max_pending_requests: 100,
                enable_tracing: false,
            };
            black_box(config)
        });
    });

    group.finish();
}

/// Benchmark manager creation with conservative settings
fn benchmark_manager_setup(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("correlation_manager_setup");

    group.bench_function("manager_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = CorrelationConfig {
                    max_pending_requests: 10,
                    enable_tracing: false,
                    cleanup_interval: Duration::from_secs(3600), // Disable cleanup
                    ..Default::default()
                };
                let manager = CorrelationManager::new_without_cleanup(config)
                    .await
                    .unwrap();
                black_box(manager)
            })
        });
    });

    group.finish();
}

/// Benchmark request registration with minimal memory usage
fn benchmark_request_registration(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("request_registration");

    for request_count in [1, 3, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("register_requests", request_count),
            request_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        rt.block_on(async {
                            let config = CorrelationConfig {
                                max_pending_requests: 10,
                                enable_tracing: false,
                                cleanup_interval: Duration::from_secs(3600),
                                ..Default::default()
                            };
                            CorrelationManager::new_without_cleanup(config)
                                .await
                                .unwrap()
                        })
                    },
                    |manager| {
                        rt.block_on(async {
                            let mut request_ids = Vec::with_capacity(count);

                            // Register minimal requests
                            for i in 0..count {
                                let payload = json!({"id": i});
                                let (request_id, receiver) = manager
                                    .register_request(Some(TimeDelta::seconds(5)), payload)
                                    .await
                                    .unwrap();
                                request_ids.push(request_id);
                                drop(receiver); // Free memory immediately
                            }

                            black_box(request_ids);
                        })
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark pending operations with minimal load
fn benchmark_pending_operations(c: &mut Criterion) {
    let rt = create_runtime();
    let mut group = c.benchmark_group("pending_operations");

    group.bench_function("pending_count_check", |b| {
        b.iter_batched(
            || {
                rt.block_on(async {
                    let config = CorrelationConfig {
                        max_pending_requests: 5,
                        enable_tracing: false,
                        cleanup_interval: Duration::from_secs(3600),
                        ..Default::default()
                    };
                    let manager = CorrelationManager::new_without_cleanup(config)
                        .await
                        .unwrap();

                    // Add a few requests
                    for i in 0..3 {
                        let payload = json!({"test": i});
                        let (_id, receiver) = manager
                            .register_request(Some(TimeDelta::seconds(5)), payload)
                            .await
                            .unwrap();
                        drop(receiver);
                    }

                    manager
                })
            },
            |manager| {
                rt.block_on(async {
                    let count = manager.pending_count().await;
                    black_box(count);
                })
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_correlation_config,
    benchmark_manager_setup,
    benchmark_request_registration,
    benchmark_pending_operations
);
criterion_main!(benches);

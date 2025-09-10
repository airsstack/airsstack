//! Message Processing Performance Benchmarks
//!
//! This module provides comprehensive benchmarks for JSON-RPC message processing
//! performance, measuring latency and throughput across different scenarios.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::json;

use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcNotification, JsonRpcRequest, RequestId};

/// Create a test request with a payload of the specified size (in KB)
fn create_test_request(size_kb: usize) -> JsonRpcRequest {
    let payload_size = size_kb * 1024;
    let data = "x".repeat(payload_size);

    JsonRpcRequest::new(
        "test_method",
        Some(json!({
            "data": data,
            "size_kb": size_kb,
            "timestamp": chrono::Utc::now()
        })),
        RequestId::new_string(format!("req_{size_kb}")),
    )
}

/// Create a test notification with a payload of the specified size (in KB)
fn create_test_notification(size_kb: usize) -> JsonRpcNotification {
    let payload_size = size_kb * 1024;
    let data = "y".repeat(payload_size);

    JsonRpcNotification::new(
        "test_notification",
        Some(json!({
            "data": data,
            "size_kb": size_kb,
            "timestamp": chrono::Utc::now()
        })),
    )
}

/// Benchmark JSON-RPC message serialization performance
fn benchmark_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");

    // Test different message sizes: 1KB, 10KB, 100KB, 1MB
    for size_kb in [1, 10, 100, 1000].iter() {
        let request = create_test_request(*size_kb);
        let notification = create_test_notification(*size_kb);

        group.throughput(Throughput::Bytes((*size_kb * 1024) as u64));

        // Benchmark request serialization
        group.bench_with_input(
            BenchmarkId::new("request_to_json", size_kb),
            size_kb,
            |b, &_size| {
                b.iter(|| black_box(request.to_json()).unwrap());
            },
        );

        // Benchmark notification serialization
        group.bench_with_input(
            BenchmarkId::new("notification_to_json", size_kb),
            size_kb,
            |b, &_size| {
                b.iter(|| black_box(notification.to_json()).unwrap());
            },
        );

        // Benchmark binary serialization
        group.bench_with_input(
            BenchmarkId::new("request_to_bytes", size_kb),
            size_kb,
            |b, &_size| {
                b.iter(|| black_box(request.to_bytes()).unwrap());
            },
        );
    }

    group.finish();
}

/// Benchmark JSON-RPC message deserialization performance
fn benchmark_message_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_deserialization");

    // Pre-serialize test messages for deserialization benchmarks
    let mut serialized_requests = Vec::new();
    let mut serialized_notifications = Vec::new();
    let mut serialized_bytes = Vec::new();

    for size_kb in [1, 10, 100, 1000].iter() {
        let request = create_test_request(*size_kb);
        let notification = create_test_notification(*size_kb);

        serialized_requests.push((*size_kb, request.to_json().unwrap()));
        serialized_notifications.push((*size_kb, notification.to_json().unwrap()));
        serialized_bytes.push((*size_kb, request.to_bytes().unwrap()));
    }

    // Benchmark request deserialization
    for (size_kb, json_str) in &serialized_requests {
        group.throughput(Throughput::Bytes((*size_kb * 1024) as u64));

        group.bench_with_input(
            BenchmarkId::new("request_from_json", size_kb),
            size_kb,
            |b, &_size| {
                b.iter(|| black_box(JsonRpcRequest::from_json(json_str)).unwrap());
            },
        );
    }

    // Benchmark notification deserialization
    for (size_kb, json_str) in &serialized_notifications {
        group.bench_with_input(
            BenchmarkId::new("notification_from_json", size_kb),
            size_kb,
            |b, &_size| {
                b.iter(|| black_box(JsonRpcNotification::from_json(json_str)).unwrap());
            },
        );
    }

    // Benchmark binary deserialization
    for (size_kb, bytes) in &serialized_bytes {
        group.bench_with_input(
            BenchmarkId::new("request_from_json_bytes", size_kb),
            size_kb,
            |b, &_size| {
                b.iter(|| black_box(JsonRpcRequest::from_json_bytes(bytes)).unwrap());
            },
        );
    }

    group.finish();
}

/// Benchmark round-trip serialization/deserialization performance
fn benchmark_round_trip_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip_processing");

    for size_kb in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Bytes((*size_kb * 1024) as u64));

        group.bench_with_input(
            BenchmarkId::new("request_round_trip", size_kb),
            size_kb,
            |b, &size| {
                b.iter(|| {
                    let request = create_test_request(size);
                    let json = black_box(request.to_json()).unwrap();
                    let parsed = black_box(JsonRpcRequest::from_json(&json)).unwrap();
                    black_box(parsed);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("notification_round_trip", size_kb),
            size_kb,
            |b, &size| {
                b.iter(|| {
                    let notification = create_test_notification(size);
                    let json = black_box(notification.to_json()).unwrap();
                    let parsed = black_box(JsonRpcNotification::from_json(&json)).unwrap();
                    black_box(parsed);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("binary_round_trip", size_kb),
            size_kb,
            |b, &size| {
                b.iter(|| {
                    let request = create_test_request(size);
                    let bytes = black_box(request.to_bytes()).unwrap();
                    let parsed = black_box(JsonRpcRequest::from_json_bytes(&bytes)).unwrap();
                    black_box(parsed);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory efficiency of message operations
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    // Benchmark message creation overhead
    group.bench_function("request_creation_1kb", |b| {
        b.iter(|| {
            black_box(create_test_request(1));
        });
    });

    group.bench_function("request_creation_100kb", |b| {
        b.iter(|| {
            black_box(create_test_request(100));
        });
    });

    // Benchmark clone performance
    let request_1kb = create_test_request(1);
    let request_100kb = create_test_request(100);

    group.bench_function("request_clone_1kb", |b| {
        b.iter(|| {
            black_box(request_1kb.clone());
        });
    });

    group.bench_function("request_clone_100kb", |b| {
        b.iter(|| {
            black_box(request_100kb.clone());
        });
    });

    group.finish();
}

/// Benchmark batch processing performance
fn benchmark_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    for batch_size in [10, 100, 1000].iter() {
        let requests: Vec<JsonRpcRequest> = (0..*batch_size)
            .map(|i| {
                JsonRpcRequest::new(
                    "batch_method",
                    Some(json!({"index": i, "data": "test"})),
                    RequestId::new_number(i as i64),
                )
            })
            .collect();

        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("serialize_batch", batch_size),
            batch_size,
            |b, &_size| {
                b.iter(|| {
                    let json_results: Vec<String> =
                        requests.iter().map(|req| req.to_json().unwrap()).collect();
                    black_box(json_results);
                });
            },
        );

        // Pre-serialize for deserialization benchmark
        let serialized_batch: Vec<String> =
            requests.iter().map(|req| req.to_json().unwrap()).collect();

        group.bench_with_input(
            BenchmarkId::new("deserialize_batch", batch_size),
            batch_size,
            |b, &_size| {
                b.iter(|| {
                    let parsed_results: Vec<JsonRpcRequest> = serialized_batch
                        .iter()
                        .map(|json| JsonRpcRequest::from_json(json).unwrap())
                        .collect();
                    black_box(parsed_results);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    message_processing_benches,
    benchmark_message_serialization,
    benchmark_message_deserialization,
    benchmark_round_trip_processing,
    benchmark_memory_efficiency,
    benchmark_batch_processing
);

criterion_main!(message_processing_benches);

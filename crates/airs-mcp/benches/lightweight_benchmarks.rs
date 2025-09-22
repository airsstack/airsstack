// Resource-conscious benchmark suite for airs-mcp v0.2.0
//
// This benchmark suite is designed for limited resource environments.
// Benchmarks are lightweight and focus on essential performance characteristics.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Runtime;

use airs_mcp::protocol::message::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification, RequestId};

// Lightweight JSON-RPC serialization benchmarks
fn bench_jsonrpc_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsonrpc_serialization");
    group.warm_up_time(Duration::from_millis(100)); // Minimal warm-up
    group.measurement_time(Duration::from_millis(500)); // Short measurement
    group.sample_size(50); // Small sample size for resource constraints
    
    // Simple request serialization
    let simple_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        method: "ping".to_string(),
        params: Some(json!({})),
    };
    
    group.bench_function("simple_request_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&simple_request)).unwrap();
        })
    });
    
    let simple_request_json = serde_json::to_string(&simple_request).unwrap();
    group.bench_function("simple_request_deserialize", |b| {
        b.iter(|| {
            let _request: JsonRpcRequest = serde_json::from_str(black_box(&simple_request_json)).unwrap();
        })
    });
    
    // Simple response serialization
    let simple_response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::new_number(1)),
        result: Some(json!({"status": "ok"})),
        error: None,
    };
    
    group.bench_function("simple_response_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&simple_response)).unwrap();
        })
    });
    
    // Notification serialization (lightweight)
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "notification".to_string(),
        params: Some(json!({"message": "test"})),
    };
    
    group.bench_function("notification_serialize", |b| {
        b.iter(|| {
            let _json = serde_json::to_string(black_box(&notification)).unwrap();
        })
    });
    
    group.finish();
}

// Basic memory allocation patterns
fn bench_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_creation");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(500));
    group.sample_size(50);
    
    group.bench_function("request_creation", |b| {
        b.iter(|| {
            let _request = JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: RequestId::new_number(black_box(1)),
                method: "test_method".to_string(),
                params: Some(json!({"param": "value"})),
            };
        })
    });
    
    group.bench_function("response_creation", |b| {
        b.iter(|| {
            let _response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: Some(RequestId::new_number(black_box(1))),
                result: Some(json!({"result": "success"})),
                error: None,
            };
        })
    });
    
    group.finish();
}

// Lightweight payload size benchmarks
fn bench_payload_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_sizes");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(300)); // Even shorter for larger payloads
    group.sample_size(25); // Smaller sample for resource conservation
    
    // Test different payload sizes (but keep them reasonable for limited resources)
    let sizes = vec![
        ("small", json!({"message": "hello"})),
        ("medium", json!({"data": "x".repeat(100)})), // 100 chars
        ("large", json!({"content": "x".repeat(1000)})), // 1KB
    ];
    
    for (size_name, payload) in sizes {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::new_number(1),
            method: "test".to_string(),
            params: Some(payload),
        };
        
        group.bench_with_input(
            BenchmarkId::new("serialize", size_name),
            &request,
            |b, request| {
                b.iter(|| {
                    let _json = serde_json::to_string(black_box(request)).unwrap();
                })
            },
        );
    }
    
    group.finish();
}

// Basic async operation benchmark (minimal)
fn bench_async_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_operations");
    group.warm_up_time(Duration::from_millis(100));
    group.measurement_time(Duration::from_millis(300));
    group.sample_size(25);
    
    group.bench_function("simple_async_task", |b| {
        let rt = Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                // Simulate minimal async work
                tokio::task::yield_now().await;
                black_box(42)
            })
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_jsonrpc_serialization,
    bench_message_creation,
    bench_payload_sizes,
    bench_async_operations
);

criterion_main!(benches);
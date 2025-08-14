//! HTTP Client Transport Usage Example
//!
//! This example demonstrates how to use the HTTP Client Transport
//! for JSON-RPC communication over HTTP.

use airs_mcp::transport::http::{HttpClientTransport, HttpTransportConfig};
use airs_mcp::transport::Transport;
use reqwest::Url;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 HTTP Client Transport Example");

    // Create HTTP transport configuration
    let config = HttpTransportConfig::new()
        .bind_address("127.0.0.1:8080".parse()?)
        .max_connections(1000)
        .request_timeout(Duration::from_secs(30))
        .enable_buffer_pool()
        .buffer_pool_size(100);

    println!("📋 Configuration:");
    println!("   Bind Address: {}", config.bind_address);
    println!("   Max Connections: {}", config.max_connections);
    println!("   Request Timeout: {:?}", config.request_timeout);
    println!(
        "   Buffer Pool Enabled: {}",
        matches!(
            config.parser.optimization_strategy,
            airs_mcp::transport::http::OptimizationStrategy::BufferPool(_)
        )
    );

    // Create transport instance
    let mut transport = HttpClientTransport::new(config);

    // Set target server (using httpbin.org for demonstration)
    let target_url = Url::parse("https://httpbin.org/post")?;
    transport.set_target(target_url.clone());
    transport.set_session_id("example-session-123".to_string());

    println!("🎯 Target: {target_url}");

    // Create a JSON-RPC request message
    let json_rpc_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "example.hello",
        "params": {
            "name": "World",
            "message": "Hello from HTTP Transport!"
        },
        "id": 1
    });

    let request_bytes = serde_json::to_vec(&json_rpc_request)?;
    println!("📤 Sending JSON-RPC request: {json_rpc_request}");

    // Send the request
    match transport.send(&request_bytes).await {
        Ok(()) => {
            println!("✅ Request sent successfully!");

            // Try to receive the response
            match transport.receive().await {
                Ok(response_bytes) => {
                    let response_str = String::from_utf8_lossy(&response_bytes);
                    println!("📥 Received response:");

                    // Pretty print JSON if possible
                    if let Ok(json_value) =
                        serde_json::from_slice::<serde_json::Value>(&response_bytes)
                    {
                        println!("{}", serde_json::to_string_pretty(&json_value)?);
                    } else {
                        println!("{response_str}");
                    }
                }
                Err(e) => {
                    println!("⚠️  No response received (expected for client-only transport): {e}");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to send request: {e}");
            println!("💡 Note: This might fail if httpbin.org is unreachable or if there are network issues.");
        }
    }

    // Get buffer statistics if available
    if let Some(stats) = transport.buffer_stats() {
        println!("📊 Buffer Pool Statistics:");
        println!("   Max Buffers: {}", stats.max_buffers);
        println!("   Available Buffers: {}", stats.available_buffers);
        println!("   Buffer Size: {} bytes", stats.buffer_size);
        println!("   Adaptive Sizing: {}", stats.adaptive_sizing);
        println!("   Pool Utilization: {:.1}%", stats.utilization());
    }

    // Clean shutdown
    transport.close().await?;
    println!("🏁 Transport closed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_transport_configuration() {
        // This test ensures the example configuration is valid
        let config = HttpTransportConfig::new()
            .bind_address("127.0.0.1:8080".parse().unwrap())
            .max_connections(1000)
            .request_timeout(Duration::from_secs(30))
            .enable_buffer_pool()
            .buffer_pool_size(100);

        let transport = HttpClientTransport::new(config);

        assert_eq!(
            transport.config().bind_address.to_string(),
            "127.0.0.1:8080"
        );
        assert_eq!(transport.config().max_connections, 1000);
        assert_eq!(transport.config().request_timeout, Duration::from_secs(30));
        assert!(transport.buffer_stats().is_some());
    }
}

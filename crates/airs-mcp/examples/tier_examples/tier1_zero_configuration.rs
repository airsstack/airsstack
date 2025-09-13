//! Tier 1: Zero Configuration Example
//!
//! This example demonstrates the simplest possible usage pattern for HTTP transport
//! with AxumHttpServer. Ideal for beginners who want to get started quickly without
//! any configuration complexity.
//!
//! # Key Features
//! - Zero configuration required
//! - Uses sensible defaults for everything
//! - Minimal code to get running
//! - Perfect for development and testing

use std::error::Error;
use std::net::SocketAddr;

use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
use airs_mcp::transport::adapters::http::{HttpTransport, HttpTransportBuilder};

/// Tier 1: Zero Configuration - The simplest possible usage
///
/// This pattern is perfect for:
/// - Quick prototyping
/// - Development environments
/// - Learning MCP fundamentals
/// - Testing scenarios
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Tier 1: Zero Configuration HTTP Transport Example");

    // Tier 1: Absolute simplest usage - everything uses defaults
    let _transport = HttpTransportBuilder::<AxumHttpServer>::with_default()?
        .bind("127.0.0.1:8080".parse::<SocketAddr>()?)
        .await?
        .build()
        .await?;

    println!("✅ HTTP Transport created with zero configuration");
    println!("   - Server type: AxumHttpServer (default)");
    println!("   - Authentication: None (default)");
    println!("   - Binding: 127.0.0.1:8080");
    println!("   - Configuration: All defaults");

    // The transport is ready to use with McpServer
    println!("🎯 Transport ready for McpServer integration");

    // In a real application, you would:
    // let server = McpServer::new(transport);
    // server.start().await?;

    Ok(())
}

/// Alternative zero-configuration pattern using builder defaults
///
/// This shows an alternative way to achieve the same result
/// using explicit builder construction with defaults.
#[allow(dead_code)]
async fn alternative_zero_config() -> Result<HttpTransport<AxumHttpServer>, Box<dyn Error>> {
    // Alternative: Explicit builder with default engine
    let transport = HttpTransportBuilder::new(AxumHttpServer::default())
        .bind("127.0.0.1:8080".parse::<SocketAddr>()?)
        .await?
        .build()
        .await?;

    Ok(transport)
}

/// Test helper to verify the tier 1 pattern works
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tier1_zero_configuration() {
        let result = HttpTransportBuilder::<AxumHttpServer>::with_default();
        assert!(result.is_ok(), "Tier 1 zero configuration should work");
    }

    #[tokio::test]
    async fn test_alternative_zero_config() {
        let result = alternative_zero_config().await;
        assert!(result.is_ok(), "Alternative zero configuration should work");
    }
}

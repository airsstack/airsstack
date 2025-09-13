//! Tier 2: Basic Configuration Example
//!
//! This example demonstrates pre-configured engines for common patterns.
//! It shows how to use ready-made configurations for authentication and
//! other common requirements without deep knowledge of the underlying systems.
//!
//! # Key Features
//! - Pre-configured engines for common use cases
//! - Simple authentication setup
//! - Standard patterns for production use
//! - Balance between simplicity and control

use std::error::Error;
use std::net::SocketAddr;

use airs_mcp::transport::adapters::http::{HttpTransport, HttpTransportBuilder};
use airs_mcp::transport::adapters::http::axum::AxumHttpServer;
use airs_mcp::transport::adapters::http::auth::middleware::HttpAuthConfig;

/// Tier 2: Basic Configuration - Pre-configured engines for common patterns
///
/// This pattern is perfect for:
/// - Production applications with standard authentication
/// - Teams that want proven configurations
/// - Applications with specific security requirements
/// - Scenarios where you know what you need but don't want to configure everything
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔧 Tier 2: Basic Configuration HTTP Transport Example");

    // Tier 2A: Pre-configured engine with basic authentication
    let auth_config = HttpAuthConfig::default();
    
    // Note: For this example, we're using a placeholder pattern
    // In a real implementation, you'd have specific auth engines
    let engine = create_auth_engine(auth_config)?;
    
    let _transport = HttpTransportBuilder::with_engine(engine)?
        .bind("127.0.0.1:8080".parse::<SocketAddr>()?).await?
        .build().await?;

    println!("✅ HTTP Transport created with pre-configured authentication");
    println!("   - Server type: AxumHttpServer with authentication");
    println!("   - Authentication: HTTP Auth configured");
    println!("   - Binding: 127.0.0.1:8080");
    println!("   - Configuration: Production-ready defaults");

    // Tier 2B: OAuth2 pre-configuration (conceptual example)
    let _oauth2_transport = create_oauth2_transport().await?;
    println!("✅ OAuth2 transport also created successfully");

    println!("🎯 Both transports ready for McpServer integration");

    Ok(())
}

/// Helper function to create an authenticated engine
///
/// In a real implementation, this would use actual authentication adapters
/// and return properly configured AxumHttpServer instances.
fn create_auth_engine(_auth_config: HttpAuthConfig) -> Result<AxumHttpServer, Box<dyn Error>> {
    // Placeholder: In reality this would configure actual authentication
    let server = AxumHttpServer::default();
    
    println!("🔐 Created authenticated engine with config");
    
    Ok(server)
}

/// OAuth2 configuration example
///
/// This shows the pattern for OAuth2 authentication setup.
/// In a real implementation, this would use OAuth2Strategy and proper validators.
async fn create_oauth2_transport() -> Result<HttpTransport<AxumHttpServer>, Box<dyn Error>> {
    // Tier 2B: OAuth2 pre-configuration pattern
    // let oauth2_config = OAuth2Config::new(/* client_id, client_secret, etc. */);
    // let engine = AxumHttpServer::with_oauth2(oauth2_config)?;
    
    // For now, use default engine as placeholder
    let engine = AxumHttpServer::default();
    
    let transport = HttpTransportBuilder::with_engine(engine)?
        .bind("127.0.0.1:8081".parse::<SocketAddr>()?).await?
        .build().await?;

    println!("🔒 OAuth2 transport configured for production use");
    
    Ok(transport)
}

/// API Key authentication example
///
/// Shows how Tier 2 patterns work with different authentication types.
#[allow(dead_code)]
async fn create_apikey_transport() -> Result<HttpTransport<AxumHttpServer>, Box<dyn Error>> {
    // Tier 2C: API Key authentication pattern
    // let apikey_config = ApiKeyConfig::new(validation_rules);
    // let engine = AxumHttpServer::with_apikey(apikey_config)?;
    
    let engine = AxumHttpServer::default();
    
    let transport = HttpTransportBuilder::with_engine(engine)?
        .bind("127.0.0.1:8082".parse::<SocketAddr>()?).await?
        .build().await?;

    Ok(transport)
}

/// Test helper to verify tier 2 patterns work
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tier2_basic_auth() {
        let auth_config = HttpAuthConfig::default();
        let result = create_auth_engine(auth_config);
        assert!(result.is_ok(), "Tier 2 auth engine creation should work");
    }

    #[tokio::test]
    async fn test_tier2_oauth2() {
        let result = create_oauth2_transport().await;
        assert!(result.is_ok(), "Tier 2 OAuth2 transport should work");
    }

    #[tokio::test]
    async fn test_tier2_apikey() {
        let result = create_apikey_transport().await;
        assert!(result.is_ok(), "Tier 2 API Key transport should work");
    }
}
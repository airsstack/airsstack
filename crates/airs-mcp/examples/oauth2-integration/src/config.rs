//! OAuth2 Configuration Module
//!
//! This module handles the creation and management of OAuth2 configuration
//! for the MCP server integration.

use std::time::Duration;

use url::Url;

use airs_mcp::oauth2::config::{CacheConfig, OAuth2Config, ValidationConfig};

/// Create OAuth2 configuration for the MCP server
pub fn create_oauth2_config() -> Result<OAuth2Config, Box<dyn std::error::Error>> {
    let jwks_url = Url::parse("http://localhost:3004/.well-known/jwks.json")?;

    Ok(OAuth2Config::builder()
        .jwks_url(jwks_url)
        .audience("mcp-server".to_string())
        .issuer("https://example.com".to_string())
        .validation_config(ValidationConfig {
            require_exp: true,
            require_aud: true,
            require_iss: true,
            validate_nbf: true,
            leeway: Duration::from_secs(60),
            algorithms: vec!["RS256".to_string()],
        })
        .cache_config(CacheConfig {
            jwks_cache_ttl: Duration::from_secs(300),
            jwks_cache_max_size: 10,
            token_cache_ttl: Duration::from_secs(60),
            token_cache_max_size: 100,
        })
        .build()?)
}

/// OAuth2 configuration constants
pub mod constants {
    #![allow(dead_code)]

    /// Default audience for MCP server tokens
    pub const DEFAULT_AUDIENCE: &str = "mcp-server";

    /// Default issuer for test tokens
    pub const DEFAULT_ISSUER: &str = "https://example.com";

    /// Default JWKS URL for testing
    pub const DEFAULT_JWKS_URL: &str = "http://localhost:3004/.well-known/jwks.json";

    /// Default MCP server address
    pub const DEFAULT_MCP_ADDRESS: &str = "127.0.0.1:3001";

    /// Default JWKS server address
    pub const DEFAULT_JWKS_ADDRESS: &str = "127.0.0.1:3004";
}

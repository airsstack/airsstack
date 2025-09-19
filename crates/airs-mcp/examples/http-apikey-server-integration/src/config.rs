//! Server Configuration
//!
//! Configuration management for the HTTP API Key MCP server.

/// Server configuration structure
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server port
    pub port: u16,
    /// Enable development mode with pre-configured API keys
    pub dev_mode: bool,
    /// Custom API key (optional)
    pub custom_api_key: Option<String>,
}

impl ServerConfig {
    /// Create new server configuration
    pub fn new(port: u16, dev_mode: bool, custom_api_key: Option<String>) -> Self {
        Self {
            port,
            dev_mode,
            custom_api_key,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            dev_mode: true,
            custom_api_key: None,
        }
    }
}
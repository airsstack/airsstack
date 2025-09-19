//! Client Configuration
//!
//! Configuration management for the HTTP API Key MCP client.

// Layer 1: Standard library imports
use std::env;
use std::time::Duration;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (none for config)

/// Authentication methods supported by the HTTP client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    /// X-API-Key header authentication
    XApiKey,
    /// Authorization Bearer token authentication
    Bearer,
    /// Query parameter authentication (?api_key=...)
    QueryParameter,
}

impl Default for AuthenticationMethod {
    fn default() -> Self {
        Self::XApiKey
    }
}

/// Configuration for the HTTP MCP client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Server endpoint URL
    pub server_url: String,

    /// API key for authentication
    pub api_key: String,

    /// Authentication method to use
    pub auth_method: AuthenticationMethod,

    /// Timeout for individual requests
    pub timeout: Duration,

    /// Enable mock server mode for testing
    pub mock_mode: bool,

    /// Mock server port (when in mock mode)
    pub mock_server_port: u16,

    /// Use production server (Phase 4.4 server)
    pub use_production_server: bool,

    /// Enable development mode with additional logging
    pub dev_mode: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:3000".to_string(),
            api_key: "".to_string(),
            auth_method: AuthenticationMethod::default(),
            timeout: Duration::from_secs(30),
            mock_mode: false,
            mock_server_port: 3001,
            use_production_server: false,
            dev_mode: false,
        }
    }
}

impl ClientConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Server configuration
        if let Ok(url) = env::var("MCP_SERVER_URL") {
            config.server_url = url;
        }

        // Authentication configuration
        if let Ok(key) = env::var("MCP_API_KEY") {
            config.api_key = key;
        }

        if let Ok(method) = env::var("MCP_AUTH_METHOD") {
            config.auth_method = match method.to_lowercase().as_str() {
                "bearer" => AuthenticationMethod::Bearer,
                "query" | "query_parameter" => AuthenticationMethod::QueryParameter,
                _ => AuthenticationMethod::XApiKey,
            };
        }

        // Timeout configuration
        if let Ok(timeout_str) = env::var("MCP_TIMEOUT") {
            if let Ok(timeout_secs) = timeout_str.parse::<u64>() {
                config.timeout = Duration::from_secs(timeout_secs);
            }
        }

        // Mode configuration
        config.mock_mode = env::var("USE_MOCK").is_ok();
        config.use_production_server = env::var("USE_PRODUCTION").is_ok();
        config.dev_mode = env::var("DEV_MODE").is_ok();

        // Mock server port
        if let Ok(port_str) = env::var("MOCK_SERVER_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                config.mock_server_port = port;
            }
        }

        config
    }

    /// Create configuration for mock server testing
    pub fn for_mock_server() -> Self {
        Self {
            server_url: "http://127.0.0.1:3001".to_string(),
            api_key: "test-key-123".to_string(),
            auth_method: AuthenticationMethod::XApiKey,
            timeout: Duration::from_secs(10),
            mock_mode: true,
            mock_server_port: 3001,
            use_production_server: false,
            dev_mode: true,
        }
    }

    /// Create configuration for production server (Phase 4.4)
    pub fn for_production_server() -> Self {
        Self {
            server_url: "http://127.0.0.1:3000".to_string(),
            api_key: "dev-key-12345".to_string(), // Default Phase 4.4 dev key
            auth_method: AuthenticationMethod::XApiKey,
            timeout: Duration::from_secs(30),
            mock_mode: false,
            mock_server_port: 3001,
            use_production_server: true,
            dev_mode: true,
        }
    }

    /// Create configuration for different authentication methods
    #[allow(dead_code)]
    pub fn with_auth_method(mut self, method: AuthenticationMethod) -> Self {
        self.auth_method = method;
        self
    }

    /// Builder method to set API key
    #[allow(dead_code)]
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = api_key;
        self
    }

    /// Builder method to set server URL
    #[allow(dead_code)]
    pub fn with_server_url(mut self, url: String) -> Self {
        self.server_url = url;
        self
    }

    /// Builder method to set timeout
    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the authentication header name for the current auth method
    #[allow(dead_code)]
    pub fn auth_header_name(&self) -> &'static str {
        match self.auth_method {
            AuthenticationMethod::XApiKey => "X-API-Key",
            AuthenticationMethod::Bearer => "Authorization",
            AuthenticationMethod::QueryParameter => "", // Not applicable for headers
        }
    }

    /// Get the authentication header value for the current auth method
    #[allow(dead_code)]
    pub fn auth_header_value(&self) -> String {
        match self.auth_method {
            AuthenticationMethod::XApiKey => self.api_key.clone(),
            AuthenticationMethod::Bearer => format!("Bearer {}", self.api_key),
            AuthenticationMethod::QueryParameter => String::new(), // Not applicable for headers
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.server_url.is_empty() {
            return Err("Server URL cannot be empty".to_string());
        }

        if !self.server_url.starts_with("http://") && !self.server_url.starts_with("https://") {
            return Err("Server URL must start with http:// or https://".to_string());
        }

        if self.api_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }

        if self.timeout.as_secs() == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.mock_server_port == 0 {
            return Err("Mock server port must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get the server URL with query parameter authentication if applicable
    pub fn server_url_with_auth(&self) -> String {
        match self.auth_method {
            AuthenticationMethod::QueryParameter => {
                format!("{}?api_key={}", self.server_url, self.api_key)
            }
            _ => self.server_url.clone(),
        }
    }
}
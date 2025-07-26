# Configuration Management

```rust,ignore
// Hierarchical configuration with environment override support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub server: Option<ServerConfig>,
    pub client: Option<ClientConfig>,
    pub transport: TransportConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub max_connections: usize,
    pub request_timeout: Duration,
    pub enable_subscriptions: bool,
    pub tool_approval_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub connect_timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub sampling_approval_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub stdio: Option<StdioConfig>,
    pub http: Option<HttpConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_oauth21: bool,
    pub oauth_config: Option<OAuth21Config>,
    pub audit_logging: bool,
    pub strict_permissions: bool,
}

// Configuration loading with environment override
impl McpConfig {
    pub fn load() -> Result<Self, ConfigError> {
        // Load from multiple sources with precedence:
        // 1. Environment variables (highest)
        // 2. Config file
        // 3. Defaults (lowest)
        
        let mut config = Self::default();
        
        // Load from config file if present
        if let Ok(contents) = std::fs::read_to_string("mcp.toml") {
            let file_config: McpConfig = toml::from_str(&contents)?;
            config = config.merge(file_config);
        }
        
        // Override with environment variables
        config = config.override_from_env()?;
        
        config.validate()?;
        Ok(config)
    }
    
    fn override_from_env(mut self) -> Result<Self, ConfigError> {
        if let Ok(timeout) = std::env::var("MCP_REQUEST_TIMEOUT") {
            if let Some(ref mut server) = self.server {
                server.request_timeout = Duration::from_secs(timeout.parse()?);
            }
        }
        
        if let Ok(transport) = std::env::var("MCP_TRANSPORT") {
            self.transport.transport_type = transport.parse()?;
        }
        
        Ok(self)
    }
}
```

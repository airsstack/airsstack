//! HTTP API Key Server Implementation
//!
//! This module provides a complete HTTP server implementation with API key authentication
//! for MCP operations. It handles the HTTP transport layer, authentication middleware,
//! and server lifecycle management.

// Layer 1: Standard library imports
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

// Layer 2: Third-party crate imports
use tracing::info;

// Layer 3: Internal module imports
use airs_mcp::{
    authentication::strategies::apikey::{
        ApiKeyAuthData, ApiKeySource, ApiKeyStrategy, InMemoryApiKeyValidator,
    },
    authentication::{AuthContext, AuthMethod},
    integration::McpServer,
    transport::adapters::http::{
        auth::{apikey::ApiKeyStrategyAdapter, middleware::HttpAuthConfig},
        axum::AxumHttpServer,
        config::HttpTransportConfig,
        connection_manager::{HealthCheckConfig, HttpConnectionManager},
        HttpTransportBuilder,
    },
};

use crate::{config::ServerConfig, tools::McpHandlers};

/// HTTP API Key Server for MCP operations
///
/// This server provides a complete HTTP MCP implementation with API key authentication.
/// It handles the HTTP transport layer, authentication middleware, and MCP handlers.
pub struct HttpApiKeyServer {
    config: ServerConfig,
    socket_addr: SocketAddr,
}

impl HttpApiKeyServer {
    /// Create a new HTTP API Key server with the given configuration
    ///
    /// # Arguments
    /// * `config` - Server configuration including port, dev mode, and API keys
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn std::error::Error>>` - The configured server or error
    pub fn new(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_addr = format!("127.0.0.1:{}", config.port).parse()?;

        Ok(Self {
            config,
            socket_addr,
        })
    }

    /// Start the HTTP server with API key authentication
    ///
    /// This method sets up the complete HTTP transport stack including:
    /// - API key authentication middleware
    /// - HTTP transport configuration
    /// - Connection management
    /// - MCP handler registration
    ///
    /// # Arguments
    /// * `handlers` - MCP handlers for tool operations
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Success or error
    pub async fn start(&self, handlers: McpHandlers) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔧 Setting up HTTP API Key server...");

        // Create API key validator
        let api_key_validator = self.create_api_key_validator()?;
        let api_key_strategy = ApiKeyStrategy::new(api_key_validator);
        let api_key_adapter = ApiKeyStrategyAdapter::with_default_config(api_key_strategy);

        // Create HTTP authentication configuration
        let auth_config = HttpAuthConfig {
            include_error_details: true,
            auth_realm: "API Key MCP Server".to_string(),
            request_timeout_secs: 30,
            skip_paths: vec![
                "/health".to_string(),
                "/status".to_string(),
                "/metrics".to_string(),
            ],
        };

        info!("🔑 API key authentication configured");

        // Create HTTP transport configuration
        let http_config = HttpTransportConfig::new().bind_address(self.socket_addr);

        // Create connection manager
        let connection_manager = Arc::new(HttpConnectionManager::new(
            1000, // max connections
            HealthCheckConfig::default(),
        ));

        info!("🌐 HTTP transport configured for {}", self.socket_addr);

        // Create the API key-enabled MCP server using AxumHttpServer
        let mut engine = AxumHttpServer::from_parts(connection_manager, http_config)?
            .with_authentication(api_key_adapter, auth_config);

        // Register the MCP handlers
        engine.register_custom_mcp_handler(handlers);

        info!("📦 MCP handlers registered");

        // Build HTTP transport
        let transport = HttpTransportBuilder::with_engine(engine)?
            .bind(self.socket_addr)
            .await?
            .build()
            .await?;

        info!("🚀 HTTP transport built successfully");

        // Create MCP server
        let mcp_server = McpServer::new(transport);

        // Log server startup information
        self.log_startup_info();

        // Start the server
        info!("🎯 Starting MCP server...");

        // Start the MCP server - this starts the HTTP transport and returns immediately
        mcp_server
            .start()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        info!("✅ HTTP API Key server is now running and ready to accept requests!");

        // Keep the server alive until interrupted
        tokio::signal::ctrl_c()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        info!("🛑 HTTP API Key server received shutdown signal");
        info!("🎯 HTTP API Key server stopping...");
        Ok(())
    }

    /// Create API key validator with configured keys
    fn create_api_key_validator(
        &self,
    ) -> Result<InMemoryApiKeyValidator, Box<dyn std::error::Error>> {
        let mut api_keys = HashMap::new();

        // Add development keys if dev mode is enabled
        if self.config.dev_mode {
            let dev_context = AuthContext::new(
                AuthMethod::new("apikey"),
                ApiKeyAuthData {
                    key_id: "dev-user".to_string(),
                    source: ApiKeySource::Header("X-API-Key".to_string()),
                },
            );
            api_keys.insert("dev-key-123".to_string(), dev_context);

            let test_context = AuthContext::new(
                AuthMethod::new("apikey"),
                ApiKeyAuthData {
                    key_id: "test-user".to_string(),
                    source: ApiKeySource::AuthorizationBearer,
                },
            );
            api_keys.insert("test-key-456".to_string(), test_context);

            let demo_context = AuthContext::new(
                AuthMethod::new("apikey"),
                ApiKeyAuthData {
                    key_id: "demo-user".to_string(),
                    source: ApiKeySource::QueryParameter("api_key".to_string()),
                },
            );
            api_keys.insert("demo-key-789".to_string(), demo_context);
        }

        // Add custom API key if provided
        if let Some(ref custom_key) = self.config.custom_api_key {
            let custom_context = AuthContext::new(
                AuthMethod::new("apikey"),
                ApiKeyAuthData {
                    key_id: "custom-user".to_string(),
                    source: ApiKeySource::Header("X-API-Key".to_string()),
                },
            );
            api_keys.insert(custom_key.clone(), custom_context);
        }

        if api_keys.is_empty() {
            return Err(
                "No API keys configured. Enable dev_mode or provide a custom API key.".into(),
            );
        }

        info!("🔑 Configured {} API keys", api_keys.len());

        Ok(InMemoryApiKeyValidator::new(api_keys))
    }

    /// Log server startup information
    fn log_startup_info(&self) {
        info!("✅ HTTP API Key server configured successfully!");
        info!("🔑 API Keys configured:");
        if self.config.dev_mode {
            info!("   • Development keys: dev-key-123, test-key-456, demo-key-789");
        }
        if let Some(ref custom_key) = self.config.custom_api_key {
            info!("   • Custom key: {}", custom_key);
        }
        info!("");
        info!("🌐 Server listening on: http://{}", self.socket_addr);
        info!("🔐 Authentication methods supported:");
        info!("   • X-API-Key header: X-API-Key: your-key");
        info!("   • Authorization Bearer: Authorization: Bearer your-key");
        info!("   • Query parameter: ?api_key=your-key");
        info!("");
        info!("💡 Example usage:");
        info!("   curl -X POST http://{}/mcp \\", self.socket_addr);
        info!("     -H \"Content-Type: application/json\" \\");
        info!("     -H \"X-API-Key: dev-key-123\" \\");
        info!(
            "     -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"list_tools\",\"params\":{{}}}}'"
        );
        info!("");
    }
}

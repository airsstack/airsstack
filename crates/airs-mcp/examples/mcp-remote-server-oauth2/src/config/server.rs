//! Server Configuration for AirsStack MCP Integration
//!
//! This module provides configuration helpers for setting up the complete
//! AirsStack MCP server with OAuth2 authentication.

use std::sync::Arc;
use std::time::Duration;
use chrono::Duration as ChronoDuration;

// AirsStack MCP components
use airs_mcp::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};
use airs_mcp::correlation::manager::{CorrelationConfig, CorrelationManager};
use airs_mcp::oauth2::validator::{Jwt, Scope};
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
    StructuredLoggingHandler,
};
use airs_mcp::transport::adapters::http::{
    auth::oauth2::OAuth2StrategyAdapter,
    axum::{AxumHttpServer, McpHandlersBuilder},
    config::HttpTransportConfig,
    connection_manager::{HealthCheckConfig, HttpConnectionManager},
    session::{SessionConfig, SessionManager},
};

/// Configuration for the OAuth2 MCP server using AirsStack components
pub struct ServerConfig {
    pub connection_manager: Arc<HttpConnectionManager>,
    pub session_manager: Arc<SessionManager>,
    pub jsonrpc_processor: Arc<ConcurrentProcessor>,
    pub handlers: McpHandlersBuilder,
    pub transport_config: HttpTransportConfig,
}

impl ServerConfig {
    /// Create server configuration with AirsStack MCP components
    ///
    /// This sets up:
    /// - HttpConnectionManager for connection pooling
    /// - CorrelationManager for request correlation
    /// - SessionManager for session handling
    /// - ConcurrentProcessor for JSON-RPC processing
    /// - McpHandlersBuilder with providers (filesystem, tools, prompts)
    /// - HttpTransportConfig for the transport layer
    pub async fn new(temp_dir_path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        // Create server infrastructure using AirsStack components
        let connection_manager = Arc::new(HttpConnectionManager::new(
            1000,
            HealthCheckConfig::default(),
        ));

        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default()).await?
        );

        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        // Configure JSON-RPC processor for concurrent request handling
        let processor_config = ProcessorConfig {
            worker_count: 4,
            queue_capacity: 1000,
            max_batch_size: 10,
            processing_timeout: ChronoDuration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));

        // Create MCP handlers with AirsStack providers
        let handlers = McpHandlersBuilder::new()
            .with_resource_provider(Arc::new(
                FileSystemResourceProvider::new(&temp_dir_path.canonicalize()?)
                    .expect("Failed to create filesystem provider")
            ))
            .with_tool_provider(Arc::new(MathToolProvider::new()))
            .with_prompt_provider(Arc::new(CodeReviewPromptProvider::new()))
            .with_logging_handler(Arc::new(StructuredLoggingHandler::new()));

        // Create HTTP transport configuration
        let transport_config = HttpTransportConfig::new()
            .bind_address("127.0.0.1:3001".parse()?)
            .max_connections(1000)
            .request_timeout(Duration::from_secs(30))
            .session_timeout(Duration::from_secs(1800)) // 30 minutes
            .enable_buffer_pool()
            .buffer_pool_size(100);

        Ok(ServerConfig {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            handlers,
            transport_config,
        })
    }

    /// Create the OAuth2-enabled AxumHttpServer
    ///
    /// This is the main AirsStack MCP server with OAuth2 authentication
    pub async fn create_server(
        self,
        oauth2_setup: crate::auth::setup::OAuth2Setup,
    ) -> Result<AxumHttpServer<OAuth2StrategyAdapter<Jwt, Scope>>, Box<dyn std::error::Error>> {
        // Create the OAuth2-enabled MCP server using AirsStack components
        let server = AxumHttpServer::with_handlers(
            self.connection_manager,
            self.session_manager,
            self.jsonrpc_processor,
            self.handlers,
            self.transport_config,
        )
        .await?
        .with_authentication(oauth2_setup.strategy_adapter, oauth2_setup.auth_config);

        Ok(server)
    }
}

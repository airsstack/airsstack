//! Axum HTTP Server Implementation for MCP Transport
//!
//! This module provides a complete HTTP server implementation using Axum framework
//! for handling MCP JSON-RPC requests. It integrates with the connection manager,
//! session manager, and MCP server infrastructure for full protocol support.
//!
//! # Architecture
//!
//! This module has been refactored to follow SOLID principles with proper separation
//! of concerns. The implementation is now split across multiple focused modules:
//!
//! - `server`: Main HTTP server implementation
//! - `handlers`: HTTP endpoint handlers
//! - `mcp_handlers`: MCP protocol handlers management
//! - `mcp_operations`: MCP protocol operations

// Import the new modular architecture
#[path = "axum_impl/mod.rs"]
mod axum_impl;

// Re-export key types for backward compatibility
pub use axum_impl::{AxumHttpServer, McpHandlers, McpHandlersBuilder, ServerState};

#[cfg(test)]
mod tests {
    // Layer 1: Standard library imports
    use std::sync::Arc;

    // Layer 2: Third-party crate imports
    use uuid::Uuid;

    // Layer 3: Internal module imports
    use crate::base::jsonrpc::concurrent::{ConcurrentProcessor, ProcessorConfig};
    use crate::correlation::manager::{CorrelationConfig, CorrelationManager};
    use crate::transport::http::config::HttpTransportConfig;
    use crate::transport::http::connection_manager::{HealthCheckConfig, HttpConnectionManager};
    use crate::transport::http::session::{SessionConfig, SessionManager};

    use super::{AxumHttpServer, McpHandlers};

    async fn create_test_server() -> AxumHttpServer {
        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        AxumHttpServer::new_with_empty_handlers(
            connection_manager,
            session_manager,
            jsonrpc_processor,
            config,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_axum_server_creation() {
        let server = create_test_server().await;
        assert!(!server.is_bound());
    }

    #[tokio::test]
    async fn test_axum_server_bind() {
        let mut server = create_test_server().await;
        let addr = "127.0.0.1:0".parse().unwrap();

        server.bind(addr).await.unwrap();
        assert!(server.is_bound());
    }

    #[tokio::test]
    async fn test_router_creation() {
        use crate::integration::mcp::server::McpServerConfig;
        use crate::transport::http::axum_server::axum_impl::handlers::create_router;
        use crate::transport::http::axum_server::axum_impl::handlers::ServerState;

        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers: Arc::new(McpHandlers {
                resource_provider: None,
                tool_provider: None,
                prompt_provider: None,
                logging_handler: None,
                config: McpServerConfig::default(),
            }),
            config,
        };

        let router = create_router(state);

        // Router should be created successfully
        // Note: Testing actual routes would require more complex setup
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[tokio::test]
    async fn test_extract_session_from_headers() {
        use crate::integration::mcp::server::McpServerConfig;
        use crate::transport::http::axum_server::axum_impl::handlers::{
            extract_or_create_session, ServerState,
        };
        use axum::http::HeaderMap;

        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers: Arc::new(McpHandlers {
                resource_provider: None,
                tool_provider: None,
                prompt_provider: None,
                logging_handler: None,
                config: McpServerConfig::default(),
            }),
            config,
        };

        let peer_addr = "127.0.0.1:8080".parse().unwrap();
        let mut headers = HeaderMap::new();

        // Test with no session header - should create new session
        let session_id = extract_or_create_session(&state, &headers, peer_addr)
            .await
            .unwrap();
        assert!(session_id != Uuid::nil());

        // Test with invalid session header - should create new session
        headers.insert("X-Session-ID", "invalid-uuid".parse().unwrap());
        let session_id2 = extract_or_create_session(&state, &headers, peer_addr)
            .await
            .unwrap();
        assert!(session_id2 != Uuid::nil());
    }

    #[tokio::test]
    async fn test_process_jsonrpc_request() {
        use crate::base::jsonrpc::message::{JsonRpcRequest, RequestId};
        use crate::integration::mcp::server::McpServerConfig;
        use crate::shared::protocol::types::common::ProtocolVersion;
        use crate::transport::http::axum_server::axum_impl::handlers::{
            process_jsonrpc_request, ServerState,
        };

        // Test initialize method with valid MCP InitializeRequest
        let init_params = serde_json::json!({
            "protocolVersion": ProtocolVersion::current(),
            "capabilities": {
                "roots": {"listChanged": false},
                "sampling": {}
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        });

        let request =
            JsonRpcRequest::new("initialize", Some(init_params), RequestId::new_number(1));
        let session_id = Uuid::new_v4();

        // Create minimal state for testing
        let connection_manager =
            Arc::new(HttpConnectionManager::new(10, HealthCheckConfig::default()));
        let correlation_manager = Arc::new(
            CorrelationManager::new(CorrelationConfig::default())
                .await
                .unwrap(),
        );
        let session_manager = Arc::new(SessionManager::new(
            correlation_manager,
            SessionConfig::default(),
        ));

        let processor_config = ProcessorConfig {
            worker_count: 2,
            queue_capacity: 100,
            max_batch_size: 10,
            processing_timeout: chrono::Duration::seconds(30),
            enable_ordering: false,
            enable_backpressure: true,
        };
        let jsonrpc_processor = Arc::new(ConcurrentProcessor::new(processor_config));
        let config = HttpTransportConfig::new();

        let state = ServerState {
            connection_manager,
            session_manager,
            jsonrpc_processor,
            mcp_handlers: Arc::new(McpHandlers {
                resource_provider: None,
                tool_provider: None,
                prompt_provider: None,
                logging_handler: None,
                config: McpServerConfig::default(),
            }),
            config,
        };

        let result = process_jsonrpc_request(&state, session_id, request)
            .await
            .unwrap();

        // Should return initialize response with MCP protocol info
        assert_eq!(result["jsonrpc"], "2.0");
        assert_eq!(result["id"], 1);
        assert!(result["result"].is_object());
        assert!(result["result"]["protocolVersion"].is_string());
        assert!(result["result"]["capabilities"].is_object());
        assert!(result["result"]["serverInfo"].is_object());
    }
}

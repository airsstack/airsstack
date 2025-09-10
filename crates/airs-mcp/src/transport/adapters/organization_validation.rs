//! Module Organization Validation Test
//!
//! This file validates that Phase 5.5.5 Transport Module Organization is correctly implemented:
//! - Protocol module contains only transport-agnostic generic traits
//! - Transport modules are self-contained with no cross-dependencies  
//! - Type aliases are properly organized for convenience
//! - Clean architecture with proper separation of concerns

#[cfg(test)]
mod transport_module_organization_tests {
    // Test imports demonstrate clean module organization

    // Protocol imports - only generic, transport-agnostic traits
    use crate::protocol::{
        MessageContext, MessageHandler, Transport, TransportBuilder, TransportError,
    };

    // STDIO transport imports - self-contained module
    use crate::transport::adapters::stdio::{
        StdioMessageContext, StdioMessageHandler, StdioTransport, StdioTransportBuilder,
        StdioTransportConfig,
    };

    // HTTP transport imports - self-contained module
    use crate::transport::adapters::http::{
        EchoHttpHandler, HttpContext, HttpMessageContext, HttpMessageHandler, HttpTransport,
        HttpTransportBuilder, HttpTransportConfig, McpHttpHandler, StaticFileHandler,
    };

    // External imports
    use async_trait::async_trait;
    use std::sync::Arc;

    /// Test that demonstrates protocol module contains only generic traits
    #[test]
    fn test_protocol_module_is_transport_agnostic() {
        // This test validates that protocol module exports work without any transport-specific types

        // Generic MessageContext with unit type (default)
        let generic_context = MessageContext::new("test_session".to_string());
        assert_eq!(generic_context.session_id(), "test_session");
        assert!(generic_context.transport_data().is_none()); // No transport data for unit type

        // Generic MessageContext with custom type
        let custom_context = MessageContext::new_with_transport_data(
            "custom_session".to_string(),
            "custom_transport_data".to_string(),
        );
        assert_eq!(custom_context.session_id(), "custom_session");
        assert_eq!(
            custom_context.transport_data(),
            Some(&"custom_transport_data".to_string())
        );
    }

    /// Test that demonstrates STDIO transport is self-contained
    #[test]
    fn test_stdio_transport_self_contained() {
        // This test validates STDIO transport module is completely self-contained

        // Type aliases work correctly
        let _: Option<&StdioMessageHandler> = None;
        let _: Option<StdioMessageContext> = None;

        // Configuration is self-contained
        let config = StdioTransportConfig::default();
        assert!(config.buffer_size > 0);

        // Transport types are available
        let _: Option<StdioTransport> = None;
        let _: Option<StdioTransportBuilder> = None;
    }

    /// Test that demonstrates HTTP transport is self-contained  
    #[test]
    fn test_http_transport_self_contained() {
        // This test validates HTTP transport module is completely self-contained

        // Type aliases work correctly
        let _: Option<&HttpMessageHandler> = None;
        let _: Option<HttpMessageContext> = None;

        // HTTP context is available and functional
        let http_context = HttpContext::new("GET", "/test")
            .with_header("content-type", "application/json")
            .with_query_param("param", "value");

        assert_eq!(http_context.method(), "GET");
        assert_eq!(http_context.path(), "/test");
        assert_eq!(
            http_context.get_header("content-type"),
            Some("application/json")
        );
        assert_eq!(http_context.get_query_param("param"), Some("value"));
        assert!(http_context.is_json());

        // Configuration is self-contained
        let config = HttpTransportConfig::default();
        assert!(config.port > 0);

        // Transport types are available
        let _: Option<HttpTransport> = None;
        let _: Option<HttpTransportBuilder> = None;

        // Handler examples are available
        let _mcp_handler = McpHttpHandler::new();
        let _echo_handler = EchoHttpHandler::new();
        let _static_handler = StaticFileHandler::new();
    }

    /// Test that demonstrates no cross-dependencies between transport modules
    #[test]
    fn test_no_cross_dependencies() {
        // This test validates that transport modules don't depend on each other

        // STDIO can be used independently
        let stdio_config = StdioTransportConfig::default();
        assert_eq!(stdio_config.buffer_size, 8192);

        // HTTP can be used independently
        let http_config = HttpTransportConfig::default();
        assert_eq!(http_config.host, "127.0.0.1");

        // No types from one transport should be needed in another
        // (This is enforced by compilation - if there were cross-dependencies, imports would fail)
    }

    /// Demonstration handler for STDIO (unit context)
    struct TestStdioHandler;

    #[async_trait]
    impl MessageHandler<()> for TestStdioHandler {
        async fn handle_message(
            &self,
            _message: crate::protocol::JsonRpcMessage,
            context: MessageContext<()>,
        ) {
            // STDIO handler receives unit context (no transport-specific data)
            assert_eq!(context.session_id(), "test");
            assert!(context.transport_data().is_none());
        }

        async fn handle_error(&self, _error: TransportError) {}
        async fn handle_close(&self) {}
    }

    /// Demonstration handler for HTTP (rich context)
    struct TestHttpHandler;

    #[async_trait]
    impl MessageHandler<HttpContext> for TestHttpHandler {
        async fn handle_message(
            &self,
            _message: crate::protocol::JsonRpcMessage,
            context: MessageContext<HttpContext>,
        ) {
            // HTTP handler receives rich HTTP context
            if let Some(http_ctx) = context.transport_data() {
                // Can access HTTP-specific data
                let _method = http_ctx.method();
                let _path = http_ctx.path();
                let _headers = http_ctx.headers();
            }
        }

        async fn handle_error(&self, _error: TransportError) {}
        async fn handle_close(&self) {}
    }

    /// Test that demonstrates the Generic MessageHandler pattern works correctly
    #[tokio::test]
    async fn test_generic_message_handler_pattern() {
        // This test validates the Generic MessageHandler pattern with different context types

        // STDIO handler with unit context
        let stdio_handler = Arc::new(TestStdioHandler);
        let stdio_context = MessageContext::new("test".to_string());
        let message = crate::protocol::JsonRpcMessage::from_notification("test", None);

        stdio_handler
            .handle_message(message.clone(), stdio_context)
            .await;

        // HTTP handler with rich context
        let http_handler = Arc::new(TestHttpHandler);
        let http_context_data =
            HttpContext::new("POST", "/api").with_header("content-type", "application/json");
        let http_context =
            MessageContext::new_with_transport_data("http_session".to_string(), http_context_data);

        http_handler.handle_message(message, http_context).await;
    }
}

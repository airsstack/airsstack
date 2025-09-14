//! Authentication Module
//!
//! This module provides the OAuth2 message handler for the MCP integration.

use tracing::{info, warn};

use airs_mcp::protocol::transport::{MessageHandler, MessageContext, TransportError};
use airs_mcp::protocol::message::JsonRpcMessage;
use airs_mcp::transport::adapters::http::{AxumMcpRequestHandler, HttpContext};
use airs_mcp::providers::{
    CodeReviewPromptProvider, FileSystemResourceProvider, MathToolProvider,
    StructuredLoggingHandler,
};

/// OAuth2 Message Handler
///
/// This handler wraps the standard AxumMcpRequestHandler and provides
/// OAuth2-specific message handling for the MCP integration.
pub struct OAuth2MessageHandler {
    _inner: AxumMcpRequestHandler<
        FileSystemResourceProvider,
        MathToolProvider,
        CodeReviewPromptProvider,
        StructuredLoggingHandler,
    >,
}

impl OAuth2MessageHandler {
    pub fn new(
        inner: AxumMcpRequestHandler<
            FileSystemResourceProvider,
            MathToolProvider,
            CodeReviewPromptProvider,
            StructuredLoggingHandler,
        >,
    ) -> Self {
        Self { _inner: inner }
    }
}

#[async_trait::async_trait]
impl MessageHandler<HttpContext> for OAuth2MessageHandler {
    async fn handle_message(
        &self,
        message: JsonRpcMessage,
        context: MessageContext<HttpContext>,
    ) {
        tracing::debug!("OAuth2MessageHandler: delegating to inner handler");
        self._inner.handle_message(message, context).await;
        tracing::debug!("OAuth2MessageHandler: inner handler returned");
    }

    async fn handle_error(&self, error: TransportError) {
        warn!("OAuth2 transport error: {:?}", error);
    }

    async fn handle_close(&self) {
        info!("OAuth2 transport connection closed");
    }
}
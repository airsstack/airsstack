//! Default Zero-Cost Provider Implementations
//!
//! This module provides zero-cost default implementations for all MCP provider types.
//! These types are used as defaults in the generic AxumMcpRequestHandler when no
//! specific provider is needed, following the zero-cost abstraction principle.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports  
use async_trait::async_trait;
use serde_json::Value;

// Layer 3: Internal module imports
use crate::integration::{LoggingHandler, McpError, McpResult};
use crate::protocol::{Content, LoggingConfig, PromptMessage, Prompt, Resource, Tool};
use crate::providers::{PromptProvider, ResourceProvider, ToolProvider};

/// Zero-cost no-op resource provider
///
/// This provider returns empty results for all resource operations,
/// providing a zero-cost default when no resource functionality is needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoResourceProvider;

#[async_trait]
impl ResourceProvider for NoResourceProvider {
    async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        Ok(Vec::new())
    }

    async fn read_resource(&self, _uri: &str) -> McpResult<Vec<Content>> {
        Err(McpError::unsupported_capability("resources"))
    }

    async fn subscribe_to_resource(&self, _uri: &str) -> McpResult<()> {
        Err(McpError::unsupported_capability("resource subscriptions"))
    }

    async fn unsubscribe_from_resource(&self, _uri: &str) -> McpResult<()> {
        Err(McpError::unsupported_capability("resource subscriptions"))
    }
}

/// Zero-cost no-op tool provider
///
/// This provider returns empty results for all tool operations,
/// providing a zero-cost default when no tool functionality is needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoToolProvider;

#[async_trait]
impl ToolProvider for NoToolProvider {
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        Ok(Vec::new())
    }

    async fn call_tool(&self, _name: &str, _arguments: Value) -> McpResult<Vec<Content>> {
        Err(McpError::unsupported_capability("tools"))
    }
}

/// Zero-cost no-op prompt provider
///
/// This provider returns empty results for all prompt operations,
/// providing a zero-cost default when no prompt functionality is needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPromptProvider;

#[async_trait]
impl PromptProvider for NoPromptProvider {
    async fn list_prompts(&self) -> McpResult<Vec<Prompt>> {
        Ok(Vec::new())
    }

    async fn get_prompt(&self, _name: &str, _arguments: HashMap<String, String>) -> McpResult<(String, Vec<PromptMessage>)> {
        Err(McpError::unsupported_capability("prompts"))
    }
}

/// Zero-cost no-op logging handler
///
/// This handler ignores all logging operations,
/// providing a zero-cost default when no logging functionality is needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoLoggingHandler;

#[async_trait]
impl LoggingHandler for NoLoggingHandler {
    async fn set_logging(&self, _config: LoggingConfig) -> McpResult<bool> {
        // No-op: logging configuration changes are ignored
        Ok(false) // Return false to indicate logging is not actually configured
    }
}

/// Type alias for the default AxumMcpRequestHandler with no providers
pub type DefaultAxumMcpRequestHandler = super::AxumMcpRequestHandler<
    NoResourceProvider,
    NoToolProvider,
    NoPromptProvider,
    NoLoggingHandler,
>;
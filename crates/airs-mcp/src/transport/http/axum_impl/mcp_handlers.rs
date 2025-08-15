//! MCP Handlers Management
//!
//! This module provides structured management of MCP protocol handlers including
//! resource providers, tool providers, prompt providers, and logging handlers.
//! Implements builder pattern for fluent configuration.

use std::sync::Arc;

use crate::integration::mcp::server::McpServerConfig;
use crate::integration::mcp::{LoggingHandler, PromptProvider, ResourceProvider, ToolProvider};

/// MCP handlers container for different provider types
///
/// This structure manages all MCP protocol handlers and provides a clean
/// interface for accessing different provider implementations.
pub struct McpHandlers {
    /// Resource provider for handling resource-related MCP requests
    pub resource_provider: Option<Arc<dyn ResourceProvider>>,
    /// Tool provider for handling tool-related MCP requests  
    pub tool_provider: Option<Arc<dyn ToolProvider>>,
    /// Prompt provider for handling prompt-related MCP requests
    pub prompt_provider: Option<Arc<dyn PromptProvider>>,
    /// Logging handler for MCP logging operations
    pub logging_handler: Option<Arc<dyn LoggingHandler>>,
    /// MCP server configuration
    pub config: McpServerConfig,
}

/// Builder for MCP handlers to enable fluent configuration
///
/// Implements the Builder pattern for clean, fluent configuration of MCP handlers.
/// Provides type-safe construction with optional components.
pub struct McpHandlersBuilder {
    resource_provider: Option<Arc<dyn ResourceProvider>>,
    tool_provider: Option<Arc<dyn ToolProvider>>,
    prompt_provider: Option<Arc<dyn PromptProvider>>,
    logging_handler: Option<Arc<dyn LoggingHandler>>,
    config: McpServerConfig,
}

impl McpHandlersBuilder {
    /// Create a new MCP handlers builder with default configuration
    pub fn new() -> Self {
        Self {
            resource_provider: None,
            tool_provider: None,
            prompt_provider: None,
            logging_handler: None,
            config: McpServerConfig::default(),
        }
    }

    /// Set the resource provider
    pub fn with_resource_provider(mut self, provider: Arc<dyn ResourceProvider>) -> Self {
        self.resource_provider = Some(provider);
        self
    }

    /// Set the tool provider
    pub fn with_tool_provider(mut self, provider: Arc<dyn ToolProvider>) -> Self {
        self.tool_provider = Some(provider);
        self
    }

    /// Set the prompt provider
    pub fn with_prompt_provider(mut self, provider: Arc<dyn PromptProvider>) -> Self {
        self.prompt_provider = Some(provider);
        self
    }

    /// Set the logging handler
    pub fn with_logging_handler(mut self, handler: Arc<dyn LoggingHandler>) -> Self {
        self.logging_handler = Some(handler);
        self
    }

    /// Set the MCP server configuration
    pub fn with_config(mut self, config: McpServerConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the MCP handlers
    pub fn build(self) -> McpHandlers {
        McpHandlers {
            resource_provider: self.resource_provider,
            tool_provider: self.tool_provider,
            prompt_provider: self.prompt_provider,
            logging_handler: self.logging_handler,
            config: self.config,
        }
    }
}

impl Default for McpHandlersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

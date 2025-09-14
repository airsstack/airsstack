//! STDIO Transport Configuration
//!
//! This module provides configuration structures for the STDIO transport
//! following ADR-011 Transport Configuration Separation Architecture.

use std::path::PathBuf;

use serde_json::{self, json};

use crate::protocol::transport::TransportConfig;
use crate::protocol::types::{ServerCapabilities, ServerConfig};

/// STDIO-specific transport configuration
///
/// This configuration structure contains both universal MCP requirements
/// (via ServerConfig) and STDIO-specific settings optimized for standard
/// input/output communication patterns.
///
/// # Design Principles
///
/// - **Buffer Management**: STDIO needs specific buffering for performance
/// - **Flush Control**: Important for interactive STDIO sessions  
/// - **Log Separation**: Logs must go to file, not stdout (would corrupt MCP protocol)
/// - **Validation Strictness**: STDIO-specific validation levels
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::stdio::StdioTransportConfig;
/// use airs_mcp::protocol::types::ServerConfig;
/// use airs_mcp::protocol::transport::TransportConfig;
/// use std::path::PathBuf;
///
/// // Create with defaults
/// let mut config = StdioTransportConfig::new();
///
/// // Set MCP server configuration
/// let server_config = ServerConfig::default();
/// config.set_server_config(server_config);
///
/// // Customize STDIO-specific settings
/// let config = StdioTransportConfig::new()
///     .buffer_size(8192)
///     .flush_on_response(true)
///     .strict_validation(true)
///     .with_log_file_path(Some("/tmp/mcp-stdio.log".into()));
/// ```
#[derive(Debug, Clone)]
pub struct StdioTransportConfig {
    /// Universal MCP requirements (transport-agnostic)
    server_config: Option<ServerConfig>,

    /// Buffer size for stdin/stdout operations
    ///
    /// Larger buffers improve performance for bulk operations,
    /// smaller buffers reduce latency for interactive sessions.
    /// Default: 8192 bytes
    buffer_size: usize,

    /// Whether to flush stdout after each response
    ///
    /// Essential for interactive STDIO sessions to ensure responses
    /// are immediately visible to the client.
    /// Default: true
    flush_on_response: bool,

    /// STDIO-specific validation strictness
    ///
    /// When enabled, performs additional validation on JSON-RPC
    /// messages before processing. Useful for debugging.
    /// Default: false
    strict_validation: bool,

    /// Whether to log operations (to file, not stdout!)
    ///
    /// Logging to stdout would corrupt the MCP protocol stream,
    /// so logs must be written to a separate file.
    /// Default: false
    log_operations: bool,

    /// Log file path (separate from stdout)
    ///
    /// Required when log_operations is true. Must not be stdout/stderr
    /// to avoid corrupting the MCP protocol stream.
    /// Default: None
    log_file_path: Option<PathBuf>,
}

impl StdioTransportConfig {
    /// Create a new STDIO transport configuration with sensible defaults
    ///
    /// Default values are optimized for typical MCP STDIO workloads:
    /// - 8KB buffer size for good performance/latency balance
    /// - Flush on response for interactive sessions
    /// - Non-strict validation for performance
    /// - No logging to avoid stdout corruption
    pub fn new() -> Self {
        Self {
            server_config: None,
            buffer_size: 8192,
            flush_on_response: true,
            strict_validation: false,
            log_operations: false,
            log_file_path: None,
        }
    }

    /// Set buffer size for stdin/stdout operations
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set whether to flush stdout after each response
    pub fn flush_on_response(mut self, flush: bool) -> Self {
        self.flush_on_response = flush;
        self
    }

    /// Set STDIO-specific validation strictness
    pub fn strict_validation(mut self, strict: bool) -> Self {
        self.strict_validation = strict;
        self
    }

    /// Enable logging with specified file path
    pub fn with_log_file_path(mut self, path: Option<PathBuf>) -> Self {
        self.log_operations = path.is_some();
        self.log_file_path = path;
        self
    }

    /// Enable/disable operation logging
    pub fn with_log_operations(mut self, log: bool) -> Self {
        self.log_operations = log;
        self
    }

    // Getters for STDIO-specific configuration

    /// Get buffer size
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Get flush on response setting
    pub fn get_flush_on_response(&self) -> bool {
        self.flush_on_response
    }

    /// Get strict validation setting
    pub fn get_strict_validation(&self) -> bool {
        self.strict_validation
    }

    /// Get logging operations setting
    pub fn get_log_operations(&self) -> bool {
        self.log_operations
    }

    /// Get log file path
    pub fn get_log_file_path(&self) -> Option<&PathBuf> {
        self.log_file_path.as_ref()
    }
}

impl Default for StdioTransportConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of TransportConfig trait for STDIO transport
///
/// This provides the standardized interface for MCP server configuration
/// management while allowing STDIO-specific capability modifications.
impl TransportConfig for StdioTransportConfig {
    fn set_server_config(&mut self, server_config: ServerConfig) {
        self.server_config = Some(server_config);
    }

    fn server_config(&self) -> Option<&ServerConfig> {
        self.server_config.as_ref()
    }

    fn effective_capabilities(&self) -> ServerCapabilities {
        if let Some(server_cfg) = &self.server_config {
            let mut caps = server_cfg.capabilities.clone();

            // STDIO-specific capability modifications
            // Set experimental to empty object for compatibility
            caps.experimental = Some(json!({}));

            // STDIO doesn't support certain streaming patterns
            if let Some(ref mut resources) = caps.resources {
                // Disable subscription for STDIO (interactive polling is better)
                resources.subscribe = Some(false);
            }

            caps
        } else {
            // Default capabilities if no server config set
            ServerCapabilities::default()
        }
    }
}

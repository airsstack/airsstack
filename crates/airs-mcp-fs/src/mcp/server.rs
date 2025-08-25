//! MCP Server implementation for filesystem operations
//!
//! This module provides the core MCP server that integrates with Claude Desktop
//! and other MCP-compatible clients to provide secure filesystem operations.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use tracing::{info, instrument};

// Layer 3: Internal module imports
use crate::config::Settings;
use crate::security::SecurityManager;
use airs_mcp::integration::mcp::{McpError, McpResult, ToolProvider};
use airs_mcp::shared::protocol::{Content, Tool};

/// Filesystem MCP server implementing MCP protocol for secure file operations
#[derive(Debug)]
pub struct FilesystemMcpServer {
    security_manager: Arc<SecurityManager>,
    settings: Arc<Settings>,
    _server_state: Arc<Mutex<ServerState>>,
}

/// Legacy MCP server wrapper (maintained for backward compatibility)
#[derive(Debug)]
pub struct McpServer {
    settings: Arc<Settings>,
    _server_state: Arc<Mutex<ServerState>>,
}

/// Internal server state
#[derive(Debug, Default)]
struct ServerState {
    #[allow(dead_code)] // Will be used in task_002
    connected: bool,
    #[allow(dead_code)] // Will be used in task_002
    tools_registered: bool,
}

impl FilesystemMcpServer {
    /// Create a new filesystem MCP server instance
    #[instrument(level = "debug")]
    pub async fn new(settings: Settings) -> Result<Self> {
        info!("Initializing AIRS MCP-FS filesystem server");

        // Initialize security manager with security config
        let security_manager = Arc::new(SecurityManager::new(settings.security.clone()));

        Ok(Self {
            security_manager,
            settings: Arc::new(settings),
            _server_state: Arc::new(Mutex::new(ServerState::default())),
        })
    }

    /// Get reference to security manager for validation
    pub fn security_manager(&self) -> &SecurityManager {
        &self.security_manager
    }

    /// Get server settings
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

#[async_trait]
impl ToolProvider for FilesystemMcpServer {
    /// List available filesystem tools for MCP clients
    #[instrument(level = "debug")]
    async fn list_tools(&self) -> McpResult<Vec<Tool>> {
        info!("Listing available filesystem tools");

        let tools = vec![
            Tool::new(
                "read_file",
                Some("Read File"),
                Some("Read file contents with security validation and encoding detection"),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to read"
                        }
                    },
                    "required": ["path"]
                }),
            ),
            Tool::new(
                "write_file",
                Some("Write File"),
                Some("Write file contents with human approval workflow and security validation"),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }),
            ),
            Tool::new(
                "list_directory",
                Some("List Directory"),
                Some("List directory contents with metadata and security validation"),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the directory to list"
                        }
                    },
                    "required": ["path"]
                }),
            ),
        ];

        info!(
            tool_count = tools.len(),
            "Filesystem tools listed successfully"
        );
        Ok(tools)
    }

    /// Execute filesystem tool operations with security validation
    #[instrument(level = "debug", fields(tool_name = %name))]
    async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<Vec<Content>> {
        info!(tool_name = %name, "Executing filesystem tool");

        match name {
            "read_file" => self.handle_read_file(arguments).await,
            "write_file" => self.handle_write_file(arguments).await,
            "list_directory" => self.handle_list_directory(arguments).await,
            _ => {
                info!(tool_name = %name, "Tool not found");
                Err(McpError::tool_not_found(name))
            }
        }
    }
}

impl FilesystemMcpServer {
    /// Handle read_file tool execution (placeholder implementation)
    async fn handle_read_file(&self, _arguments: Value) -> McpResult<Vec<Content>> {
        // TODO: Implement in task_003 - Core File Operations
        info!("read_file tool called - placeholder implementation");
        Ok(vec![Content::text(
            "File reading will be implemented in task_003",
        )])
    }

    /// Handle write_file tool execution (placeholder implementation)
    async fn handle_write_file(&self, _arguments: Value) -> McpResult<Vec<Content>> {
        // TODO: Implement in task_003 - Core File Operations
        info!("write_file tool called - placeholder implementation");
        Ok(vec![Content::text(
            "File writing will be implemented in task_003",
        )])
    }

    /// Handle list_directory tool execution (placeholder implementation)
    async fn handle_list_directory(&self, _arguments: Value) -> McpResult<Vec<Content>> {
        // TODO: Implement in task_003 - Core File Operations
        info!("list_directory tool called - placeholder implementation");
        Ok(vec![Content::text(
            "Directory listing will be implemented in task_003",
        )])
    }
}

impl McpServer {
    /// Create a new MCP server instance
    pub async fn new(settings: Settings) -> Result<Self> {
        info!("Initializing AIRS MCP-FS server");

        Ok(Self {
            settings: Arc::new(settings),
            _server_state: Arc::new(Mutex::new(ServerState::default())),
        })
    }

    /// Run the MCP server (placeholder implementation)
    pub async fn run(&self) -> Result<()> {
        info!("Starting AIRS MCP-FS server: {}", self.settings.server.name);

        // TODO: Implement actual MCP server loop in task_002
        // For now, just log that we're ready
        info!("MCP server ready for connections");

        // Placeholder - prevent immediate exit
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let settings = Settings::default();
        let result = McpServer::new(settings).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_server_run() {
        let settings = Settings::default();
        let server = McpServer::new(settings).await.unwrap();

        // Test that run() completes without error
        let result = server.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_mcp_server_creation() {
        let settings = Settings::default();
        let result = FilesystemMcpServer::new(settings).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_mcp_server_security_manager_access() {
        let settings = Settings::default();
        let server = FilesystemMcpServer::new(settings).await.unwrap();

        // Test that we can access the security manager
        let _security_manager = server.security_manager();
        let _settings = server.settings();
    }

    #[tokio::test]
    async fn test_tool_provider_list_tools() {
        let settings = Settings::default();
        let server = FilesystemMcpServer::new(settings).await.unwrap();

        // Test tool listing functionality
        let tools = server.list_tools().await.unwrap();
        assert_eq!(tools.len(), 3);

        // Verify the tools are correctly registered
        let tool_names: Vec<&String> = tools.iter().map(|t| &t.name).collect();
        assert!(tool_names.contains(&&"read_file".to_string()));
        assert!(tool_names.contains(&&"write_file".to_string()));
        assert!(tool_names.contains(&&"list_directory".to_string()));
    }

    #[tokio::test]
    async fn test_tool_provider_call_tool_read_file() {
        let settings = Settings::default();
        let server = FilesystemMcpServer::new(settings).await.unwrap();

        // Test calling read_file tool (placeholder implementation)
        let args = serde_json::json!({"path": "/test/file.txt"});
        let result = server.call_tool("read_file", args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert!(text.contains("task_003"));
            }
        }
    }

    #[tokio::test]
    async fn test_tool_provider_call_tool_unknown() {
        let settings = Settings::default();
        let server = FilesystemMcpServer::new(settings).await.unwrap();

        // Test calling unknown tool
        let args = serde_json::json!({"invalid": "data"});
        let result = server.call_tool("unknown_tool", args).await;

        assert!(result.is_err());
    }
}

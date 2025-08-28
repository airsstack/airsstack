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
// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::integration::mcp::{McpError, McpResult, ToolProvider};
use airs_mcp::shared::protocol::{Content, Tool};

// Layer 3b: Local crate modules
use crate::config::Settings;
use crate::mcp::handlers::{DirectoryHandler, DirectoryOperations, FileHandler, FileOperations};
use crate::security::SecurityManager;

/// Filesystem MCP server implementing MCP protocol for secure file operations
#[derive(Debug)]
pub struct FilesystemMcpServer<F, D>
where
    F: FileOperations,
    D: DirectoryOperations,
{
    file_handler: F,
    directory_handler: D,
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

impl<F, D> FilesystemMcpServer<F, D>
where
    F: FileOperations,
    D: DirectoryOperations,
{
    /// Create a new filesystem MCP server instance with dependency injection
    #[instrument(level = "debug")]
    pub async fn new(settings: Settings, file_handler: F, directory_handler: D) -> Result<Self> {
        info!("Initializing AIRS MCP-FS filesystem server with injected handlers");

        Ok(Self {
            file_handler,
            directory_handler,
            settings: Arc::new(settings),
            _server_state: Arc::new(Mutex::new(ServerState::default())),
        })
    }

    /// Get reference to file handler
    pub fn file_handler(&self) -> &F {
        &self.file_handler
    }

    /// Get reference to directory handler  
    pub fn directory_handler(&self) -> &D {
        &self.directory_handler
    }

    /// Get server settings
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

// Convenience type alias for the default implementation
pub type DefaultFilesystemMcpServer = FilesystemMcpServer<FileHandler, DirectoryHandler>;

impl DefaultFilesystemMcpServer {
    /// Create a new filesystem MCP server instance with default handlers
    #[instrument(level = "debug")]
    pub async fn with_default_handlers(settings: Settings) -> Result<Self> {
        info!("Initializing AIRS MCP-FS filesystem server with default handlers");

        // Initialize security manager with security config
        let security_manager = Arc::new(SecurityManager::new(settings.security.clone())?);

        // Create handlers with shared security manager
        let file_handler = FileHandler::new(Arc::clone(&security_manager));
        let directory_handler = DirectoryHandler::new(Arc::clone(&security_manager));

        Self::new(settings, file_handler, directory_handler).await
    }
}

#[async_trait]
impl<F, D> ToolProvider for FilesystemMcpServer<F, D>
where
    F: FileOperations,
    D: DirectoryOperations,
{
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
            "read_file" => self.file_handler.handle_read_file(arguments).await,
            "write_file" => self.file_handler.handle_write_file(arguments).await,
            "list_directory" => {
                self.directory_handler
                    .handle_list_directory(arguments)
                    .await
            }
            _ => {
                info!(tool_name = %name, "Tool not found");
                Err(McpError::tool_not_found(name))
            }
        }
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::config::{
        BinaryConfig, FilesystemConfig, OperationConfig, SecurityConfig, ServerConfig, Settings,
    };
    use std::collections::HashMap;
    use tempfile;

    fn create_permissive_test_settings() -> Settings {
        Settings {
            security: SecurityConfig {
                filesystem: FilesystemConfig {
                    allowed_paths: vec!["/**/*".to_string()], // Allow all paths for testing
                    denied_paths: vec![],                     // No denied paths for testing
                },
                operations: OperationConfig {
                    read_allowed: true,
                    write_requires_policy: false, // Permissive for testing
                    delete_requires_explicit_allow: false, // Permissive for testing
                    create_dir_allowed: true,
                },
                policies: HashMap::new(), // No policies needed for permissive testing
            },
            binary: BinaryConfig {
                max_file_size: 100 * 1024 * 1024, // 100MB
                enable_image_processing: true,
                enable_pdf_processing: true,
            },
            server: ServerConfig {
                name: "airs-mcp-fs".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let settings = create_permissive_test_settings();
        let result = McpServer::new(settings).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_server_run() {
        let settings = create_permissive_test_settings();
        let server = McpServer::new(settings).await.unwrap();

        // Test that run() completes without error
        let result = server.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_mcp_server_creation() {
        let settings = Settings::default();
        let result = DefaultFilesystemMcpServer::with_default_handlers(settings).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_mcp_server_handler_access() {
        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

        // Test that we can access the handlers
        let _file_handler = server.file_handler();
        let _directory_handler = server.directory_handler();
        let _settings = server.settings();
    }

    #[tokio::test]
    async fn test_tool_provider_list_tools() {
        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

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
        use std::io::Write;
        use tempfile::NamedTempFile;

        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

        // Create a temporary file for testing
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = "Hello, World! This is test content.";
        temp_file.write_all(test_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Test calling read_file tool with real file
        let args = serde_json::json!({"path": temp_file.path().to_string_lossy()});
        let result = server.call_tool("read_file", args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert_eq!(text, test_content);
            }
        }
    }

    #[tokio::test]
    async fn test_tool_provider_call_tool_read_file_not_found() {
        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

        // Test calling read_file tool with non-existent file
        let args = serde_json::json!({"path": "/non/existent/file.txt"});
        let result = server.call_tool("read_file", args).await;

        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("File not found"));
        }
    }

    #[tokio::test]
    async fn test_tool_provider_call_tool_unknown() {
        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

        // Test calling unknown tool
        let args = serde_json::json!({"invalid": "data"});
        let result = server.call_tool("unknown_tool", args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_write_file_tool() {
        use tempfile::TempDir;

        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_write.txt");
        let test_content = "Hello from write_file tool!";

        // Test writing a file
        let args = serde_json::json!({
            "path": test_file_path.to_string_lossy(),
            "content": test_content
        });
        let result = server.call_tool("write_file", args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert!(text.contains("File written successfully"));
                assert!(text.contains(&test_file_path.to_string_lossy().to_string()));
            }
        }

        // Verify the file was actually written
        let written_content = tokio::fs::read_to_string(&test_file_path).await.unwrap();
        assert_eq!(written_content, test_content);
    }

    #[tokio::test]
    async fn test_list_directory_tool() {
        use tempfile::TempDir;

        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

        // Create a temporary directory with some files for testing
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.md");
        tokio::fs::write(&file1_path, "content1").await.unwrap();
        tokio::fs::write(&file2_path, "# content2").await.unwrap();

        // Test listing directory
        let args = serde_json::json!({
            "path": temp_dir.path().to_string_lossy()
        });
        let result = server.call_tool("list_directory", args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                // Parse the JSON response
                let response: serde_json::Value = serde_json::from_str(text).unwrap();

                assert_eq!(response["total_entries"].as_u64().unwrap(), 2);
                assert!(text.contains("file1.txt"));
                assert!(text.contains("file2.md"));

                // Check that entries have type "file" - be flexible with JSON formatting
                let entries = response["entries"].as_array().unwrap();
                assert_eq!(entries.len(), 2);
                for entry in entries {
                    assert_eq!(entry["type"].as_str().unwrap(), "file");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_read_file_base64_encoding() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let settings = Settings::default();
        let server = DefaultFilesystemMcpServer::with_default_handlers(settings)
            .await
            .unwrap();

        // Create a temporary file with binary content
        let mut temp_file = NamedTempFile::new().unwrap();
        let binary_data = vec![0u8, 1u8, 2u8, 255u8]; // Some binary data
        temp_file.write_all(&binary_data).unwrap();
        temp_file.flush().unwrap();

        // Test reading with base64 encoding
        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy(),
            "encoding": "base64"
        });
        let result = server.call_tool("read_file", args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert!(text.contains("Base64 encoded content"));
                assert!(text.contains("4 bytes"));
                // The base64 encoding of [0, 1, 2, 255] should be "AAEC/w=="
                assert!(text.contains("AAEC/w=="));
            }
        }
    }
}

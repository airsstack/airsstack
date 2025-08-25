//! Directory Operation Handler
//!
//! Handles list_directory MCP tool operations with security validation,
//! recursive listing, and metadata collection.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use chrono;
use serde_json::Value;
use tracing::{info, instrument};

// Layer 3: Internal module imports
// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::integration::mcp::{McpError, McpResult};
use airs_mcp::shared::protocol::Content;

// Layer 3b: Local crate modules
use crate::filesystem::FileOperation;
use crate::mcp::OperationType;
use crate::security::SecurityManager;

/// Handler for directory operations (list_directory)
#[derive(Debug)]
pub struct DirectoryHandler {
    security_manager: Arc<SecurityManager>,
}

impl DirectoryHandler {
    /// Create a new directory handler instance
    pub fn new(security_manager: Arc<SecurityManager>) -> Self {
        Self { security_manager }
    }

    /// Handle list_directory tool execution with security validation and metadata
    #[instrument(level = "debug", skip(self))]
    pub async fn handle_list_directory(&self, arguments: Value) -> McpResult<Vec<Content>> {
        info!("Processing list_directory tool request");

        // Parse arguments from JSON
        #[derive(serde::Deserialize)]
        struct ListDirectoryArgs {
            path: String,
            #[serde(default)]
            include_hidden: Option<bool>,
            #[serde(default)]
            include_metadata: Option<bool>,
            #[serde(default)]
            recursive: Option<bool>,
            #[serde(default)]
            max_depth: Option<u32>,
        }

        let args: ListDirectoryArgs = serde_json::from_value(arguments).map_err(|e| {
            McpError::invalid_request(format!("Invalid list_directory arguments: {e}"))
        })?;

        // Create filesystem operation for security validation
        let operation =
            FileOperation::new(OperationType::List, std::path::PathBuf::from(&args.path));

        // Security validation
        self.security_manager
            .validate_read_access(&operation)
            .await
            .map_err(|e| McpError::invalid_request(format!("Security validation failed: {e}")))?;

        // Validate directory exists and is actually a directory
        self.validate_directory(&args.path).await?;

        // Extract listing options
        let options = ListingOptions {
            include_hidden: args.include_hidden.unwrap_or(false),
            include_metadata: args.include_metadata.unwrap_or(true),
            recursive: args.recursive.unwrap_or(false),
            max_depth: args.max_depth.unwrap_or(10),
        };

        // List directory contents
        let mut entries = Vec::new();
        if options.recursive {
            self.collect_entries_recursive(&args.path, &mut entries, &options, 0)
                .await?;
        } else {
            self.collect_entries_single(&args.path, &mut entries, &options)
                .await?;
        }

        // Sort entries by name
        entries.sort_by(|a, b| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .cmp(b["name"].as_str().unwrap_or(""))
        });

        // Format response
        let response = self.format_response(&args.path, &entries, &options);

        info!(
            directory_path = %args.path,
            entry_count = entries.len(),
            recursive = options.recursive,
            "Successfully listed directory contents"
        );

        Ok(vec![Content::text(
            serde_json::to_string_pretty(&response).map_err(|e| {
                McpError::internal_error(format!("Failed to serialize response: {e}"))
            })?,
        )])
    }

    /// Validate that the path exists and is a directory
    async fn validate_directory(&self, path: &str) -> McpResult<()> {
        // Check if directory exists
        if !tokio::fs::try_exists(path).await.map_err(|e| {
            McpError::internal_error(format!("Failed to check directory existence: {e}"))
        })? {
            return Err(McpError::invalid_request(format!(
                "Directory not found: {path}"
            )));
        }

        // Check if path is actually a directory
        let metadata = tokio::fs::metadata(path).await.map_err(|e| {
            McpError::internal_error(format!("Failed to read directory metadata: {e}"))
        })?;

        if !metadata.is_dir() {
            return Err(McpError::invalid_request(format!(
                "Path is not a directory: {path}"
            )));
        }

        Ok(())
    }

    /// Collect directory entries recursively
    fn collect_entries_recursive<'a>(
        &'a self,
        path: &'a str,
        entries: &'a mut Vec<serde_json::Value>,
        options: &'a ListingOptions,
        current_depth: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = McpResult<()>> + 'a + Send>> {
        Box::pin(async move {
            if current_depth >= options.max_depth {
                return Ok(());
            }

            let mut dir_entries = tokio::fs::read_dir(path)
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to read directory: {e}")))?;

            while let Some(entry) = dir_entries.next_entry().await.map_err(|e| {
                McpError::internal_error(format!("Failed to read directory entry: {e}"))
            })? {
                let entry_path = entry.path();
                let entry_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?");

                // Skip hidden files if not requested
                if !options.include_hidden && entry_name.starts_with('.') {
                    continue;
                }

                let relative_path = entry_path
                    .strip_prefix(path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| entry_path.to_string_lossy().to_string());

                let mut entry_info = serde_json::json!({
                    "name": entry_name,
                    "path": entry_path.to_string_lossy(),
                    "relative_path": relative_path,
                    "depth": current_depth
                });

                if options.include_metadata {
                    if let Ok(metadata) = entry.metadata().await {
                        self.add_metadata_to_entry(&mut entry_info, &metadata);
                    }
                }

                entries.push(entry_info);

                // Recurse into subdirectories
                if entry_path.is_dir() {
                    self.collect_entries_recursive(
                        &entry_path.to_string_lossy(),
                        entries,
                        options,
                        current_depth + 1,
                    )
                    .await?;
                }
            }

            Ok(())
        })
    }

    /// Collect directory entries for single-level listing
    async fn collect_entries_single(
        &self,
        path: &str,
        entries: &mut Vec<serde_json::Value>,
        options: &ListingOptions,
    ) -> McpResult<()> {
        let mut dir_entries = tokio::fs::read_dir(path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read directory: {e}")))?;

        while let Some(entry) = dir_entries
            .next_entry()
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read directory entry: {e}")))?
        {
            let entry_name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files if not requested
            if !options.include_hidden && entry_name.starts_with('.') {
                continue;
            }

            let mut entry_info = serde_json::json!({
                "name": entry_name,
                "path": entry.path().to_string_lossy()
            });

            if options.include_metadata {
                if let Ok(metadata) = entry.metadata().await {
                    self.add_metadata_to_entry(&mut entry_info, &metadata);
                }
            }

            entries.push(entry_info);
        }

        Ok(())
    }

    /// Add metadata information to directory entry
    fn add_metadata_to_entry(
        &self,
        entry_info: &mut serde_json::Value,
        metadata: &std::fs::Metadata,
    ) {
        entry_info["type"] = if metadata.is_dir() {
            "directory"
        } else {
            "file"
        }
        .into();
        entry_info["size"] = metadata.len().into();
        entry_info["modified"] = metadata
            .modified()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
            .unwrap_or_else(|_| "unknown".to_string())
            .into();
    }

    /// Format the final response
    fn format_response(
        &self,
        directory_path: &str,
        entries: &[serde_json::Value],
        options: &ListingOptions,
    ) -> serde_json::Value {
        serde_json::json!({
            "directory": directory_path,
            "total_entries": entries.len(),
            "entries": entries,
            "options": {
                "include_hidden": options.include_hidden,
                "include_metadata": options.include_metadata,
                "recursive": options.recursive,
                "max_depth": options.max_depth
            }
        })
    }
}

/// Configuration options for directory listing
#[derive(Debug)]
struct ListingOptions {
    include_hidden: bool,
    include_metadata: bool,
    recursive: bool,
    max_depth: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::security::SecurityManager;
    use tempfile::TempDir;

    fn create_test_handler() -> DirectoryHandler {
        let settings = Settings::default();
        let security_manager = Arc::new(SecurityManager::new(settings.security));
        DirectoryHandler::new(security_manager)
    }

    #[tokio::test]
    async fn test_list_directory() {
        let handler = create_test_handler();

        // Create a temporary directory with some files
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.md");
        tokio::fs::write(&file1_path, "content1").await.unwrap();
        tokio::fs::write(&file2_path, "# content2").await.unwrap();

        // Test listing directory
        let args = serde_json::json!({
            "path": temp_dir.path().to_string_lossy()
        });
        let result = handler.handle_list_directory(args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                let response: serde_json::Value = serde_json::from_str(text).unwrap();
                assert_eq!(response["total_entries"].as_u64().unwrap(), 2);

                let entries = response["entries"].as_array().unwrap();
                assert_eq!(entries.len(), 2);
                for entry in entries {
                    assert_eq!(entry["type"].as_str().unwrap(), "file");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_list_directory_not_found() {
        let handler = create_test_handler();

        // Test listing non-existent directory
        let args = serde_json::json!({"path": "/non/existent/directory"});
        let result = handler.handle_list_directory(args).await;

        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("Directory not found"));
        }
    }

    #[tokio::test]
    async fn test_list_directory_recursive() {
        let handler = create_test_handler();

        // Create a nested directory structure
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        tokio::fs::create_dir(&sub_dir).await.unwrap();

        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = sub_dir.join("file2.txt");
        tokio::fs::write(&file1_path, "content1").await.unwrap();
        tokio::fs::write(&file2_path, "content2").await.unwrap();

        // Test recursive listing
        let args = serde_json::json!({
            "path": temp_dir.path().to_string_lossy(),
            "recursive": true
        });
        let result = handler.handle_list_directory(args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                let response: serde_json::Value = serde_json::from_str(text).unwrap();
                // Should have 3 entries: file1.txt, subdir, and file2.txt
                assert_eq!(response["total_entries"].as_u64().unwrap(), 3);
            }
        }
    }
}

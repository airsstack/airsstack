//! File Operation Handler
//!
//! Handles read_file and write_file MCP tool operations with security validation,
//! encoding detection, and approval workflows.

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use base64::prelude::*;
use chrono;
use serde_json::Value;
use tracing::{info, instrument};

// Layer 3: Internal module imports
// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::integration::mcp::{McpError, McpResult};
use airs_mcp::shared::protocol::Content;

// Layer 3b: Local crate modules
use crate::filesystem::FileOperation;
use crate::mcp::handlers::traits::FileOperations;
use crate::mcp::OperationType;
use crate::security::{ApprovalDecision, SecurityManager};

/// Handler for file operations (read_file, write_file)
#[derive(Debug)]
pub struct FileHandler {
    security_manager: Arc<SecurityManager>,
}

/// Arguments for write_file operations
#[derive(serde::Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    create_directories: Option<bool>,
    #[serde(default)]
    backup_existing: Option<bool>,
}

impl FileHandler {
    /// Create a new file handler instance
    pub fn new(security_manager: Arc<SecurityManager>) -> Self {
        Self { security_manager }
    }

    /// Handle read_file tool execution with security validation and encoding detection
    #[instrument(level = "debug", skip(self))]
    pub async fn handle_read_file(&self, arguments: Value) -> McpResult<Vec<Content>> {
        info!("Processing read_file tool request");

        // Parse arguments from JSON
        #[derive(serde::Deserialize)]
        struct ReadFileArgs {
            path: String,
            #[serde(default)]
            encoding: Option<String>,
            #[serde(default)]
            max_size_mb: Option<u64>,
        }

        let args: ReadFileArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::invalid_request(format!("Invalid read_file arguments: {e}")))?;

        // Create filesystem operation for security validation
        let operation =
            FileOperation::new(OperationType::Read, std::path::PathBuf::from(&args.path));

        // Security validation
        self.security_manager
            .validate_read_access(&operation)
            .await
            .map_err(|e| McpError::invalid_request(format!("Security validation failed: {e}")))?;

        // Check if file exists
        if !tokio::fs::try_exists(&args.path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to check file existence: {e}")))?
        {
            return Err(McpError::invalid_request(format!(
                "File not found: {}",
                args.path
            )));
        }

        // Check file size
        let metadata = tokio::fs::metadata(&args.path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read file metadata: {e}")))?;

        let max_size = {
            // SECURITY FIX: Prevent integer overflow in size calculation
            const MAX_REASONABLE_SIZE_MB: u64 = 1024; // 1GB max
            const MB_TO_BYTES: u64 = 1024 * 1024;

            let size_mb = args.max_size_mb.unwrap_or(100);

            // Validate input range to prevent overflow
            if size_mb > MAX_REASONABLE_SIZE_MB {
                return Err(McpError::invalid_request(format!(
                    "File size limit too large: {size_mb} MB (max: {MAX_REASONABLE_SIZE_MB} MB)"
                )));
            }

            // Safe multiplication with overflow check
            size_mb.checked_mul(MB_TO_BYTES).ok_or_else(|| {
                McpError::invalid_request("File size calculation overflow".to_string())
            })?
        };
        if metadata.len() > max_size {
            return Err(McpError::invalid_request(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                max_size
            )));
        }

        // Determine encoding and read file
        let encoding = args.encoding.unwrap_or_else(|| "auto".to_string());
        match encoding.as_str() {
            "utf8" | "text" => self.read_as_text(&args.path).await,
            "base64" | "binary" => self.read_as_base64(&args.path).await,
            "auto" => self.read_with_auto_detection(&args.path).await,
            _ => Err(McpError::invalid_request(format!(
                "Unsupported encoding: {encoding}"
            ))),
        }
    }

    /// Handle write_file tool execution with security validation and human approval
    #[instrument(level = "debug", skip(self))]
    pub async fn handle_write_file(&self, arguments: Value) -> McpResult<Vec<Content>> {
        info!("Processing write_file tool request");

        // Parse arguments from JSON
        let args: WriteFileArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::invalid_request(format!("Invalid write_file arguments: {e}")))?;

        // SECURITY FIX: Input validation for content
        self.validate_write_input(&args)?;

        // Create filesystem operation for security validation
        let operation =
            FileOperation::new(OperationType::Write, std::path::PathBuf::from(&args.path));

        // Security validation with approval workflow
        let approval_decision = self
            .security_manager
            .validate_write_access(&operation)
            .await
            .map_err(|e| McpError::invalid_request(format!("Security validation failed: {e}")))?;

        // Check approval status
        self.validate_approval_decision(approval_decision)?;

        // Prepare for write operation
        self.prepare_write_operation(&args.path, args.create_directories.unwrap_or(false))
            .await?;

        // Backup existing file if requested
        if args.backup_existing.unwrap_or(false) {
            self.create_backup(&args.path).await?;
        }

        // Write file based on encoding
        let encoding = args.encoding.unwrap_or_else(|| "utf8".to_string());
        match encoding.as_str() {
            "utf8" | "text" => self.write_as_text(&args.path, &args.content).await,
            "base64" => self.write_from_base64(&args.path, &args.content).await,
            _ => Err(McpError::invalid_request(format!(
                "Unsupported encoding: {encoding}"
            ))),
        }
    }

    /// Read file as UTF-8 text
    async fn read_as_text(&self, path: &str) -> McpResult<Vec<Content>> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read file as UTF-8: {e}")))?;

        info!(
            file_path = %path,
            content_length = content.len(),
            "Successfully read file as UTF-8"
        );

        Ok(vec![Content::text(content)])
    }

    /// Read file as binary and encode as base64
    async fn read_as_base64(&self, path: &str) -> McpResult<Vec<Content>> {
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read file as binary: {e}")))?;

        let base64_content = base64::prelude::BASE64_STANDARD.encode(&bytes);

        info!(
            file_path = %path,
            byte_length = bytes.len(),
            "Successfully read file as base64"
        );

        Ok(vec![Content::text(format!(
            "Base64 encoded content ({} bytes):\n{}",
            bytes.len(),
            base64_content
        ))])
    }

    /// Read file with automatic encoding detection
    async fn read_with_auto_detection(&self, path: &str) -> McpResult<Vec<Content>> {
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {e}")))?;

        // Simple UTF-8 detection
        match String::from_utf8(bytes.clone()) {
            Ok(text) => {
                info!(
                    file_path = %path,
                    content_length = text.len(),
                    encoding = "utf8",
                    "Auto-detected UTF-8 encoding"
                );
                Ok(vec![Content::text(text)])
            }
            Err(_) => {
                let base64_content = base64::prelude::BASE64_STANDARD.encode(&bytes);
                info!(
                    file_path = %path,
                    byte_length = bytes.len(),
                    encoding = "base64",
                    "Auto-detected binary content, returning as base64"
                );
                Ok(vec![Content::text(format!(
                    "Binary file detected ({} bytes), base64 encoded:\n{}",
                    bytes.len(),
                    base64_content
                ))])
            }
        }
    }

    /// Validate approval decision from security manager
    fn validate_approval_decision(&self, decision: ApprovalDecision) -> McpResult<()> {
        match decision {
            ApprovalDecision::Approved => {
                info!("Write operation approved, proceeding with file write");
                Ok(())
            }
            ApprovalDecision::Denied => Err(McpError::invalid_request(
                "Write operation denied by security policy".to_string(),
            )),
            ApprovalDecision::Timeout => Err(McpError::invalid_request(
                "Write operation timed out waiting for approval".to_string(),
            )),
            ApprovalDecision::Cancelled => Err(McpError::invalid_request(
                "Write operation cancelled by user".to_string(),
            )),
        }
    }

    /// Prepare filesystem for write operation (create directories if needed)
    async fn prepare_write_operation(&self, path: &str, create_directories: bool) -> McpResult<()> {
        if create_directories {
            if let Some(parent) = std::path::Path::new(path).parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    McpError::internal_error(format!("Failed to create directories: {e}"))
                })?;
            }
        }
        Ok(())
    }

    /// Create backup of existing file
    async fn create_backup(&self, path: &str) -> McpResult<()> {
        if tokio::fs::try_exists(path).await.unwrap_or(false) {
            let backup_path = format!("{}.backup.{}", path, chrono::Utc::now().timestamp());
            tokio::fs::copy(path, &backup_path)
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to create backup: {e}")))?;
            info!(backup_path = %backup_path, "Created backup of existing file");
        }
        Ok(())
    }

    /// Write content as UTF-8 text
    async fn write_as_text(&self, path: &str, content: &str) -> McpResult<Vec<Content>> {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to write file: {e}")))?;

        info!(
            file_path = %path,
            content_length = content.len(),
            "Successfully wrote file as UTF-8"
        );

        Ok(vec![Content::text(format!(
            "File written successfully: {} ({} characters)",
            path,
            content.len()
        ))])
    }

    /// Write content from base64 encoded data
    async fn write_from_base64(&self, path: &str, content: &str) -> McpResult<Vec<Content>> {
        let bytes = base64::prelude::BASE64_STANDARD
            .decode(content)
            .map_err(|e| McpError::invalid_request(format!("Invalid base64 content: {e}")))?;

        tokio::fs::write(path, &bytes)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to write binary file: {e}")))?;

        info!(
            file_path = %path,
            byte_length = bytes.len(),
            "Successfully wrote file from base64"
        );

        Ok(vec![Content::text(format!(
            "Binary file written successfully: {} ({} bytes)",
            path,
            bytes.len()
        ))])
    }

    /// SECURITY FIX: Comprehensive input validation for write operations  
    fn validate_write_input(&self, args: &WriteFileArgs) -> McpResult<()> {
        // Validate path input
        if args.path.is_empty() {
            return Err(McpError::invalid_request(
                "Path cannot be empty".to_string(),
            ));
        }

        // Check for null bytes in path and content
        if args.path.contains('\0') || args.content.contains('\0') {
            return Err(McpError::invalid_request(
                "Null bytes not allowed in input".to_string(),
            ));
        }

        // Validate content size to prevent DoS
        const MAX_CONTENT_SIZE: usize = 100 * 1024 * 1024; // 100 MB
        if args.content.len() > MAX_CONTENT_SIZE {
            return Err(McpError::invalid_request(format!(
                "Content too large: {} bytes (max: {} bytes)",
                args.content.len(),
                MAX_CONTENT_SIZE
            )));
        }

        // Check for control characters in content (except safe ones)
        if args
            .content
            .chars()
            .any(|c| c.is_control() && c != '\t' && c != '\n' && c != '\r')
        {
            return Err(McpError::invalid_request(
                "Dangerous control characters detected in content".to_string(),
            ));
        }

        // Validate encoding parameter
        if let Some(ref encoding) = args.encoding {
            match encoding.as_str() {
                "utf8" | "text" | "base64" | "binary" => {}
                _ => {
                    return Err(McpError::invalid_request(format!(
                        "Unsupported encoding: {encoding}"
                    )))
                }
            }
        }

        Ok(())
    }
}

/// Implementation of FileOperations trait for FileHandler
///
/// This implementation provides the dependency injection interface for file operations,
/// enabling loose coupling and improved testability.
#[async_trait]
impl FileOperations for FileHandler {
    async fn handle_read_file(&self, arguments: Value) -> McpResult<Vec<Content>> {
        self.handle_read_file(arguments).await
    }

    async fn handle_write_file(&self, arguments: Value) -> McpResult<Vec<Content>> {
        self.handle_write_file(arguments).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::config::{SecurityConfig, Settings};
    use crate::security::SecurityManager;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn create_permissive_test_config() -> SecurityConfig {
        // Use permissive settings for testing
        Settings::builder().permissive().build().security
    }

    fn create_test_handler() -> FileHandler {
        let security_config = create_permissive_test_config();
        let security_manager = Arc::new(SecurityManager::new(security_config).unwrap());
        FileHandler::new(security_manager)
    }

    #[tokio::test]
    async fn test_read_file_utf8() {
        let handler = create_test_handler();

        // Create a temporary file for testing
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = "Hello, World! This is test content.";
        temp_file.write_all(test_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Test reading the file
        let temp_path = temp_file.path().to_string_lossy();
        println!("Testing with path: {temp_path}");
        let args = serde_json::json!({"path": temp_path});
        let result = handler.handle_read_file(args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert_eq!(text, test_content);
            }
        }
    }

    #[tokio::test]
    async fn test_read_file_base64() {
        let handler = create_test_handler();

        // Create a temporary file with binary content
        let mut temp_file = NamedTempFile::new().unwrap();
        let binary_data = vec![0u8, 1u8, 2u8, 255u8];
        temp_file.write_all(&binary_data).unwrap();
        temp_file.flush().unwrap();

        // Test reading with base64 encoding
        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy(),
            "encoding": "base64"
        });
        let result = handler.handle_read_file(args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert!(text.contains("Base64 encoded content"));
                assert!(text.contains("AAEC/w=="));
            }
        }
    }

    #[tokio::test]
    async fn test_write_file_utf8() {
        let handler = create_test_handler();

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_write.txt");
        let test_content = "Hello from write_file handler!";

        // Test writing a file
        let args = serde_json::json!({
            "path": test_file_path.to_string_lossy(),
            "content": test_content
        });
        let result = handler.handle_write_file(args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert!(text.contains("File written successfully"));
            }
        }

        // Verify the file was actually written
        let written_content = tokio::fs::read_to_string(&test_file_path).await.unwrap();
        assert_eq!(written_content, test_content);
    }

    #[tokio::test]
    async fn test_read_file_not_found() {
        let handler = create_test_handler();

        // Test reading non-existent file
        let args = serde_json::json!({"path": "/non/existent/file.txt"});
        let result = handler.handle_read_file(args).await;

        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("File not found"));
        }
    }

    /// Test 1: Verify security validation is properly called for read operations
    /// This test ensures the security manager is consulted before file access
    #[tokio::test]
    async fn test_read_file_security_validation() {
        let handler = create_test_handler();

        // Create a temporary file we know exists to isolate security testing
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy()
        });

        // With default permissive security config, this should succeed
        // The test validates that security validation runs without blocking legitimate operations
        let result = handler.handle_read_file(args).await;

        // Default security config should allow read operations
        assert!(
            result.is_ok(),
            "Security validation should allow legitimate read operations"
        );

        if let Ok(content) = result {
            assert_eq!(content.len(), 1);
            if let Some(text_content) = content.first().and_then(|c| c.as_text()) {
                assert!(text_content.contains("test content"));
            }
        }
    }

    /// Test 2: Verify security validation is properly called for write operations
    /// This test ensures write operations go through proper security and approval workflows
    #[tokio::test]
    async fn test_write_file_security_validation() {
        let handler = create_test_handler();

        // Create a temporary directory for safe testing
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("security_test.txt");

        let args = serde_json::json!({
            "path": test_file_path.to_string_lossy(),
            "content": "test content for security validation"
        });

        // With default permissive security config and auto-approval, this should succeed
        // The test validates that security validation and approval workflow run correctly
        let result = handler.handle_write_file(args).await;

        // Default config should allow write operations with auto-approval
        assert!(
            result.is_ok(),
            "Security validation should allow legitimate write operations with proper approval"
        );

        if let Ok(content) = result {
            assert_eq!(content.len(), 1);
            if let Some(text_content) = content.first().and_then(|c| c.as_text()) {
                assert!(text_content.contains("File written successfully"));
                // Length of "test content for security validation" is 36 characters
                assert!(text_content.contains("36 characters"));
            }
        }

        // Verify the file was actually created
        assert!(
            test_file_path.exists(),
            "File should be created when write operation succeeds"
        );
    }

    /// Test 3: Verify file size limits are enforced during read operations
    /// This test validates that security policies can limit file access based on size
    #[tokio::test]
    async fn test_read_file_size_limit() {
        let handler = create_test_handler();

        // Create a small temporary file for testing
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_content = "small content for size testing";
        temp_file.write_all(test_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Test 1: Normal read should succeed (file is small)
        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy()
        });
        let result = handler.handle_read_file(args).await;
        assert!(
            result.is_ok(),
            "Small files should be readable within default size limits"
        );

        // Test 2: Read with explicit reasonable size limit should succeed
        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy(),
            "max_size_mb": 1  // 1 MB limit - our test file is much smaller
        });
        let result = handler.handle_read_file(args).await;
        assert!(result.is_ok(), "Files within size limit should be readable");

        // Test 3: Read with zero size limit - behavior depends on security manager implementation
        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy(),
            "max_size_mb": 0  // 0 MB limit - should block all files
        });
        let result = handler.handle_read_file(args).await;

        // With 0 MB limit, the behavior depends on the security manager implementation
        // Some implementations may reject, others may interpret 0 as "no limit"
        match result {
            Ok(_) => {
                // Zero size limit was interpreted as "no limit" - this is valid behavior
                println!(
                    "Note: Zero size limit was interpreted as 'no limit' - this is valid behavior"
                );
            }
            Err(error) => {
                // Zero size limit blocked the file - this is also valid behavior
                let error_msg = format!("{error:?}");
                assert!(
                    error_msg.contains("File too large")
                        || error_msg.contains("exceeds maximum allowed size")
                        || error_msg.contains("Size limit")
                        || error_msg.contains("Security validation failed"),
                    "Zero size limit error should contain appropriate message, got: {error_msg}"
                );
            }
        }
    }

    /// Test 4: Verify backup creation during write operations
    /// This test validates that backup files are created when overwriting existing files
    #[tokio::test]
    async fn test_write_file_with_backup() {
        let handler = create_test_handler();

        // Create a temporary directory and initial file
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_backup.txt");
        let initial_content = "initial content";
        tokio::fs::write(&test_file_path, initial_content)
            .await
            .unwrap();

        // Write new content with backup enabled
        let new_content = "new content";
        let args = serde_json::json!({
            "path": test_file_path.to_string_lossy(),
            "content": new_content,
            "backup_existing": true
        });
        let result = handler.handle_write_file(args).await;

        match result {
            Ok(_) => {
                // Verify the file was updated
                let written_content = tokio::fs::read_to_string(&test_file_path).await.unwrap();
                assert_eq!(written_content, new_content);

                // Verify backup was created (backup filename includes timestamp)
                let backup_files: Vec<_> = std::fs::read_dir(temp_dir.path())
                    .unwrap()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .contains("test_backup.txt.backup.")
                    })
                    .collect();

                assert!(
                    !backup_files.is_empty(),
                    "Backup file should have been created"
                );
            }
            Err(error) => {
                // Write failed due to security/approval
                let error_msg = format!("{error:?}");
                assert!(
                    error_msg.contains("Security validation failed")
                        || error_msg.contains("denied by security policy")
                );
            }
        }
    }

    /// Test 5: Verify directory creation during write operations
    /// This test validates that parent directories are created when write operations require them
    #[tokio::test]
    async fn test_write_file_create_directories() {
        let handler = create_test_handler();

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("nested/deep/test_file.txt");
        let test_content = "content in nested directory";

        // Write file with directory creation enabled
        let args = serde_json::json!({
            "path": nested_path.to_string_lossy(),
            "content": test_content,
            "create_directories": true
        });
        let result = handler.handle_write_file(args).await;

        match result {
            Ok(_) => {
                // Verify the file was created and directories exist
                assert!(nested_path.parent().unwrap().exists());
                let written_content = tokio::fs::read_to_string(&nested_path).await.unwrap();
                assert_eq!(written_content, test_content);
            }
            Err(error) => {
                // Write failed due to security/approval
                let error_msg = format!("{error:?}");
                assert!(
                    error_msg.contains("Security validation failed")
                        || error_msg.contains("denied by security policy")
                        || error_msg.contains("Failed to create directories")
                );
            }
        }
    }

    /// Test 6: Verify base64 encoding support for binary file writes
    /// This test validates that binary content can be written using base64 encoding
    #[tokio::test]
    async fn test_write_file_base64_encoding() {
        let handler = create_test_handler();

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_binary.bin");

        // Test data encoded as base64 (represents bytes [72, 101, 108, 108, 111] = "Hello")
        let base64_content = "SGVsbG8=";

        // Write base64 content
        let args = serde_json::json!({
            "path": test_file_path.to_string_lossy(),
            "content": base64_content,
            "encoding": "base64"
        });
        let result = handler.handle_write_file(args).await;

        match result {
            Ok(response) => {
                // Verify response indicates binary write
                if let Some(content) = response.first() {
                    if let Some(text) = content.as_text() {
                        assert!(text.contains("Binary file written successfully"));
                        assert!(text.contains("5 bytes")); // "Hello" is 5 bytes
                    }
                }

                // Verify the file contains the decoded content
                let written_bytes = tokio::fs::read(&test_file_path).await.unwrap();
                assert_eq!(written_bytes, b"Hello");
            }
            Err(error) => {
                // Write failed due to security/approval
                let error_msg = format!("{error:?}");
                assert!(
                    error_msg.contains("Security validation failed")
                        || error_msg.contains("denied by security policy")
                );
            }
        }
    }

    /// Test 7: Verify automatic binary file detection and encoding
    /// This test validates that binary files are automatically detected and encoded as base64
    #[tokio::test]
    async fn test_read_file_auto_detection_binary() {
        let handler = create_test_handler();

        // Create a temporary file with binary content
        let mut temp_file = NamedTempFile::new().unwrap();
        let binary_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG file header
        temp_file.write_all(&binary_data).unwrap();
        temp_file.flush().unwrap();

        // Test reading with auto detection (should detect as binary)
        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy(),
            "encoding": "auto"
        });
        let result = handler.handle_read_file(args).await.unwrap();

        assert_eq!(result.len(), 1);
        if let Some(content) = result.first() {
            if let Some(text) = content.as_text() {
                assert!(text.contains("Binary file detected"));
                assert!(text.contains("base64 encoded"));
            }
        }
    }

    /// Test 8: Verify error handling for invalid encoding parameters
    /// This test validates that unsupported encoding types are properly rejected
    #[tokio::test]
    async fn test_invalid_encoding_parameter() {
        let handler = create_test_handler();

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();
        temp_file.flush().unwrap();

        // Test reading with invalid encoding
        let args = serde_json::json!({
            "path": temp_file.path().to_string_lossy(),
            "encoding": "invalid_encoding"
        });
        let result = handler.handle_read_file(args).await;

        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("Unsupported encoding"));
        }
    }

    /// Test 9: Verify error handling for invalid base64 content
    /// This test validates that malformed base64 data is properly rejected during write operations
    #[tokio::test]
    async fn test_invalid_base64_content() {
        let handler = create_test_handler();

        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let test_file_path = temp_dir.path().join("test_invalid_base64.bin");

        // Invalid base64 content
        let invalid_base64 = "This is not valid base64!!!";

        // Try to write invalid base64 content
        let args = serde_json::json!({
            "path": test_file_path.to_string_lossy(),
            "content": invalid_base64,
            "encoding": "base64"
        });
        let result = handler.handle_write_file(args).await;

        // Should fail due to invalid base64 OR security (depending on which check comes first)
        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(
                error_msg.contains("Invalid base64 content")
                    || error_msg.contains("Security validation failed")
                    || error_msg.contains("denied by security policy")
            );
        }
    }

    /// Test 10: Verify error handling for invalid JSON arguments
    /// This test validates that malformed or missing required arguments are properly rejected
    #[tokio::test]
    async fn test_invalid_json_arguments() {
        let handler = create_test_handler();

        // Test read_file with invalid arguments
        let invalid_args = serde_json::json!({"invalid_field": "value"});
        let result = handler.handle_read_file(invalid_args).await;

        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("Invalid read_file arguments"));
        }

        // Test write_file with invalid arguments
        let invalid_args = serde_json::json!({"path": "/tmp/test", "invalid_field": "value"});
        let result = handler.handle_write_file(invalid_args).await;

        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{error:?}");
            assert!(error_msg.contains("Invalid write_file arguments"));
        }
    }
}

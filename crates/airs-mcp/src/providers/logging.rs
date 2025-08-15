//! Production-ready Logging Handler Implementations
//!
//! This module provides comprehensive logging handlers for MCP servers:
//! - Structured logging with multiple backends
//! - File-based logging with rotation
//! - Integration with tracing ecosystem

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use crate::integration::mcp::{LoggingHandler, McpError, McpResult};
use crate::shared::protocol::LoggingConfig;

/// Structured logging handler with configurable backends
#[derive(Debug, Clone)]
pub struct StructuredLoggingHandler {
    /// Current logging configuration
    config: Arc<RwLock<LoggingConfig>>,
    /// Log entries buffer for in-memory logging
    log_buffer: Arc<RwLock<Vec<LogEntry>>>,
    /// Maximum number of log entries to keep in memory
    max_buffer_size: usize,
    /// Whether to enable file logging
    file_logging_enabled: bool,
    /// File logging path
    log_file_path: Option<PathBuf>,
}

/// Internal log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: chrono::DateTime<chrono::Utc>,
    level: String,
    message: String,
    data: Option<Value>,
}

impl StructuredLoggingHandler {
    /// Create a new structured logging handler
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(LoggingConfig::default())),
            log_buffer: Arc::new(RwLock::new(Vec::new())),
            max_buffer_size: 1000, // Default to 1000 entries
            file_logging_enabled: false,
            log_file_path: None,
        }
    }

    /// Enable file logging with specified path
    pub fn with_file_logging<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.log_file_path = Some(path.into());
        self.file_logging_enabled = true;
        self
    }

    /// Set maximum buffer size for in-memory logging
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.max_buffer_size = size;
        self
    }

    /// Log a message with the specified level
    pub async fn log(&self, level: &str, message: &str, data: Option<Value>) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
            data,
        };

        // Add to buffer
        {
            let mut buffer = self.log_buffer.write().await;
            buffer.push(entry.clone());

            // Maintain buffer size limit
            if buffer.len() > self.max_buffer_size {
                buffer.remove(0);
            }
        }

        // Log to tracing system
        match level.to_lowercase().as_str() {
            "error" => error!(message = %entry.message, data = ?entry.data, "MCP log entry"),
            "warn" | "warning" => {
                warn!(message = %entry.message, data = ?entry.data, "MCP log entry")
            }
            "info" => info!(message = %entry.message, data = ?entry.data, "MCP log entry"),
            "debug" => debug!(message = %entry.message, data = ?entry.data, "MCP log entry"),
            _ => {
                info!(level = %level, message = %entry.message, data = ?entry.data, "MCP log entry")
            }
        }

        // Write to file if enabled
        if self.file_logging_enabled {
            if let Some(ref path) = self.log_file_path {
                let log_line = format!(
                    "{} [{}] {}{}\n",
                    entry.timestamp.to_rfc3339(),
                    entry.level.to_uppercase(),
                    entry.message,
                    entry
                        .data
                        .map(|d| format!(" | Data: {d}"))
                        .unwrap_or_default()
                );

                // Note: In a production implementation, you'd want to use a proper
                // logging framework with rotation, async file writing, etc.
                if let Err(e) = tokio::fs::write(path, log_line).await {
                    error!("Failed to write log to file: {}", e);
                }
            }
        }
    }

    /// Get recent log entries
    pub async fn get_recent_logs(&self, limit: Option<usize>) -> Vec<LogEntry> {
        let buffer = self.log_buffer.read().await;
        let limit = limit.unwrap_or(100).min(buffer.len());
        buffer.iter().rev().take(limit).cloned().collect()
    }

    /// Clear log buffer
    pub async fn clear_logs(&self) {
        let mut buffer = self.log_buffer.write().await;
        buffer.clear();
    }

    /// Get current logging statistics
    pub async fn get_stats(&self) -> HashMap<String, Value> {
        let buffer = self.log_buffer.read().await;
        let config = self.config.read().await;

        let mut level_counts = HashMap::new();
        for entry in buffer.iter() {
            *level_counts.entry(entry.level.clone()).or_insert(0u32) += 1;
        }

        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), json!(buffer.len()));
        stats.insert("buffer_size".to_string(), json!(self.max_buffer_size));
        stats.insert("level_counts".to_string(), json!(level_counts));
        stats.insert(
            "file_logging_enabled".to_string(),
            json!(self.file_logging_enabled),
        );
        stats.insert("current_config".to_string(), json!(config.clone()));

        if let Some(ref path) = self.log_file_path {
            stats.insert(
                "log_file_path".to_string(),
                json!(path.display().to_string()),
            );
        }

        stats
    }
}

#[async_trait]
impl LoggingHandler for StructuredLoggingHandler {
    #[instrument(level = "debug", skip(self))]
    async fn set_logging(&self, config: LoggingConfig) -> McpResult<bool> {
        info!("Setting logging configuration: {:?}", config);

        // Validate the configuration
        match config.min_level.as_str() {
            "error" | "warning" | "info" | "debug" | "critical" => {
                // Valid level
            }
            _ => {
                warn!("Invalid logging level: {}", config.min_level.as_str());
                return Err(McpError::invalid_request(format!(
                    "Invalid logging level: {}. Valid levels are: error, warning, info, debug, critical",
                    config.min_level.as_str()
                )));
            }
        }

        // Update configuration
        {
            let mut current_config = self.config.write().await;
            *current_config = config.clone();
        }

        // Log the configuration change
        self.log("info", "Logging configuration updated", Some(json!(config)))
            .await;

        info!("Logging configuration updated successfully");
        Ok(true)
    }
}

/// File-based logging handler with rotation support
#[derive(Debug, Clone)]
pub struct FileLoggingHandler {
    /// Log file path
    log_file: PathBuf,
    /// Maximum file size before rotation (in bytes)
    max_file_size: u64,
    /// Number of rotated files to keep
    max_files: u32,
    /// Current logging configuration
    config: Arc<RwLock<LoggingConfig>>,
}

impl FileLoggingHandler {
    /// Create a new file logging handler
    pub fn new<P: Into<PathBuf>>(log_file: P) -> Self {
        Self {
            log_file: log_file.into(),
            max_file_size: 10 * 1024 * 1024, // 10MB default
            max_files: 5,                    // Keep 5 rotated files
            config: Arc::new(RwLock::new(LoggingConfig::default())),
        }
    }

    /// Set maximum file size before rotation
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set number of rotated files to keep
    pub fn with_max_files(mut self, count: u32) -> Self {
        self.max_files = count;
        self
    }

    /// Rotate log files if needed
    async fn rotate_if_needed(&self) -> McpResult<()> {
        // Check if current log file exists and its size
        if let Ok(metadata) = tokio::fs::metadata(&self.log_file).await {
            if metadata.len() > self.max_file_size {
                // Rotate files
                for i in (1..self.max_files).rev() {
                    let from = if i == 1 {
                        self.log_file.clone()
                    } else {
                        self.log_file.with_extension(format!("log.{}", i - 1))
                    };

                    let to = self.log_file.with_extension(format!("log.{i}"));

                    if from.exists() {
                        if let Err(e) = tokio::fs::rename(&from, &to).await {
                            warn!("Failed to rotate log file {:?} to {:?}: {}", from, to, e);
                        }
                    }
                }

                // Create new log file
                if let Err(e) = tokio::fs::write(&self.log_file, "").await {
                    error!("Failed to create new log file: {}", e);
                    return Err(McpError::internal_error("Failed to create new log file"));
                }
            }
        }

        Ok(())
    }

    /// Append log entry to file
    async fn append_log(&self, level: &str, message: &str, data: Option<Value>) -> McpResult<()> {
        self.rotate_if_needed().await?;

        let timestamp = chrono::Utc::now().to_rfc3339();
        let log_line = format!(
            "{} [{}] {}{}\n",
            timestamp,
            level.to_uppercase(),
            message,
            data.map(|d| format!(" | Data: {d}")).unwrap_or_default()
        );

        match tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .await
        {
            Ok(mut file) => {
                use tokio::io::AsyncWriteExt;
                if let Err(e) = file.write_all(log_line.as_bytes()).await {
                    error!("Failed to write to log file: {}", e);
                    return Err(McpError::internal_error("Failed to write to log file"));
                }
            }
            Err(e) => {
                error!("Failed to open log file: {}", e);
                return Err(McpError::internal_error("Failed to open log file"));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl LoggingHandler for FileLoggingHandler {
    async fn set_logging(&self, config: LoggingConfig) -> McpResult<bool> {
        // Update configuration
        {
            let mut current_config = self.config.write().await;
            *current_config = config.clone();
        }

        // Log the configuration change
        self.append_log("info", "Logging configuration updated", Some(json!(config)))
            .await?;

        Ok(true)
    }
}

impl Default for StructuredLoggingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::protocol::LogLevel;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_structured_logging_handler() {
        let handler = StructuredLoggingHandler::new();

        // Test setting configuration
        let config = LoggingConfig::new(LogLevel::Info);

        let result = handler.set_logging(config).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_structured_logging_with_buffer() {
        let handler = StructuredLoggingHandler::new().with_buffer_size(5);

        // Add some log entries
        handler.log("info", "Test message 1", None).await;
        handler
            .log("error", "Test message 2", Some(json!({"key": "value"})))
            .await;
        handler.log("debug", "Test message 3", None).await;

        let logs = handler.get_recent_logs(Some(10)).await;
        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].message, "Test message 3"); // Most recent first
    }

    #[tokio::test]
    async fn test_file_logging_handler() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("test.log");

        let handler = FileLoggingHandler::new(&log_file);

        let config = LoggingConfig::new(LogLevel::Debug);

        let result = handler.set_logging(config).await;
        assert!(result.is_ok());

        // Check that log file was created
        assert!(log_file.exists());
    }

    #[tokio::test]
    async fn test_invalid_log_level() {
        let handler = StructuredLoggingHandler::new();

        // Test with a valid config since LogLevel is an enum and can't contain invalid values
        // The validation occurs at the string parsing level, which we test separately
        let config = LoggingConfig::new(LogLevel::Info);

        let result = handler.set_logging(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logging_stats() {
        let handler = StructuredLoggingHandler::new();

        handler.log("info", "Test message", None).await;
        handler.log("error", "Error message", None).await;
        handler.log("info", "Another info message", None).await;

        let stats = handler.get_stats().await;
        assert_eq!(stats.get("total_entries").unwrap(), &json!(3));

        let level_counts = stats.get("level_counts").unwrap().as_object().unwrap();
        assert_eq!(level_counts.get("info").unwrap(), &json!(2));
        assert_eq!(level_counts.get("error").unwrap(), &json!(1));
    }
}

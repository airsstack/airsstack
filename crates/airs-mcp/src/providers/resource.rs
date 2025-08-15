//! Production-ready Resource Provider Implementations
//!
//! This module provides comprehensive resource providers for common use cases:
//! - File system access with security constraints
//! - Configuration management
//! - Database resource exposure

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use serde_yml;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use crate::integration::mcp::{McpError, McpResult, ResourceProvider};
use crate::shared::protocol::{Content, MimeType, Resource, Uri};

/// File system resource provider with security constraints
#[derive(Debug, Clone)]
pub struct FileSystemResourceProvider {
    /// Base directory for file access (security constraint)
    base_path: PathBuf,
    /// Allowed file extensions
    allowed_extensions: Vec<String>,
    /// Maximum file size in bytes
    max_file_size: u64,
    /// Cache of discovered resources
    resource_cache: Arc<RwLock<HashMap<String, Resource>>>,
}

impl FileSystemResourceProvider {
    /// Create a new file system resource provider
    ///
    /// # Arguments
    /// * `base_path` - Base directory for file access (security constraint)
    ///
    /// # Security
    /// All file access is restricted to within the base_path directory.
    /// Attempts to access files outside this directory will be rejected.
    pub fn new<P: AsRef<Path>>(base_path: P) -> McpResult<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        if !base_path.exists() {
            return Err(McpError::invalid_request(format!(
                "Base path does not exist: {}",
                base_path.display()
            )));
        }

        if !base_path.is_dir() {
            return Err(McpError::invalid_request(format!(
                "Base path is not a directory: {}",
                base_path.display()
            )));
        }

        Ok(Self {
            base_path,
            allowed_extensions: vec![
                "txt".to_string(),
                "md".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "toml".to_string(),
                "xml".to_string(),
                "csv".to_string(),
                "log".to_string(),
            ],
            max_file_size: 10 * 1024 * 1024, // 10MB default
            resource_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add allowed file extensions
    pub fn with_allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = extensions;
        self
    }

    /// Set maximum file size
    pub fn with_max_file_size(mut self, max_size: u64) -> Self {
        self.max_file_size = max_size;
        self
    }

    /// Validate if a file path is safe and allowed
    fn validate_path(&self, file_path: &str) -> McpResult<PathBuf> {
        // Remove file:// prefix if present
        let clean_path = if let Some(stripped) = file_path.strip_prefix("file://") {
            stripped
        } else {
            file_path
        };

        let path = Path::new(clean_path);
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_path.join(path)
        };

        // Canonicalize to resolve any .. or . components
        let canonical_path = absolute_path
            .canonicalize()
            .map_err(|e| McpError::resource_not_found(format!("Path error: {e}")))?;

        // Ensure the canonical path is within our base directory
        if !canonical_path.starts_with(&self.base_path) {
            return Err(McpError::invalid_request(
                "Access outside base directory not allowed",
            ));
        }

        // Check file extension
        if let Some(ext) = canonical_path.extension() {
            if let Some(ext_str) = ext.to_str() {
                if !self.allowed_extensions.contains(&ext_str.to_lowercase()) {
                    return Err(McpError::invalid_request(format!(
                        "File extension '{ext_str}' not allowed"
                    )));
                }
            }
        }

        Ok(canonical_path)
    }

    /// Get MIME type for a file based on extension
    fn get_mime_type(&self, path: &Path) -> McpResult<MimeType> {
        let mime_str = match path.extension().and_then(|e| e.to_str()) {
            Some("txt") | Some("log") => "text/plain",
            Some("md") => "text/markdown",
            Some("json") => "application/json",
            Some("yaml") | Some("yml") => "application/yaml",
            Some("toml") => "application/toml",
            Some("xml") => "application/xml",
            Some("csv") => "text/csv",
            _ => "text/plain", // Default fallback
        };

        MimeType::new(mime_str)
            .map_err(|e| McpError::internal_error(format!("Invalid MIME type: {e}")))
    }

    /// Discover resources in a directory recursively
    async fn discover_resources_in_dir(&self, dir_path: &Path) -> McpResult<Vec<Resource>> {
        let mut resources = Vec::new();

        // Use iterative approach to avoid Send issues with recursive async
        let mut dirs_to_process = vec![dir_path.to_path_buf()];

        while let Some(current_dir) = dirs_to_process.pop() {
            let mut entries = fs::read_dir(&current_dir)
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to read directory: {e}")))?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                McpError::internal_error(format!("Failed to read directory entry: {e}"))
            })? {
                let path = entry.path();

                if path.is_file() {
                    // Check if file extension is allowed
                    if let Some(ext) = path.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            if self.allowed_extensions.contains(&ext_str.to_lowercase()) {
                                // Check file size
                                if let Ok(metadata) = entry.metadata().await {
                                    if metadata.len() <= self.max_file_size {
                                        let relative_path =
                                            path.strip_prefix(&self.base_path).map_err(|e| {
                                                McpError::internal_error(format!("Path error: {e}"))
                                            })?;

                                        let uri_str = format!("file://{}", path.display());
                                        let uri = Uri::new(&uri_str).map_err(|e| {
                                            McpError::internal_error(format!("Invalid URI: {e}"))
                                        })?;

                                        let mime_type = self.get_mime_type(&path)?;

                                        let resource = Resource {
                                            uri,
                                            name: relative_path.display().to_string(),
                                            description: Some(format!(
                                                "File: {} ({} bytes)",
                                                relative_path.display(),
                                                metadata.len()
                                            )),
                                            mime_type: Some(mime_type),
                                        };

                                        resources.push(resource);
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() {
                    // Add subdirectory to processing queue
                    dirs_to_process.push(path);
                }
            }
        }

        Ok(resources)
    }
}

#[async_trait]
impl ResourceProvider for FileSystemResourceProvider {
    #[instrument(level = "debug", skip(self))]
    async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        info!(
            "Discovering file system resources in: {}",
            self.base_path.display()
        );

        // Check cache first
        {
            let cache = self.resource_cache.read().await;
            if !cache.is_empty() {
                debug!("Returning cached resources (count: {})", cache.len());
                return Ok(cache.values().cloned().collect());
            }
        }

        let resources = self.discover_resources_in_dir(&self.base_path).await?;

        // Update cache
        {
            let mut cache = self.resource_cache.write().await;
            cache.clear();
            for resource in &resources {
                cache.insert(resource.uri.to_string(), resource.clone());
            }
        }

        info!(
            resource_count = resources.len(),
            base_path = %self.base_path.display(),
            "File system resources discovered"
        );

        Ok(resources)
    }

    #[instrument(level = "debug", skip(self), fields(uri = %uri))]
    async fn read_resource(&self, uri: &str) -> McpResult<Vec<Content>> {
        info!(uri = %uri, "Reading file system resource");

        let file_path = self.validate_path(uri)?;

        // Check file exists and get metadata
        let metadata = fs::metadata(&file_path)
            .await
            .map_err(|e| McpError::resource_not_found(format!("File not found: {e}")))?;

        if !metadata.is_file() {
            return Err(McpError::resource_not_found("Path is not a file"));
        }

        if metadata.len() > self.max_file_size {
            return Err(McpError::invalid_request(format!(
                "File too large: {} bytes (max: {} bytes)",
                metadata.len(),
                self.max_file_size
            )));
        }

        // Read file content
        let content_bytes = fs::read(&file_path)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to read file: {e}")))?;

        let content_str = String::from_utf8(content_bytes)
            .map_err(|e| McpError::internal_error(format!("File is not valid UTF-8: {e}")))?;

        let content = vec![Content::text_with_uri(&content_str, uri)
            .map_err(|e| McpError::internal_error(format!("Failed to create content: {e}")))?];

        info!(
            uri = %uri,
            file_path = %file_path.display(),
            size = content_str.len(),
            "File system resource read successfully"
        );

        Ok(content)
    }
}

/// Configuration resource provider for application settings
#[derive(Debug, Clone)]
pub struct ConfigurationResourceProvider {
    /// Configuration values
    config: Arc<RwLock<HashMap<String, Value>>>,
    /// Configuration file path for persistence
    #[allow(dead_code)] // Framework for future persistence functionality
    config_file: Option<PathBuf>,
}

impl ConfigurationResourceProvider {
    /// Create a new configuration resource provider
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(HashMap::new())),
            config_file: None,
        }
    }

    /// Create a configuration provider from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> McpResult<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| McpError::resource_not_found(format!("Config file error: {e}")))?;

        let config: HashMap<String, Value> =
            if path.extension() == Some(std::ffi::OsStr::new("json")) {
                serde_json::from_str(&content)
                    .map_err(|e| McpError::invalid_request(format!("Invalid JSON config: {e}")))?
            } else if path.extension() == Some(std::ffi::OsStr::new("yaml"))
                || path.extension() == Some(std::ffi::OsStr::new("yml"))
            {
                serde_yml::from_str(&content)
                    .map_err(|e| McpError::invalid_request(format!("Invalid YAML config: {e}")))?
            } else {
                return Err(McpError::invalid_request("Unsupported config file format"));
            };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_file: Some(path),
        })
    }

    /// Set a configuration value
    pub async fn set_config(&self, key: String, value: Value) {
        let mut config = self.config.write().await;
        config.insert(key, value);
    }

    /// Get a configuration value
    pub async fn get_config(&self, key: &str) -> Option<Value> {
        let config = self.config.read().await;
        config.get(key).cloned()
    }
}

#[async_trait]
impl ResourceProvider for ConfigurationResourceProvider {
    async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        let config = self.config.read().await;
        let mut resources = Vec::new();

        for key in config.keys() {
            let uri_str = format!("config://{key}");
            let uri = Uri::new(&uri_str)
                .map_err(|e| McpError::internal_error(format!("Invalid URI: {e}")))?;

            let resource = Resource {
                uri,
                name: format!("Config: {key}"),
                description: Some(format!("Configuration value for '{key}'")),
                mime_type: Some(MimeType::new("application/json").unwrap()),
            };

            resources.push(resource);
        }

        Ok(resources)
    }

    async fn read_resource(&self, uri: &str) -> McpResult<Vec<Content>> {
        if !uri.starts_with("config://") {
            return Err(McpError::invalid_request("Invalid config URI"));
        }

        let key = &uri[9..]; // Remove "config://" prefix
        let config = self.config.read().await;

        match config.get(key) {
            Some(value) => {
                let content_str = serde_json::to_string_pretty(value).map_err(|e| {
                    McpError::internal_error(format!("JSON serialization error: {e}"))
                })?;

                let content = vec![Content::text_with_uri(&content_str, uri).map_err(|e| {
                    McpError::internal_error(format!("Failed to create content: {e}"))
                })?];

                Ok(content)
            }
            None => Err(McpError::resource_not_found(format!(
                "Configuration key '{key}' not found"
            ))),
        }
    }
}

/// Database resource provider for exposing database content
#[derive(Debug)]
pub struct DatabaseResourceProvider {
    /// Database connection string or identifier
    connection_id: String,
    /// Available tables/collections
    tables: Vec<String>,
}

impl DatabaseResourceProvider {
    /// Create a new database resource provider
    pub fn new(connection_id: String, tables: Vec<String>) -> Self {
        Self {
            connection_id,
            tables,
        }
    }
}

#[async_trait]
impl ResourceProvider for DatabaseResourceProvider {
    async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        let mut resources = Vec::new();

        for table in &self.tables {
            let uri_str = format!("db://{}/{}", self.connection_id, table);
            let uri = Uri::new(&uri_str)
                .map_err(|e| McpError::internal_error(format!("Invalid URI: {e}")))?;

            let resource = Resource {
                uri,
                name: format!("Table: {table}"),
                description: Some(format!(
                    "Database table '{}' in '{}'",
                    table, self.connection_id
                )),
                mime_type: Some(MimeType::new("application/json").unwrap()),
            };

            resources.push(resource);
        }

        Ok(resources)
    }

    async fn read_resource(&self, uri: &str) -> McpResult<Vec<Content>> {
        // This is a placeholder implementation
        // In a real implementation, you would:
        // 1. Parse the URI to extract connection and table info
        // 2. Connect to the database
        // 3. Execute a query to fetch data
        // 4. Return the results as JSON content

        if !uri.starts_with(&format!("db://{}/", self.connection_id)) {
            return Err(McpError::invalid_request("Invalid database URI"));
        }

        // For demonstration, return a placeholder
        let content = vec![Content::text_with_uri(
            format!(r#"{{"message": "Database resource not yet implemented", "uri": "{uri}"}}"#),
            uri,
        )
        .map_err(|e| McpError::internal_error(format!("Failed to create content: {e}")))?];

        Ok(content)
    }
}

impl Default for ConfigurationResourceProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_file_system_provider_creation() {
        let temp_dir = TempDir::new().unwrap();
        let provider = FileSystemResourceProvider::new(temp_dir.path()).unwrap();
        assert_eq!(provider.base_path, temp_dir.path());
    }

    #[tokio::test]
    async fn test_file_system_provider_invalid_path() {
        let result = FileSystemResourceProvider::new("/nonexistent/path");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_system_provider_list_resources() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.txt"), "Hello World")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("config.json"), r#"{"key": "value"}"#)
            .await
            .unwrap();

        let provider = FileSystemResourceProvider::new(temp_dir.path()).unwrap();
        let resources = provider.list_resources().await.unwrap();

        assert_eq!(resources.len(), 2);
        assert!(resources.iter().any(|r| r.name == "test.txt"));
        assert!(resources.iter().any(|r| r.name == "config.json"));
    }

    #[tokio::test]
    async fn test_configuration_provider() {
        let provider = ConfigurationResourceProvider::new();

        // Set some config values
        provider
            .set_config(
                "app_name".to_string(),
                Value::String("Test App".to_string()),
            )
            .await;
        provider
            .set_config(
                "port".to_string(),
                Value::Number(serde_json::Number::from(8080)),
            )
            .await;

        let resources = provider.list_resources().await.unwrap();
        assert_eq!(resources.len(), 2);

        // Test reading config
        let content = provider.read_resource("config://app_name").await.unwrap();
        assert_eq!(content.len(), 1);
        if let Content::Text { text, .. } = &content[0] {
            assert!(text.contains("Test App"));
        }
    }
}

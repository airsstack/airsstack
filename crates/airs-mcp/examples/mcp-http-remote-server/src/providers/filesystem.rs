//! File system resource provider with security boundaries
//!
//! Provides secure access to file system resources through MCP protocol.
//! Implements proper path validation and access control.

// Layer 1: Standard library imports
use std::path::{Path, PathBuf};

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc}; // Per workspace standards §3.2
use tokio::fs;
use tracing::{debug, warn, instrument};

// Layer 3: Internal module imports
use airs_mcp::integration::mcp::{ResourceProvider, McpError, McpResult};
use airs_mcp::shared::protocol::{Resource, Content, Uri, MimeType};

/// File system resource provider with security boundaries
#[derive(Debug, Clone)]
pub struct FileSystemResourceProvider {
    base_path: PathBuf,
    max_file_size: usize,
    allowed_extensions: Vec<String>,
}

impl FileSystemResourceProvider {
    /// Create new file system provider with security constraints
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path: base_path.canonicalize().unwrap_or(base_path),
            max_file_size: 1024 * 1024, // 1MB limit
            allowed_extensions: vec![
                "rs".to_string(),
                "toml".to_string(),
                "md".to_string(),
                "txt".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
            ],
        }
    }

    /// Validate path is within security boundaries
    fn validate_path(&self, path: &Path) -> Result<PathBuf, String> {
        let requested_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_path.join(path)
        };

        let canonical_path = requested_path.canonicalize()
            .map_err(|e| format!("Path resolution failed: {}", e))?;

        // Ensure path is within base directory
        if !canonical_path.starts_with(&self.base_path) {
            return Err("Access denied: Path outside allowed directory".to_string());
        }

        // Check file extension
        if let Some(extension) = canonical_path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            if !self.allowed_extensions.contains(&ext_str) {
                return Err(format!("Access denied: File extension '{}' not allowed", ext_str));
            }
        }

        Ok(canonical_path)
    }

    /// List directory contents safely
    #[instrument(skip(self))]
    async fn list_directory(&self, dir_path: &Path) -> Result<Vec<Resource>, String> {
        let validated_path = self.validate_path(dir_path)?;
        
        if !validated_path.is_dir() {
            return Err("Path is not a directory".to_string());
        }

        let mut entries = fs::read_dir(&validated_path)
            .await
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut resources = Vec::new();
        
        while let Some(entry) = entries.next_entry()
            .await
            .map_err(|e| format!("Failed to read directory entry: {}", e))? {
            
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.base_path)
                .unwrap_or(&path)
                .to_path_buf();

            let metadata = entry.metadata()
                .await
                .map_err(|e| format!("Failed to read metadata: {}", e))?;

            let _modified = metadata.modified()
                .ok()
                .and_then(|time| DateTime::from_timestamp(
                    time.duration_since(std::time::UNIX_EPOCH)
                        .ok()?
                        .as_secs() as i64, 0
                ))
                .unwrap_or_else(Utc::now);

            let resource = Resource {
                uri: Uri::new(format!("file://{}", relative_path.display()))
                    .map_err(|e| format!("Failed to create URI: {}", e))?,
                name: entry.file_name().to_string_lossy().into_owned(),
                description: Some(if metadata.is_dir() {
                    "Directory".to_string()
                } else {
                    format!("File ({} bytes)", metadata.len())
                }),
                mime_type: if metadata.is_dir() {
                    Some(MimeType::new("inode/directory")
                        .map_err(|e| format!("Failed to create MIME type: {}", e))?)
                } else {
                    Some(MimeType::new("text/plain")
                        .map_err(|e| format!("Failed to create MIME type: {}", e))?)
                },
            };

            resources.push(resource);
        }

        debug!("Listed {} resources in directory: {}", resources.len(), dir_path.display());
        Ok(resources)
    }

    /// Read file content safely
    #[instrument(skip(self))]
    async fn read_file(&self, file_path: &Path) -> Result<Vec<Content>, String> {
        let validated_path = self.validate_path(file_path)?;
        
        if !validated_path.is_file() {
            return Err("Path is not a file".to_string());
        }

        let metadata = fs::metadata(&validated_path)
            .await
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        if metadata.len() > self.max_file_size as u64 {
            return Err(format!("File too large: {} bytes (max: {} bytes)", 
                metadata.len(), self.max_file_size));
        }

        let content = fs::read_to_string(&validated_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        debug!("Read file: {} ({} bytes)", validated_path.display(), content.len());

        let text_content = Content::text(content);
        Ok(vec![text_content])
    }
}

#[async_trait]
impl ResourceProvider for FileSystemResourceProvider {
    #[instrument(skip(self))]
    async fn list_resources(&self) -> McpResult<Vec<Resource>> {
        match self.list_directory(&self.base_path).await {
            Ok(resources) => Ok(resources),
            Err(e) => {
                warn!("Failed to list resources: {}", e);
                Err(McpError::internal_error(e))
            }
        }
    }

    #[instrument(skip(self))]
    async fn read_resource(&self, uri: &str) -> McpResult<Vec<Content>> {
        // Parse URI to extract file path
        let path = if uri.starts_with("file://") {
            PathBuf::from(&uri[7..])
        } else {
            PathBuf::from(uri)
        };

        match self.read_file(&path).await {
            Ok(content) => Ok(content),
            Err(e) => {
                warn!("Failed to read resource '{}': {}", uri, e);
                Err(McpError::internal_error(e))
            }
        }
    }
}

//! Configuration settings management for AIRS MCP-FS

// Layer 1: Standard library imports
// (None needed for this module)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (None needed yet)

/// Main configuration structure for AIRS MCP-FS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Security configuration
    pub security: SecurityConfig,
    /// Binary processing settings
    pub binary: BinaryConfig,
    /// MCP server configuration
    pub server: ServerConfig,
}

/// Security-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Allowed file paths patterns
    pub allowed_paths: Vec<String>,
    /// Denied file paths patterns
    pub denied_paths: Vec<String>,
    /// Require approval for write operations
    pub require_approval: bool,
}

/// Binary processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryConfig {
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Enable image processing
    pub enable_image_processing: bool,
    /// Enable PDF processing
    pub enable_pdf_processing: bool,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name for MCP capabilities
    pub name: String,
    /// Server version
    pub version: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            security: SecurityConfig {
                allowed_paths: vec!["**/*".to_string()], // Allow all by default
                denied_paths: vec![],
                require_approval: true,
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
}

impl Settings {
    /// Load settings from configuration file or use defaults
    pub fn load() -> anyhow::Result<Self> {
        // TODO: Implement actual configuration loading in subsequent tasks
        Ok(Self::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        
        assert_eq!(settings.server.name, "airs-mcp-fs");
        assert!(settings.security.require_approval);
        assert_eq!(settings.binary.max_file_size, 100 * 1024 * 1024);
        assert!(settings.binary.enable_image_processing);
        assert!(settings.binary.enable_pdf_processing);
    }

    #[test]
    fn test_settings_load() {
        let result = Settings::load();
        assert!(result.is_ok());
        
        let settings = result.unwrap();
        assert_eq!(settings.server.name, "airs-mcp-fs");
    }
}

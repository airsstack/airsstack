//! Configuration settings management for AIRS MCP-FS

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use anyhow::Context;
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (None needed yet)
use crate::config::loader::ConfigurationLoader;

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

/// Security-related configuration with comprehensive policy framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Filesystem access configuration
    pub filesystem: FilesystemConfig,
    /// Operation-level security rules
    pub operations: OperationConfig,
    /// Named security policies for different file types and patterns
    pub policies: HashMap<String, SecurityPolicy>,
}

/// Filesystem access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    /// Allowed file paths patterns (glob syntax)
    pub allowed_paths: Vec<String>,
    /// Denied file paths patterns (glob syntax, takes precedence)
    pub denied_paths: Vec<String>,
}

/// Operation-level security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationConfig {
    /// Allow read operations in allowed_paths
    pub read_allowed: bool,
    /// Write operations require explicit policy match
    pub write_requires_policy: bool,
    /// Delete operations require explicit "delete" permission in policy
    pub delete_requires_explicit_allow: bool,
    /// Directory creation allowed in allowed_paths
    pub create_dir_allowed: bool,
}

/// Named security policy for specific file patterns and operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// File patterns this policy applies to (glob syntax)
    pub patterns: Vec<String>,
    /// Allowed operations for files matching patterns
    pub operations: Vec<String>,
    /// Risk level for audit logging and monitoring
    pub risk_level: RiskLevel,
    /// Optional description of this policy
    #[serde(default)]
    pub description: Option<String>,
}

/// Risk level for operations and audit logging
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low risk operations (normal source code, documentation)
    Low,
    /// Medium risk operations (configuration files, build scripts)
    Medium,
    /// High risk operations (system files, credentials)
    High,
    /// Critical risk operations (security-sensitive files)
    Critical,
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

/// Builder for creating Settings with different security configurations
pub struct SettingsBuilder {
    security_mode: SecurityMode,
}

/// Security mode for configuring Settings behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// Production mode: Secure by default, requires explicit policies for sensitive operations
    Production,
    /// Development mode: Balanced security, allows most development tasks
    Development,
    /// Permissive mode: Minimal restrictions, suitable for testing and development
    Permissive,
}

impl SettingsBuilder {
    /// Create a new builder with production security mode (secure by default)
    pub fn new() -> Self {
        Self {
            security_mode: SecurityMode::Production,
        }
    }

    /// Set security mode to production (secure by default)
    pub fn secure(mut self) -> Self {
        self.security_mode = SecurityMode::Production;
        self
    }

    /// Set security mode to development (balanced security)
    pub fn development(mut self) -> Self {
        self.security_mode = SecurityMode::Development;
        self
    }

    /// Set security mode to permissive (minimal restrictions)
    pub fn permissive(mut self) -> Self {
        self.security_mode = SecurityMode::Permissive;
        self
    }

    /// Build the Settings with the configured security mode
    pub fn build(self) -> Settings {
        // Create default security policies that apply to all modes
        let mut policies = HashMap::new();

        // Source code policy - low risk, read and write allowed
        policies.insert(
            "source_code".to_string(),
            SecurityPolicy {
                patterns: vec![
                    "**/*.{rs,py,js,ts,jsx,tsx}".to_string(),
                    "**/*.{c,cpp,h,hpp}".to_string(),
                    "**/*.{java,kt,scala}".to_string(),
                ],
                operations: vec!["read".to_string(), "write".to_string()],
                risk_level: RiskLevel::Low,
                description: Some("Source code files - safe for development".to_string()),
            },
        );

        // Documentation policy - low risk, read and write allowed
        policies.insert(
            "documentation".to_string(),
            SecurityPolicy {
                patterns: vec![
                    "**/*.{md,txt,rst}".to_string(),
                    "**/README*".to_string(),
                    "**/CHANGELOG*".to_string(),
                ],
                operations: vec!["read".to_string(), "write".to_string()],
                risk_level: RiskLevel::Low,
                description: Some("Documentation files - safe for editing".to_string()),
            },
        );

        // Configuration policy - medium risk, read and write with caution
        policies.insert(
            "config_files".to_string(),
            SecurityPolicy {
                patterns: vec![
                    "**/Cargo.toml".to_string(),
                    "**/*.{json,yaml,yml,toml}".to_string(),
                    "**/*.{xml,ini,conf}".to_string(),
                ],
                operations: vec!["read".to_string(), "write".to_string()],
                risk_level: RiskLevel::Medium,
                description: Some("Configuration files - moderate risk".to_string()),
            },
        );

        // Build artifacts policy - low risk, delete allowed for cleanup
        policies.insert(
            "build_artifacts".to_string(),
            SecurityPolicy {
                patterns: vec![
                    "**/target/**".to_string(),
                    "**/dist/**".to_string(),
                    "**/build/**".to_string(),
                    "**/*.{tmp,bak,log}".to_string(),
                ],
                operations: vec!["read".to_string(), "delete".to_string()],
                risk_level: RiskLevel::Low,
                description: Some(
                    "Build artifacts and temporary files - safe to clean".to_string(),
                ),
            },
        );

        // Configure security settings based on mode
        let (allowed_paths, write_requires_policy, delete_requires_explicit_allow) =
            match self.security_mode {
                SecurityMode::Permissive => {
                    // Permissive mode: Allow all operations, suitable for testing
                    policies.insert(
                        "permissive_universal".to_string(),
                        SecurityPolicy {
                            patterns: vec!["**/*".to_string()], // Match all paths
                            operations: vec![
                                "read".to_string(),
                                "write".to_string(),
                                "delete".to_string(),
                                "list".to_string(),
                                "create_dir".to_string(),
                                "move".to_string(),
                                "copy".to_string(),
                            ],
                            risk_level: RiskLevel::Low,
                            description: Some(
                                "Universal permissive policy - allows all operations".to_string(),
                            ),
                        },
                    );

                    (
                        vec![
                            "/**/*".to_string(), // Allow all absolute paths
                            "**/*".to_string(),  // Allow all relative paths
                        ],
                        false, // Don't require policies for writes
                        false, // Don't require explicit delete permissions
                    )
                }
                SecurityMode::Development => {
                    // Development mode: Balanced security, reasonable for development work
                    (
                        vec![
                            "~/projects/**/*".to_string(),
                            "~/Documents/**/*".to_string(),
                            "~/Desktop/**/*".to_string(),
                            "./**/*".to_string(), // Current directory and subdirectories
                        ],
                        false, // Allow writes without strict policy requirements
                        true,  // Still require explicit delete permissions for safety
                    )
                }
                SecurityMode::Production => {
                    // Production mode: Secure by default
                    (
                        vec![
                            "~/projects/**/*".to_string(),
                            "~/Documents/**/*.{md,txt,rst}".to_string(),
                        ],
                        true, // Require policies for writes
                        true, // Require explicit delete permissions
                    )
                }
            };

        Settings {
            security: SecurityConfig {
                filesystem: FilesystemConfig {
                    allowed_paths,
                    denied_paths: vec![
                        "**/.git/**".to_string(),
                        "**/.env*".to_string(),
                        "~/.*/**".to_string(),         // Hidden directories
                        "**/id_rsa*".to_string(),      // SSH keys
                        "**/credentials*".to_string(), // Credential files
                        "**/secrets*".to_string(),     // Secret files
                    ],
                },
                operations: OperationConfig {
                    read_allowed: true,
                    write_requires_policy,
                    delete_requires_explicit_allow,
                    create_dir_allowed: true,
                },
                policies,
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

impl Default for SettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Settings {
    /// Default Settings use production security mode (secure by default)
    fn default() -> Self {
        SettingsBuilder::new().secure().build()
    }
}

impl Settings {
    /// Create a new SettingsBuilder for configuring security modes
    pub fn builder() -> SettingsBuilder {
        SettingsBuilder::new()
    }

    /// Load settings from configuration file or use defaults with validation
    pub fn load() -> anyhow::Result<Self> {
        // Use the new configuration loader for real configuration loading
        let loader = ConfigurationLoader::new();
        let (settings, source_info) = loader
            .load()
            .context("Failed to load configuration using ConfigurationLoader")?;

        // Log configuration source information in non-test mode
        if !cfg!(test) {
            tracing::info!(
                "📋 Configuration loaded from {} environment",
                source_info.environment
            );
            if !source_info.files.is_empty() {
                tracing::info!("   Configuration files: {:?}", source_info.files);
            }
            if !source_info.env_vars.is_empty() {
                tracing::info!(
                    "   Environment variables: {} overrides",
                    source_info.env_vars.len()
                );
            }
            if source_info.uses_defaults {
                tracing::info!("   Using built-in defaults as base configuration");
            }
        }

        // Validate the loaded configuration before returning
        Self::validate_and_warn(&settings)?;

        Ok(settings)
    }

    /// Validate configuration and display warnings/errors
    pub fn validate_and_warn(settings: &Settings) -> anyhow::Result<()> {
        use crate::config::validation::ConfigurationValidator;

        let validation_result = ConfigurationValidator::validate_settings(settings)
            .context("Failed to validate configuration")?;

        // Log warnings if any
        if !validation_result.warnings.is_empty() {
            tracing::warn!("Configuration warnings:");
            for warning in &validation_result.warnings {
                tracing::warn!("  ⚠️  {warning}");
            }
        }

        // If there are errors, fail the configuration load
        if !validation_result.is_valid {
            tracing::error!("Configuration errors:");
            for error in &validation_result.errors {
                tracing::error!("  ❌ {error}");
            }
            return Err(anyhow::anyhow!(
                "Configuration validation failed with {} error(s)",
                validation_result.errors.len()
            ));
        }

        // In non-test mode, also log a success message
        if !cfg!(test) && validation_result.warnings.is_empty() {
            tracing::info!("✅ Configuration validation passed");
        }

        Ok(())
    }

    /// Validate configuration and return detailed results for programmatic use
    pub fn validate(&self) -> anyhow::Result<crate::config::validation::ValidationResult> {
        use crate::config::validation::ConfigurationValidator;
        ConfigurationValidator::validate_settings(self)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert_eq!(settings.server.name, "airs-mcp-fs");

        // Default should be secure (production mode)
        assert!(settings.security.operations.write_requires_policy);
        assert!(settings.security.operations.delete_requires_explicit_allow);

        assert_eq!(settings.binary.max_file_size, 100 * 1024 * 1024);
        assert!(settings.binary.enable_image_processing);
        assert!(settings.binary.enable_pdf_processing);

        // Test that security policies are properly configured
        assert!(settings.security.policies.contains_key("source_code"));
        assert!(settings.security.policies.contains_key("documentation"));
        assert!(settings.security.policies.contains_key("config_files"));
        assert!(settings.security.policies.contains_key("build_artifacts"));
    }

    #[test]
    fn test_settings_builder_modes() {
        // Test production mode (secure)
        let production_settings = Settings::builder().secure().build();
        assert!(
            production_settings
                .security
                .operations
                .write_requires_policy
        );
        assert!(
            production_settings
                .security
                .operations
                .delete_requires_explicit_allow
        );

        // Test development mode (balanced)
        let dev_settings = Settings::builder().development().build();
        assert!(!dev_settings.security.operations.write_requires_policy);
        assert!(
            dev_settings
                .security
                .operations
                .delete_requires_explicit_allow
        );

        // Test permissive mode (minimal restrictions)
        let permissive_settings = Settings::builder().permissive().build();
        assert!(
            !permissive_settings
                .security
                .operations
                .write_requires_policy
        );
        assert!(
            !permissive_settings
                .security
                .operations
                .delete_requires_explicit_allow
        );
        assert!(permissive_settings
            .security
            .policies
            .contains_key("permissive_universal"));
    }

    #[test]
    fn test_settings_builder_default() {
        // Test that builder defaults to secure mode
        let default_builder_settings = Settings::builder().build();
        let explicit_secure_settings = Settings::builder().secure().build();

        assert_eq!(
            default_builder_settings
                .security
                .operations
                .write_requires_policy,
            explicit_secure_settings
                .security
                .operations
                .write_requires_policy
        );
        assert_eq!(
            default_builder_settings
                .security
                .operations
                .delete_requires_explicit_allow,
            explicit_secure_settings
                .security
                .operations
                .delete_requires_explicit_allow
        );
    }

    #[test]
    fn test_settings_load() {
        let result = Settings::load();
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.server.name, "airs-mcp-fs");
    }

    #[test]
    fn test_settings_validation() {
        let settings = Settings::default();
        let validation_result = settings.validate();

        assert!(validation_result.is_ok());
        let result = validation_result.unwrap();
        assert!(
            result.is_valid,
            "Default settings should be valid. Errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_validate_and_warn_success() {
        let settings = Settings::default();
        let result = Settings::validate_and_warn(&settings);
        assert!(
            result.is_ok(),
            "validate_and_warn should succeed for default settings"
        );
    }
}

//! Configuration loading system for AIRS MCP-FS
//!
//! Provides enterprise-grade configuration management with:
//! - Multi-format support (TOML, YAML, JSON)
//! - Environment-specific configuration layering
//! - Environment variable overrides (12-factor app compliance)
//! - Configuration schema validation
//! - Secure defaults and error handling

// Layer 1: Standard library imports
use std::env;
use std::path::{Path, PathBuf};

// Layer 2: Third-party crate imports
use anyhow::{Context, Result};
use config::{Config, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use super::settings::Settings;

/// Configuration environment type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigEnvironment {
    /// Development environment - permissive settings for local development
    Development,
    /// Staging environment - production-like settings for testing
    Staging,
    /// Production environment - secure settings for live deployment
    Production,
    /// Test environment - minimal settings for unit tests
    Test,
}

impl ConfigEnvironment {
    /// Detect environment from environment variables
    pub fn detect() -> Self {
        match env::var("AIRS_MCP_FS_ENV")
            .or_else(|_| env::var("NODE_ENV"))
            .or_else(|_| env::var("ENVIRONMENT"))
            .as_deref()
        {
            Ok("development") | Ok("dev") => Self::Development,
            Ok("staging") | Ok("stage") => Self::Staging,
            Ok("production") | Ok("prod") => Self::Production,
            Ok("test") => Self::Test,
            _ => {
                // Default based on compile-time environment
                if cfg!(test) {
                    Self::Test
                } else if cfg!(debug_assertions) {
                    Self::Development
                } else {
                    Self::Production
                }
            }
        }
    }

    /// Get environment-specific configuration file name
    pub fn config_filename(&self) -> &'static str {
        match self {
            Self::Development => "development.toml",
            Self::Staging => "staging.toml",
            Self::Production => "production.toml",
            Self::Test => "test.toml",
        }
    }

    /// Get environment name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Staging => "staging",
            Self::Production => "production",
            Self::Test => "test",
        }
    }
}

/// Configuration source information for debugging and logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSource {
    /// Configuration files loaded in order
    pub files: Vec<String>,
    /// Environment variables applied
    pub env_vars: Vec<String>,
    /// Environment type detected
    pub environment: String,
    /// Whether defaults were used
    pub uses_defaults: bool,
}

/// Configuration loader with environment-specific layering
pub struct ConfigurationLoader {
    /// Current environment
    environment: ConfigEnvironment,
    /// Base configuration directory
    config_dir: PathBuf,
    /// Environment variable prefix
    env_prefix: String,
}

impl ConfigurationLoader {
    /// Create new configuration loader with automatic environment detection
    pub fn new() -> Self {
        Self::with_environment(ConfigEnvironment::detect())
    }

    /// Create configuration loader for specific environment
    pub fn with_environment(environment: ConfigEnvironment) -> Self {
        let config_dir = Self::default_config_dir();
        Self {
            environment,
            config_dir,
            env_prefix: "AIRS_MCP_FS".to_string(),
        }
    }

    /// Set custom configuration directory
    pub fn with_config_dir<P: Into<PathBuf>>(mut self, config_dir: P) -> Self {
        self.config_dir = config_dir.into();
        self
    }

    /// Set custom environment variable prefix
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.env_prefix = prefix.into();
        self
    }

    /// Load configuration with environment-specific layering
    pub fn load(&self) -> Result<(Settings, ConfigurationSource)> {
        let mut builder = Config::builder();
        let mut source_info = ConfigurationSource {
            files: Vec::new(),
            env_vars: Vec::new(),
            environment: self.environment.as_str().to_string(),
            uses_defaults: false,
        };

        // Layer 1: Built-in defaults (always start with this)
        let default_settings = Settings::default();
        builder = builder.add_source(
            config::Config::try_from(&default_settings)
                .context("Failed to convert default settings to config source")?,
        );
        source_info.uses_defaults = true;

        // Layer 2: Base configuration file (config.toml)
        let base_config_path = self.config_dir.join("config.toml");
        if base_config_path.exists() {
            builder = builder.add_source(
                File::from(base_config_path.as_path())
                    .format(FileFormat::Toml)
                    .required(false),
            );
            source_info
                .files
                .push(base_config_path.display().to_string());
        }

        // Layer 3: Environment-specific configuration
        let env_config_path = self.config_dir.join(self.environment.config_filename());
        if env_config_path.exists() {
            builder = builder.add_source(
                File::from(env_config_path.as_path())
                    .format(FileFormat::Toml)
                    .required(false),
            );
            source_info
                .files
                .push(env_config_path.display().to_string());
        }

        // Layer 4: Local overrides (local.toml) - for development only
        if matches!(self.environment, ConfigEnvironment::Development) {
            let local_config_path = self.config_dir.join("local.toml");
            if local_config_path.exists() {
                builder = builder.add_source(
                    File::from(local_config_path.as_path())
                        .format(FileFormat::Toml)
                        .required(false),
                );
                source_info
                    .files
                    .push(local_config_path.display().to_string());
            }
        }

        // Layer 5: Environment variables (12-factor app compliance)
        builder = builder.add_source(
            Environment::with_prefix(&self.env_prefix)
                .separator("__") // Use double underscore for nested keys
                .prefix_separator("_"),
        );

        // Collect environment variables that were used
        for (key, _value) in env::vars() {
            if key.starts_with(&format!("{}_", self.env_prefix)) {
                source_info.env_vars.push(key);
            }
        }

        // Build final configuration
        let config = builder.build().context("Failed to build configuration")?;

        // Deserialize into Settings struct
        let settings: Settings = config
            .try_deserialize()
            .context("Failed to deserialize configuration into Settings struct")?;

        Ok((settings, source_info))
    }

    /// Load configuration from specific file path
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<Settings> {
        let path = file_path.as_ref();

        // Determine file format from extension
        let format = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => FileFormat::Toml,
            Some("yaml") | Some("yml") => FileFormat::Yaml,
            Some("json") => FileFormat::Json,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported configuration file format. Supported: .toml, .yaml, .yml, .json"
                ))
            }
        };

        let config = Config::builder()
            .add_source(File::from(path).format(format))
            .build()
            .with_context(|| format!("Failed to load configuration from {}", path.display()))?;

        let settings: Settings = config
            .try_deserialize()
            .with_context(|| format!("Failed to parse configuration file {}", path.display()))?;

        Ok(settings)
    }

    /// Get default configuration directory
    fn default_config_dir() -> PathBuf {
        // Try environment variable first
        if let Ok(config_dir) = env::var("AIRS_MCP_FS_CONFIG_DIR") {
            return PathBuf::from(config_dir);
        }

        // Default to config/ in current directory for now
        // In production, this would be more sophisticated (e.g., /etc/airs-mcp-fs/)
        PathBuf::from("config")
    }

    /// Validate configuration file without loading
    pub fn validate_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<String>> {
        let settings = Self::load_from_file(file_path)?;

        // Use existing validation system
        let validation_result = settings
            .validate()
            .context("Failed to validate configuration")?;

        let mut issues = Vec::new();
        issues.extend(validation_result.errors);
        issues.extend(validation_result.warnings);

        Ok(issues)
    }
}

impl Default for ConfigurationLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::uninlined_format_args)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create a basic config.toml
        let config_content = r#"
[server]
name = "test-airs-mcp-fs"
version = "0.1.0"

[binary]
max_file_size = 52428800  # 50MB
enable_image_processing = true
enable_pdf_processing = false

[security.filesystem]
allowed_paths = ["./test/**/*"]
denied_paths = ["**/.env*"]

[security.operations]
read_allowed = true
write_requires_policy = false
delete_requires_explicit_allow = false
create_dir_allowed = true

[security.policies.test_policy]
patterns = ["*.txt"]
operations = ["read", "write"]
risk_level = "low"
description = "Test policy for txt files"
"#;

        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, config_content).unwrap();

        temp_dir
    }

    #[test]
    fn test_environment_detection() {
        // Test default detection
        let env = ConfigEnvironment::detect();
        assert!(matches!(
            env,
            ConfigEnvironment::Test | ConfigEnvironment::Development
        ));
    }

    #[test]
    fn test_environment_config_filenames() {
        assert_eq!(
            ConfigEnvironment::Development.config_filename(),
            "development.toml"
        );
        assert_eq!(ConfigEnvironment::Staging.config_filename(), "staging.toml");
        assert_eq!(
            ConfigEnvironment::Production.config_filename(),
            "production.toml"
        );
        assert_eq!(ConfigEnvironment::Test.config_filename(), "test.toml");
    }

    #[test]
    fn test_configuration_loader_creation() {
        let loader = ConfigurationLoader::new();
        assert!(matches!(
            loader.environment,
            ConfigEnvironment::Test | ConfigEnvironment::Development
        ));

        let prod_loader = ConfigurationLoader::with_environment(ConfigEnvironment::Production);
        assert!(matches!(
            prod_loader.environment,
            ConfigEnvironment::Production
        ));
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = create_test_config_dir();
        let config_path = temp_dir.path().join("config.toml");

        let settings = ConfigurationLoader::load_from_file(&config_path).unwrap();
        assert_eq!(settings.server.name, "test-airs-mcp-fs");
        assert_eq!(settings.binary.max_file_size, 52428800);
        assert!(!settings.binary.enable_pdf_processing);
    }

    #[test]
    fn test_load_with_config_dir() {
        let temp_dir = create_test_config_dir();

        let loader = ConfigurationLoader::with_environment(ConfigEnvironment::Test)
            .with_config_dir(temp_dir.path());

        let (settings, source_info) = loader.load().unwrap();

        // Should have loaded from our test config
        assert_eq!(settings.server.name, "test-airs-mcp-fs");
        assert!(source_info.uses_defaults);
        assert!(!source_info.files.is_empty());
        assert_eq!(source_info.environment, "test");
    }

    #[test]
    fn test_validate_file() {
        let temp_dir = create_test_config_dir();
        let config_path = temp_dir.path().join("config.toml");

        let issues = ConfigurationLoader::validate_file(&config_path).unwrap();
        // Validation should complete without panicking
        // Issues may contain warnings or errors, which is acceptable for testing
        println!("Validation issues found: {:?}", issues);

        // The main goal is that validation completes successfully
        // Individual issues will be addressed by the validation system
    }

    #[test]
    fn test_unsupported_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("config.txt");
        fs::write(&invalid_path, "invalid content").unwrap();

        let result = ConfigurationLoader::load_from_file(&invalid_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported configuration file format"));
    }
}

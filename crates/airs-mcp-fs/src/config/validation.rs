//! Configuration validation for AIRS MCP-FS security settings
//!
//! Provides comprehensive validation of security configurations to ensure safe operation
//! and prevent misconfiguration that could lead to security vulnerabilities or operational failures.

// Layer 1: Standard library imports
use std::collections::HashSet;

// Layer 2: Third-party crate imports
use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSetBuilder};

// Layer 3: Internal module imports
use crate::config::settings::{
    FilesystemConfig, OperationConfig, RiskLevel, SecurityConfig, SecurityPolicy, Settings,
};

/// Validation result for a configuration component
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// List of validation errors found
    pub errors: Vec<String>,
    /// List of validation warnings (non-fatal issues)
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a new failed validation result with an error
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            errors: vec![message.into()],
            warnings: Vec::new(),
        }
    }

    /// Add an error to this validation result
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.is_valid = false;
    }

    /// Add a warning to this validation result
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// Combine this validation result with another
    pub fn combine(mut self, other: ValidationResult) -> Self {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.is_valid = self.is_valid && other.is_valid;
        self
    }
}

/// Configuration validator for security settings
pub struct ConfigurationValidator;

impl ConfigurationValidator {
    /// Validate complete settings configuration
    pub fn validate_settings(settings: &Settings) -> Result<ValidationResult> {
        let mut result = ValidationResult::success();

        // Validate security configuration
        let security_result = Self::validate_security_config(&settings.security)
            .context("Failed to validate security configuration")?;
        result = result.combine(security_result);

        // Validate binary configuration
        let binary_result = Self::validate_binary_config(&settings.binary);
        result = result.combine(binary_result);

        // Validate server configuration
        let server_result = Self::validate_server_config(&settings.server);
        result = result.combine(server_result);

        Ok(result)
    }

    /// Validate security configuration comprehensively
    pub fn validate_security_config(config: &SecurityConfig) -> Result<ValidationResult> {
        let mut result = ValidationResult::success();

        // Validate filesystem configuration
        let filesystem_result = Self::validate_filesystem_config(&config.filesystem)
            .context("Failed to validate filesystem configuration")?;
        result = result.combine(filesystem_result);

        // Validate operation configuration
        let operation_result = Self::validate_operation_config(&config.operations);
        result = result.combine(operation_result);

        // Validate security policies
        let policies_result = Self::validate_security_policies(&config.policies)
            .context("Failed to validate security policies")?;
        result = result.combine(policies_result);

        // Cross-validation: ensure policies support required operations
        let cross_validation_result =
            Self::validate_policy_operation_consistency(&config.operations, &config.policies);
        result = result.combine(cross_validation_result);

        Ok(result)
    }

    /// Validate filesystem configuration (paths and glob patterns)
    pub fn validate_filesystem_config(config: &FilesystemConfig) -> Result<ValidationResult> {
        let mut result = ValidationResult::success();

        // Validate allowed paths
        if config.allowed_paths.is_empty() {
            result.add_error("At least one allowed path pattern must be specified");
        }

        for (i, pattern) in config.allowed_paths.iter().enumerate() {
            if let Err(e) = Self::validate_glob_pattern(pattern) {
                result.add_error(format!("Invalid allowed path pattern #{}: {}", i + 1, e));
            }
        }

        // Validate denied paths
        for (i, pattern) in config.denied_paths.iter().enumerate() {
            if let Err(e) = Self::validate_glob_pattern(pattern) {
                result.add_error(format!("Invalid denied path pattern #{}: {}", i + 1, e));
            }
        }

        // Check for overly permissive patterns
        if config.allowed_paths.iter().any(|p| p == "**" || p == "/**") {
            result.add_warning(
                "Extremely permissive allowed path pattern detected - consider restricting access"
                    .to_string(),
            );
        }

        // Check for conflicts between allowed and denied paths (only if all patterns are valid)
        if result.is_valid {
            match Self::check_path_conflicts(&config.allowed_paths, &config.denied_paths) {
                Ok(conflicts) => {
                    if !conflicts.is_empty() {
                        result.add_warning(format!(
                            "Potential conflicts between allowed and denied paths: {}",
                            conflicts.join(", ")
                        ));
                    }
                }
                Err(_) => {
                    // If we can't check conflicts due to invalid patterns, skip this check
                    // The invalid patterns will already be reported as errors above
                }
            }
        }

        Ok(result)
    }

    /// Validate operation configuration
    pub fn validate_operation_config(config: &OperationConfig) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check for insecure configurations
        if !config.write_requires_policy && !cfg!(test) {
            result.add_warning(
                "write_requires_policy is disabled in non-test mode - this may be insecure"
                    .to_string(),
            );
        }

        if !config.delete_requires_explicit_allow && !cfg!(test) {
            result.add_warning(
                "delete_requires_explicit_allow is disabled in non-test mode - this may be insecure"
                    .to_string(),
            );
        }

        if !config.read_allowed {
            result.add_warning(
                "read_allowed is disabled - this may prevent normal operation".to_string(),
            );
        }

        result
    }

    /// Validate security policies comprehensively
    pub fn validate_security_policies(
        policies: &std::collections::HashMap<String, SecurityPolicy>,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::success();

        if policies.is_empty() {
            result.add_error("At least one security policy must be defined");
            return Ok(result);
        }

        for (name, policy) in policies {
            let policy_result = Self::validate_security_policy(name, policy)
                .context(format!("Failed to validate policy '{}'", name))?;
            result = result.combine(policy_result);
        }

        // Check for policy coverage gaps
        let coverage_result = Self::validate_policy_coverage(policies);
        result = result.combine(coverage_result);

        Ok(result)
    }

    /// Validate a single security policy
    pub fn validate_security_policy(
        name: &str,
        policy: &SecurityPolicy,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::success();

        // Validate policy name
        if name.is_empty() {
            result.add_error("Policy name cannot be empty");
        }

        // Validate patterns
        if policy.patterns.is_empty() {
            result.add_error(format!("Policy '{}' must have at least one pattern", name));
        }

        for (i, pattern) in policy.patterns.iter().enumerate() {
            if let Err(e) = Self::validate_glob_pattern(pattern) {
                result.add_error(format!(
                    "Policy '{}' has invalid pattern #{}: {}",
                    name,
                    i + 1,
                    e
                ));
            }
        }

        // Validate operations
        if policy.operations.is_empty() {
            result.add_error(format!(
                "Policy '{}' must specify at least one operation",
                name
            ));
        }

        let valid_operations = [
            "read",
            "write",
            "delete",
            "list",
            "create_dir",
            "move",
            "copy",
        ];
        for operation in &policy.operations {
            if !valid_operations.contains(&operation.as_str()) {
                result.add_error(format!(
                    "Policy '{}' has invalid operation '{}'. Valid operations: {}",
                    name,
                    operation,
                    valid_operations.join(", ")
                ));
            }
        }

        // Validate risk level consistency
        Self::validate_risk_level_consistency(name, policy, &mut result);

        Ok(result)
    }

    /// Validate consistency between operation config and security policies
    pub fn validate_policy_operation_consistency(
        operations: &OperationConfig,
        policies: &std::collections::HashMap<String, SecurityPolicy>,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // If write_requires_policy is enabled, ensure at least one policy allows write
        if operations.write_requires_policy {
            let has_write_policy = policies
                .values()
                .any(|policy| policy.operations.contains(&"write".to_string()));

            if !has_write_policy {
                result.add_error(
                    "write_requires_policy is enabled but no policies allow 'write' operations"
                        .to_string(),
                );
            }
        }

        // If delete_requires_explicit_allow is enabled, ensure at least one policy allows delete
        if operations.delete_requires_explicit_allow {
            let has_delete_policy = policies
                .values()
                .any(|policy| policy.operations.contains(&"delete".to_string()));

            if !has_delete_policy {
                result.add_error(
                    "delete_requires_explicit_allow is enabled but no policies allow 'delete' operations"
                        .to_string(),
                );
            }
        }

        result
    }

    /// Validate binary configuration
    fn validate_binary_config(config: &crate::config::settings::BinaryConfig) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check for reasonable file size limits
        if config.max_file_size == 0 {
            result.add_error("max_file_size cannot be zero");
        } else if config.max_file_size > 1_000_000_000 {
            // 1GB
            result.add_warning(format!(
                "max_file_size is very large ({} bytes) - this may cause memory issues",
                config.max_file_size
            ));
        }

        result
    }

    /// Validate server configuration
    fn validate_server_config(config: &crate::config::settings::ServerConfig) -> ValidationResult {
        let mut result = ValidationResult::success();

        if config.name.is_empty() {
            result.add_error("Server name cannot be empty");
        }

        if config.version.is_empty() {
            result.add_error("Server version cannot be empty");
        }

        result
    }

    /// Validate a glob pattern for correctness
    fn validate_glob_pattern(pattern: &str) -> Result<()> {
        Glob::new(pattern).map_err(|e| anyhow!("Invalid glob pattern '{}': {}", pattern, e))?;
        Ok(())
    }

    /// Check for conflicts between allowed and denied path patterns
    fn check_path_conflicts(allowed: &[String], denied: &[String]) -> Result<Vec<String>> {
        let mut conflicts = Vec::new();

        // Build glob sets for efficient matching
        let mut allowed_builder = GlobSetBuilder::new();
        for pattern in allowed {
            allowed_builder.add(Glob::new(pattern)?);
        }
        let _allowed_set = allowed_builder.build()?;

        let mut denied_builder = GlobSetBuilder::new();
        for pattern in denied {
            denied_builder.add(Glob::new(pattern)?);
        }
        let _denied_set = denied_builder.build()?;

        // Check for patterns that might conflict
        for allowed_pattern in allowed {
            for denied_pattern in denied {
                // Simple heuristic: if patterns overlap, it might be a conflict
                if Self::patterns_might_overlap(allowed_pattern, denied_pattern) {
                    conflicts.push(format!("'{}' vs '{}'", allowed_pattern, denied_pattern));
                }
            }
        }

        Ok(conflicts)
    }

    /// Check if two glob patterns might overlap (simple heuristic)
    fn patterns_might_overlap(pattern1: &str, pattern2: &str) -> bool {
        // Very basic overlap detection - this could be more sophisticated
        let p1_parts: Vec<&str> = pattern1.split('/').collect();
        let p2_parts: Vec<&str> = pattern2.split('/').collect();

        // Check if one pattern could be a subset of another
        for i in 0..std::cmp::min(p1_parts.len(), p2_parts.len()) {
            let part1 = p1_parts[i];
            let part2 = p2_parts[i];

            // If parts are completely different (no wildcards), no overlap
            if part1 != part2 && !part1.contains('*') && !part2.contains('*') {
                return false;
            }
        }

        true // Conservative: assume overlap if we can't rule it out
    }

    /// Validate policy coverage for common file types
    fn validate_policy_coverage(
        policies: &std::collections::HashMap<String, SecurityPolicy>,
    ) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Check if common file types are covered
        let common_patterns = [
            "*.rs", "*.py", "*.js", "*.ts", "*.md", "*.toml", "*.json", "*.yaml",
        ];

        let all_patterns: HashSet<String> = policies
            .values()
            .flat_map(|policy| policy.patterns.iter().cloned())
            .collect();

        let mut uncovered_types = Vec::new();
        for pattern in &common_patterns {
            let is_covered = all_patterns
                .iter()
                .any(|p| p.contains(pattern) || p.contains("*.*") || p.contains("**/*"));

            if !is_covered {
                uncovered_types.push(*pattern);
            }
        }

        if !uncovered_types.is_empty() {
            result.add_warning(format!(
                "Common file types may not be covered by policies: {}",
                uncovered_types.join(", ")
            ));
        }

        result
    }

    /// Validate risk level consistency with operations
    fn validate_risk_level_consistency(
        name: &str,
        policy: &SecurityPolicy,
        result: &mut ValidationResult,
    ) {
        // High risk operations with delete permission should be flagged
        if policy.risk_level >= RiskLevel::High && policy.operations.contains(&"delete".to_string())
        {
            result.add_warning(format!(
                "Policy '{}' has high risk level but allows delete operations - consider review",
                name
            ));
        }

        // Critical risk operations should be very limited
        if policy.risk_level == RiskLevel::Critical {
            let risky_ops = ["write", "delete", "move"];
            let has_risky_ops = policy
                .operations
                .iter()
                .any(|op| risky_ops.contains(&op.as_str()));

            if has_risky_ops {
                result.add_warning(format!(
                    "Policy '{}' has critical risk level but allows write/delete/move - consider restrictions",
                    name
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::Settings;
    use std::collections::HashMap;

    #[test]
    fn test_validate_default_settings() {
        let settings = Settings::default();
        let result = ConfigurationValidator::validate_settings(&settings).unwrap();

        // Default settings should be valid
        assert!(result.is_valid, "Errors: {:?}", result.errors);

        // May have warnings but should not have errors
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_empty_allowed_paths() {
        let config = FilesystemConfig {
            allowed_paths: vec![],
            denied_paths: vec![],
        };

        let result = ConfigurationValidator::validate_filesystem_config(&config).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("At least one allowed path")));
    }

    #[test]
    fn test_validate_invalid_glob_pattern() {
        let config = FilesystemConfig {
            allowed_paths: vec!["[invalid".to_string()], // Invalid glob pattern
            denied_paths: vec![],
        };

        let result = ConfigurationValidator::validate_filesystem_config(&config);
        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert!(!validation_result.is_valid);
        assert!(validation_result
            .errors
            .iter()
            .any(|e| e.contains("Invalid allowed path pattern")));
    }

    #[test]
    fn test_validate_empty_policies() {
        let policies = HashMap::new();
        let result = ConfigurationValidator::validate_security_policies(&policies).unwrap();

        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("At least one security policy")));
    }

    #[test]
    fn test_validate_policy_with_invalid_operation() {
        let mut policies = HashMap::new();
        policies.insert(
            "test".to_string(),
            SecurityPolicy {
                patterns: vec!["*.rs".to_string()],
                operations: vec!["invalid_operation".to_string()],
                risk_level: RiskLevel::Low,
                description: None,
            },
        );

        let result = ConfigurationValidator::validate_security_policies(&policies).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("invalid operation")));
    }

    #[test]
    fn test_validate_policy_operation_consistency() {
        let operations = OperationConfig {
            read_allowed: true,
            write_requires_policy: true,
            delete_requires_explicit_allow: false,
            create_dir_allowed: true,
        };

        // Policy that doesn't allow write operations
        let mut policies = HashMap::new();
        policies.insert(
            "read_only".to_string(),
            SecurityPolicy {
                patterns: vec!["*.rs".to_string()],
                operations: vec!["read".to_string()],
                risk_level: RiskLevel::Low,
                description: None,
            },
        );

        let result =
            ConfigurationValidator::validate_policy_operation_consistency(&operations, &policies);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("write_requires_policy is enabled but no policies allow 'write'")));
    }

    #[test]
    fn test_validate_binary_config_zero_size() {
        let config = crate::config::settings::BinaryConfig {
            max_file_size: 0,
            enable_image_processing: true,
            enable_pdf_processing: true,
        };

        let result = ConfigurationValidator::validate_binary_config(&config);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("max_file_size cannot be zero")));
    }

    #[test]
    fn test_validate_server_config_empty_name() {
        let config = crate::config::settings::ServerConfig {
            name: "".to_string(),
            version: "1.0.0".to_string(),
        };

        let result = ConfigurationValidator::validate_server_config(&config);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Server name cannot be empty")));
    }

    #[test]
    fn test_validation_result_combine() {
        let mut result1 = ValidationResult::success();
        result1.add_error("Error 1");
        result1.add_warning("Warning 1");

        let mut result2 = ValidationResult::success();
        result2.add_error("Error 2");
        result2.add_warning("Warning 2");

        let combined = result1.combine(result2);
        assert!(!combined.is_valid);
        assert_eq!(combined.errors.len(), 2);
        assert_eq!(combined.warnings.len(), 2);
    }

    #[test]
    fn test_validate_high_risk_delete_policy() {
        let mut policies = HashMap::new();
        policies.insert(
            "high_risk_delete".to_string(),
            SecurityPolicy {
                patterns: vec!["*.important".to_string()],
                operations: vec!["read".to_string(), "delete".to_string()],
                risk_level: RiskLevel::High,
                description: None,
            },
        );

        let result = ConfigurationValidator::validate_security_policies(&policies).unwrap();
        assert!(result.is_valid); // Should be valid but with warnings
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("high risk level but allows delete")));
    }
}

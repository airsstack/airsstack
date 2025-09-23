//! Security manager for access control and validation

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Instant;

// Layer 2: Third-party crate imports
use anyhow::Result;
use globset;

// Layer 3: Internal module imports
use crate::binary::format::{FileFormat, FormatDetector};
use crate::config::settings::{RiskLevel, SecurityConfig};
use crate::filesystem::{
    validation::{PathValidator, SecurityError},
    FileOperation,
};
use crate::mcp::OperationType;
use crate::security::approval::ApprovalDecision;
use crate::security::audit::{AuditLogger, CorrelationId};
use crate::security::permissions::{
    PathPermissionRule, PathPermissionValidator, PermissionEvaluation, PermissionLevel,
};
use crate::security::policy::PolicyEngine;

/// Main security manager for filesystem operations
#[derive(Debug)]
pub struct SecurityManager {
    path_validator: PathValidator,
    policy_engine: PolicyEngine,
    permission_validator: PathPermissionValidator,
    audit_logger: AuditLogger,
    format_detector: FormatDetector,
    config: Arc<SecurityConfig>,
}

impl SecurityManager {
    /// Create a new security manager with configuration
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let path_validator = PathValidator::new(
            config.filesystem.allowed_paths.clone(),
            config.filesystem.denied_paths.clone(),
        );

        // Create policy engine from configured policies
        let policy_engine = PolicyEngine::new(config.policies.clone())?;

        // Create permission validator in strict mode for security-first approach
        // Only policies should define permissions - no auto-generated rules
        let mut permission_validator = PathPermissionValidator::new(true); // strict mode

        // Convert security policies to permission rules
        // This ensures all permissions are explicitly defined in configuration
        for (name, policy) in &config.policies {
            permission_validator.add_policy(name.clone(), policy.clone());

            // Create permission rules from policy patterns and operations
            for pattern in &policy.patterns {
                // Determine permission level based on operations allowed
                let permission_level = if policy.operations.contains(&"delete".to_string()) {
                    PermissionLevel::Full
                } else if policy.operations.contains(&"write".to_string()) {
                    PermissionLevel::ReadWrite
                } else if policy.operations.contains(&"read".to_string()) {
                    PermissionLevel::ReadOnly
                } else {
                    PermissionLevel::None
                };

                let rule = PathPermissionRule::new(
                    pattern.clone(),
                    permission_level,
                    policy.operations.iter().map(|s| s.as_str()).collect(),
                    100, // Standard priority for policy-based rules
                    format!("Policy '{name}' rule for pattern: {pattern}"),
                )?;

                permission_validator.add_rule(rule);
            }
        }

        Ok(Self {
            path_validator,
            policy_engine,
            permission_validator,
            audit_logger: AuditLogger::new(),
            format_detector: FormatDetector::new(),
            config: Arc::new(config),
        })
    }

    /// Validate operation-specific permissions
    ///
    /// This method provides granular validation for specific operation types,
    /// integrating with the path permission system and policy engine.
    pub async fn validate_operation_permission(
        &self,
        operation: &FileOperation,
    ) -> Result<ApprovalDecision> {
        let correlation_id = CorrelationId::new();
        let start_time = Instant::now();

        // Log operation request
        self.audit_logger
            .log_operation_requested(correlation_id, operation);

        // 0. Validate binary file restrictions (security hardening)
        if let Err(err) = self
            .validate_binary_file_restriction(&operation.path, correlation_id)
            .await
        {
            let execution_time_ms = start_time.elapsed().as_millis() as u64;
            self.audit_logger.log_operation_failed(
                correlation_id,
                operation,
                &err.to_string(),
                execution_time_ms,
            );
            return Err(err);
        }

        // 1. Validate basic path security first
        match self.path_validator.validate_path(&operation.path) {
            Ok(_validated_path) => {}
            Err(security_error) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                let error_msg = match security_error {
                    SecurityError::AccessDenied => "Access denied",
                    SecurityError::InvalidInput => "Invalid input detected",
                    SecurityError::PolicyViolation => "Security policy violation",
                };
                self.audit_logger.log_operation_failed(
                    correlation_id,
                    operation,
                    error_msg,
                    execution_time_ms,
                );
                return Err(anyhow::anyhow!("{}", error_msg));
            }
        }

        // 2. Validate operation-specific permissions
        let operations = std::iter::once(operation.operation_type).collect();
        let permission_result = self.permission_validator.evaluate_permissions(
            &operation.path,
            &operations,
            Some(&format!(
                "operation_validation_{:?}",
                operation.operation_type
            )),
        );

        if !permission_result.allowed {
            let execution_time_ms = start_time.elapsed().as_millis() as u64;
            let reason = format!(
                "Operation {:?} denied by permission system: {}",
                operation.operation_type, permission_result.decision_reason
            );
            self.audit_logger.log_operation_failed(
                correlation_id,
                operation,
                &reason,
                execution_time_ms,
            );
            return Err(anyhow::anyhow!("{}", reason));
        }

        // 3. Apply operation-specific configuration rules
        let operation_allowed = match operation.operation_type {
            OperationType::Read => {
                // Read operations: always allowed if path permissions pass
                self.config.operations.read_allowed
            }
            OperationType::Write => {
                // Write operations: check if policy is required
                if self.config.operations.write_requires_policy {
                    // Policy validation required for writes
                    self.validate_operation_against_policies(operation, correlation_id)
                        .await?
                } else {
                    // Write allowed by configuration
                    true
                }
            }
            OperationType::Delete => {
                // Delete operations: check explicit allow requirement
                if self.config.operations.delete_requires_explicit_allow {
                    // Must have explicit delete permission in a policy
                    self.validate_delete_permission(operation, correlation_id)
                        .await?
                } else {
                    // Delete allowed by configuration
                    true
                }
            }
            OperationType::CreateDir => {
                // Directory creation: check configuration
                self.config.operations.create_dir_allowed
            }
            OperationType::List | OperationType::Move | OperationType::Copy => {
                // Other operations: allowed if permission system passes
                true
            }
        };

        if !operation_allowed {
            let execution_time_ms = start_time.elapsed().as_millis() as u64;
            let reason = format!(
                "Operation {:?} denied by operation configuration",
                operation.operation_type
            );
            self.audit_logger.log_operation_failed(
                correlation_id,
                operation,
                &reason,
                execution_time_ms,
            );
            return Err(anyhow::anyhow!("{}", reason));
        }

        // 4. Final policy engine validation
        let policy_start = Instant::now();
        let policy_decision = self.policy_engine.evaluate_operation(operation);
        let policy_time_ms = policy_start.elapsed().as_millis() as u64;

        // Log policy evaluation
        self.audit_logger.log_policy_evaluated(
            correlation_id,
            &policy_decision,
            None,
            policy_time_ms,
        );

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        if policy_decision.is_allowed() {
            // Log successful operation
            self.audit_logger.log_operation_completed(
                correlation_id,
                operation,
                execution_time_ms,
                None,
            );
            Ok(ApprovalDecision::Approved)
        } else {
            // Log failed operation
            let reason = format!(
                "Operation {:?} denied by policy engine: {}",
                operation.operation_type,
                policy_decision.reason()
            );
            self.audit_logger.log_operation_failed(
                correlation_id,
                operation,
                &reason,
                execution_time_ms,
            );
            Err(anyhow::anyhow!("{}", reason))
        }
    }

    /// Validate binary file restrictions for security hardening
    /// This method rejects all binary file operations to prevent security risks
    async fn validate_binary_file_restriction(
        &self,
        path: &std::path::Path,
        correlation_id: CorrelationId,
    ) -> Result<()> {
        // First, try to detect format from file extension (fast check)
        let format_from_extension = self.format_detector.detect_from_extension(path);

        // Check if it's a known binary format based on extension
        if self.is_binary_format(&format_from_extension) {
            let error_msg = format!(
                "Binary file access denied for security reasons: {} (detected format: {:?})",
                path.display(),
                format_from_extension
            );

            // Log security violation for binary file access attempt
            self.audit_logger.log_security_violation(
                correlation_id,
                "binary_file_denied",
                path,
                &error_msg,
                RiskLevel::High,
            );

            return Err(anyhow::anyhow!("{}", error_msg));
        }

        // For file operations on existing files, read a small sample to verify format
        // Only check if file exists and we can read it safely
        if path.exists() && path.is_file() {
            match std::fs::read(path) {
                Ok(bytes) if !bytes.is_empty() => {
                    // Only check first 512 bytes for format detection (efficient)
                    let sample_size = std::cmp::min(bytes.len(), 512);
                    let format_from_content = self
                        .format_detector
                        .detect_from_bytes(&bytes[..sample_size]);

                    if self.is_binary_format(&format_from_content) {
                        let error_msg = format!(
                            "Binary file access denied for security reasons: {} (content analysis: {:?})",
                            path.display(),
                            format_from_content
                        );

                        // Log security violation for binary content detection
                        self.audit_logger.log_security_violation(
                            correlation_id,
                            "binary_content_denied",
                            path,
                            &error_msg,
                            RiskLevel::High,
                        );

                        return Err(anyhow::anyhow!("{}", error_msg));
                    }
                }
                Err(_) => {
                    // If we can't read the file, let the regular file system handle the error
                    // This avoids false positives for permission issues
                }
                Ok(_) => {
                    // Empty file - allow operation (common for new text files)
                }
            }
        }

        // File is safe (text or unknown but not detectably binary)
        Ok(())
    }

    /// Helper method to determine if a file format is considered binary
    fn is_binary_format(&self, format: &FileFormat) -> bool {
        match format {
            // All image formats are binary
            FileFormat::Jpeg
            | FileFormat::Png
            | FileFormat::Gif
            | FileFormat::WebP
            | FileFormat::Tiff
            | FileFormat::Bmp => true,

            // Document formats are binary
            FileFormat::Pdf => true,

            // Text formats are allowed
            FileFormat::Text => false,

            // Unknown formats are allowed (benefit of the doubt for text files)
            // This prevents false positives while maintaining security
            FileFormat::Unknown => false,
        }
    }

    /// Validate operation against security policies (for write operations)
    async fn validate_operation_against_policies(
        &self,
        operation: &FileOperation,
        correlation_id: CorrelationId,
    ) -> Result<bool> {
        // Check if any policy explicitly allows this operation
        for policy in self.config.policies.values() {
            // Check if the file path matches any pattern in this policy
            for pattern in &policy.patterns {
                if let Ok(glob) = globset::Glob::new(pattern) {
                    if glob.compile_matcher().is_match(&operation.path) {
                        // Check if the operation is allowed by this policy
                        let operation_name = operation.operation_type.as_str();

                        if policy.operations.contains(&operation_name.to_string()) {
                            // Log successful policy match as information (not a violation)
                            // Use the existing audit infrastructure for successful operations
                            return Ok(true);
                        }
                    }
                }
            }
        }

        // No policy allows this operation - log as security violation
        self.audit_logger.log_security_violation(
            correlation_id,
            "policy_denied",
            &operation.path,
            &format!(
                "Operation {:?} denied - no matching policy found",
                operation.operation_type
            ),
            RiskLevel::High,
        );

        Ok(false)
    }

    /// Validate explicit delete permission requirement
    async fn validate_delete_permission(
        &self,
        operation: &FileOperation,
        correlation_id: CorrelationId,
    ) -> Result<bool> {
        // For delete operations, we need explicit "delete" permission in a policy
        for policy in self.config.policies.values() {
            // Check if the file path matches any pattern in this policy
            for pattern in &policy.patterns {
                if let Ok(glob) = globset::Glob::new(pattern) {
                    if glob.compile_matcher().is_match(&operation.path) {
                        // Check if delete is explicitly allowed
                        if policy.operations.contains(&"delete".to_string()) {
                            // Log successful delete permission (not a violation)
                            return Ok(true);
                        }
                    }
                }
            }
        }

        // No explicit delete permission found - log as security violation
        self.audit_logger.log_security_violation(
            correlation_id,
            "delete_denied",
            &operation.path,
            "Delete operation denied - no explicit delete permission found",
            RiskLevel::High,
        );

        Ok(false)
    }

    /// Validate read access to a path
    pub async fn validate_read_access(&self, operation: &FileOperation) -> Result<()> {
        let correlation_id = CorrelationId::new();
        let start_time = Instant::now();

        // Log operation request
        self.audit_logger
            .log_operation_requested(correlation_id, operation);

        // Validate path security
        match self.path_validator.validate_path(&operation.path) {
            Ok(_validated_path) => {}
            Err(security_error) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                let error_msg = match security_error {
                    SecurityError::AccessDenied => "Access denied",
                    SecurityError::InvalidInput => "Invalid input detected",
                    SecurityError::PolicyViolation => "Security policy violation",
                };
                self.audit_logger.log_operation_failed(
                    correlation_id,
                    operation,
                    error_msg,
                    execution_time_ms,
                );
                return Err(anyhow::anyhow!("{}", error_msg));
            }
        }

        // Validate path permissions
        let operations = std::iter::once(operation.operation_type).collect();
        let permission_result = self.permission_validator.evaluate_permissions(
            &operation.path,
            &operations,
            Some("read_access_validation"),
        );

        if !permission_result.allowed {
            let execution_time_ms = start_time.elapsed().as_millis() as u64;
            self.audit_logger.log_operation_failed(
                correlation_id,
                operation,
                &format!("Permission denied: {}", permission_result.decision_reason),
                execution_time_ms,
            );
            return Err(anyhow::anyhow!(
                "Permission denied: {}",
                permission_result.decision_reason
            ));
        }

        // Use policy engine to validate read access
        let policy_start = Instant::now();
        let policy_decision = self.policy_engine.evaluate_operation(operation);
        let policy_time_ms = policy_start.elapsed().as_millis() as u64;

        // Log policy evaluation
        self.audit_logger.log_policy_evaluated(
            correlation_id,
            &policy_decision,
            None, // Policy name will be included in the decision
            policy_time_ms,
        );

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        if policy_decision.is_allowed() {
            // Log successful operation
            self.audit_logger.log_operation_completed(
                correlation_id,
                operation,
                execution_time_ms,
                None, // No size information available
            );
            Ok(())
        } else {
            // Log failed operation
            self.audit_logger.log_operation_failed(
                correlation_id,
                operation,
                &format!("Policy denied read access: {}", policy_decision.reason()),
                execution_time_ms,
            );
            Err(anyhow::anyhow!(
                "Policy denied read access: {}",
                policy_decision.reason()
            ))
        }
    }

    /// Validate write access to a path (requires policy evaluation)
    pub async fn validate_write_access(
        &self,
        operation: &FileOperation,
    ) -> Result<ApprovalDecision> {
        let correlation_id = CorrelationId::new();
        let start_time = Instant::now();

        // Log operation request
        self.audit_logger
            .log_operation_requested(correlation_id, operation);

        // Validate path security first
        match self.path_validator.validate_path(&operation.path) {
            Ok(_validated_path) => {}
            Err(security_error) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                let error_msg = match security_error {
                    SecurityError::AccessDenied => "Access denied",
                    SecurityError::InvalidInput => "Invalid input detected",
                    SecurityError::PolicyViolation => "Security policy violation",
                };
                self.audit_logger.log_operation_failed(
                    correlation_id,
                    operation,
                    error_msg,
                    execution_time_ms,
                );
                return Err(anyhow::anyhow!("{}", error_msg));
            }
        }

        // Validate path permissions
        let operations = std::iter::once(operation.operation_type).collect();
        let permission_result = self.permission_validator.evaluate_permissions(
            &operation.path,
            &operations,
            Some("write_access_validation"),
        );

        if !permission_result.allowed {
            let execution_time_ms = start_time.elapsed().as_millis() as u64;
            self.audit_logger.log_operation_failed(
                correlation_id,
                operation,
                &format!("Permission denied: {}", permission_result.decision_reason),
                execution_time_ms,
            );
            return Err(anyhow::anyhow!(
                "Permission denied: {}",
                permission_result.decision_reason
            ));
        }

        // Use policy engine for real security evaluation instead of auto-approval
        let policy_start = Instant::now();
        let policy_decision = self.policy_engine.evaluate_operation(operation);
        let policy_time_ms = policy_start.elapsed().as_millis() as u64;

        // Log policy evaluation
        self.audit_logger.log_policy_evaluated(
            correlation_id,
            &policy_decision,
            None, // Policy name will be included in the decision
            policy_time_ms,
        );

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        if self.config.operations.write_requires_policy
            && self.requires_approval(operation.operation_type)
        {
            if policy_decision.is_allowed() {
                // Policy allows operation - auto-approve and log completion
                let decision = ApprovalDecision::Approved;

                self.audit_logger.log_operation_completed(
                    correlation_id,
                    operation,
                    execution_time_ms,
                    None, // No size information available
                );

                Ok(decision)
            } else {
                // Policy denies operation - log and convert to ApprovalDecision::Denied
                self.audit_logger.log_operation_failed(
                    correlation_id,
                    operation,
                    &format!("Policy denied write access: {}", policy_decision.reason()),
                    execution_time_ms,
                );
                Ok(ApprovalDecision::Denied)
            }
        } else {
            // For operations that don't require policy evaluation, still check with policy engine
            if policy_decision.is_allowed() {
                self.audit_logger.log_operation_completed(
                    correlation_id,
                    operation,
                    execution_time_ms,
                    None, // No size information available
                );
                Ok(ApprovalDecision::Approved)
            } else {
                self.audit_logger.log_operation_failed(
                    correlation_id,
                    operation,
                    &format!("Policy denied write access: {}", policy_decision.reason()),
                    execution_time_ms,
                );
                Ok(ApprovalDecision::Denied)
            }
        }
    }

    /// Add a permission rule to the path permission validator
    pub fn add_permission_rule(&mut self, rule: PathPermissionRule) {
        self.permission_validator.add_rule(rule);
    }

    /// Get permission evaluation for a specific path and operations
    pub fn evaluate_path_permissions(
        &self,
        path: &std::path::Path,
        operations: &std::collections::HashSet<OperationType>,
        context: Option<&str>,
    ) -> PermissionEvaluation {
        self.permission_validator
            .evaluate_permissions(path, operations, context)
    }

    /// Get permission coverage statistics
    pub fn get_permission_coverage(&self) -> std::collections::HashMap<String, usize> {
        self.permission_validator.get_coverage_stats()
    }

    /// Check if an operation type requires human approval
    fn requires_approval(&self, operation_type: OperationType) -> bool {
        match operation_type {
            OperationType::Read | OperationType::List => false,
            OperationType::Write
            | OperationType::CreateDir
            | OperationType::Delete
            | OperationType::Move
            | OperationType::Copy => true,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::config::SecurityConfig;
    use std::path::PathBuf;

    fn create_test_config() -> SecurityConfig {
        use crate::config::settings::{
            FilesystemConfig, OperationConfig, RiskLevel, SecurityPolicy,
        };
        use std::collections::HashMap;

        let mut policies = HashMap::new();

        // Simple universal policy that allows all operations on all patterns
        policies.insert(
            "universal_test_policy".to_string(),
            SecurityPolicy {
                patterns: vec!["**/*".to_string()], // Match everything
                operations: vec![
                    "read".to_string(),
                    "write".to_string(),
                    "list".to_string(),
                    "create_dir".to_string(),
                    "move".to_string(),
                    "copy".to_string(),
                    "delete".to_string(),
                ],
                risk_level: RiskLevel::Low,
                description: Some("Universal test policy allowing all operations".to_string()),
            },
        );

        SecurityConfig {
            filesystem: FilesystemConfig {
                allowed_paths: vec!["**/*".to_string()], // Allow everything for testing
                denied_paths: vec!["**/.git/**".to_string()],
            },
            operations: OperationConfig {
                read_allowed: true,
                write_requires_policy: false, // Don't require policies for writes in test
                delete_requires_explicit_allow: false, // Don't require explicit delete permissions in test
                create_dir_allowed: true,
            },
            policies,
        }
    }

    #[test]
    fn test_security_manager_creation() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Test should pass with our permissive test config
        assert!(manager.config.operations.read_allowed);
        assert!(!manager.config.operations.write_requires_policy); // Permissive in test config
        assert!(manager.config.operations.create_dir_allowed);
    }

    #[tokio::test]
    async fn test_validate_read_access_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Read, PathBuf::from("src/main.rs"));

        let result = manager.validate_read_access(&operation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_read_access_denied_path() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Read, PathBuf::from(".git/config"));

        let result = manager.validate_read_access(&operation).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_write_access_with_approval() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Write, PathBuf::from("src/new_file.rs"));

        let result = manager.validate_write_access(&operation).await;
        assert!(result.is_ok());
        // Returns approval decision (currently auto-approved after policy validation)
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[test]
    fn test_requires_approval() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        assert!(!manager.requires_approval(OperationType::Read));
        assert!(!manager.requires_approval(OperationType::List));
        assert!(manager.requires_approval(OperationType::Write));
        assert!(manager.requires_approval(OperationType::Delete));
        assert!(manager.requires_approval(OperationType::CreateDir));
    }

    // ===== NEW OPERATION-TYPE RESTRICTIONS TESTS =====

    #[tokio::test]
    async fn test_validate_operation_permission_read_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Read, PathBuf::from("src/main.rs"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn test_validate_operation_permission_write_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Write, PathBuf::from("src/new_file.rs"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn test_validate_operation_permission_delete_success() {
        use crate::config::settings::{RiskLevel, SecurityPolicy};

        let mut config = create_test_config();

        // Add a policy that allows delete operations
        let delete_policy = SecurityPolicy {
            patterns: vec!["src/**".to_string()],
            operations: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
            risk_level: RiskLevel::Low,
            description: None,
        };
        config
            .policies
            .insert("delete_policy".to_string(), delete_policy);

        let manager = SecurityManager::new(config).unwrap();

        // Test with a file in src/ directory that has explicit delete permission
        let operation =
            FileOperation::new(OperationType::Delete, PathBuf::from("src/temp_file.txt"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn test_validate_operation_permission_delete_denied() {
        use crate::config::settings::{
            FilesystemConfig, OperationConfig, RiskLevel, SecurityPolicy,
        };
        use std::collections::HashMap;

        // Create a restrictive config that denies delete operations
        let mut policies = HashMap::new();
        policies.insert(
            "no_delete_policy".to_string(),
            SecurityPolicy {
                patterns: vec!["**/*".to_string()],
                operations: vec!["read".to_string(), "write".to_string()], // No delete operation
                risk_level: RiskLevel::Low,
                description: None,
            },
        );

        let restrictive_config = SecurityConfig {
            filesystem: FilesystemConfig {
                allowed_paths: vec!["**/*".to_string()],
                denied_paths: vec![],
            },
            operations: OperationConfig {
                read_allowed: true,
                write_requires_policy: false,
                delete_requires_explicit_allow: true, // Require explicit delete permission
                create_dir_allowed: true,
            },
            policies,
        };

        let manager = SecurityManager::new(restrictive_config).unwrap();

        // Test with source code that doesn't have explicit delete permission
        let operation = FileOperation::new(OperationType::Delete, PathBuf::from("src/main.rs"));

        let result = manager.validate_operation_permission(&operation).await;
        // Should fail because policy doesn't include "delete" operation
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Delete") || error_msg.contains("delete"));
    }

    #[tokio::test]
    async fn test_validate_operation_permission_create_dir_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation =
            FileOperation::new(OperationType::CreateDir, PathBuf::from("src/new_module"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn test_validate_operation_permission_list_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // List the src directory itself
        let operation = FileOperation::new(OperationType::List, PathBuf::from("src"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn test_validate_operation_permission_move_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Move, PathBuf::from("src/old_file.rs"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn test_validate_operation_permission_copy_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Copy, PathBuf::from("src/main.rs"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[tokio::test]
    async fn test_validate_operation_permission_denied_path() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        // Test with denied path (.git directory)
        let operation = FileOperation::new(OperationType::Read, PathBuf::from(".git/config"));

        let result = manager.validate_operation_permission(&operation).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Be more flexible about the error message - it could be path validation or permission denied
        assert!(
            error_msg.contains("Path validation failed")
                || error_msg.contains("denied")
                || error_msg.contains("not allowed")
                || error_msg.contains("Security policy violation")
        );
    }

    #[tokio::test]
    async fn test_validate_operation_against_policies_write_allowed() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).unwrap();

        let operation = FileOperation::new(OperationType::Write, PathBuf::from("src/main.rs"));
        let correlation_id = CorrelationId::new();

        let result = manager
            .validate_operation_against_policies(&operation, correlation_id)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true for allowed operation
    }

    #[tokio::test]
    async fn test_validate_operation_against_policies_write_denied() {
        use crate::config::settings::{
            FilesystemConfig, OperationConfig, RiskLevel, SecurityPolicy,
        };
        use std::collections::HashMap;

        // Create a restrictive config for this specific test
        let mut policies = HashMap::new();
        policies.insert(
            "read_only_policy".to_string(),
            SecurityPolicy {
                patterns: vec!["**/*".to_string()],
                operations: vec!["read".to_string()], // Only allow read, not write
                risk_level: RiskLevel::Low,
                description: None,
            },
        );

        let restrictive_config = SecurityConfig {
            filesystem: FilesystemConfig {
                allowed_paths: vec!["**/*".to_string()],
                denied_paths: vec![],
            },
            operations: OperationConfig {
                read_allowed: true,
                write_requires_policy: true, // Require policy for writes
                delete_requires_explicit_allow: false,
                create_dir_allowed: true,
            },
            policies,
        };

        let manager = SecurityManager::new(restrictive_config).unwrap();

        // Test with a write operation that should be denied by the read-only policy
        let operation = FileOperation::new(OperationType::Write, PathBuf::from("test/file.txt"));
        let correlation_id = CorrelationId::new();

        let result = manager
            .validate_operation_against_policies(&operation, correlation_id)
            .await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false for denied operation
    }

    #[tokio::test]
    async fn test_validate_delete_permission_allowed() {
        use crate::config::settings::{RiskLevel, SecurityPolicy};

        let mut config = create_test_config();

        // Add a policy that explicitly allows delete operations
        config.policies.insert(
            "delete_policy".to_string(),
            SecurityPolicy {
                patterns: vec!["src/**".to_string()],
                operations: vec!["delete".to_string()],
                risk_level: RiskLevel::Low,
                description: None,
            },
        );

        let manager = SecurityManager::new(config).unwrap();

        // Test with a file in src/ that has explicit delete permission
        let operation =
            FileOperation::new(OperationType::Delete, PathBuf::from("src/temp_file.txt"));
        let correlation_id = CorrelationId::new();

        let result = manager
            .validate_delete_permission(&operation, correlation_id)
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true for allowed delete
    }

    #[tokio::test]
    async fn test_validate_delete_permission_denied() {
        use crate::config::settings::{
            FilesystemConfig, OperationConfig, RiskLevel, SecurityPolicy,
        };
        use std::collections::HashMap;

        // Create a restrictive config for this test
        let mut policies = HashMap::new();
        policies.insert(
            "restrictive_policy".to_string(),
            SecurityPolicy {
                patterns: vec!["**/*".to_string()],
                operations: vec!["read".to_string(), "write".to_string()], // No delete operation
                risk_level: RiskLevel::Low,
                description: None,
            },
        );

        let restrictive_config = SecurityConfig {
            filesystem: FilesystemConfig {
                allowed_paths: vec!["**/*".to_string()],
                denied_paths: vec![],
            },
            operations: OperationConfig {
                read_allowed: true,
                write_requires_policy: false,
                delete_requires_explicit_allow: true, // Require explicit delete permission
                create_dir_allowed: true,
            },
            policies,
        };

        let manager = SecurityManager::new(restrictive_config).unwrap();

        // Test with source code that doesn't have explicit delete permission
        let operation = FileOperation::new(OperationType::Delete, PathBuf::from("src/main.rs"));
        let correlation_id = CorrelationId::new();

        let result = manager
            .validate_delete_permission(&operation, correlation_id)
            .await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false for denied delete
    }

    #[tokio::test]
    async fn test_operation_type_specific_configuration() {
        use crate::config::settings::Settings;

        // Test with production settings (default)
        let production_settings = Settings::default();
        let production_manager = SecurityManager::new(production_settings.security).unwrap();

        // Test that configuration settings are properly applied
        assert!(production_manager.config.operations.read_allowed);
        assert!(production_manager.config.operations.write_requires_policy);
        assert!(
            production_manager
                .config
                .operations
                .delete_requires_explicit_allow
        );
        assert!(production_manager.config.operations.create_dir_allowed);

        // Test with permissive settings for testing
        let permissive_settings = Settings::builder().permissive().build();
        let permissive_manager = SecurityManager::new(permissive_settings.security).unwrap();

        // Test that permissive configuration is properly applied
        assert!(permissive_manager.config.operations.read_allowed);
        assert!(!permissive_manager.config.operations.write_requires_policy);
        assert!(
            !permissive_manager
                .config
                .operations
                .delete_requires_explicit_allow
        );
        assert!(permissive_manager.config.operations.create_dir_allowed);

        // Also test with explicit restrictive configuration
        let restrictive_config = create_test_config();
        let restrictive_manager = SecurityManager::new(restrictive_config).unwrap();

        // Our test config is actually permissive now
        assert!(restrictive_manager.config.operations.read_allowed);
        assert!(!restrictive_manager.config.operations.write_requires_policy); // Permissive
        assert!(
            !restrictive_manager
                .config
                .operations
                .delete_requires_explicit_allow
        ); // Permissive
        assert!(restrictive_manager.config.operations.create_dir_allowed);
    }

    /// Test binary file rejection for security hardening (Task 011)
    #[tokio::test]
    async fn test_binary_file_rejection_security_hardening() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).expect("Should create security manager");

        // Test various binary file types should be rejected
        let binary_files = vec![
            ("/tmp/test.jpg", "JPEG image"),
            ("/tmp/test.png", "PNG image"),
            ("/tmp/test.gif", "GIF image"),
            ("/tmp/test.pdf", "PDF document"),
            ("/tmp/test.bmp", "BMP image"),
            ("/tmp/test.tiff", "TIFF image"),
            ("/tmp/test.webp", "WebP image"),
        ];

        for (path, description) in binary_files {
            let operation = FileOperation::new(OperationType::Read, PathBuf::from(path));

            let result = manager.validate_operation_permission(&operation).await;
            assert!(
                result.is_err(),
                "Binary file {description} should be rejected for security"
            );

            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("Binary file access denied")
                    || error_msg.contains("binary")
                    || error_msg.contains("security"),
                "Error message should indicate binary file security restriction: {error_msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_text_files_allowed_after_binary_hardening() {
        let config = create_test_config();
        let manager = SecurityManager::new(config).expect("Should create security manager");

        // Test that text files are still allowed
        let text_files = vec![
            "/tmp/test.txt",
            "/tmp/test.md",
            "/tmp/test.rs",
            "/tmp/test.py",
            "/tmp/test.js",
            "/tmp/test.json",
            "/tmp/test.toml",
            "/tmp/test.yml",
        ];

        for path in text_files {
            let operation = FileOperation::new(OperationType::Read, PathBuf::from(path));

            let result = manager.validate_operation_permission(&operation).await;
            // Should not fail due to binary restriction (may fail for other policy reasons)
            if let Err(err) = &result {
                let error_msg = err.to_string();
                assert!(
                    !error_msg.contains("Binary file access denied")
                        && !error_msg.contains("binary")
                        && !error_msg.contains("Binary"),
                    "Text file {path} should not trigger binary file restriction: {error_msg}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_binary_file_content_detection() {
        use std::fs;
        use tempfile::TempDir;

        let config = create_test_config();
        let manager = SecurityManager::new(config).expect("Should create security manager");

        // Create temporary directory for test files
        let temp_dir = TempDir::new().expect("Should create temp dir");

        // Create a file with binary content (fake JPEG header)
        let binary_file = temp_dir.path().join("fake.txt"); // .txt extension but binary content
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic bytes
        fs::write(&binary_file, &jpeg_header).expect("Should write binary file");

        let operation = FileOperation::new(OperationType::Read, binary_file);

        let result = manager.validate_operation_permission(&operation).await;
        assert!(
            result.is_err(),
            "File with binary content should be rejected regardless of extension"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Binary file access denied")
                || error_msg.contains("content analysis"),
            "Error should indicate content-based binary detection: {error_msg}"
        );
    }
}

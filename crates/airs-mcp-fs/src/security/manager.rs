//! Security manager for access control and validation

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Instant;

// Layer 2: Third-party crate imports
use anyhow::Result;

// Layer 3: Internal module imports
use crate::config::settings::SecurityConfig;
use crate::filesystem::{validation::PathValidator, FileOperation};
use crate::mcp::OperationType;
use crate::security::approval::{ApprovalDecision, ApprovalWorkflow};
use crate::security::audit::{AuditLogger, CorrelationId};
use crate::security::permissions::PathPermissionValidator;
use crate::security::policy::PolicyEngine;

/// Main security manager for filesystem operations
#[derive(Debug)]
pub struct SecurityManager {
    path_validator: PathValidator,
    approval_workflow: ApprovalWorkflow,
    policy_engine: PolicyEngine,
    permission_validator: PathPermissionValidator,
    audit_logger: AuditLogger,
    config: Arc<SecurityConfig>,
}

impl SecurityManager {
    /// Create a new security manager with configuration
    pub fn new(config: SecurityConfig) -> Self {
        let path_validator = PathValidator::new(
            config.filesystem.allowed_paths.clone(),
            config.filesystem.denied_paths.clone(),
        );

        // Create policy engine from configured policies
        let policy_engine = PolicyEngine::new(config.policies.clone())
            .expect("Failed to create policy engine - invalid security policies");

        // Create permission validator in permissive mode for backward compatibility
        // Can be switched to strict mode later with explicit configuration
        let mut permission_validator = PathPermissionValidator::new(false);
        
        // Add security policies to the permission validator
        for (name, policy) in &config.policies {
            permission_validator.add_policy(name.clone(), policy.clone());
        }

        Self {
            path_validator,
            approval_workflow: ApprovalWorkflow::new(),
            policy_engine,
            permission_validator,
            audit_logger: AuditLogger::new(),
            config: Arc::new(config),
        }
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
            Err(e) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                self.audit_logger.log_operation_failed(
                    correlation_id,
                    operation,
                    &format!("Path validation failed: {e}"),
                    execution_time_ms,
                );
                return Err(e);
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
            return Err(anyhow::anyhow!("Permission denied: {}", permission_result.decision_reason));
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
            Err(e) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;
                self.audit_logger.log_operation_failed(
                    correlation_id,
                    operation,
                    &format!("Path validation failed: {e}"),
                    execution_time_ms,
                );
                return Err(e);
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
            return Err(anyhow::anyhow!("Permission denied: {}", permission_result.decision_reason));
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
                // Policy allows operation - still check approval workflow if needed
                let decision = self.approval_workflow.request_approval(operation).await;

                // Log based on approval decision
                match decision {
                    ApprovalDecision::Approved => {
                        self.audit_logger.log_operation_completed(
                            correlation_id,
                            operation,
                            execution_time_ms,
                            None, // No size information available
                        );
                    }
                    ApprovalDecision::Denied => {
                        self.audit_logger.log_operation_failed(
                            correlation_id,
                            operation,
                            "Human approval denied",
                            execution_time_ms,
                        );
                    }
                    ApprovalDecision::Timeout => {
                        self.audit_logger.log_operation_failed(
                            correlation_id,
                            operation,
                            "Human approval timed out",
                            execution_time_ms,
                        );
                    }
                    ApprovalDecision::Cancelled => {
                        self.audit_logger.log_operation_failed(
                            correlation_id,
                            operation,
                            "Human approval cancelled",
                            execution_time_ms,
                        );
                    }
                }

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
    pub fn add_permission_rule(&mut self, rule: crate::security::permissions::PathPermissionRule) {
        self.permission_validator.add_rule(rule);
    }

    /// Get permission evaluation for a specific path and operations
    pub fn evaluate_path_permissions(
        &self,
        path: &std::path::Path,
        operations: &std::collections::HashSet<OperationType>,
        context: Option<&str>,
    ) -> crate::security::permissions::PermissionEvaluation {
        self.permission_validator.evaluate_permissions(path, operations, context)
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
        policies.insert(
            "test_policy".to_string(),
            SecurityPolicy {
                patterns: vec!["src/**".to_string(), "tests/**".to_string()],
                operations: vec!["read".to_string(), "write".to_string()],
                risk_level: RiskLevel::Low,
                description: None,
            },
        );

        SecurityConfig {
            filesystem: FilesystemConfig {
                allowed_paths: vec!["src/**".to_string(), "tests/**".to_string()],
                denied_paths: vec!["**/.git/**".to_string()],
            },
            operations: OperationConfig {
                read_allowed: true,
                write_requires_policy: true,
                delete_requires_explicit_allow: true,
                create_dir_allowed: true,
            },
            policies,
        }
    }

    #[test]
    fn test_security_manager_creation() {
        let config = create_test_config();
        let manager = SecurityManager::new(config);
        assert!(manager.config.operations.write_requires_policy);
    }

    #[tokio::test]
    async fn test_validate_read_access_success() {
        let config = create_test_config();
        let manager = SecurityManager::new(config);

        let operation = FileOperation::new(OperationType::Read, PathBuf::from("src/main.rs"));

        let result = manager.validate_read_access(&operation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_read_access_denied_path() {
        let config = create_test_config();
        let manager = SecurityManager::new(config);

        let operation = FileOperation::new(OperationType::Read, PathBuf::from(".git/config"));

        let result = manager.validate_read_access(&operation).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_write_access_with_approval() {
        let config = create_test_config();
        let manager = SecurityManager::new(config);

        let operation = FileOperation::new(OperationType::Write, PathBuf::from("src/new_file.rs"));

        let result = manager.validate_write_access(&operation).await;
        assert!(result.is_ok());
        // Should return approval decision (placeholder returns Approved)
        assert_eq!(result.unwrap(), ApprovalDecision::Approved);
    }

    #[test]
    fn test_requires_approval() {
        let config = create_test_config();
        let manager = SecurityManager::new(config);

        assert!(!manager.requires_approval(OperationType::Read));
        assert!(!manager.requires_approval(OperationType::List));
        assert!(manager.requires_approval(OperationType::Write));
        assert!(manager.requires_approval(OperationType::Delete));
        assert!(manager.requires_approval(OperationType::CreateDir));
    }
}

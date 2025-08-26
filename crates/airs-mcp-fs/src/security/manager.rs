//! Security manager for access control and validation

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use anyhow::Result;

// Layer 3: Internal module imports
use crate::config::settings::SecurityConfig;
use crate::filesystem::{validation::PathValidator, FileOperation};
use crate::mcp::OperationType;
use crate::security::approval::{ApprovalDecision, ApprovalWorkflow};

/// Main security manager for filesystem operations
#[derive(Debug)]
pub struct SecurityManager {
    path_validator: PathValidator,
    approval_workflow: ApprovalWorkflow,
    config: Arc<SecurityConfig>,
}

impl SecurityManager {
    /// Create a new security manager with configuration
    pub fn new(config: SecurityConfig) -> Self {
        let path_validator = PathValidator::new(
            config.filesystem.allowed_paths.clone(),
            config.filesystem.denied_paths.clone(),
        );
        
        Self {
            path_validator,
            approval_workflow: ApprovalWorkflow::new(),
            config: Arc::new(config),
        }
    }

    /// Validate read access to a path
    pub async fn validate_read_access(&self, operation: &FileOperation) -> Result<()> {
        // Validate path security
        self.path_validator.validate_path(&operation.path)?;
        
        // Read operations generally don't require approval
        // Additional security checks can be added here
        
        Ok(())
    }

    /// Validate write access to a path (requires approval if configured)
    pub async fn validate_write_access(&self, operation: &FileOperation) -> Result<ApprovalDecision> {
        // Validate path security first
        self.path_validator.validate_path(&operation.path)?;
        
        // Check if approval is required for write operations
        if self.config.operations.write_requires_policy && self.requires_approval(operation.operation_type) {
            let decision = self.approval_workflow.request_approval(operation).await;
            Ok(decision)
        } else {
            Ok(ApprovalDecision::Approved)
        }
    }

    /// Check if an operation type requires human approval
    fn requires_approval(&self, operation_type: OperationType) -> bool {
        match operation_type {
            OperationType::Read | OperationType::List => false,
            OperationType::Write | OperationType::CreateDir | OperationType::Delete 
            | OperationType::Move | OperationType::Copy => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecurityConfig;
    use std::path::PathBuf;

    fn create_test_config() -> SecurityConfig {
        use std::collections::HashMap;
        use crate::config::settings::{FilesystemConfig, OperationConfig, SecurityPolicy, RiskLevel};
        
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
        
        let operation = FileOperation::new(
            OperationType::Read,
            PathBuf::from("src/main.rs"),
        );
        
        let result = manager.validate_read_access(&operation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_read_access_denied_path() {
        let config = create_test_config();
        let manager = SecurityManager::new(config);
        
        let operation = FileOperation::new(
            OperationType::Read,
            PathBuf::from(".git/config"),
        );
        
        let result = manager.validate_read_access(&operation).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_write_access_with_approval() {
        let config = create_test_config();
        let manager = SecurityManager::new(config);
        
        let operation = FileOperation::new(
            OperationType::Write,
            PathBuf::from("src/new_file.rs"),
        );
        
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

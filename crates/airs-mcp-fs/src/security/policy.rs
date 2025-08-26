//! Policy engine for real-time security evaluation
//!
//! This module provides the core policy evaluation engine that replaces the auto-approval
//! security bypass with real policy-based decision making.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

// Layer 3: Internal module imports
use crate::config::settings::{RiskLevel, SecurityPolicy};
use crate::filesystem::FileOperation;
use crate::mcp::OperationType;

/// Result of policy evaluation for a filesystem operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// Operation is allowed by policy
    Allow {
        /// Policy that granted permission
        policy_name: String,
        /// Risk level of the operation
        risk_level: RiskLevel,
        /// Reason for allowing
        reason: String,
    },
    /// Operation is denied
    Deny {
        /// Reason for denial
        reason: String,
    },
}

impl PolicyDecision {
    /// Check if the decision allows the operation
    pub fn is_allowed(&self) -> bool {
        matches!(self, PolicyDecision::Allow { .. })
    }

    /// Get the risk level if operation is allowed
    pub fn risk_level(&self) -> Option<&RiskLevel> {
        match self {
            PolicyDecision::Allow { risk_level, .. } => Some(risk_level),
            PolicyDecision::Deny { .. } => None,
        }
    }

    /// Get the reason for the decision
    pub fn reason(&self) -> &str {
        match self {
            PolicyDecision::Allow { reason, .. } => reason,
            PolicyDecision::Deny { reason } => reason,
        }
    }
}

/// Compiled policy matcher for efficient pattern matching
#[derive(Debug)]
struct PolicyMatcher {
    /// Policy name
    name: String,
    /// Policy configuration
    policy: SecurityPolicy,
    /// Compiled glob set for path matching
    glob_set: GlobSet,
}

impl PolicyMatcher {
    /// Create a new policy matcher from a security policy
    fn new(name: String, policy: SecurityPolicy) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();

        // Add all patterns from the policy
        for pattern in &policy.patterns {
            let glob = Glob::new(pattern).with_context(|| {
                format!("Invalid glob pattern in policy '{}': {}", name, pattern)
            })?;
            builder.add(glob);
        }

        let glob_set = builder
            .build()
            .with_context(|| format!("Failed to build glob set for policy '{}'", name))?;

        Ok(PolicyMatcher {
            name,
            policy,
            glob_set,
        })
    }

    /// Check if this policy matches the given path
    fn matches_path(&self, path: &std::path::Path) -> bool {
        // Convert path to string for glob matching
        let path_str = path.to_string_lossy();
        self.glob_set.is_match(path_str.as_ref())
    }

    /// Check if this policy allows the given operation type
    fn allows_operation(&self, operation_type: OperationType) -> bool {
        let operation_str = match operation_type {
            OperationType::Read => "read",
            OperationType::Write => "write",
            OperationType::Delete => "delete",
            OperationType::CreateDir => "create_dir",
            OperationType::List => "list",
            OperationType::Move => "move",
            OperationType::Copy => "copy",
        };

        self.policy.operations.contains(&operation_str.to_string())
    }
}

/// Policy engine for real-time security evaluation
#[derive(Debug)]
pub struct PolicyEngine {
    /// Compiled policy matchers for efficient evaluation
    matchers: Vec<PolicyMatcher>,
}

impl PolicyEngine {
    /// Create a new policy engine from security policies
    pub fn new(policies: HashMap<String, SecurityPolicy>) -> Result<Self> {
        let mut matchers = Vec::new();

        // Compile all policies into matchers
        for (name, policy) in policies {
            let matcher = PolicyMatcher::new(name, policy)
                .with_context(|| "Failed to create policy matcher")?;
            matchers.push(matcher);
        }

        Ok(PolicyEngine { matchers })
    }

    /// Evaluate a filesystem operation against all policies
    pub fn evaluate_operation(&self, operation: &FileOperation) -> PolicyDecision {
        // Find all policies that match the file path
        let matching_policies: Vec<&PolicyMatcher> = self
            .matchers
            .iter()
            .filter(|matcher| matcher.matches_path(&operation.path))
            .collect();

        // If no policies match the path, deny by default (security-first)
        if matching_policies.is_empty() {
            return PolicyDecision::Deny {
                reason: format!(
                    "No policy matches path '{}' for {} operation",
                    operation.path.display(),
                    self.operation_type_str(operation.operation_type)
                ),
            };
        }

        // Check if any matching policy allows the operation
        for matcher in &matching_policies {
            if matcher.allows_operation(operation.operation_type) {
                return PolicyDecision::Allow {
                    policy_name: matcher.name.clone(),
                    risk_level: matcher.policy.risk_level.clone(),
                    reason: format!(
                        "Policy '{}' allows {} operation on '{}'",
                        matcher.name,
                        self.operation_type_str(operation.operation_type),
                        operation.path.display()
                    ),
                };
            }
        }

        // Path matches policies but no policy allows the operation
        let policy_names: Vec<&str> = matching_policies.iter().map(|m| m.name.as_str()).collect();

        PolicyDecision::Deny {
            reason: format!(
                "Path '{}' matches policies [{}] but none allow {} operation",
                operation.path.display(),
                policy_names.join(", "),
                self.operation_type_str(operation.operation_type)
            ),
        }
    }

    /// Convert operation type to string for display
    fn operation_type_str(&self, operation_type: OperationType) -> &'static str {
        match operation_type {
            OperationType::Read => "read",
            OperationType::Write => "write",
            OperationType::Delete => "delete",
            OperationType::CreateDir => "create_dir",
            OperationType::List => "list",
            OperationType::Move => "move",
            OperationType::Copy => "copy",
        }
    }

    /// Get statistics about loaded policies
    pub fn policy_count(&self) -> usize {
        self.matchers.len()
    }

    /// Get all policy names
    pub fn policy_names(&self) -> Vec<&str> {
        self.matchers.iter().map(|m| m.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::RiskLevel;
    use std::path::PathBuf;

    fn create_test_policy() -> SecurityPolicy {
        SecurityPolicy {
            patterns: vec!["**/*.rs".to_string(), "**/*.md".to_string()],
            operations: vec!["read".to_string(), "write".to_string()],
            risk_level: RiskLevel::Low,
            description: Some("Test policy".to_string()),
        }
    }

    fn create_read_only_policy() -> SecurityPolicy {
        SecurityPolicy {
            patterns: vec!["**/secret/**".to_string()],
            operations: vec!["read".to_string()],
            risk_level: RiskLevel::High,
            description: Some("Read-only policy for secrets".to_string()),
        }
    }

    #[test]
    fn test_policy_decision_is_allowed() {
        let allow_decision = PolicyDecision::Allow {
            policy_name: "test".to_string(),
            risk_level: RiskLevel::Low,
            reason: "test reason".to_string(),
        };
        assert!(allow_decision.is_allowed());

        let deny_decision = PolicyDecision::Deny {
            reason: "test denial".to_string(),
        };
        assert!(!deny_decision.is_allowed());
    }

    #[test]
    fn test_policy_decision_risk_level() {
        let allow_decision = PolicyDecision::Allow {
            policy_name: "test".to_string(),
            risk_level: RiskLevel::Medium,
            reason: "test reason".to_string(),
        };
        assert_eq!(allow_decision.risk_level(), Some(&RiskLevel::Medium));

        let deny_decision = PolicyDecision::Deny {
            reason: "test denial".to_string(),
        };
        assert_eq!(deny_decision.risk_level(), None);
    }

    #[test]
    fn test_policy_matcher_creation() {
        let policy = create_test_policy();
        let matcher = PolicyMatcher::new("test_policy".to_string(), policy);
        assert!(matcher.is_ok());
    }

    #[test]
    fn test_policy_matcher_path_matching() {
        let policy = create_test_policy();
        let matcher = PolicyMatcher::new("test_policy".to_string(), policy).unwrap();

        assert!(matcher.matches_path(&PathBuf::from("src/main.rs")));
        assert!(matcher.matches_path(&PathBuf::from("docs/README.md")));
        assert!(!matcher.matches_path(&PathBuf::from("src/main.py")));
    }

    #[test]
    fn test_policy_matcher_operation_checking() {
        let policy = create_test_policy();
        let matcher = PolicyMatcher::new("test_policy".to_string(), policy).unwrap();

        assert!(matcher.allows_operation(OperationType::Read));
        assert!(matcher.allows_operation(OperationType::Write));
        assert!(!matcher.allows_operation(OperationType::Delete));
    }

    #[test]
    fn test_policy_engine_creation() {
        let mut policies = HashMap::new();
        policies.insert("test_policy".to_string(), create_test_policy());

        let engine = PolicyEngine::new(policies);
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert_eq!(engine.policy_count(), 1);
        assert_eq!(engine.policy_names(), vec!["test_policy"]);
    }

    #[test]
    fn test_policy_engine_allow_operation() {
        let mut policies = HashMap::new();
        policies.insert("test_policy".to_string(), create_test_policy());

        let engine = PolicyEngine::new(policies).unwrap();
        let operation = FileOperation::new(OperationType::Read, PathBuf::from("src/main.rs"));

        let decision = engine.evaluate_operation(&operation);
        assert!(decision.is_allowed());
        assert_eq!(decision.risk_level(), Some(&RiskLevel::Low));
    }

    #[test]
    fn test_policy_engine_deny_operation_no_policy() {
        let mut policies = HashMap::new();
        policies.insert("test_policy".to_string(), create_test_policy());

        let engine = PolicyEngine::new(policies).unwrap();
        let operation = FileOperation::new(
            OperationType::Read,
            PathBuf::from("src/main.py"), // No policy matches .py files
        );

        let decision = engine.evaluate_operation(&operation);
        assert!(!decision.is_allowed());
        assert!(decision.reason().contains("No policy matches"));
    }

    #[test]
    fn test_policy_engine_deny_operation_wrong_operation() {
        let mut policies = HashMap::new();
        policies.insert("read_only".to_string(), create_read_only_policy());

        let engine = PolicyEngine::new(policies).unwrap();
        let operation = FileOperation::new(
            OperationType::Write, // Policy only allows read
            PathBuf::from("secret/config.txt"),
        );

        let decision = engine.evaluate_operation(&operation);
        assert!(!decision.is_allowed());
        assert!(decision.reason().contains("none allow write operation"));
    }

    #[test]
    fn test_policy_engine_allow_with_multiple_policies() {
        let mut policies = HashMap::new();
        policies.insert("source_code".to_string(), create_test_policy());
        policies.insert("read_only".to_string(), create_read_only_policy());

        let engine = PolicyEngine::new(policies).unwrap();

        // Test source code access
        let operation = FileOperation::new(OperationType::Write, PathBuf::from("src/main.rs"));
        let decision = engine.evaluate_operation(&operation);
        assert!(decision.is_allowed());

        // Test read-only access
        let operation =
            FileOperation::new(OperationType::Read, PathBuf::from("secret/password.txt"));
        let decision = engine.evaluate_operation(&operation);
        assert!(decision.is_allowed());
        assert_eq!(decision.risk_level(), Some(&RiskLevel::High));
    }
}

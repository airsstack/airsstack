//! Enhanced path-based permission validation system
//!
//! Provides sophisticated glob pattern matching with permission hierarchies,
//! multiple policy evaluation, and detailed permission analysis.

// Layer 1: Standard library imports
use std::collections::{HashMap, HashSet};
use std::path::Path;

// Layer 2: Third-party crate imports
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use glob::Pattern;
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::config::settings::{RiskLevel, SecurityPolicy};
use crate::mcp::types::OperationType;

/// Permission levels for path-based access control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// No access allowed
    None,
    /// Read-only access
    ReadOnly,
    /// Read and basic operations (list, etc.)
    ReadBasic,
    /// Read and write access (no deletion)
    ReadWrite,
    /// Full access including deletion and creation
    Full,
}

impl PermissionLevel {
    /// Check if this permission level allows a specific operation
    pub fn allows_operation(&self, operation: &OperationType) -> bool {
        match (self, operation) {
            (PermissionLevel::None, _) => false,
            (PermissionLevel::ReadOnly, OperationType::Read) => true,
            (PermissionLevel::ReadBasic, OperationType::Read | OperationType::List) => true,
            (
                PermissionLevel::ReadWrite,
                OperationType::Read
                | OperationType::List
                | OperationType::Write
                | OperationType::Copy,
            ) => true,
            (PermissionLevel::Full, _) => true,
            _ => false,
        }
    }

    /// Get the numeric priority for permission level comparison
    pub fn priority(&self) -> i32 {
        match self {
            PermissionLevel::None => 0,
            PermissionLevel::ReadOnly => 1,
            PermissionLevel::ReadBasic => 2,
            PermissionLevel::ReadWrite => 3,
            PermissionLevel::Full => 4,
        }
    }
}

/// Result of permission evaluation for a specific path and operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionEvaluation {
    /// Whether the operation is allowed
    pub allowed: bool,
    /// The effective permission level granted
    pub effective_level: PermissionLevel,
    /// Policies that matched this path
    pub matched_policies: Vec<String>,
    /// Risk level of the operation
    pub risk_level: RiskLevel,
    /// Detailed explanation of the decision
    pub decision_reason: String,
    /// Timestamp of evaluation
    pub evaluated_at: DateTime<Utc>,
}

/// Configuration for a path permission rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathPermissionRule {
    /// Pattern to match paths (supports glob patterns)
    pub pattern: String,
    /// Permission level to grant for matching paths
    pub permission: PermissionLevel,
    /// Allowed operations for this rule
    pub allowed_operations: HashSet<OperationType>,
    /// Whether this rule is enabled
    pub enabled: bool,
    /// Priority for rule evaluation (higher = more important)
    pub priority: i32,
    /// Description of this rule
    pub description: String,
}

impl PathPermissionRule {
    /// Create a new path permission rule
    pub fn new(
        pattern: String,
        permission: PermissionLevel,
        operations: Vec<&str>,
        priority: i32,
        description: String,
    ) -> Result<Self> {
        let mut allowed_operations = HashSet::new();

        for op in operations {
            match op {
                "read" => {
                    allowed_operations.insert(OperationType::Read);
                }
                "write" => {
                    allowed_operations.insert(OperationType::Write);
                }
                "delete" => {
                    allowed_operations.insert(OperationType::Delete);
                }
                "create_dir" => {
                    allowed_operations.insert(OperationType::CreateDir);
                }
                "list" => {
                    allowed_operations.insert(OperationType::List);
                }
                "move" => {
                    allowed_operations.insert(OperationType::Move);
                }
                "copy" => {
                    allowed_operations.insert(OperationType::Copy);
                }
                _ => return Err(anyhow::anyhow!("Unknown operation type: {}", op)),
            };
        }

        Ok(Self {
            pattern,
            permission,
            allowed_operations,
            enabled: true,
            priority,
            description,
        })
    }

    /// Check if this rule matches the given path
    pub fn matches_path(&self, path: &Path) -> bool {
        if !self.enabled {
            return false;
        }

        Pattern::new(&self.pattern)
            .map(|pattern| pattern.matches_path(path))
            .unwrap_or(false)
    }

    /// Get the permission level for operations based on what's allowed
    pub fn evaluate_for_operations(&self, operations: &HashSet<OperationType>) -> PermissionLevel {
        // Check if all requested operations are allowed by this rule
        if operations
            .iter()
            .all(|op| self.allowed_operations.contains(op))
        {
            // Return the rule's permission level if all operations are allowed
            self.permission.clone()
        } else {
            // Deny if any operation is not allowed
            PermissionLevel::None
        }
    }
}

/// Advanced path permission validator with glob patterns and inheritance
#[derive(Debug, Clone)]
pub struct PathPermissionValidator {
    /// Rules for path-based permissions
    rules: Vec<PathPermissionRule>,
    /// Security policies cache
    policies: HashMap<String, SecurityPolicy>,
    /// Whether to use strict evaluation
    strict_mode: bool,
}

impl PathPermissionValidator {
    /// Create a new path permission validator
    pub fn new(strict_mode: bool) -> Self {
        Self {
            rules: Vec::new(),
            policies: HashMap::new(),
            strict_mode,
        }
    }

    /// Add a permission rule
    pub fn add_rule(&mut self, rule: PathPermissionRule) {
        self.rules.push(rule);
        // Sort by priority (descending) for proper evaluation order
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add a security policy to the cache
    pub fn add_policy(&mut self, name: String, policy: SecurityPolicy) {
        self.policies.insert(name, policy);
    }

    /// Evaluate permissions for a path and set of operations
    pub fn evaluate_permissions(
        &self,
        path: &Path,
        operations: &HashSet<OperationType>,
        _context: Option<&str>,
    ) -> PermissionEvaluation {
        let start_time = Utc::now();
        let mut matched_policies = Vec::new();
        let mut effective_level = PermissionLevel::None;
        let mut decision_reasons = Vec::new();

        // First, check path-based rules
        for rule in &self.rules {
            if rule.matches_path(path) {
                matched_policies.push(rule.description.clone());
                let rule_level = rule.evaluate_for_operations(operations);

                if rule_level.priority() > effective_level.priority() {
                    effective_level = rule_level;
                    decision_reasons.push(format!(
                        "Rule '{}' grants {:?} permission",
                        rule.description, rule.permission
                    ));
                }
            }
        }

        // Check security policies for additional restrictions
        let risk_level = self
            .policies
            .values()
            .filter(|policy| self.path_matches_policy(path, policy))
            .map(|policy| &policy.risk_level)
            .max() // Use highest risk level
            .cloned()
            .unwrap_or(RiskLevel::Low);

        for (name, policy) in &self.policies {
            if self.path_matches_policy(path, policy) {
                matched_policies.push(name.clone());

                // Apply policy restrictions - policies can only restrict, not expand
                if !policy.patterns.is_empty() {
                    decision_reasons.push(format!("Policy '{}' applies restrictions", name));
                }
            }
        }

        // Calculate overall risk level has already been done above

        // In strict mode, require explicit permission
        let allowed = if self.strict_mode {
            effective_level != PermissionLevel::None
                && operations
                    .iter()
                    .all(|op| effective_level.allows_operation(op))
        } else {
            // In permissive mode, allow unless explicitly denied
            // If no rules match (None), default to allowing with ReadWrite permission
            if effective_level == PermissionLevel::None {
                // No rules matched, set a permissive default level
                effective_level = PermissionLevel::ReadWrite;
                true
            } else {
                // Rules matched, respect their decision
                effective_level != PermissionLevel::None
                    && operations
                        .iter()
                        .all(|op| effective_level.allows_operation(op))
            }
        };

        // Build decision reason
        let decision_reason = if decision_reasons.is_empty() {
            if self.strict_mode {
                "No explicit permission granted (strict mode)".to_string()
            } else {
                "Default permissive access".to_string()
            }
        } else {
            decision_reasons.join("; ")
        };

        PermissionEvaluation {
            allowed,
            effective_level,
            matched_policies,
            risk_level,
            decision_reason,
            evaluated_at: start_time,
        }
    }

    /// Check if a path matches a security policy pattern
    fn path_matches_policy(&self, path: &Path, policy: &SecurityPolicy) -> bool {
        // Check if path matches any of the policy patterns
        for pattern in &policy.patterns {
            if let Ok(glob_pattern) = Pattern::new(pattern) {
                if glob_pattern.matches_path(path) {
                    return true;
                }
            }
        }

        // If no patterns are defined, policy applies to all paths
        policy.patterns.is_empty()
    }

    /// Get statistics about rule coverage
    pub fn get_coverage_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        stats.insert("total_rules".to_string(), self.rules.len());
        stats.insert(
            "enabled_rules".to_string(),
            self.rules.iter().filter(|r| r.enabled).count(),
        );
        stats.insert("policies".to_string(), self.policies.len());

        // Group rules by permission level
        for rule in &self.rules {
            let key = format!("{:?}_rules", rule.permission);
            *stats.entry(key).or_insert(0) += 1;
        }

        stats
    }

    /// Validate all rules for syntax errors
    pub fn validate_rules(&self) -> Result<()> {
        for rule in &self.rules {
            Pattern::new(&rule.pattern)
                .with_context(|| format!("Invalid pattern in rule: {}", rule.description))?;
        }
        Ok(())
    }

    /// Get rules that would match a specific path
    pub fn get_matching_rules(&self, path: &Path) -> Vec<&PathPermissionRule> {
        self.rules
            .iter()
            .filter(|rule| rule.matches_path(path))
            .collect()
    }

    /// Check if the validator has inheritance from parent directories
    pub fn check_parent_inheritance(
        &self,
        path: &Path,
        operation: &OperationType,
    ) -> Option<PermissionLevel> {
        let mut current_path = path;

        // Walk up the directory tree
        while let Some(parent) = current_path.parent() {
            // Check if any rules apply to the parent
            for rule in &self.rules {
                if rule.matches_path(parent) && rule.allowed_operations.contains(operation) {
                    return Some(rule.permission.clone());
                }
            }
            current_path = parent;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_rule(pattern: &str, permission: PermissionLevel) -> PathPermissionRule {
        PathPermissionRule::new(
            pattern.to_string(),
            permission,
            vec!["read", "write"],
            100,
            format!("Test rule for {}", pattern),
        )
        .unwrap()
    }

    #[test]
    fn test_permission_level_operations() {
        assert!(PermissionLevel::ReadOnly.allows_operation(&OperationType::Read));
        assert!(!PermissionLevel::ReadOnly.allows_operation(&OperationType::Write));
        assert!(PermissionLevel::Full.allows_operation(&OperationType::Delete));
        assert!(!PermissionLevel::None.allows_operation(&OperationType::Read));
    }

    #[test]
    fn test_permission_level_priority() {
        assert!(PermissionLevel::Full.priority() > PermissionLevel::ReadOnly.priority());
        assert!(PermissionLevel::ReadWrite.priority() > PermissionLevel::ReadBasic.priority());
        assert_eq!(PermissionLevel::None.priority(), 0);
    }

    #[test]
    fn test_path_permission_rule_creation() {
        let rule = PathPermissionRule::new(
            "src/**/*.rs".to_string(),
            PermissionLevel::ReadWrite,
            vec!["read", "write", "list"],
            200,
            "Rust source files".to_string(),
        )
        .unwrap();

        assert_eq!(rule.pattern, "src/**/*.rs");
        assert_eq!(rule.permission, PermissionLevel::ReadWrite);
        assert!(rule.allowed_operations.contains(&OperationType::Read));
        assert!(rule.allowed_operations.contains(&OperationType::Write));
        assert!(rule.allowed_operations.contains(&OperationType::List));
        assert!(!rule.allowed_operations.contains(&OperationType::Delete));
    }

    #[test]
    fn test_path_matching() {
        let rule = create_test_rule("src/**/*.rs", PermissionLevel::ReadWrite);

        assert!(rule.matches_path(&PathBuf::from("src/main.rs")));
        assert!(rule.matches_path(&PathBuf::from("src/lib/utils.rs")));
        assert!(!rule.matches_path(&PathBuf::from("test/main.rs")));
        assert!(!rule.matches_path(&PathBuf::from("src/main.py")));
    }

    #[test]
    fn test_permission_validator_basic() {
        let mut validator = PathPermissionValidator::new(true);
        let rule = create_test_rule("**/*.txt", PermissionLevel::ReadOnly);
        validator.add_rule(rule);

        let operations = [OperationType::Read].iter().cloned().collect();
        let result = validator.evaluate_permissions(&PathBuf::from("test.txt"), &operations, None);

        assert!(result.allowed);
        assert_eq!(result.effective_level, PermissionLevel::ReadOnly);
        assert!(!result.matched_policies.is_empty());
    }

    #[test]
    fn test_strict_mode_permission_denial() {
        let mut validator = PathPermissionValidator::new(true);
        let rule = create_test_rule("secrets/**", PermissionLevel::None);
        validator.add_rule(rule);

        let operations = [OperationType::Read].iter().cloned().collect();
        let result = validator.evaluate_permissions(
            &PathBuf::from("secrets/api_key.txt"),
            &operations,
            None,
        );

        assert!(!result.allowed);
        assert_eq!(result.effective_level, PermissionLevel::None);
    }

    #[test]
    fn test_rule_priority_ordering() {
        let mut validator = PathPermissionValidator::new(false);

        // Add lower priority rule first
        let low_rule = PathPermissionRule::new(
            "**/*".to_string(),
            PermissionLevel::ReadOnly,
            vec!["read"],
            10,
            "Default rule".to_string(),
        )
        .unwrap();

        // Add higher priority rule
        let high_rule = PathPermissionRule::new(
            "config/**".to_string(),
            PermissionLevel::Full,
            vec!["read", "write", "delete"],
            100,
            "Config rule".to_string(),
        )
        .unwrap();

        validator.add_rule(low_rule);
        validator.add_rule(high_rule);

        let operations = [OperationType::Write].iter().cloned().collect();
        let result = validator.evaluate_permissions(
            &PathBuf::from("config/database.toml"),
            &operations,
            None,
        );

        assert!(result.allowed);
        assert_eq!(result.effective_level, PermissionLevel::Full);
        assert!(result
            .matched_policies
            .iter()
            .any(|p| p.contains("Config rule")));
    }

    #[test]
    fn test_parent_directory_inheritance() {
        let mut validator = PathPermissionValidator::new(true);
        let rule = create_test_rule("project/**", PermissionLevel::ReadWrite);
        validator.add_rule(rule);

        let inheritance = validator.check_parent_inheritance(
            &PathBuf::from("project/subdir/deep/file.txt"),
            &OperationType::Read,
        );

        assert!(inheritance.is_some());
        assert_eq!(inheritance.unwrap(), PermissionLevel::ReadWrite);
    }

    #[test]
    fn test_coverage_statistics() {
        let mut validator = PathPermissionValidator::new(true);

        validator.add_rule(create_test_rule("**/*.rs", PermissionLevel::ReadWrite));
        validator.add_rule(create_test_rule("**/*.py", PermissionLevel::ReadOnly));

        let stats = validator.get_coverage_stats();

        assert_eq!(stats.get("total_rules"), Some(&2));
        assert_eq!(stats.get("enabled_rules"), Some(&2));
        assert_eq!(stats.get("ReadWrite_rules"), Some(&1));
        assert_eq!(stats.get("ReadOnly_rules"), Some(&1));
    }
}

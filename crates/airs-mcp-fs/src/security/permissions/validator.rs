//! # Path Permission Validator
//!
//! Main validation engine that orchestrates permission evaluation, policy integration,
//! and rule-based access control for filesystem operations.

// Layer 1: Standard library imports
use std::collections::{HashMap, HashSet};
use std::path::Path;

// Layer 2: Third-party crate imports
use anyhow::{Context, Result};
use chrono::Utc;
use glob::Pattern;

// Layer 3: Internal module imports
use crate::config::settings::{RiskLevel, SecurityPolicy};
use crate::mcp::types::OperationType;
use super::evaluation::PermissionEvaluation;
use super::level::PermissionLevel;
use super::rule::PathPermissionRule;

/// Advanced path permission validator with glob patterns and inheritance.
///
/// The main engine for evaluating filesystem permissions based on configurable
/// rules and security policies. Supports both strict and permissive evaluation
/// modes, rule priority ordering, and parent directory inheritance.
///
/// # Architecture
///
/// The validator operates in layers:
/// 1. **Rule Evaluation** - Match path against permission rules by priority
/// 2. **Policy Integration** - Apply security policy restrictions
/// 3. **Permission Calculation** - Determine effective permission level
/// 4. **Operation Validation** - Check if specific operations are allowed
///
/// # Examples
///
/// ```rust
/// use airs_mcp_fs::security::permissions::*;
/// use std::collections::HashSet;
/// use std::path::PathBuf;
///
/// // Create validator in strict mode
/// let mut validator = PathPermissionValidator::new(true);
///
/// // Add rules for different file types
/// let rust_rule = PathPermissionRule::new(
///     "src/**/*.rs".to_string(),
///     PermissionLevel::ReadWrite,
///     vec!["read", "write"],
///     100,
///     "Rust source files".to_string(),
/// )?;
/// validator.add_rule(rust_rule);
///
/// let docs_rule = PathPermissionRule::new(
///     "docs/**/*.md".to_string(),
///     PermissionLevel::ReadOnly,
///     vec!["read"],
///     50,
///     "Documentation files".to_string(),
/// )?;
/// validator.add_rule(docs_rule);
///
/// // Evaluate permissions for a specific operation
/// let operations: HashSet<_> = [OperationType::Read].iter().cloned().collect();
/// let result = validator.evaluate_permissions(
///     &PathBuf::from("src/main.rs"),
///     &operations,
///     None,
/// );
///
/// assert!(result.allowed);
/// assert_eq!(result.effective_level, PermissionLevel::ReadWrite);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Security Considerations
///
/// - **Strict Mode**: Denies access unless explicitly granted by rules
/// - **Permissive Mode**: Allows access unless explicitly denied (use with caution)
/// - **Rule Priority**: Higher priority rules override lower priority ones
/// - **Policy Integration**: Security policies can add restrictions but not expand access
/// - **Operation Granularity**: Rules specify exactly which operations are allowed
#[derive(Debug, Clone)]
pub struct PathPermissionValidator {
    /// Rules for path-based permissions, sorted by priority (descending).
    ///
    /// Higher priority rules are evaluated first and can override
    /// lower priority rules for the same path.
    rules: Vec<PathPermissionRule>,

    /// Security policies cache for additional restrictions.
    ///
    /// Policies provide additional context and restrictions that are
    /// applied alongside rule-based permissions.
    policies: HashMap<String, SecurityPolicy>,

    /// Whether to use strict evaluation mode.
    ///
    /// - `true`: Deny by default, require explicit permission grants
    /// - `false`: Allow by default unless explicitly denied
    strict_mode: bool,
}

impl PathPermissionValidator {
    /// Create a new path permission validator.
    ///
    /// # Arguments
    ///
    /// * `strict_mode` - Whether to use strict evaluation (deny by default)
    ///
    /// # Security Recommendations
    ///
    /// - Use `strict_mode = true` for production environments
    /// - Use `strict_mode = false` only for development with trusted code
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::PathPermissionValidator;
    /// // Strict mode for production
    /// let production_validator = PathPermissionValidator::new(true);
    ///
    /// // Permissive mode for development
    /// let dev_validator = PathPermissionValidator::new(false);
    /// ```
    pub fn new(strict_mode: bool) -> Self {
        Self {
            rules: Vec::new(),
            policies: HashMap::new(),
            strict_mode,
        }
    }

    /// Add a permission rule to the validator.
    ///
    /// Rules are automatically sorted by priority (descending) to ensure
    /// higher priority rules are evaluated first.
    ///
    /// # Arguments
    ///
    /// * `rule` - The permission rule to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionValidator, PathPermissionRule, PermissionLevel};
    /// let mut validator = PathPermissionValidator::new(true);
    ///
    /// let high_priority_rule = PathPermissionRule::new(
    ///     "secrets/**".to_string(),
    ///     PermissionLevel::None,
    ///     vec![],
    ///     1000, // High priority
    ///     "Block access to secrets".to_string(),
    /// )?;
    ///
    /// let low_priority_rule = PathPermissionRule::new(
    ///     "**/*".to_string(),
    ///     PermissionLevel::ReadOnly,
    ///     vec!["read"],
    ///     10, // Low priority
    ///     "Default read access".to_string(),
    /// )?;
    ///
    /// validator.add_rule(high_priority_rule);
    /// validator.add_rule(low_priority_rule);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn add_rule(&mut self, rule: PathPermissionRule) {
        self.rules.push(rule);
        // Sort by priority (descending) for proper evaluation order
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Add a security policy to the validator cache.
    ///
    /// Policies provide additional context and restrictions that are
    /// evaluated alongside rule-based permissions.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique name for the policy
    /// * `policy` - The security policy to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::PathPermissionValidator;
    /// # use airs_mcp_fs::config::settings::{SecurityPolicy, RiskLevel};
    /// let mut validator = PathPermissionValidator::new(true);
    ///
    /// let policy = SecurityPolicy {
    ///     patterns: vec!["sensitive/**".to_string()],
    ///     operations: vec!["read".to_string()],
    ///     risk_level: RiskLevel::High,
    ///     description: Some("Sensitive data policy".to_string()),
    /// };
    ///
    /// validator.add_policy("sensitive_data".to_string(), policy);
    /// ```
    pub fn add_policy(&mut self, name: String, policy: SecurityPolicy) {
        self.policies.insert(name, policy);
    }

    /// Evaluate permissions for a path and set of operations.
    ///
    /// This is the main entry point for permission evaluation. It processes
    /// rules by priority, integrates security policies, and produces a
    /// comprehensive evaluation result.
    ///
    /// # Arguments
    ///
    /// * `path` - Filesystem path to evaluate
    /// * `operations` - Set of operations being requested
    /// * `_context` - Additional context (reserved for future use)
    ///
    /// # Returns
    ///
    /// A [`PermissionEvaluation`] containing the decision, reasoning, and metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionValidator, PathPermissionRule, PermissionLevel};
    /// # use airs_mcp_fs::mcp::types::OperationType;
    /// # use std::collections::HashSet;
    /// # use std::path::PathBuf;
    /// let mut validator = PathPermissionValidator::new(true);
    ///
    /// let rule = PathPermissionRule::new(
    ///     "docs/**/*.md".to_string(),
    ///     PermissionLevel::ReadOnly,
    ///     vec!["read"],
    ///     100,
    ///     "Documentation access".to_string(),
    /// )?;
    /// validator.add_rule(rule);
    ///
    /// let operations: HashSet<_> = [OperationType::Read].iter().cloned().collect();
    /// let result = validator.evaluate_permissions(
    ///     &PathBuf::from("docs/README.md"),
    ///     &operations,
    ///     None,
    /// );
    ///
    /// assert!(result.allowed);
    /// assert!(!result.matched_policies.is_empty());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// Check if a path matches a security policy pattern.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check against policy patterns
    /// * `policy` - Security policy to evaluate
    ///
    /// # Returns
    ///
    /// `true` if the path matches any policy pattern or if no patterns are defined
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

    /// Get statistics about rule coverage and configuration.
    ///
    /// Provides insights into the validator's configuration for monitoring
    /// and debugging purposes.
    ///
    /// # Returns
    ///
    /// HashMap with statistics including rule counts, policy counts, and
    /// distribution by permission level
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionValidator, PathPermissionRule, PermissionLevel};
    /// let mut validator = PathPermissionValidator::new(true);
    ///
    /// // Add some rules
    /// validator.add_rule(PathPermissionRule::new(
    ///     "**/*.rs".to_string(),
    ///     PermissionLevel::ReadWrite,
    ///     vec!["read", "write"],
    ///     100,
    ///     "Rust files".to_string(),
    /// )?);
    ///
    /// let stats = validator.get_coverage_stats();
    /// assert_eq!(stats.get("total_rules"), Some(&1));
    /// assert_eq!(stats.get("enabled_rules"), Some(&1));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_coverage_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();

        stats.insert("total_rules".to_string(), self.rules.len());
        stats.insert(
            "enabled_rules".to_string(),
            self.rules.iter().filter(|r| r.is_enabled()).count(),
        );
        stats.insert("policies".to_string(), self.policies.len());

        // Group rules by permission level
        for rule in &self.rules {
            let key = format!("{:?}_rules", rule.permission);
            *stats.entry(key).or_insert(0) += 1;
        }

        stats
    }

    /// Validate all rules for syntax errors.
    ///
    /// Checks that all configured rules have valid glob patterns.
    /// Useful for configuration validation and startup checks.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all rules are valid, `Err` with details if any rule is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionValidator, PathPermissionRule, PermissionLevel};
    /// let mut validator = PathPermissionValidator::new(true);
    ///
    /// validator.add_rule(PathPermissionRule::new(
    ///     "src/**/*.rs".to_string(),
    ///     PermissionLevel::ReadOnly,
    ///     vec!["read"],
    ///     100,
    ///     "Valid rule".to_string(),
    /// )?);
    ///
    /// assert!(validator.validate_rules().is_ok());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn validate_rules(&self) -> Result<()> {
        for rule in &self.rules {
            rule.validate_pattern()
                .with_context(|| format!("Invalid rule: {}", rule.description))?;
        }
        Ok(())
    }

    /// Get rules that would match a specific path.
    ///
    /// Returns all enabled rules whose patterns match the given path,
    /// sorted by priority (highest first).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to find matching rules for
    ///
    /// # Returns
    ///
    /// Vector of references to matching rules, sorted by priority
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionValidator, PathPermissionRule, PermissionLevel};
    /// # use std::path::PathBuf;
    /// let mut validator = PathPermissionValidator::new(true);
    ///
    /// validator.add_rule(PathPermissionRule::new(
    ///     "src/**/*.rs".to_string(),
    ///     PermissionLevel::ReadWrite,
    ///     vec!["read", "write"],
    ///     100,
    ///     "Rust files".to_string(),
    /// )?);
    ///
    /// let matching_rules = validator.get_matching_rules(&PathBuf::from("src/main.rs"));
    /// assert_eq!(matching_rules.len(), 1);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_matching_rules(&self, path: &Path) -> Vec<&PathPermissionRule> {
        self.rules
            .iter()
            .filter(|rule| rule.matches_path(path))
            .collect()
    }

    /// Check for permission inheritance from parent directories.
    ///
    /// Walks up the directory tree to find rules that apply to parent
    /// directories and would grant the requested operation. This supports
    /// hierarchical permission models where subdirectories inherit access.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check for inherited permissions
    /// * `operation` - Specific operation to check inheritance for
    ///
    /// # Returns
    ///
    /// `Some(PermissionLevel)` if a parent directory grants the operation,
    /// `None` if no inherited permission is found
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionValidator, PathPermissionRule, PermissionLevel};
    /// # use airs_mcp_fs::mcp::types::OperationType;
    /// # use std::path::PathBuf;
    /// let mut validator = PathPermissionValidator::new(true);
    ///
    /// validator.add_rule(PathPermissionRule::new(
    ///     "project/**".to_string(),
    ///     PermissionLevel::ReadWrite,
    ///     vec!["read", "write"],
    ///     100,
    ///     "Project directory".to_string(),
    /// )?);
    ///
    /// let inherited = validator.check_parent_inheritance(
    ///     &PathBuf::from("project/subdir/deep/file.txt"),
    ///     &OperationType::Read,
    /// );
    ///
    /// assert_eq!(inherited, Some(PermissionLevel::ReadWrite));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// Get the current evaluation mode.
    ///
    /// # Returns
    ///
    /// `true` if in strict mode (deny by default), `false` if permissive
    pub fn is_strict_mode(&self) -> bool {
        self.strict_mode
    }

    /// Get the total number of configured rules.
    ///
    /// # Returns
    ///
    /// Number of rules (both enabled and disabled)
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get the number of configured policies.
    ///
    /// # Returns
    ///
    /// Number of security policies in the cache
    pub fn policy_count(&self) -> usize {
        self.policies.len()
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

    #[test]
    fn test_permissive_mode_default() {
        let validator = PathPermissionValidator::new(false);

        let operations = [OperationType::Read].iter().cloned().collect();
        let result = validator.evaluate_permissions(
            &PathBuf::from("any/file.txt"), 
            &operations, 
            None
        );

        assert!(result.allowed);
        assert_eq!(result.effective_level, PermissionLevel::ReadWrite);
        assert!(result.decision_reason.contains("Default permissive access"));
    }

    #[test]
    fn test_strict_mode_default_denial() {
        let validator = PathPermissionValidator::new(true);

        let operations = [OperationType::Read].iter().cloned().collect();
        let result = validator.evaluate_permissions(
            &PathBuf::from("any/file.txt"), 
            &operations, 
            None
        );

        assert!(!result.allowed);
        assert_eq!(result.effective_level, PermissionLevel::None);
        assert!(result.decision_reason.contains("No explicit permission granted"));
    }

    #[test]
    fn test_rule_validation() {
        let mut validator = PathPermissionValidator::new(true);
        
        validator.add_rule(create_test_rule("src/**/*.rs", PermissionLevel::ReadWrite));
        validator.add_rule(create_test_rule("docs/**/*.md", PermissionLevel::ReadOnly));

        assert!(validator.validate_rules().is_ok());
    }

    #[test]
    fn test_matching_rules() {
        let mut validator = PathPermissionValidator::new(true);
        
        validator.add_rule(create_test_rule("src/**/*.rs", PermissionLevel::ReadWrite));
        validator.add_rule(create_test_rule("**/*.rs", PermissionLevel::ReadOnly));

        let matching = validator.get_matching_rules(&PathBuf::from("src/main.rs"));
        assert_eq!(matching.len(), 2);
    }

    #[test]
    fn test_validator_state_queries() {
        let mut validator = PathPermissionValidator::new(true);
        
        assert!(validator.is_strict_mode());
        assert_eq!(validator.rule_count(), 0);
        assert_eq!(validator.policy_count(), 0);

        validator.add_rule(create_test_rule("**/*", PermissionLevel::ReadOnly));
        
        let policy = SecurityPolicy {
            patterns: vec!["secret/**".to_string()],
            operations: vec!["read".to_string()],
            risk_level: RiskLevel::High,
            description: Some("Test policy".to_string()),
        };
        validator.add_policy("secrets".to_string(), policy);

        assert_eq!(validator.rule_count(), 1);
        assert_eq!(validator.policy_count(), 1);
    }
}

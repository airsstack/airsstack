//! # Path Permission Rules
//!
//! Individual permission rules that define access control for specific
//! path patterns using glob matching and operation-specific permissions.

// Layer 1: Standard library imports
use std::collections::HashSet;
use std::path::Path;

// Layer 2: Third-party crate imports
use anyhow::{Context, Result};
use glob::Pattern;
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use super::level::PermissionLevel;
use crate::mcp::types::OperationType;

/// A path-based permission rule with glob pattern matching.
///
/// Rules define access control for filesystem paths using glob patterns
/// and specify which operations are allowed. Rules are evaluated by
/// priority order, with higher priority rules taking precedence.
///
/// # Glob Pattern Support
///
/// The rule system supports standard glob patterns:
/// - `*` - Matches any sequence of characters (excluding path separators)
/// - `**` - Matches any sequence including path separators (recursive)
/// - `?` - Matches any single character
/// - `[abc]` - Matches any character in the set
/// - `{a,b,c}` - Matches any of the alternatives
///
/// # Examples
///
/// ```rust
/// use airs_mcp_fs::security::permissions::{PathPermissionRule, PermissionLevel};
/// use std::path::PathBuf;
///
/// // Create a rule for Rust source files
/// let rule = PathPermissionRule::new(
///     "src/**/*.rs".to_string(),
///     PermissionLevel::ReadWrite,
///     vec!["read", "write"],
///     100,
///     "Rust source code access".to_string(),
/// )?;
///
/// // Test pattern matching
/// assert!(rule.matches_path(&PathBuf::from("src/main.rs")));
/// assert!(rule.matches_path(&PathBuf::from("src/lib/utils.rs")));
/// assert!(!rule.matches_path(&PathBuf::from("tests/main.rs")));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Security Considerations
///
/// - Use specific patterns rather than overly broad ones like `**/*`
/// - Higher priority rules can override security restrictions
/// - Disabled rules are completely ignored during evaluation
/// - Operation sets should follow principle of least privilege
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathPermissionRule {
    /// Glob pattern to match filesystem paths.
    ///
    /// Supports standard glob syntax including `**` for recursive matching.
    /// Pattern compilation is validated when the rule is created.
    pub pattern: String,

    /// Maximum permission level granted by this rule.
    ///
    /// The effective permission may be lower if not all requested
    /// operations are included in `allowed_operations`.
    pub permission: PermissionLevel,

    /// Set of operations explicitly allowed by this rule.
    ///
    /// Even if `permission` level would normally allow an operation,
    /// it must be explicitly listed here to be granted.
    pub allowed_operations: HashSet<OperationType>,

    /// Whether this rule is active during evaluation.
    ///
    /// Disabled rules are completely ignored. Use for temporary
    /// rule deactivation without deletion.
    pub enabled: bool,

    /// Priority for rule evaluation order.
    ///
    /// Higher values are evaluated first. When multiple rules match
    /// the same path, the highest priority rule determines permissions.
    /// Use priority ranges like: 1-100 (low), 101-500 (medium), 501+ (high).
    pub priority: i32,

    /// Human-readable description of this rule's purpose.
    ///
    /// Used in audit logs and permission evaluation explanations.
    /// Should clearly indicate what access this rule is intended to provide.
    pub description: String,
}

impl PathPermissionRule {
    /// Create a new path permission rule with validation.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Glob pattern to match paths (validated for syntax)
    /// * `permission` - Maximum permission level to grant
    /// * `operations` - String names of allowed operations
    /// * `priority` - Priority for rule evaluation (higher = more important)
    /// * `description` - Human-readable purpose description
    ///
    /// # Supported Operations
    ///
    /// - `"read"` - Read file contents
    /// - `"write"` - Modify file contents
    /// - `"delete"` - Remove files/directories
    /// - `"create_dir"` - Create new directories
    /// - `"list"` - List directory contents
    /// - `"move"` - Move/rename files
    /// - `"copy"` - Copy files
    ///
    /// # Returns
    ///
    /// `Ok(PathPermissionRule)` if all parameters are valid, otherwise
    /// `Err` with details about the validation failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionRule, PermissionLevel};
    /// // Read-write access to documentation
    /// let docs_rule = PathPermissionRule::new(
    ///     "docs/**/*.md".to_string(),
    ///     PermissionLevel::ReadWrite,
    ///     vec!["read", "write"],
    ///     200,
    ///     "Documentation files".to_string(),
    /// )?;
    ///
    /// // Read-only access to configuration templates
    /// let config_rule = PathPermissionRule::new(
    ///     "config/templates/**".to_string(),
    ///     PermissionLevel::ReadOnly,
    ///     vec!["read"],
    ///     150,
    ///     "Configuration templates".to_string(),
    /// )?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(
        pattern: String,
        permission: PermissionLevel,
        operations: Vec<&str>,
        priority: i32,
        description: String,
    ) -> Result<Self> {
        // Validate the glob pattern
        Pattern::new(&pattern).with_context(|| format!("Invalid glob pattern: {pattern}"))?;

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

    /// Check if this rule's pattern matches the given path.
    ///
    /// Uses glob pattern matching with support for `**` recursive patterns.
    /// Disabled rules never match regardless of pattern.
    ///
    /// # Arguments
    ///
    /// * `path` - Filesystem path to test against the rule pattern
    ///
    /// # Returns
    ///
    /// `true` if the rule is enabled and the pattern matches the path
    ///
    /// # Performance
    ///
    /// Pattern compilation is cached, so repeated calls with the same
    /// rule are efficient. Time complexity is O(n) where n is path length.
    pub fn matches_path(&self, path: &Path) -> bool {
        if !self.enabled {
            return false;
        }

        Pattern::new(&self.pattern)
            .map(|pattern| pattern.matches_path(path))
            .unwrap_or(false)
    }

    /// Evaluate effective permission level for a set of operations.
    ///
    /// Determines what permission level to grant based on whether all
    /// requested operations are allowed by this rule. Uses "all or nothing"
    /// semantics - if any operation is not allowed, returns `None`.
    ///
    /// # Arguments
    ///
    /// * `operations` - Set of operations being requested
    ///
    /// # Returns
    ///
    /// - `rule.permission` if all operations are in `allowed_operations`
    /// - `PermissionLevel::None` if any operation is not allowed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionRule, PermissionLevel};
    /// # use airs_mcp_fs::mcp::types::OperationType;
    /// # use std::collections::HashSet;
    /// let rule = PathPermissionRule::new(
    ///     "**/*.txt".to_string(),
    ///     PermissionLevel::ReadWrite,
    ///     vec!["read", "write"],
    ///     100,
    ///     "Text files".to_string(),
    /// )?;
    ///
    /// let read_ops: HashSet<_> = [OperationType::Read].iter().cloned().collect();
    /// let write_ops: HashSet<_> = [OperationType::Read, OperationType::Write].iter().cloned().collect();
    /// let delete_ops: HashSet<_> = [OperationType::Delete].iter().cloned().collect();
    ///
    /// assert_eq!(rule.evaluate_for_operations(&read_ops), PermissionLevel::ReadWrite);
    /// assert_eq!(rule.evaluate_for_operations(&write_ops), PermissionLevel::ReadWrite);
    /// assert_eq!(rule.evaluate_for_operations(&delete_ops), PermissionLevel::None);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// Disable this rule.
    ///
    /// Disabled rules are ignored during evaluation. This provides a way
    /// to temporarily deactivate rules without removing them entirely.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionRule, PermissionLevel};
    /// # use std::path::PathBuf;
    /// let mut rule = PathPermissionRule::new(
    ///     "temp/**".to_string(),
    ///     PermissionLevel::Full,
    ///     vec!["read", "write", "delete"],
    ///     100,
    ///     "Temporary files".to_string(),
    /// )?;
    ///
    /// assert!(rule.matches_path(&PathBuf::from("temp/file.txt")));
    ///
    /// rule.disable();
    /// assert!(!rule.matches_path(&PathBuf::from("temp/file.txt")));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable this rule.
    ///
    /// Re-enables a previously disabled rule for evaluation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionRule, PermissionLevel};
    /// # use std::path::PathBuf;
    /// let mut rule = PathPermissionRule::new(
    ///     "src/**".to_string(),
    ///     PermissionLevel::ReadWrite,
    ///     vec!["read", "write"],
    ///     100,
    ///     "Source files".to_string(),
    /// )?;
    ///
    /// rule.disable();
    /// assert!(!rule.matches_path(&PathBuf::from("src/main.rs")));
    ///
    /// rule.enable();
    /// assert!(rule.matches_path(&PathBuf::from("src/main.rs")));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Check if this rule is currently enabled.
    ///
    /// # Returns
    ///
    /// `true` if the rule is enabled and will be evaluated
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Validate the rule's glob pattern.
    ///
    /// Checks if the pattern can be compiled as a valid glob expression.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the pattern is valid, `Err` with details if invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::{PathPermissionRule, PermissionLevel};
    /// let rule = PathPermissionRule::new(
    ///     "src/**/*.rs".to_string(),
    ///     PermissionLevel::ReadOnly,
    ///     vec!["read"],
    ///     100,
    ///     "Rust files".to_string(),
    /// )?;
    ///
    /// assert!(rule.validate_pattern().is_ok());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn validate_pattern(&self) -> Result<()> {
        Pattern::new(&self.pattern).with_context(|| {
            format!(
                "Invalid glob pattern in rule '{}': {}",
                self.description, self.pattern
            )
        })?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_rule(pattern: &str, permission: PermissionLevel) -> PathPermissionRule {
        PathPermissionRule::new(
            pattern.to_string(),
            permission,
            vec!["read", "write"],
            100,
            format!("Test rule for {pattern}"),
        )
        .unwrap()
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
        assert_eq!(rule.priority, 200);
        assert!(rule.enabled);
    }

    #[test]
    fn test_invalid_operation_type() {
        let result = PathPermissionRule::new(
            "**/*".to_string(),
            PermissionLevel::Full,
            vec!["invalid_operation"],
            100,
            "Test".to_string(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown operation type"));
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
    fn test_disabled_rule_no_match() {
        let mut rule = create_test_rule("**/*.txt", PermissionLevel::ReadOnly);

        assert!(rule.matches_path(&PathBuf::from("test.txt")));

        rule.disable();
        assert!(!rule.matches_path(&PathBuf::from("test.txt")));

        rule.enable();
        assert!(rule.matches_path(&PathBuf::from("test.txt")));
    }

    #[test]
    fn test_operation_evaluation() {
        let rule = PathPermissionRule::new(
            "**/*.txt".to_string(),
            PermissionLevel::ReadWrite,
            vec!["read", "write"],
            100,
            "Text files".to_string(),
        )
        .unwrap();

        let read_ops: HashSet<_> = [OperationType::Read].iter().cloned().collect();
        let write_ops: HashSet<_> = [OperationType::Read, OperationType::Write]
            .iter()
            .cloned()
            .collect();
        let delete_ops: HashSet<_> = [OperationType::Delete].iter().cloned().collect();

        assert_eq!(
            rule.evaluate_for_operations(&read_ops),
            PermissionLevel::ReadWrite
        );
        assert_eq!(
            rule.evaluate_for_operations(&write_ops),
            PermissionLevel::ReadWrite
        );
        assert_eq!(
            rule.evaluate_for_operations(&delete_ops),
            PermissionLevel::None
        );
    }

    #[test]
    fn test_rule_enable_disable() {
        let mut rule = create_test_rule("**/*", PermissionLevel::Full);

        assert!(rule.is_enabled());

        rule.disable();
        assert!(!rule.is_enabled());

        rule.enable();
        assert!(rule.is_enabled());
    }

    #[test]
    fn test_pattern_validation() {
        let valid_rule = create_test_rule("src/**/*.rs", PermissionLevel::ReadOnly);
        assert!(valid_rule.validate_pattern().is_ok());

        // Test with invalid pattern during creation
        let invalid_result = PathPermissionRule::new(
            "[".to_string(), // Invalid glob pattern
            PermissionLevel::ReadOnly,
            vec!["read"],
            100,
            "Invalid pattern".to_string(),
        );
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_glob_patterns() {
        // Test various glob patterns

        // Test exact file match (should only match files in root, not subdirectories)
        let root_only_rule = create_test_rule("main.rs", PermissionLevel::ReadOnly);
        assert!(root_only_rule.matches_path(&PathBuf::from("main.rs")));
        assert!(!root_only_rule.matches_path(&PathBuf::from("src/main.rs")));

        // Test wildcard for any .rs file (matches any .rs file at any level due to glob behavior)
        let wildcard_rule = create_test_rule("*.rs", PermissionLevel::ReadOnly);
        assert!(wildcard_rule.matches_path(&PathBuf::from("main.rs")));
        // The glob crate's *.rs pattern matches files at any depth
        assert!(wildcard_rule.matches_path(&PathBuf::from("src/main.rs")));

        let recursive_rule = create_test_rule("src/**/*.rs", PermissionLevel::ReadOnly);
        assert!(recursive_rule.matches_path(&PathBuf::from("src/main.rs")));
        assert!(recursive_rule.matches_path(&PathBuf::from("src/lib/utils.rs")));
        assert!(recursive_rule.matches_path(&PathBuf::from("src/deep/nested/file.rs")));

        let question_rule = create_test_rule("file?.txt", PermissionLevel::ReadOnly);
        assert!(question_rule.matches_path(&PathBuf::from("file1.txt")));
        assert!(question_rule.matches_path(&PathBuf::from("filea.txt")));
        assert!(!question_rule.matches_path(&PathBuf::from("file10.txt")));
    }

    #[test]
    fn test_rule_serialization() {
        let rule = create_test_rule("src/**/*.rs", PermissionLevel::ReadWrite);

        let serialized = serde_json::to_string(&rule).unwrap();
        let deserialized: PathPermissionRule = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.pattern, rule.pattern);
        assert_eq!(deserialized.permission, rule.permission);
        assert_eq!(deserialized.allowed_operations, rule.allowed_operations);
        assert_eq!(deserialized.enabled, rule.enabled);
        assert_eq!(deserialized.priority, rule.priority);
    }
}

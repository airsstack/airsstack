//! # Permission Evaluation Results
//!
//! Structures and types for representing the results of permission evaluation,
//! including decision reasoning, risk assessment, and audit information.

// Layer 1: Standard library imports
// (none required for this module)

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use super::level::PermissionLevel;
use crate::config::settings::RiskLevel;

/// Result of permission evaluation for a specific path and operation.
///
/// Contains comprehensive information about a permission decision including
/// the decision itself, reasoning, risk assessment, and audit metadata.
///
/// # Examples
///
/// ```rust
/// use airs_mcpserver_fs::security::permissions::{PermissionEvaluation, PermissionLevel};
/// use airs_mcpserver_fs::config::settings::RiskLevel;
/// use chrono::Utc;
///
/// let evaluation = PermissionEvaluation {
///     allowed: true,
///     effective_level: PermissionLevel::ReadWrite,
///     matched_policies: vec!["source_code_policy".to_string()],
///     risk_level: RiskLevel::Low,
///     decision_reason: "Rule 'Rust source files' grants ReadWrite permission".to_string(),
///     evaluated_at: Utc::now(),
/// };
///
/// assert!(evaluation.allowed);
/// assert_eq!(evaluation.effective_level, PermissionLevel::ReadWrite);
/// ```
///
/// # Security Considerations
///
/// - Always check `allowed` field before proceeding with operations
/// - Review `risk_level` for high-risk operations that may need additional oversight
/// - Use `decision_reason` for audit logging and user feedback
/// - Include `evaluated_at` timestamp in audit records for compliance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionEvaluation {
    /// Whether the operation is allowed.
    ///
    /// This is the primary decision field that should be checked
    /// before proceeding with any filesystem operation.
    pub allowed: bool,

    /// The effective permission level granted.
    ///
    /// Represents the highest permission level that would be granted
    /// based on the evaluation. May be `None` even if `allowed` is true
    /// in permissive mode scenarios.
    pub effective_level: PermissionLevel,

    /// Policies that matched this path.
    ///
    /// List of policy names or rule descriptions that were considered
    /// during evaluation. Useful for debugging and audit purposes.
    pub matched_policies: Vec<String>,

    /// Risk level of the operation.
    ///
    /// Assessment of the potential risk associated with allowing
    /// this operation. Higher risk operations may require additional
    /// oversight or approval workflows.
    pub risk_level: RiskLevel,

    /// Detailed explanation of the decision.
    ///
    /// Human-readable explanation of why the permission was granted
    /// or denied. Includes information about which rules or policies
    /// influenced the decision.
    pub decision_reason: String,

    /// Timestamp of evaluation.
    ///
    /// When this permission evaluation was performed, using UTC
    /// timezone per workspace standard §3.2. Essential for audit
    /// trails and compliance logging.
    pub evaluated_at: DateTime<Utc>,
}

impl PermissionEvaluation {
    /// Create a new permission evaluation result.
    ///
    /// # Arguments
    ///
    /// * `allowed` - Whether the operation should be permitted
    /// * `effective_level` - The permission level that would be granted
    /// * `matched_policies` - Names of policies that matched during evaluation
    /// * `risk_level` - Assessed risk level of the operation
    /// * `decision_reason` - Human-readable explanation of the decision
    ///
    /// # Returns
    ///
    /// A new `PermissionEvaluation` with the current timestamp
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcpserver_fs::security::permissions::{PermissionEvaluation, PermissionLevel};
    /// # use airs_mcpserver_fs::config::settings::RiskLevel;
    /// let evaluation = PermissionEvaluation::new(
    ///     true,
    ///     PermissionLevel::ReadOnly,
    ///     vec!["public_docs".to_string()],
    ///     RiskLevel::Low,
    ///     "Public documentation access granted".to_string(),
    /// );
    ///
    /// assert!(evaluation.allowed);
    /// assert_eq!(evaluation.risk_level, RiskLevel::Low);
    /// ```
    pub fn new(
        allowed: bool,
        effective_level: PermissionLevel,
        matched_policies: Vec<String>,
        risk_level: RiskLevel,
        decision_reason: String,
    ) -> Self {
        Self {
            allowed,
            effective_level,
            matched_policies,
            risk_level,
            decision_reason,
            evaluated_at: Utc::now(),
        }
    }

    /// Create a denied permission evaluation.
    ///
    /// Convenience method for creating evaluations that deny access.
    /// Sets `allowed` to false and `effective_level` to `None`.
    ///
    /// # Arguments
    ///
    /// * `reason` - Reason why the permission was denied
    /// * `risk_level` - Risk level of the attempted operation
    ///
    /// # Returns
    ///
    /// A new `PermissionEvaluation` indicating denial
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcpserver_fs::security::permissions::PermissionEvaluation;
    /// # use airs_mcpserver_fs::config::settings::RiskLevel;
    /// let evaluation = PermissionEvaluation::denied(
    ///     "Path matches security exclusion pattern".to_string(),
    ///     RiskLevel::High,
    /// );
    ///
    /// assert!(!evaluation.allowed);
    /// assert_eq!(evaluation.risk_level, RiskLevel::High);
    /// ```
    pub fn denied(reason: String, risk_level: RiskLevel) -> Self {
        Self::new(false, PermissionLevel::None, Vec::new(), risk_level, reason)
    }

    /// Create an allowed permission evaluation.
    ///
    /// Convenience method for creating evaluations that grant access.
    /// Sets `allowed` to true with the specified permission level.
    ///
    /// # Arguments
    ///
    /// * `level` - Permission level to grant
    /// * `matched_policies` - Policies that matched and granted access
    /// * `reason` - Reason why the permission was granted
    /// * `risk_level` - Risk level of the operation
    ///
    /// # Returns
    ///
    /// A new `PermissionEvaluation` indicating approval
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcpserver_fs::security::permissions::{PermissionEvaluation, PermissionLevel};
    /// # use airs_mcpserver_fs::config::settings::RiskLevel;
    /// let evaluation = PermissionEvaluation::allowed(
    ///     PermissionLevel::ReadWrite,
    ///     vec!["dev_workspace".to_string()],
    ///     "Development workspace access".to_string(),
    ///     RiskLevel::Medium,
    /// );
    ///
    /// assert!(evaluation.allowed);
    /// assert_eq!(evaluation.effective_level, PermissionLevel::ReadWrite);
    /// ```
    pub fn allowed(
        level: PermissionLevel,
        matched_policies: Vec<String>,
        reason: String,
        risk_level: RiskLevel,
    ) -> Self {
        Self::new(true, level, matched_policies, risk_level, reason)
    }

    /// Check if the evaluation allows a specific operation.
    ///
    /// Combines the `allowed` flag with the effective permission level's
    /// operation checking to determine if a specific operation is permitted.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to check permission for
    ///
    /// # Returns
    ///
    /// `true` if both `allowed` is true and the effective level permits the operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcpserver_fs::security::permissions::{PermissionEvaluation, PermissionLevel};
    /// # use airs_mcpserver_fs::config::settings::RiskLevel;
    /// # use airs_mcpserver_fs::mcp::types::OperationType;
    /// let evaluation = PermissionEvaluation::allowed(
    ///     PermissionLevel::ReadOnly,
    ///     vec![],
    ///     "Read access granted".to_string(),
    ///     RiskLevel::Low,
    /// );
    ///
    /// assert!(evaluation.allows_operation(&OperationType::Read));
    /// assert!(!evaluation.allows_operation(&OperationType::Write));
    /// ```
    pub fn allows_operation(&self, operation: &crate::mcp::types::OperationType) -> bool {
        self.allowed && self.effective_level.allows_operation(operation)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::mcp::types::OperationType;

    #[test]
    fn test_permission_evaluation_creation() {
        let evaluation = PermissionEvaluation::new(
            true,
            PermissionLevel::ReadWrite,
            vec!["test_policy".to_string()],
            RiskLevel::Low,
            "Test evaluation".to_string(),
        );

        assert!(evaluation.allowed);
        assert_eq!(evaluation.effective_level, PermissionLevel::ReadWrite);
        assert_eq!(evaluation.matched_policies, vec!["test_policy"]);
        assert_eq!(evaluation.risk_level, RiskLevel::Low);
        assert_eq!(evaluation.decision_reason, "Test evaluation");
    }

    #[test]
    fn test_permission_evaluation_denied() {
        let evaluation = PermissionEvaluation::denied("Access denied".to_string(), RiskLevel::High);

        assert!(!evaluation.allowed);
        assert_eq!(evaluation.effective_level, PermissionLevel::None);
        assert!(evaluation.matched_policies.is_empty());
        assert_eq!(evaluation.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_permission_evaluation_allowed() {
        let evaluation = PermissionEvaluation::allowed(
            PermissionLevel::Full,
            vec!["admin_policy".to_string()],
            "Admin access granted".to_string(),
            RiskLevel::Medium,
        );

        assert!(evaluation.allowed);
        assert_eq!(evaluation.effective_level, PermissionLevel::Full);
        assert_eq!(evaluation.matched_policies, vec!["admin_policy"]);
        assert_eq!(evaluation.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_allows_operation() {
        let read_evaluation = PermissionEvaluation::allowed(
            PermissionLevel::ReadOnly,
            vec![],
            "Read access".to_string(),
            RiskLevel::Low,
        );

        assert!(read_evaluation.allows_operation(&OperationType::Read));
        assert!(!read_evaluation.allows_operation(&OperationType::Write));

        let denied_evaluation =
            PermissionEvaluation::denied("No access".to_string(), RiskLevel::High);

        assert!(!denied_evaluation.allows_operation(&OperationType::Read));
    }

    #[test]
    fn test_evaluation_timestamp() {
        let before = Utc::now();
        let evaluation = PermissionEvaluation::new(
            true,
            PermissionLevel::ReadOnly,
            vec![],
            RiskLevel::Low,
            "Test".to_string(),
        );
        let after = Utc::now();

        assert!(evaluation.evaluated_at >= before);
        assert!(evaluation.evaluated_at <= after);
    }

    #[test]
    fn test_evaluation_serialization() {
        let evaluation = PermissionEvaluation::new(
            true,
            PermissionLevel::ReadWrite,
            vec!["policy1".to_string()],
            RiskLevel::Medium,
            "Test serialization".to_string(),
        );

        let serialized = serde_json::to_string(&evaluation).unwrap();
        let deserialized: PermissionEvaluation = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.allowed, evaluation.allowed);
        assert_eq!(deserialized.effective_level, evaluation.effective_level);
        assert_eq!(deserialized.matched_policies, evaluation.matched_policies);
    }
}

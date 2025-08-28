//! Human-in-the-loop approval workflow

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

/// Result of a human approval decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    /// Operation approved by human
    Approved,
    /// Operation denied by human
    Denied,
    /// Approval request timed out
    Timeout,
    /// Approval workflow was cancelled
    Cancelled,
}

impl fmt::Display for ApprovalDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApprovalDecision::Approved => write!(f, "Approved"),
            ApprovalDecision::Denied => write!(f, "Denied"),
            ApprovalDecision::Timeout => write!(f, "Timeout"),
            ApprovalDecision::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_decision_display() {
        assert_eq!(ApprovalDecision::Approved.to_string(), "Approved");
        assert_eq!(ApprovalDecision::Denied.to_string(), "Denied");
        assert_eq!(ApprovalDecision::Timeout.to_string(), "Timeout");
        assert_eq!(ApprovalDecision::Cancelled.to_string(), "Cancelled");
    }

    #[test]
    fn test_approval_decision_equality() {
        assert_eq!(ApprovalDecision::Approved, ApprovalDecision::Approved);
        assert_ne!(ApprovalDecision::Approved, ApprovalDecision::Denied);
    }

    #[test]
    fn test_approval_decision_serialization() {
        let decision = ApprovalDecision::Approved;
        let serialized = serde_json::to_string(&decision).unwrap();
        let deserialized: ApprovalDecision = serde_json::from_str(&serialized).unwrap();
        assert_eq!(decision, deserialized);
    }
}

//! Comprehensive audit logging system for security compliance
//!
//! Provides structured logging of all filesystem operations, policy decisions, and security events
//! to support compliance requirements and operational monitoring.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::path::Path;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::config::settings::RiskLevel;
use crate::filesystem::FileOperation;
use crate::mcp::OperationType;
use crate::security::policy::PolicyDecision;

/// Unique identifier for correlating related audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(Uuid);

impl CorrelationId {
    /// Generate a new correlation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of audit events for security tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEventType {
    /// Operation request initiated
    OperationRequested {
        operation: OperationType,
        path: String,
        size_bytes: Option<u64>,
    },
    /// Policy evaluation completed
    PolicyEvaluated {
        policy_name: Option<String>,
        decision: PolicyDecisionLog,
        evaluation_time_ms: u64,
    },
    /// Operation completed successfully
    OperationCompleted {
        operation: OperationType,
        path: String,
        execution_time_ms: u64,
        bytes_processed: Option<u64>,
    },
    /// Operation failed
    OperationFailed {
        operation: OperationType,
        path: String,
        error: String,
        execution_time_ms: u64,
    },
    /// Security violation detected
    SecurityViolation {
        violation_type: String,
        path: String,
        reason: String,
        risk_level: RiskLevel,
    },
}

/// Serializable version of PolicyDecision for audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "decision")]
pub enum PolicyDecisionLog {
    Allow {
        policy_name: String,
        risk_level: RiskLevel,
        reason: String,
    },
    Deny {
        reason: String,
    },
}

impl From<&PolicyDecision> for PolicyDecisionLog {
    fn from(decision: &PolicyDecision) -> Self {
        match decision {
            PolicyDecision::Allow {
                policy_name,
                risk_level,
                reason,
            } => PolicyDecisionLog::Allow {
                policy_name: policy_name.clone(),
                risk_level: risk_level.clone(),
                reason: reason.clone(),
            },
            PolicyDecision::Deny { reason } => PolicyDecisionLog::Deny {
                reason: reason.clone(),
            },
        }
    }
}

/// Complete audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Correlation ID for related events
    pub correlation_id: CorrelationId,
    /// Event timestamp (UTC per workspace standard §3.2)
    pub timestamp: DateTime<Utc>,
    /// Event type and details
    #[serde(flatten)]
    pub event_type: AuditEventType,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(correlation_id: CorrelationId, event_type: AuditEventType) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            correlation_id,
            timestamp: Utc::now(), // Per workspace standard §3.2
            event_type,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Comprehensive audit logging system
#[derive(Debug)]
pub struct AuditLogger {
    /// Current correlation ID for operation tracking
    current_correlation_id: Option<CorrelationId>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            current_correlation_id: None,
        }
    }

    /// Start a new operation with a correlation ID
    pub fn start_operation(&mut self) -> CorrelationId {
        let correlation_id = CorrelationId::new();
        self.current_correlation_id = Some(correlation_id);
        correlation_id
    }

    /// Log an operation request
    pub fn log_operation_requested(
        &self,
        correlation_id: CorrelationId,
        operation: &FileOperation,
    ) {
        let event = AuditEvent::new(
            correlation_id,
            AuditEventType::OperationRequested {
                operation: operation.operation_type,
                path: operation.path.to_string_lossy().to_string(),
                size_bytes: None, // Size not available in FileOperation
            },
        );

        self.emit_audit_event(&event);
    }

    /// Log a policy evaluation
    pub fn log_policy_evaluated(
        &self,
        correlation_id: CorrelationId,
        decision: &PolicyDecision,
        policy_name: Option<&str>,
        evaluation_time_ms: u64,
    ) {
        let event = AuditEvent::new(
            correlation_id,
            AuditEventType::PolicyEvaluated {
                policy_name: policy_name.map(|s| s.to_string()),
                decision: PolicyDecisionLog::from(decision),
                evaluation_time_ms,
            },
        );

        self.emit_audit_event(&event);
    }

    /// Log successful operation completion
    pub fn log_operation_completed(
        &self,
        correlation_id: CorrelationId,
        operation: &FileOperation,
        execution_time_ms: u64,
        bytes_processed: Option<u64>,
    ) {
        let event = AuditEvent::new(
            correlation_id,
            AuditEventType::OperationCompleted {
                operation: operation.operation_type,
                path: operation.path.to_string_lossy().to_string(),
                execution_time_ms,
                bytes_processed,
            },
        );

        self.emit_audit_event(&event);
    }

    /// Log failed operation
    pub fn log_operation_failed(
        &self,
        correlation_id: CorrelationId,
        operation: &FileOperation,
        error: &str,
        execution_time_ms: u64,
    ) {
        let event = AuditEvent::new(
            correlation_id,
            AuditEventType::OperationFailed {
                operation: operation.operation_type,
                path: operation.path.to_string_lossy().to_string(),
                error: error.to_string(),
                execution_time_ms,
            },
        );

        self.emit_audit_event(&event);
    }

    /// Log security violation
    pub fn log_security_violation(
        &self,
        correlation_id: CorrelationId,
        violation_type: &str,
        path: &Path,
        reason: &str,
        risk_level: RiskLevel,
    ) {
        let event = AuditEvent::new(
            correlation_id,
            AuditEventType::SecurityViolation {
                violation_type: violation_type.to_string(),
                path: path.to_string_lossy().to_string(),
                reason: reason.to_string(),
                risk_level,
            },
        );

        self.emit_audit_event(&event);
    }

    /// Emit audit event to structured logging system
    fn emit_audit_event(&self, event: &AuditEvent) {
        let json = match serde_json::to_string(event) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize audit event: {}", e);
                return;
            }
        };

        match &event.event_type {
            AuditEventType::OperationRequested { .. } => {
                info!(
                    target: "audit",
                    correlation_id = %event.correlation_id,
                    event_id = %event.event_id,
                    "{}",
                    json
                );
            }
            AuditEventType::PolicyEvaluated { .. } => {
                info!(
                    target: "audit",
                    correlation_id = %event.correlation_id,
                    event_id = %event.event_id,
                    "{}",
                    json
                );
            }
            AuditEventType::OperationCompleted { .. } => {
                info!(
                    target: "audit",
                    correlation_id = %event.correlation_id,
                    event_id = %event.event_id,
                    "{}",
                    json
                );
            }
            AuditEventType::OperationFailed { .. } => {
                warn!(
                    target: "audit",
                    correlation_id = %event.correlation_id,
                    event_id = %event.event_id,
                    "{}",
                    json
                );
            }
            AuditEventType::SecurityViolation { .. } => {
                error!(
                    target: "audit",
                    correlation_id = %event.correlation_id,
                    event_id = %event.event_id,
                    "{}",
                    json
                );
            }
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::OperationType;
    use std::path::PathBuf;

    #[test]
    fn test_correlation_id_generation() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_audit_event_creation() {
        let correlation_id = CorrelationId::new();
        let event = AuditEvent::new(
            correlation_id,
            AuditEventType::OperationRequested {
                operation: OperationType::Read,
                path: "/test/file.txt".to_string(),
                size_bytes: Some(1024),
            },
        );

        assert_eq!(event.correlation_id, correlation_id);
        assert!(event.timestamp <= Utc::now());
    }

    #[test]
    fn test_policy_decision_log_conversion() {
        let allow_decision = PolicyDecision::Allow {
            policy_name: "test_policy".to_string(),
            risk_level: RiskLevel::Low,
            reason: "Policy allows read access".to_string(),
        };

        let log_decision = PolicyDecisionLog::from(&allow_decision);
        match log_decision {
            PolicyDecisionLog::Allow {
                policy_name,
                risk_level,
                reason,
            } => {
                assert_eq!(policy_name, "test_policy");
                assert_eq!(risk_level, RiskLevel::Low);
                assert_eq!(reason, "Policy allows read access");
            }
            _ => panic!("Expected Allow decision"),
        }
    }

    #[test]
    fn test_audit_event_serialization() {
        let correlation_id = CorrelationId::new();
        let event = AuditEvent::new(
            correlation_id,
            AuditEventType::OperationCompleted {
                operation: OperationType::Write,
                path: "/test/output.txt".to_string(),
                execution_time_ms: 150,
                bytes_processed: Some(2048),
            },
        );

        let json = serde_json::to_string(&event).expect("Should serialize");
        let deserialized: AuditEvent = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.correlation_id, correlation_id);
        assert_eq!(deserialized.event_id, event.event_id);
    }

    #[test]
    fn test_audit_logger_operation_tracking() {
        let mut logger = AuditLogger::new();
        let correlation_id = logger.start_operation();

        let operation = FileOperation::new(OperationType::Read, PathBuf::from("/test/file.txt"));

        // These calls should not panic and should emit structured logs
        logger.log_operation_requested(correlation_id, &operation);
        logger.log_operation_completed(correlation_id, &operation, 100, Some(1024));
    }
}

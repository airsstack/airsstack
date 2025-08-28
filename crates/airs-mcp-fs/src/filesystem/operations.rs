//! Core filesystem operations and tracking

// Layer 1: Standard library imports
use std::path::PathBuf;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::mcp::OperationType;

/// Represents a filesystem operation with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    /// Type of operation being performed
    pub operation_type: OperationType,
    /// Path to the file or directory
    pub path: PathBuf,
    /// Timestamp when operation was created
    pub timestamp: DateTime<Utc>,
    /// Unique operation identifier
    pub operation_id: String,
}

impl FileOperation {
    /// Create a new file operation
    pub fn new(operation_type: OperationType, path: PathBuf) -> Self {
        Self {
            operation_type,
            path,
            timestamp: Utc::now(), // § 3.2 compliance
            operation_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Get the operation as a human-readable string
    pub fn description(&self) -> String {
        format!(
            "{:?} operation on '{}'",
            self.operation_type,
            self.path.display()
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operation_creation() {
        let path = PathBuf::from("/test/path.txt");
        let op = FileOperation::new(OperationType::Read, path.clone());
        
        assert_eq!(op.operation_type, OperationType::Read);
        assert_eq!(op.path, path);
        assert!(!op.operation_id.is_empty());
        // Timestamp should be recent (within last second)
        let now = Utc::now();
        let diff = now.signed_duration_since(op.timestamp);
        assert!(diff.num_seconds() < 1);
    }

    #[test]
    fn test_operation_description() {
        let path = PathBuf::from("/test/file.txt");
        let op = FileOperation::new(OperationType::Write, path);
        
        let desc = op.description();
        assert!(desc.contains("Write"));
        assert!(desc.contains("/test/file.txt"));
    }

    #[test]
    fn test_operation_serialization() {
        let path = PathBuf::from("/test/serialize.txt");
        let op = FileOperation::new(OperationType::List, path);
        
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: FileOperation = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(op.operation_type, deserialized.operation_type);
        assert_eq!(op.path, deserialized.path);
        assert_eq!(op.operation_id, deserialized.operation_id);
    }
}

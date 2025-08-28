//! MCP operation types and core data structures

// Layer 1: Standard library imports
// (None needed for this module)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (None needed yet)

/// Types of filesystem operations supported by MCP tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Read file contents
    Read,
    /// Write file contents
    Write,
    /// List directory contents
    List,
    /// Create directory
    CreateDir,
    /// Delete file or directory
    Delete,
    /// Move/rename file or directory
    Move,
    /// Copy file or directory
    Copy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_type_serialization() {
        let op = OperationType::Read;
        let serialized = serde_json::to_string(&op).unwrap();
        let deserialized: OperationType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(op, deserialized);
    }

    #[test]
    fn test_operation_type_equality() {
        assert_eq!(OperationType::Read, OperationType::Read);
        assert_ne!(OperationType::Read, OperationType::Write);
    }
}

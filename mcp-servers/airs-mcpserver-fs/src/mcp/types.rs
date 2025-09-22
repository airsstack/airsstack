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

impl OperationType {
    /// Get the string representation of the operation type
    ///
    /// Returns the lowercase string identifier used in security policies
    /// and throughout the system for operation matching.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcpserver_fs::mcp::OperationType;
    ///
    /// assert_eq!(OperationType::Read.as_str(), "read");
    /// assert_eq!(OperationType::Write.as_str(), "write");
    /// assert_eq!(OperationType::CreateDir.as_str(), "create_dir");
    /// ```
    pub const fn as_str(self) -> &'static str {
        match self {
            OperationType::Read => "read",
            OperationType::Write => "write",
            OperationType::List => "list",
            OperationType::CreateDir => "create_dir",
            OperationType::Delete => "delete",
            OperationType::Move => "move",
            OperationType::Copy => "copy",
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    #[test]
    fn test_operation_type_as_str() {
        assert_eq!(OperationType::Read.as_str(), "read");
        assert_eq!(OperationType::Write.as_str(), "write");
        assert_eq!(OperationType::List.as_str(), "list");
        assert_eq!(OperationType::CreateDir.as_str(), "create_dir");
        assert_eq!(OperationType::Delete.as_str(), "delete");
        assert_eq!(OperationType::Move.as_str(), "move");
        assert_eq!(OperationType::Copy.as_str(), "copy");
    }
}

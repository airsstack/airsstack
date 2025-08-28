//! # Permission Level Hierarchy
//!
//! Defines the five-level permission hierarchy used throughout the filesystem
//! security system, from no access to full administrative permissions.

// Layer 1: Standard library imports
// (none required for this module)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::mcp::types::OperationType;

/// Five-level permission hierarchy for filesystem access control.
///
/// The permission levels form a strict hierarchy where higher levels
/// include all capabilities of lower levels. This ensures consistent
/// and predictable permission evaluation.
///
/// # Hierarchy (from lowest to highest)
///
/// 1. [`None`](PermissionLevel::None) - No access
/// 2. [`ReadOnly`](PermissionLevel::ReadOnly) - Read files only
/// 3. [`ReadBasic`](PermissionLevel::ReadBasic) - Read + List directories
/// 4. [`ReadWrite`](PermissionLevel::ReadWrite) - Read + Write + Copy operations
/// 5. [`Full`](PermissionLevel::Full) - All operations including delete
///
/// # Examples
///
/// ```rust
/// use airs_mcp_fs::security::permissions::PermissionLevel;
/// use airs_mcp_fs::mcp::types::OperationType;
///
/// // Check what operations are allowed
/// assert!(PermissionLevel::ReadOnly.allows_operation(&OperationType::Read));
/// assert!(!PermissionLevel::ReadOnly.allows_operation(&OperationType::Write));
/// assert!(PermissionLevel::Full.allows_operation(&OperationType::Delete));
///
/// // Compare permission levels
/// assert!(PermissionLevel::Full.priority() > PermissionLevel::ReadOnly.priority());
/// ```
///
/// # Security Notes
///
/// - Use the most restrictive level that meets functional requirements
/// - `Full` permissions should be granted sparingly and with explicit justification
/// - Consider using `ReadWrite` instead of `Full` when deletion isn't required
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    /// No access allowed to the resource.
    ///
    /// This level blocks all operations and is used for explicitly
    /// denied paths or as a secure default in strict mode.
    None,

    /// Read-only access to files.
    ///
    /// Allows:
    /// - Reading file contents
    ///
    /// Use case: Public documentation, configuration templates
    ReadOnly,

    /// Read access plus basic directory operations.
    ///
    /// Allows:
    /// - Reading file contents
    /// - Listing directory contents
    ///
    /// Use case: Browse-only access to project structures
    ReadBasic,

    /// Read and write access without destructive operations.
    ///
    /// Allows:
    /// - Reading file contents
    /// - Listing directory contents  
    /// - Writing/modifying files
    /// - Copying files
    ///
    /// Use case: Active development on source code
    ReadWrite,

    /// Complete access including destructive operations.
    ///
    /// Allows:
    /// - All ReadWrite operations
    /// - Deleting files and directories
    /// - Moving files
    /// - Creating directories
    ///
    /// Use case: Administrative operations, cleanup tasks
    ///
    /// ⚠️ **Security Warning**: Grant sparingly and with explicit justification
    Full,
}

impl PermissionLevel {
    /// Check if this permission level allows a specific operation.
    ///
    /// # Arguments
    ///
    /// * `operation` - The filesystem operation to check
    ///
    /// # Returns
    ///
    /// `true` if the operation is allowed by this permission level
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::PermissionLevel;
    /// # use airs_mcp_fs::mcp::types::OperationType;
    /// assert!(PermissionLevel::ReadWrite.allows_operation(&OperationType::Write));
    /// assert!(!PermissionLevel::ReadOnly.allows_operation(&OperationType::Delete));
    /// ```
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

    /// Get the numeric priority for permission level comparison.
    ///
    /// Higher numbers indicate more permissive levels. This is used
    /// internally for rule priority evaluation.
    ///
    /// # Returns
    ///
    /// Priority value (0-4) where 0 = None, 4 = Full
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp_fs::security::permissions::PermissionLevel;
    /// assert_eq!(PermissionLevel::None.priority(), 0);
    /// assert_eq!(PermissionLevel::Full.priority(), 4);
    /// assert!(PermissionLevel::ReadWrite.priority() > PermissionLevel::ReadOnly.priority());
    /// ```
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
    fn test_permission_level_hierarchy() {
        // Test that higher levels include capabilities of lower levels
        assert!(PermissionLevel::ReadBasic.allows_operation(&OperationType::Read));
        assert!(PermissionLevel::ReadWrite.allows_operation(&OperationType::Read));
        assert!(PermissionLevel::ReadWrite.allows_operation(&OperationType::List));
        assert!(PermissionLevel::Full.allows_operation(&OperationType::Read));
        assert!(PermissionLevel::Full.allows_operation(&OperationType::Write));
    }

    #[test]
    fn test_permission_level_ordering() {
        // Test PartialOrd implementation
        assert!(PermissionLevel::Full > PermissionLevel::ReadWrite);
        assert!(PermissionLevel::ReadWrite > PermissionLevel::ReadBasic);
        assert!(PermissionLevel::ReadBasic > PermissionLevel::ReadOnly);
        assert!(PermissionLevel::ReadOnly > PermissionLevel::None);
    }

    #[test]
    fn test_permission_level_serialization() {
        // Test serde serialization (useful for configuration files)
        let level = PermissionLevel::ReadWrite;
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(serialized, "\"readwrite\"");

        let deserialized: PermissionLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, level);
    }
}

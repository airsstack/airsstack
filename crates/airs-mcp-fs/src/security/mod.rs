//! Security framework and human approval workflows
//! 
//! Provides access control, approval workflows, and audit logging for filesystem operations.

// Layer 1: Standard library imports
// (None needed for pure module coordinator)

// Layer 2: Third-party crate imports
// (None needed for pure module coordinator)

// Layer 3: Internal module declarations
pub mod approval;
pub mod manager;

// Public API re-exports
pub use approval::ApprovalDecision;
pub use manager::SecurityManager;

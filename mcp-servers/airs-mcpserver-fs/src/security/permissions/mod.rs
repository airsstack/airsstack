//! # Path-Based Permission Validation System
//!
//! This module provides a sophisticated permission validation framework for filesystem
//! operations with glob pattern matching, hierarchical permissions, and policy integration.
//!
//! ## Architecture Overview
//!
//! The permission system consists of four main components:
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │ PermissionLevel │    │ PathPermissionRule│    │ PermissionEval  │
//! │                 │    │                   │    │                 │
//! │ • None          │    │ • Glob patterns   │    │ • Decision      │
//! │ • ReadOnly      │◄───┤ • Priority system │───►│ • Risk level    │
//! │ • ReadBasic     │    │ • Operation sets  │    │ • Reasoning     │
//! │ • ReadWrite     │    │                   │    │                 │
//! │ • Full          │    └──────────────────┘    └─────────────────┘
//! └─────────────────┘              │
//!                                  │
//!                     ┌──────────────────────┐
//!                     │ PathPermissionValidator│
//!                     │                        │
//!                     │ • Rule evaluation      │
//!                     │ • Policy integration   │
//!                     │ • Inheritance logic    │
//!                     │ • Security enforcement │
//!                     └──────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use airs_mcpserver_fs::security::permissions::*;
//! use airs_mcpserver_fs::OperationType;
//! use std::collections::HashSet;
//! use std::path::PathBuf;
//!
//! // Create a validator with strict mode
//! let mut validator = PathPermissionValidator::new(true);
//!
//! // Add a rule for source code files
//! let rule = PathPermissionRule::new(
//!     "src/**/*.rs".to_string(),
//!     PermissionLevel::ReadWrite,
//!     vec!["read", "write"],
//!     100,
//!     "Rust source files".to_string(),
//! )?;
//! validator.add_rule(rule);
//!
//! // Evaluate permissions
//! let operations = [OperationType::Read].iter().cloned().collect();
//! let result = validator.evaluate_permissions(
//!     &PathBuf::from("src/main.rs"),
//!     &operations,
//!     None
//! );
//!
//! assert!(result.allowed);
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Security Considerations
//!
//! - **Strict vs Permissive Mode**: Choose based on security requirements
//! - **Rule Priority**: Higher priority rules override lower priority ones
//! - **Glob Pattern Security**: Avoid overly broad patterns in production
//! - **Operation Granularity**: Use specific operation sets rather than blanket permissions
//!
//! ## Integration with Security Framework
//!
//! This module integrates with:
//! - [`crate::security::manager::SecurityManager`] for access control
//! - [`crate::security::policy::PolicyEngine`] for policy evaluation  
//! - [`crate::security::audit::AuditLogger`] for compliance logging
//!
//! ## Component Modules
//!
//! - [`level`] - Permission level hierarchy and operation checking
//! - [`rule`] - Individual permission rule definition and matching
//! - [`evaluation`] - Permission evaluation results and analysis  
//! - [`validator`] - Main validation engine and policy integration

pub mod evaluation;
pub mod level;
pub mod rule;
pub mod validator;

// Re-exports for external consumers (maintains API compatibility)
pub use evaluation::PermissionEvaluation;
pub use level::PermissionLevel;
pub use rule::PathPermissionRule;
pub use validator::PathPermissionValidator;

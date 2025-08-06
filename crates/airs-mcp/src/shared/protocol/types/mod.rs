//! Core MCP Protocol Types
//!
//! This module provides domain-specific newtypes and core protocol structures
//! with validation and proper encapsulation.

pub mod common;
pub mod content;

// Re-export public API
pub use common::*;
pub use content::*;

//! CLI command implementations
//!
//! This module contains all command implementations for the airs-memspec CLI tool.
//! Each command module provides specific functionality for interacting with Multi-Project
//! Memory Bank structures and managing workspace context.
//!
//! # Available Commands
//!
//! - **Context**: Workspace context analysis and switching
//! - **Install**: Memory bank structure installation and setup
//! - **Progress Analyzer**: Progress tracking and analysis functionality
//! - **Status**: Workspace and project status overview
//! - **Tasks**: Task tracking and viewing operations (read-only)

/// Context command implementation
pub mod context;
/// Install command implementation  
pub mod install;
/// Progress analysis command implementation
pub mod progress_analyzer;
/// Status command implementation
pub mod status;
/// Tasks command implementation
pub mod tasks;

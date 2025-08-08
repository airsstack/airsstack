//! Shared utilities and helper functions
//!
//! This module provides common utilities used throughout the airs-memspec crate,
//! including file system operations, output formatting, configuration management,
//! and template handling.
//!
//! # Utility Modules
//!
//! - **Config**: Configuration management and validation
//! - **FS**: File system operations with error handling
//! - **Layout**: Sophisticated CLI output formatting and layout engine
//! - **Output**: Terminal output configuration and color management
//! - **Templates**: Template processing and content generation

/// Configuration management and validation
pub mod config;
/// File system operations with comprehensive error handling
pub mod fs;
/// Advanced layout engine for structured CLI output
pub mod layout;
/// Terminal output configuration and formatting
pub mod output;
/// Template processing and content generation
pub mod templates;

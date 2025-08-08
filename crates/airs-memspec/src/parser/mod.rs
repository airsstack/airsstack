//! Parsing and content analysis modules
//!
//! This module provides comprehensive parsing functionality for Multi-Project Memory Bank
//! content, including markdown processing, context correlation, navigation, and
//! instruction template handling.
//!
//! # Core Parsing Components
//!
//! - **Context**: Context correlation and workspace state analysis
//! - **Instructions**: GitHub Copilot instruction template parsing
//! - **Markdown**: Markdown content parsing and task extraction
//! - **Memory Bank**: Memory bank structure parsing and validation
//! - **Navigation**: File system navigation and discovery

/// Context correlation and workspace state analysis
pub mod context;
/// GitHub Copilot instruction template parsing
pub mod instructions;
/// Markdown content parsing and task extraction
pub mod markdown;
/// Memory bank structure parsing and validation
pub mod memory_bank;
/// File system navigation and structure discovery
pub mod navigation;

//! Provider implementations for MCP HTTP Remote Server
//!
//! This module contains the concrete implementations of MCP providers:
//! - ResourceProvider: File system access with security
//! - ToolProvider: Calculator and utility tools
//! - PromptProvider: Code review and documentation prompts

pub mod filesystem;
pub mod calculator;
pub mod documentation;

pub use filesystem::FileSystemResourceProvider;
pub use calculator::CalculatorToolProvider;
pub use documentation::DocumentationPromptProvider;

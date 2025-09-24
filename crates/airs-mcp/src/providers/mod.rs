//! Production-ready MCP Provider Implementations
//!
//! This module provides comprehensive, production-ready implementations of MCP providers
//! that can be used directly in applications or serve as reference implementations.
//!
//! # Provider Types
//!
//! - **Resource Providers**: File system, configuration, database access
//! - **Tool Providers**: Mathematical operations, system tools, AI utilities  
//! - **Prompt Providers**: Code review templates, documentation generators
//! - **Logging Handlers**: Structured logging with various backends
//!
//! # Usage Example
//!
//! ```text
//! Provider implementations will be available in future versions
//! This module is currently a placeholder for production-ready
//! MCP provider implementations that integrate with the generic
//! MessageHandler architecture
//! ```

pub mod logging;
pub mod prompt;
pub mod resource;
pub mod tool;

// Re-export main provider types for convenience
pub use logging::{FileLoggingHandler, StructuredLoggingHandler};
pub use prompt::{
    AnalysisPromptProvider, CodeReviewPromptProvider, DocumentationPromptProvider, PromptProvider,
};
pub use resource::{
    ConfigurationResourceProvider, DatabaseResourceProvider, FileSystemResourceProvider,
    ResourceProvider,
};
pub use tool::{MathToolProvider, SystemToolProvider, TextToolProvider, ToolProvider};

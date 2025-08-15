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
//! ```rust,no_run
//! use airs_mcp::providers::{FileSystemResourceProvider, MathToolProvider, CodeReviewPromptProvider};
//! use airs_mcp::integration::mcp::McpServerBuilder;
//! use airs_mcp::transport::stdio::StdioTransport;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a transport (using stdio as example)
//!     let transport = StdioTransport::new().await?;
//!     
//!     let server = McpServerBuilder::new()
//!         .server_info("production-server", "1.0.0")
//!         .with_resource_provider(FileSystemResourceProvider::new("/safe/path")?)
//!         .with_tool_provider(MathToolProvider::new())
//!         .with_prompt_provider(CodeReviewPromptProvider::new())
//!         .build(transport)
//!         .await?;
//!     
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

pub mod logging;
pub mod prompt;
pub mod resource;
pub mod tool;

// Re-export main provider types for convenience
pub use logging::{FileLoggingHandler, StructuredLoggingHandler};
pub use prompt::{AnalysisPromptProvider, CodeReviewPromptProvider, DocumentationPromptProvider};
pub use resource::{
    ConfigurationResourceProvider, DatabaseResourceProvider, FileSystemResourceProvider,
};
pub use tool::{MathToolProvider, SystemToolProvider, TextToolProvider};

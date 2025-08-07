//! MCP Protocol Layer Implementation
//!
//! This module provides the Model Context Protocol (MCP) specific message types
//! and abstractions built on top of the existing JSON-RPC 2.0 foundation.
//!
//! # Architecture
//!
//! The MCP protocol layer is organized as follows:
//! - `types`: Core protocol types and domain-specific newtypes with validation
//! - `messages`: MCP-specific message structures for all protocol operations
//! - `errors`: Protocol-specific error types extending the existing error system
//!
//! # Design Principles
//!
//! - **Type Safety**: Domain-specific newtypes prevent invalid protocol messages at compile time
//! - **Validation**: All user inputs validated at construction time with clear error messages
//! - **Integration**: Seamless integration with existing JSON-RPC foundation and correlation system
//! - **Performance**: Maintains exceptional 8.5+ GiB/s throughput characteristics
//! - **Encapsulation**: Private newtype fields with controlled access through validated methods
//!
//! # Examples
//!
//! ```rust
//! use airs_mcp::shared::protocol::{
//!     types::{Uri, ProtocolVersion, ClientInfo},
//!     messages::InitializeRequest,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Type-safe protocol construction
//! let uri = Uri::new("file:///path/to/resource")?;
//! let version = ProtocolVersion::current();
//! let client_info = ClientInfo {
//!     name: "example-client".to_string(),
//!     version: "1.0.0".to_string(),
//! };
//!
//! // All validation happens at construction time
//! assert_eq!(uri.scheme(), Some("file"));
//! assert_eq!(version.as_str(), "2024-11-05");
//! # Ok(())
//! # }
//! ```

pub mod errors;
pub mod messages;
pub mod types;

// Re-export public API for convenient access
pub use errors::*;
pub use messages::*;
pub use types::*;

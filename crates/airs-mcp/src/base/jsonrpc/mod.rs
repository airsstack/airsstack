//! JSON-RPC 2.0 Foundation Implementation
//!
//! This module provides a complete JSON-RPC 2.0 implementation focused on
//! core message types with shared serialization behavior through traits.
//!
//! # Architecture
//!
//! The JSON-RPC foundation is organized as follows:
//! - `message`: Core message types with JsonRpcMessage trait for consistent serialization
//! - `error`: JSON-RPC 2.0 compliant error handling (future)
//! - `id`: Request ID implementation with string/numeric support (future)
//! - `validation`: Message structure validation (future)
//!
//! # Examples
//!
//! ```rust
//! use airs_mcp::base::jsonrpc::{JsonRpcRequest, JsonRpcMessage, RequestId};
//! use serde_json::json;
//!
//! let request = JsonRpcRequest::new(
//!     "ping",
//!     Some(json!({"message": "hello"})),
//!     RequestId::new_string("req-123")
//! );
//!
//! // Use trait methods for consistent serialization
//! let json = request.to_json().unwrap();
//! let pretty_json = request.to_json_pretty().unwrap();
//! let parsed = JsonRpcRequest::from_json(&json).unwrap();
//! 
//! assert_eq!(request, parsed);
//! ```

pub mod message;

// Re-export public API for convenient access
pub use message::*;
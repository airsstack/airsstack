//! Error Types - Consolidated Protocol Error Handling
//!
//! This module consolidates error types from multiple sources:
//! - `src/shared/protocol/errors.rs` (MCP protocol errors)
//! - `src/transport/mcp/error.rs` (MCP transport errors)
//! - Error handling patterns from `src/base/jsonrpc/` (JSON-RPC errors)
//!
//! # Consolidation Strategy
//!
//! **Phase 2 Migration Plan:**
//! - Consolidate all protocol-related error types into a unified hierarchy
//! - Preserve specific error context and diagnostic information
//! - Maintain error conversion traits for backward compatibility
//! - Follow Rust error handling best practices
//!
//! # Architecture Goals
//!
//! - **Comprehensive Coverage**: Handle all protocol error scenarios
//! - **Type Safety**: Strong typing for different error categories
//! - **Diagnostic Information**: Rich error context for debugging
//! - **Conversion Support**: Easy conversion between error types

// Layer 1: Standard library imports
// (None required for current thiserror implementation)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Layer 3: Internal module imports
use crate::protocol::transport::TransportError;
// (Will be added during Phase 2 migration)

// PHASE 1: Placeholder implementations
// These will be replaced with actual consolidated implementations in Phase 2

/// Placeholder for Protocol Error enumeration
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum ProtocolError {
    /// JSON-RPC related errors
    #[error("JSON-RPC error: {message}")]
    JsonRpc { message: String },

    /// MCP protocol specific errors
    #[error("MCP protocol error: {message}")]
    Mcp { message: String },

    /// Transport layer errors
    #[error("Transport error: {message}")]
    Transport { message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Invalid message format errors
    #[error("Invalid message: {message}")]
    InvalidMessage { message: String },

    /// Invalid base64 data
    #[error("Invalid base64 data")]
    InvalidBase64Data,

    /// Invalid protocol version
    #[error("Invalid protocol version: {0}")]
    InvalidProtocolVersion(String),

    /// Invalid URI format
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// Invalid MIME type format
    #[error("Invalid MIME type: {0}")]
    InvalidMimeType(String),
}

/// Convenient result type for protocol operations
pub type ProtocolResult<T> = Result<T, ProtocolError>;

impl From<serde_json::Error> for ProtocolError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            message: err.to_string(),
        }
    }
}

impl From<TransportError> for ProtocolError {
    fn from(err: TransportError) -> Self {
        Self::Transport {
            message: err.to_string(),
        }
    }
}

/// Placeholder for JSON-RPC specific errors
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum JsonRpcError {
    /// Parse error (-32700)
    #[error("Parse error: {message}")]
    ParseError { message: String },

    /// Invalid request (-32600)
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    /// Method not found (-32601)
    #[error("Method not found: {method}")]
    MethodNotFound { method: String },

    /// Invalid parameters (-32602)
    #[error("Invalid parameters: {message}")]
    InvalidParams { message: String },

    /// Internal error (-32603)
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Server error (custom error codes)
    #[error("Server error {code}: {message}")]
    ServerError { code: i32, message: String },
}

/// Placeholder for MCP specific errors
/// Will be populated with actual implementation during Phase 2 migration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum McpError {
    /// Protocol version mismatch
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    /// Capability not supported
    #[error("Unsupported capability: {capability}")]
    UnsupportedCapability { capability: String },

    /// Resource not found
    #[error("Resource not found: {uri}")]
    ResourceNotFound { uri: String },

    /// Authorization failed
    #[error("Authorization failed: {reason}")]
    AuthorizationFailed { reason: String },

    /// Invalid URI format
    #[error("Invalid URI: {uri} - {reason}")]
    InvalidUri { uri: String, reason: String },

    /// Request timeout
    #[error("Request timeout after {timeout_ms}ms")]
    RequestTimeout { timeout_ms: u64 },
}

// Convenience constructors and JSON-RPC error code mappings
impl JsonRpcError {
    /// JSON-RPC 2.0 error codes as defined in the specification
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Get the JSON-RPC error code for this error
    pub fn error_code(&self) -> i32 {
        match self {
            JsonRpcError::ParseError { .. } => Self::PARSE_ERROR,
            JsonRpcError::InvalidRequest { .. } => Self::INVALID_REQUEST,
            JsonRpcError::MethodNotFound { .. } => Self::METHOD_NOT_FOUND,
            JsonRpcError::InvalidParams { .. } => Self::INVALID_PARAMS,
            JsonRpcError::InternalError { .. } => Self::INTERNAL_ERROR,
            JsonRpcError::ServerError { code, .. } => *code,
        }
    }

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
        }
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::MethodNotFound {
            method: method.into(),
        }
    }

    /// Create an invalid parameters error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::InvalidParams {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Create a server error with custom code
    pub fn server_error(code: i32, message: impl Into<String>) -> Self {
        Self::ServerError {
            code,
            message: message.into(),
        }
    }
}

// Convenience constructors for ProtocolError
impl ProtocolError {
    /// Create a JSON-RPC error
    pub fn jsonrpc(message: impl Into<String>) -> Self {
        Self::JsonRpc {
            message: message.into(),
        }
    }

    /// Create an MCP protocol error
    pub fn mcp(message: impl Into<String>) -> Self {
        Self::Mcp {
            message: message.into(),
        }
    }

    /// Create a transport error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    /// Create an invalid message error
    pub fn invalid_message(message: impl Into<String>) -> Self {
        Self::InvalidMessage {
            message: message.into(),
        }
    }
}

// Convenience constructors for McpError
impl McpError {
    /// Create a version mismatch error
    pub fn version_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::VersionMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create an unsupported capability error
    pub fn unsupported_capability(capability: impl Into<String>) -> Self {
        Self::UnsupportedCapability {
            capability: capability.into(),
        }
    }

    /// Create a resource not found error
    pub fn resource_not_found(uri: impl Into<String>) -> Self {
        Self::ResourceNotFound { uri: uri.into() }
    }

    /// Create an authorization failed error
    pub fn authorization_failed(reason: impl Into<String>) -> Self {
        Self::AuthorizationFailed {
            reason: reason.into(),
        }
    }

    /// Create an invalid URI error
    pub fn invalid_uri(uri: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidUri {
            uri: uri.into(),
            reason: reason.into(),
        }
    }

    /// Create a request timeout error
    pub fn request_timeout(timeout_ms: u64) -> Self {
        Self::RequestTimeout { timeout_ms }
    }
}

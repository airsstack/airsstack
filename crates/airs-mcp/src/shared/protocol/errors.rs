//! MCP Protocol Error Types
//!
//! This module defines error types specific to the MCP protocol layer,
//! extending the existing structured error system with protocol-specific variants.

use thiserror::Error;

/// Errors that can occur during MCP protocol operations
///
/// These errors represent protocol-level issues such as invalid message formats,
/// capability negotiation failures, or validation errors in protocol-specific types.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::{ProtocolError, Uri, ProtocolVersion};
///
/// // Invalid URI construction
/// let result = Uri::new("invalid-uri");
/// assert!(matches!(result, Err(ProtocolError::InvalidUri(_))));
///
/// // Invalid protocol version
/// let result = ProtocolVersion::new("invalid-version");
/// assert!(matches!(result, Err(ProtocolError::InvalidProtocolVersion(_))));
/// ```
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// Invalid protocol version format (must be YYYY-MM-DD)
    #[error("Invalid protocol version: {0}")]
    InvalidProtocolVersion(String),

    /// Invalid URI format or structure
    #[error("Invalid URI: {0}")]
    InvalidUri(String),

    /// Invalid MIME type format
    #[error("Invalid MIME type: {0}")]
    InvalidMimeType(String),

    /// Invalid base64 encoded data
    #[error("Invalid base64 data")]
    InvalidBase64Data,

    /// Capability negotiation failed during initialization
    #[error("Capability negotiation failed: {0}")]
    CapabilityNegotiationFailed(String),

    /// Unsupported protocol version requested
    #[error("Unsupported protocol version: {0}")]
    UnsupportedProtocolVersion(String),

    /// Missing required capability for operation
    #[error("Missing required capability: {0}")]
    MissingCapability(String),

    /// Invalid resource template format
    #[error("Invalid resource template: {0}")]
    InvalidResourceTemplate(String),

    /// Tool execution safety violation
    #[error("Tool execution safety violation: {0}")]
    ToolSafetyViolation(String),
}

/// Result type for protocol operations
pub type ProtocolResult<T> = Result<T, ProtocolError>;

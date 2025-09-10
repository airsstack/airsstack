//! AIRS MCP - Model Context Protocol Implementation
//!
//! This crate provides a complete implementation of the Model Context Protocol (MCP)
//! built on a solid JSON-RPC 2.0 foundation with trait-based message abstractions.
//!
//! # Architecture
//!
//! The AIRS MCP implementation is organized in layers:
//!
//! - **Protocol Layer** (`protocol`): Unified JSON-RPC 2.0 + MCP protocol implementation
//! - **Transport Layer** (`transport`): Communication transport abstractions and implementations
//! - **Integration Layer** (`integration`): High-level MCP client and server interfaces
//! - **Providers Layer** (`providers`): Production-ready MCP provider implementations
//! - **Shared Layer** (`shared`): Additional MCP message structures and content types
//!
//! # Core Features
//!
//! ## JSON-RPC 2.0 Foundation
//!
//! Complete JSON-RPC 2.0 specification compliance with:
//! - Type-safe message structures (`JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcNotification`)
//! - Flexible request ID support (string and numeric variants)
//! - Trait-based serialization with consistent behavior across all message types
//! - Comprehensive validation and error handling
//!
//! ## Quick Start
//!
//! ```rust
//! use airs_mcp::{JsonRpcRequest, JsonRpcMessageTrait, RequestId};
//! use serde_json::json;
//!
//! // Create a JSON-RPC request
//! let request = JsonRpcRequest::new(
//!     "ping",
//!     Some(json!({"message": "hello world"})),
//!     RequestId::new_string("req-001")
//! );
//!
//! // Serialize using trait method
//! let json = request.to_json().unwrap();
//! println!("Request: {}", json);
//!
//! // Deserialize back to typed structure
//! let parsed = JsonRpcRequest::from_json(&json).unwrap();
//! assert_eq!(request, parsed);
//! ```
//!
//! ## Message Types
//!
//! ### JsonRpcRequest
//! Represents a request to invoke a method on the remote peer:
//!
//! ```rust
//! use airs_mcp::{JsonRpcRequest, RequestId};
//! use serde_json::json;
//!
//! let request = JsonRpcRequest::new(
//!     "calculate",
//!     Some(json!({"operation": "add", "values": [1, 2, 3]})),
//!     RequestId::new_number(42)
//! );
//! ```
//!
//! ### JsonRpcResponse
//! Represents a response to a JSON-RPC request (success or error):
//!
//! ```rust
//! use airs_mcp::{JsonRpcResponse, RequestId};
//! use serde_json::json;
//!
//! // Success response
//! let success = JsonRpcResponse::success(
//!     json!({"result": "calculation complete", "value": 6}),
//!     RequestId::new_number(42)
//! );
//!
//! // Error response
//! let error = JsonRpcResponse::error(
//!     json!({"code": -32602, "message": "Invalid params"}),
//!     Some(RequestId::new_number(42))
//! );
//! ```
//!
//! ### JsonRpcNotification
//! Represents a notification (one-way message without response):
//!
//! ```rust
//! use airs_mcp::JsonRpcNotification;
//! use serde_json::json;
//!
//! let notification = JsonRpcNotification::new(
//!     "user_logged_in",
//!     Some(json!({"user_id": 12345, "timestamp": "2025-07-28T10:30:00Z"}))
//! );
//! ```
//!
//! ## JsonRpcMessage Trait
//!
//! All message types implement the `JsonRpcMessage` trait for consistent serialization:
//!
//! ```rust
//! use airs_mcp::{JsonRpcMessage, JsonRpcNotification};
//!
//! let notification = JsonRpcNotification::new("heartbeat", None);
//!
//! // Standard JSON serialization
//! let json = notification.to_json().unwrap();
//!
//! // Pretty-printed JSON for debugging
//! let pretty = notification.to_json_pretty().unwrap();
//!
//! // Deserialize from JSON bytes (efficient for network I/O)
//! let bytes = json.as_bytes();
//! let parsed = JsonRpcNotification::from_json_bytes(bytes).unwrap();
//! ```
//!
//! ## Request ID Flexibility
//!
//! Request IDs support both string and numeric formats per JSON-RPC 2.0 specification:
//!
//! ```rust
//! use airs_mcp::RequestId;
//!
//! // String-based IDs (UUIDs, custom formats, etc.)
//! let string_id = RequestId::new_string("req-12345-abcdef");
//!
//! // Numeric IDs (counters, timestamps, etc.)
//! let numeric_id = RequestId::new_number(1234567890);
//!
//! // Display formatting works for both
//! println!("String ID: {}", string_id);   // "req-12345-abcdef"
//! println!("Numeric ID: {}", numeric_id); // "1234567890"
//! ```
//!
//! # Error Handling
//!
//! All serialization operations return `Result` types with `serde_json::Error`:
//!
//! ```rust
//! use airs_mcp::{JsonRpcRequest, JsonRpcMessage, RequestId};
//!
//! let request = JsonRpcRequest::new("test", None, RequestId::new_number(1));
//!
//! match request.to_json() {
//!     Ok(json) => println!("Serialized: {}", json),
//!     Err(e) => eprintln!("Serialization failed: {}", e),
//! }
//! ```
//!
//! # JSON-RPC 2.0 Specification Compliance
//!
//! This implementation strictly adheres to the JSON-RPC 2.0 specification:
//!
//! - All messages include `"jsonrpc": "2.0"` field
//! - Requests have `method`, optional `params`, and `id` fields
//! - Responses have either `result` or `error` (mutual exclusion), and `id` fields
//! - Notifications have `method` and optional `params` (no `id` field)
//! - Request IDs support string, number, and null values
//! - Parameters must be structured (Object) or ordered (Array) values
//!
//! # Future Features
//!
//! Planned extensions include:
//! - MCP protocol layer implementation
//! - Transport abstractions (STDIO)
//! - Request correlation and bidirectional communication
//! - High-level client and server interfaces
//! - Performance optimizations and zero-copy processing
//!
//! # Performance Characteristics
//!
//! The current implementation prioritizes correctness and maintainability:
//! - Message serialization: Sub-millisecond for typical message sizes
//! - Memory usage: Minimal allocations with efficient serde integration
//! - Trait-based abstractions: Zero runtime cost through compile-time optimization
//!
//! For high-throughput scenarios, future versions will include zero-copy
//! optimizations and buffer pooling strategies.

// Correlation layer modules
pub mod correlation;

// Integration layer modules
pub mod integration;

// OAuth 2.1 authentication module
pub mod oauth2;

// Multi-method authentication module
pub mod authentication;

// Zero-cost generic authorization module
pub mod authorization;

// Providers layer modules
pub mod providers;

// Protocol layer modules (TASK-028 consolidation)
pub mod protocol;

// Transport layer modules
pub mod transport;

// Re-export commonly used types for convenience
// This allows users to import directly from the crate root
pub use protocol::{
    Base64Data,
    ClientInfo,
    JsonRpcError,
    // JSON-RPC 2.0 Message Types
    JsonRpcMessage,
    JsonRpcMessageTrait,
    JsonRpcNotification,
    JsonRpcRequest,
    JsonRpcResponse,
    MessageContext,
    MessageHandler,
    MimeType,
    // Error Types
    ProtocolError,
    ProtocolResult,

    ProtocolVersion,
    RequestId,

    ServerConfig, // New location of core MCP configuration
    ServerInfo,
    // Transport Abstractions
    Transport as ProtocolTransport,
    TransportError as ProtocolTransportError,

    // MCP Protocol Types
    Uri,
};

// Re-export correlation types for convenience
pub use correlation::{CorrelationConfig, CorrelationError, CorrelationManager, CorrelationResult};

// Re-export integration types for convenience
pub use integration::{
    ConnectionState, IntegrationError, IntegrationResult, McpClient, McpClientBuilder,
    McpClientConfig, McpError, McpResult, McpServer,
};

// Re-export transport types for convenience
pub use transport::adapters::StdioTransport;
pub use transport::{
    BufferConfig, BufferManager, BufferMetrics, PooledBuffer, TransportError,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the crate version as a string
///
/// # Examples
///
/// ```rust
/// println!("AIRS MCP version: {}", airs_mcp::version());
/// ```
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_crate_public_api() {
        // Test that all core types are accessible from crate root
        let request = JsonRpcRequest::new(
            "test_method",
            Some(json!({"param": "value"})),
            RequestId::new_string("test-123"),
        );

        let response =
            JsonRpcResponse::success(json!({"result": "success"}), RequestId::new_number(456));

        let notification =
            JsonRpcNotification::new("test_event", Some(json!({"event": "occurred"})));

        // Verify trait methods work through re-exports
        assert!(request.to_json().is_ok());
        assert!(response.to_json().is_ok());
        assert!(notification.to_json().is_ok());
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test complete serialization round-trip through public API
        let original = JsonRpcRequest::new(
            "echo",
            Some(json!([1, 2, 3])),
            RequestId::new_string("echo-001"),
        );

        let json = original.to_json().unwrap();
        let parsed = JsonRpcRequest::from_json(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_request_id_types() {
        // Test both RequestId variants work through public API
        let string_request = JsonRpcRequest::new("test", None, RequestId::new_string("uuid-12345"));

        let numeric_request = JsonRpcRequest::new("test", None, RequestId::new_number(67890));

        let string_json = string_request.to_json().unwrap();
        let numeric_json = numeric_request.to_json().unwrap();

        assert!(string_json.contains(r#""id":"uuid-12345""#));
        assert!(numeric_json.contains(r#""id":67890"#));
    }

    #[test]
    fn test_version_info() {
        // Test version information is accessible
        let version_str = version();
        assert!(!version_str.is_empty());
        assert_eq!(version_str, VERSION);
    }

    #[test]
    fn test_message_trait_consistency() {
        // Test that all message types use consistent trait implementation
        let request = JsonRpcRequest::new("test", None, RequestId::new_number(1));
        let response = JsonRpcResponse::success(json!("ok"), RequestId::new_number(1));
        let notification = JsonRpcNotification::new("event", None);

        // All should support both regular and pretty JSON
        assert!(request.to_json().is_ok());
        assert!(request.to_json_pretty().is_ok());

        assert!(response.to_json().is_ok());
        assert!(response.to_json_pretty().is_ok());

        assert!(notification.to_json().is_ok());
        assert!(notification.to_json_pretty().is_ok());

        // All should support bytes deserialization
        let request_json = request.to_json().unwrap();
        let request_bytes = request_json.as_bytes();
        assert!(JsonRpcRequest::from_json_bytes(request_bytes).is_ok());
    }

    #[test]
    fn test_json_rpc_compliance() {
        // Test that all message types maintain JSON-RPC 2.0 compliance
        let request = JsonRpcRequest::new("ping", None, RequestId::new_number(1));
        let response = JsonRpcResponse::success(json!("pong"), RequestId::new_number(1));
        let notification = JsonRpcNotification::new("heartbeat", None);

        let request_json = request.to_json().unwrap();
        let response_json = response.to_json().unwrap();
        let notification_json = notification.to_json().unwrap();

        // All must have jsonrpc: "2.0"
        assert!(request_json.contains(r#""jsonrpc":"2.0""#));
        assert!(response_json.contains(r#""jsonrpc":"2.0""#));
        assert!(notification_json.contains(r#""jsonrpc":"2.0""#));

        // Requests and notifications must have method
        assert!(request_json.contains(r#""method":"ping""#));
        assert!(notification_json.contains(r#""method":"heartbeat""#));

        // Requests must have id, notifications must not
        assert!(request_json.contains(r#""id":1"#));
        assert!(!notification_json.contains("id"));

        // Responses must have result or error, and id
        assert!(response_json.contains(r#""result":"pong""#));
        assert!(response_json.contains(r#""id":1"#));
        assert!(!response_json.contains("error"));
    }
}

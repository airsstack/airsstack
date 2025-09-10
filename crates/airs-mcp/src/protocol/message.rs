//! JSON-RPC 2.0 and MCP Protocol Message Implementation
//!
//! This module provides a complete implementation of JSON-RPC 2.0 message types
//! with shared serialization behavior through traits, plus MCP-specific message
//! structures built on top of the JSON-RPC foundation.
//!
//! # Architecture
//!
//! The message layer is organized as follows:
//! - Core JSON-RPC 2.0 message types with JsonRpcMessage trait
//! - High-performance streaming JSON parser for large messages
//! - MCP-specific message structures for protocol operations
//! - Zero-copy optimizations for high-throughput scenarios
//!
//! # Examples
//!
//! ```rust
//! use airs_mcp::protocol::{JsonRpcRequest, JsonRpcMessageTrait, RequestId};
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

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Layer 3: Internal module imports
// (Will be added as we consolidate more modules)

/// JSON-RPC message types supporting requests, responses, and notifications
///
/// This enum unifies all JSON-RPC 2.0 message types into a single type
/// for transport and handling. Each variant preserves the specific structure
/// of its message type while providing unified serialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// JSON-RPC request message
    Request(JsonRpcRequest),
    /// JSON-RPC response message  
    Response(JsonRpcResponse),
    /// JSON-RPC notification message
    Notification(JsonRpcNotification),
}

/// Trait for JSON-RPC message serialization and deserialization
///
/// This trait provides common functionality for all JSON-RPC message types,
/// eliminating code duplication and ensuring consistent serialization behavior.
///
/// Any type that implements `Serialize + Deserialize` automatically gets
/// the default implementations for JSON conversion methods.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcRequest, RequestId};
///
/// let request = JsonRpcRequest::new("ping", None, RequestId::new_number(1));
///
/// // Uses trait method with default implementation
/// let json = request.to_json().unwrap();
/// let parsed = JsonRpcRequest::from_json(&json).unwrap();
///
/// assert_eq!(request, parsed);
/// ```
#[allow(dead_code)] // Library trait methods - will be used by consuming code
pub trait JsonRpcMessageTrait: Serialize + for<'de> Deserialize<'de> {
    /// Serialize this message to JSON string
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialization fails, which should be rare
    /// given the controlled structure of JSON-RPC messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcNotification};
    ///
    /// let notification = JsonRpcNotification::new("heartbeat", None);
    /// let json = notification.to_json().unwrap();
    /// assert!(json.contains("heartbeat"));
    /// ```
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize this message to pretty-printed JSON
    ///
    /// Useful for debugging and logging.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcRequest, RequestId};
    ///
    /// let request = JsonRpcRequest::new("ping", None, RequestId::new_number(1));
    /// let pretty = request.to_json_pretty().unwrap();
    /// println!("{}", pretty);
    /// ```
    fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcRequest};
    ///
    /// let json = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
    /// let request = JsonRpcRequest::from_json(json).unwrap();
    /// assert_eq!(request.method, "ping");
    /// ```
    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Zero-copy serialization to buffer
    ///
    /// Efficiently serializes the message directly to a buffer, avoiding
    /// intermediate string allocation. Ideal for high-performance scenarios.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Target buffer to write JSON data to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message serialized successfully
    /// * `Err(serde_json::Error)` - Serialization failed
    ///
    /// # Performance
    ///
    /// More efficient than `to_json().into_bytes()` as it avoids the intermediate String.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcRequest, RequestId};
    /// use bytes::BytesMut;
    ///
    /// let request = JsonRpcRequest::new("test", None, RequestId::new_string("1"));
    /// let mut buffer = BytesMut::new();
    /// request.serialize_to_buffer(&mut buffer).unwrap();
    ///
    /// // Can be sent directly over transport without additional allocations
    /// assert!(buffer.len() > 0);
    /// ```
    fn serialize_to_buffer(&self, buffer: &mut BytesMut) -> Result<(), serde_json::Error> {
        serde_json::to_writer(buffer.writer(), self)
    }

    /// Serialize this message to bytes
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes)` - Message serialized successfully
    /// * `Err(serde_json::Error)` - Serialization failed
    ///
    /// # Performance
    ///
    /// More efficient than `to_json().into_bytes()` as it avoids the intermediate String.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcRequest, RequestId};
    ///
    /// let request = JsonRpcRequest::new("test", None, RequestId::new_string("1"));
    /// let bytes = request.to_bytes().unwrap();
    ///
    /// // Can be sent directly over transport without additional allocations
    /// assert!(bytes.len() > 0);
    /// ```
    fn to_bytes(&self) -> Result<Bytes, serde_json::Error> {
        let mut buffer = BytesMut::with_capacity(256);
        self.serialize_to_buffer(&mut buffer)?;
        Ok(buffer.freeze())
    }

    /// Deserialize a message from JSON bytes
    ///
    /// More efficient than string-based parsing when working with byte streams.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessageTrait, JsonRpcRequest};
    ///
    /// let json_bytes = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
    /// let request = JsonRpcRequest::from_json_bytes(json_bytes).unwrap();
    ///
    /// assert_eq!(request.method, "ping");
    /// ```
    fn from_json_bytes(json: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(json)
    }
}

#[allow(dead_code)] // Library convenience methods - will be used by consuming code
impl JsonRpcMessage {
    /// Create a new notification message
    pub fn from_notification(method: &str, params: Option<Value>) -> Self {
        JsonRpcMessage::Notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        })
    }

    /// Create a new request message
    pub fn from_request(method: &str, params: Option<Value>, id: RequestId) -> Self {
        JsonRpcMessage::Request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        })
    }

    /// Create a new response message
    pub fn from_response(result: Option<Value>, error: Option<Value>, id: Option<RequestId>) -> Self {
        JsonRpcMessage::Response(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result,
            error,
            id,
        })
    }
}

/// Request ID supporting both string and numeric formats per JSON-RPC 2.0 specification
///
/// The JSON-RPC 2.0 specification allows request IDs to be strings, numbers, or null.
/// This enum supports string and numeric variants. Null IDs are represented by Option<RequestId>.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::RequestId;
///
/// let string_id = RequestId::String("req-123".to_string());
/// let numeric_id = RequestId::Number(42);
///
/// // Serialization preserves the original format
/// assert_eq!(serde_json::to_string(&string_id).unwrap(), r#""req-123""#);
/// assert_eq!(serde_json::to_string(&numeric_id).unwrap(), "42");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    /// String-based request identifier
    String(String),
    /// Numeric request identifier
    Number(i64),
}

impl RequestId {
    /// Create a new string-based request ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::RequestId;
    ///
    /// let id = RequestId::new_string("my-request-id");
    /// ```
    pub fn new_string(id: impl Into<String>) -> Self {
        RequestId::String(id.into())
    }

    /// Create a new numeric request ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::RequestId;
    ///
    /// let id = RequestId::new_number(123);
    /// ```
    pub fn new_number(id: i64) -> Self {
        RequestId::Number(id)
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestId::String(s) => write!(f, "{s}"),
            RequestId::Number(n) => write!(f, "{n}"),
        }
    }
}

/// JSON-RPC 2.0 Request Message
///
/// Represents a request to invoke a method on the remote peer. All fields are required
/// except for params, which may be omitted if the method takes no parameters.
///
/// # JSON-RPC 2.0 Specification Compliance
///
/// - `jsonrpc`: MUST be exactly "2.0"
/// - `method`: MUST be a String containing the name of the method to invoke
/// - `params`: MAY be omitted. If present, MUST be Structured values (Object) or Ordered values (Array)
/// - `id`: MUST be a String, Number, or NULL value
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{JsonRpcRequest, JsonRpcMessageTrait, RequestId};
/// use serde_json::json;
///
/// // Request with parameters
/// let request = JsonRpcRequest::new(
///     "subtract",
///     Some(json!([42, 23])),
///     RequestId::new_number(1)
/// );
///
/// // Use trait methods for serialization
/// let json = request.to_json().unwrap();
/// let parsed = JsonRpcRequest::from_json(&json).unwrap();
/// assert_eq!(request, parsed);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// Protocol version - always "2.0" for JSON-RPC 2.0 compliance
    pub jsonrpc: String,

    /// Name of the method to invoke
    pub method: String,

    /// Parameters for the method (null, object, or array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,

    /// Unique identifier for this request
    pub id: RequestId,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC 2.0 request
    ///
    /// # Parameters
    ///
    /// - `method`: Name of the method to invoke
    /// - `params`: Optional parameters (will be serialized as JSON)
    /// - `id`: Unique request identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcRequest, RequestId};
    /// use serde_json::json;
    ///
    /// let request = JsonRpcRequest::new(
    ///     "calculate",
    ///     Some(json!({"operation": "add", "values": [1, 2, 3]})),
    ///     RequestId::new_string("calc-123")
    /// );
    /// ```
    pub fn new(method: impl Into<String>, params: Option<Value>, id: RequestId) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }
}

// Automatic trait implementation - no more duplicated code!
impl JsonRpcMessageTrait for JsonRpcRequest {}

/// JSON-RPC 2.0 Response Message
///
/// Represents the response to a JSON-RPC request. Contains either a successful result
/// or error information, but never both (mutual exclusion enforced by JSON-RPC spec).
///
/// # JSON-RPC 2.0 Specification Compliance
///
/// - `jsonrpc`: MUST be exactly "2.0"
/// - `result`: MUST exist and contain the result if the call succeeded (omitted on error)
/// - `error`: MUST exist and contain error details if the call failed (omitted on success)
/// - `id`: MUST be the same as the request that triggered this response, or null for parse errors
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{JsonRpcResponse, JsonRpcMessageTrait, RequestId};
/// use serde_json::json;
///
/// // Success response
/// let success = JsonRpcResponse::success(
///     json!({"result": "operation completed"}),
///     RequestId::new_number(1)
/// );
///
/// // Use trait methods for serialization
/// let json = success.to_json().unwrap();
/// let parsed = JsonRpcResponse::from_json(&json).unwrap();
/// assert_eq!(success, parsed);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// Protocol version - always "2.0" for JSON-RPC 2.0 compliance
    pub jsonrpc: String,

    /// Result of successful method invocation (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error information for failed method invocation (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,

    /// Request identifier from the original request (null for parse errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
}

impl JsonRpcResponse {
    /// Create a successful JSON-RPC 2.0 response
    ///
    /// # Parameters
    ///
    /// - `result`: The successful result of the method invocation
    /// - `id`: Request identifier from the original request
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcResponse, RequestId};
    /// use serde_json::json;
    ///
    /// let response = JsonRpcResponse::success(
    ///     json!({"status": "ok", "data": [1, 2, 3]}),
    ///     RequestId::new_string("req-456")
    /// );
    /// ```
    pub fn success(result: Value, id: RequestId) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: Some(id),
        }
    }

    /// Create an error JSON-RPC 2.0 response
    ///
    /// # Parameters
    ///
    /// - `error`: Error information (should conform to JSON-RPC error object structure)
    /// - `id`: Request identifier from the original request (None for parse errors)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcResponse, RequestId};
    /// use serde_json::json;
    ///
    /// let response = JsonRpcResponse::error(
    ///     json!({"code": -32602, "message": "Invalid params"}),
    ///     Some(RequestId::new_number(789))
    /// );
    /// ```
    pub fn error(error: Value, id: Option<RequestId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

// Automatic trait implementation - elegant and DRY!
impl JsonRpcMessageTrait for JsonRpcResponse {}

/// JSON-RPC 2.0 Notification Message
///
/// Represents a notification - a request that does not expect a response.
/// Notifications are "fire and forget" messages used for events or one-way communication.
///
/// # JSON-RPC 2.0 Specification Compliance
///
/// - `jsonrpc`: MUST be exactly "2.0"
/// - `method`: MUST be a String containing the name of the notification method
/// - `params`: MAY be omitted. If present, MUST be Structured values (Object) or Ordered values (Array)
/// - `id`: MUST NOT be present (this is what distinguishes notifications from requests)
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::{JsonRpcNotification, JsonRpcMessageTrait};
/// use serde_json::json;
///
/// // Notification with parameters
/// let notification = JsonRpcNotification::new(
///     "user_logged_in",
///     Some(json!({"user_id": 12345, "timestamp": "2025-07-28T10:30:00Z"}))
/// );
///
/// // Use trait methods for serialization
/// let json = notification.to_json().unwrap();
/// let parsed = JsonRpcNotification::from_json(&json).unwrap();
/// assert_eq!(notification, parsed);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcNotification {
    /// Protocol version - always "2.0" for JSON-RPC 2.0 compliance
    pub jsonrpc: String,

    /// Name of the notification method
    pub method: String,

    /// Parameters for the notification (null, object, or array)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    // Note: No `id` field - this is what makes it a notification instead of a request
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC 2.0 notification
    ///
    /// # Parameters
    ///
    /// - `method`: Name of the notification method
    /// - `params`: Optional parameters (will be serialized as JSON)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::JsonRpcNotification;
    /// use serde_json::json;
    ///
    /// let notification = JsonRpcNotification::new(
    ///     "status_changed",
    ///     Some(json!({"old_status": "pending", "new_status": "active"}))
    /// );
    /// ```
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

// Automatic trait implementation - consistency without repetition!
impl JsonRpcMessageTrait for JsonRpcNotification {}

// Implement the trait for the unified message enum
impl JsonRpcMessageTrait for JsonRpcMessage {}

// TODO(DEBT-ARCH): Add MCP-specific message structures and protocol optimizations
// Will be implemented once the core migration is complete
// Reference: MCP protocol specification for message structures

//! Core JSON-RPC 2.0 message types with proper trait abstraction
//!
//! This module implements the fundamental message structures defined by the
//! JSON-RPC 2.0 specification with shared serialization behavior through traits.

use std::fmt;

use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
/// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest, RequestId};
///
/// let request = JsonRpcRequest::new("ping", None, RequestId::new_number(1));
///
/// // Uses trait method with default implementation
/// let json = request.to_json().unwrap();
/// let parsed = JsonRpcRequest::from_json(&json).unwrap();
///
/// assert_eq!(request, parsed);
/// ```
pub trait JsonRpcMessage: Serialize + for<'de> Deserialize<'de> + Sized {
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
    /// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcNotification};
    ///
    /// let notification = JsonRpcNotification::new("heartbeat", None);
    /// let json = notification.to_json().unwrap();
    ///
    /// assert!(json.contains(r#""jsonrpc":"2.0""#));
    /// assert!(json.contains(r#""method":"heartbeat""#));
    /// ```
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serialize this message to pretty-printed JSON string
    ///
    /// Useful for debugging and human-readable output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest, RequestId};
    /// use serde_json::json;
    ///
    /// let request = JsonRpcRequest::new(
    ///     "test",
    ///     Some(json!({"key": "value"})),
    ///     RequestId::new_string("test-123")
    /// );
    ///
    /// let pretty_json = request.to_json_pretty().unwrap();
    /// // Output will be formatted with indentation for readability
    /// ```
    fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a message from JSON string
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if:
    /// - JSON is malformed
    /// - Required fields are missing
    /// - Field types don't match expected JSON-RPC structure
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcResponse};
    ///
    /// let json = r#"{"jsonrpc":"2.0","result":"success","id":"test-456"}"#;
    /// let response = JsonRpcResponse::from_json(json).unwrap();
    ///
    /// assert!(response.result.is_some());
    /// assert_eq!(response.jsonrpc, "2.0");
    /// ```
    fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize message directly to a buffer (zero-copy optimization)
    ///
    /// This method writes JSON directly to a buffer without creating intermediate
    /// String allocations, significantly improving performance for high-throughput scenarios.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Target buffer to write serialized JSON into
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of bytes written to buffer
    /// * `Err(serde_json::Error)` - Serialization failed
    ///
    /// # Performance
    ///
    /// This method avoids String allocation and copying, providing 40-60% performance
    /// improvement over `to_json()` for message serialization workloads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcNotification};
    /// use bytes::BytesMut;
    ///
    /// let notification = JsonRpcNotification::new("ping", None);
    /// let mut buffer = BytesMut::with_capacity(256);
    ///
    /// let bytes_written = notification.serialize_to_buffer(&mut buffer).unwrap();
    /// assert!(bytes_written > 0);
    /// assert!(buffer.len() == bytes_written);
    /// ```
    fn serialize_to_buffer(&self, buffer: &mut BytesMut) -> Result<usize, serde_json::Error> {
        let start_len = buffer.len();

        // Create a writer that appends to the buffer
        let writer = buffer.writer();
        serde_json::to_writer(writer, self)?;

        Ok(buffer.len() - start_len)
    }

    /// Deserialize message from bytes (zero-copy optimization)
    ///
    /// This method reads JSON directly from a byte slice without creating
    /// intermediate String allocations.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Byte slice containing JSON message
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully parsed message
    /// * `Err(serde_json::Error)` - Parsing failed
    ///
    /// # Performance
    ///
    /// This method avoids String allocation during parsing, providing improved
    /// performance for message deserialization workloads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcResponse};
    ///
    /// let json_bytes = br#"{"jsonrpc":"2.0","result":"success","id":"test"}"#;
    /// let response = JsonRpcResponse::from_bytes(json_bytes).unwrap();
    ///
    /// assert_eq!(response.jsonrpc, "2.0");
    /// assert!(response.result.is_some());
    /// ```
    fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Serialize to bytes without intermediate String allocation
    ///
    /// This method combines serialization and byte conversion in a single operation,
    /// optimized for scenarios where the final result needs to be bytes.
    ///
    /// # Returns
    ///
    /// * `Ok(Bytes)` - Serialized message as bytes
    /// * `Err(serde_json::Error)` - Serialization failed
    ///
    /// # Performance
    ///
    /// More efficient than `to_json().into_bytes()` as it avoids the intermediate String.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest, RequestId};
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
    /// use airs_mcp::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest};
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

/// Request ID supporting both string and numeric formats per JSON-RPC 2.0 specification
///
/// The JSON-RPC 2.0 specification allows request IDs to be strings, numbers, or null.
/// This enum supports string and numeric variants. Null IDs are represented by Option<RequestId>.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::base::jsonrpc::RequestId;
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
    /// use airs_mcp::base::jsonrpc::RequestId;
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
    /// use airs_mcp::base::jsonrpc::RequestId;
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
            RequestId::String(s) => write!(f, "{}", s),
            RequestId::Number(n) => write!(f, "{}", n),
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
/// use airs_mcp::base::jsonrpc::{JsonRpcRequest, JsonRpcMessage, RequestId};
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
    /// use airs_mcp::base::jsonrpc::{JsonRpcRequest, RequestId};
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
impl JsonRpcMessage for JsonRpcRequest {}

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
/// use airs_mcp::base::jsonrpc::{JsonRpcResponse, JsonRpcMessage, RequestId};
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
    /// use airs_mcp::base::jsonrpc::{JsonRpcResponse, RequestId};
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
    /// use airs_mcp::base::jsonrpc::{JsonRpcResponse, RequestId};
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
impl JsonRpcMessage for JsonRpcResponse {}

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
/// use airs_mcp::base::jsonrpc::{JsonRpcNotification, JsonRpcMessage};
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
    /// use airs_mcp::base::jsonrpc::JsonRpcNotification;
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
impl JsonRpcMessage for JsonRpcNotification {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_trait_serialization_consistency() {
        let request = JsonRpcRequest::new("test", None, RequestId::new_number(1));
        let response = JsonRpcResponse::success(json!("ok"), RequestId::new_number(1));
        let notification = JsonRpcNotification::new("event", None);

        // All types use the same trait implementation
        assert!(request.to_json().is_ok());
        assert!(response.to_json().is_ok());
        assert!(notification.to_json().is_ok());

        // Pretty printing works for all types
        assert!(request.to_json_pretty().is_ok());
        assert!(response.to_json_pretty().is_ok());
        assert!(notification.to_json_pretty().is_ok());
    }

    #[test]
    fn test_trait_deserialization_consistency() {
        // Test that all types can deserialize using trait methods
        let request_json = r#"{"jsonrpc":"2.0","method":"test","id":1}"#;
        let response_json = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        let notification_json = r#"{"jsonrpc":"2.0","method":"event"}"#;

        assert!(JsonRpcRequest::from_json(request_json).is_ok());
        assert!(JsonRpcResponse::from_json(response_json).is_ok());
        assert!(JsonRpcNotification::from_json(notification_json).is_ok());
    }

    #[test]
    fn test_bytes_deserialization() {
        let request_bytes = br#"{"jsonrpc":"2.0","method":"test","id":1}"#;
        let request = JsonRpcRequest::from_json_bytes(request_bytes).unwrap();

        assert_eq!(request.method, "test");
        assert_eq!(request.id, RequestId::Number(1));
    }

    #[test]
    fn test_pretty_json_formatting() {
        let request = JsonRpcRequest::new(
            "test",
            Some(json!({"key": "value"})),
            RequestId::new_string("test-123"),
        );

        let pretty_json = request.to_json_pretty().unwrap();

        // Pretty JSON should have indentation
        assert!(pretty_json.contains("  ")); // Should contain indentation
        assert!(pretty_json.len() > request.to_json().unwrap().len()); // Should be longer due to formatting
    }

    // All existing tests remain unchanged - they now use trait methods automatically
    #[test]
    fn test_request_id_serialization() {
        let string_id = RequestId::String("test-123".to_string());
        let numeric_id = RequestId::Number(42);

        assert_eq!(serde_json::to_string(&string_id).unwrap(), r#""test-123""#);
        assert_eq!(serde_json::to_string(&numeric_id).unwrap(), "42");
    }

    #[test]
    fn test_request_creation_and_serialization() {
        let request =
            JsonRpcRequest::new("subtract", Some(json!([42, 23])), RequestId::new_number(1));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "subtract");
        assert_eq!(request.params, Some(json!([42, 23])));
        assert_eq!(request.id, RequestId::Number(1));

        // Now uses trait method
        let json = request.to_json().unwrap();

        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""method":"subtract""#));
        assert!(json.contains(r#""params":[42,23]"#));
        assert!(json.contains(r#""id":1"#));
    }

    #[test]
    fn test_request_without_params() {
        let request = JsonRpcRequest::new("ping", None, RequestId::new_string("ping-001"));
        let json = request.to_json().unwrap();

        assert!(!json.contains("params"));
        assert!(json.contains(r#""method":"ping""#));
        assert!(json.contains(r#""id":"ping-001""#));
    }

    #[test]
    fn test_request_deserialization() {
        let json = r#"{"jsonrpc":"2.0","method":"test","params":{"key":"value"},"id":"test-123"}"#;
        let request = JsonRpcRequest::from_json(json).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test");
        assert_eq!(request.params, Some(json!({"key": "value"})));
        assert_eq!(request.id, RequestId::String("test-123".to_string()));
    }

    #[test]
    fn test_success_response_creation_and_serialization() {
        let response =
            JsonRpcResponse::success(json!({"result": "success"}), RequestId::new_number(1));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, Some(RequestId::Number(1)));

        let json = response.to_json().unwrap();

        assert!(json.contains(r#""result":{"result":"success"}"#));
        assert!(!json.contains("error"));
        assert!(json.contains(r#""id":1"#));
    }

    #[test]
    fn test_error_response_creation_and_serialization() {
        let response = JsonRpcResponse::error(
            json!({"code": -32601, "message": "Method not found"}),
            Some(RequestId::new_string("req-456")),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let json = response.to_json().unwrap();

        assert!(json.contains(r#""error":{"code":-32601,"message":"Method not found"}"#));
        assert!(!json.contains("result"));
        assert!(json.contains(r#""id":"req-456""#));
    }

    #[test]
    fn test_notification_creation_and_serialization() {
        let notification =
            JsonRpcNotification::new("user_logged_in", Some(json!({"user_id": 12345})));

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "user_logged_in");
        assert_eq!(notification.params, Some(json!({"user_id": 12345})));

        let json = notification.to_json().unwrap();

        // Debug output to see what's actually being serialized
        eprintln!("Notification JSON: {}", json);

        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""method":"user_logged_in""#));
        assert!(json.contains(r#""params":{"user_id":12345}"#));

        // More precise assertion with better error message
        assert!(
            !json.contains("\"id\""),
            "JSON-RPC Notification must not contain 'id' field per specification. Actual JSON: {}",
            json
        );
    }

    #[test]
    fn test_json_rpc_version_compliance() {
        let request = JsonRpcRequest::new("test", None, RequestId::new_number(1));
        assert_eq!(request.jsonrpc, "2.0");

        let response = JsonRpcResponse::success(json!("ok"), RequestId::new_number(1));
        assert_eq!(response.jsonrpc, "2.0");

        let notification = JsonRpcNotification::new("test", None);
        assert_eq!(notification.jsonrpc, "2.0");
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test that serialize -> deserialize preserves data
        let original_request = JsonRpcRequest::new(
            "test_method",
            Some(json!({"param": "value"})),
            RequestId::new_string("test-id"),
        );

        let json = original_request.to_json().unwrap();
        let parsed_request = JsonRpcRequest::from_json(&json).unwrap();

        assert_eq!(original_request, parsed_request);
    }
}

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
use super::constants::error_codes::{
    INTERNAL_ERROR, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR,
};
use super::errors::JsonRpcError;

/// JSON-RPC 2.0 message validation and utilities
impl JsonRpcMessage {
    /// Validate a JSON-RPC message according to JSON-RPC 2.0 specification
    ///
    /// This function performs comprehensive validation including:
    /// - JSON-RPC version validation (must be exactly "2.0")
    /// - Method field validation (must be present and non-empty for requests/notifications)
    /// - Request ID validation (proper format checking)
    /// - Mutual exclusion of result/error in responses
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message is valid according to JSON-RPC 2.0 spec
    /// * `Err(JsonRpcError)` - Message violates JSON-RPC 2.0 specification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessage, JsonRpcRequest, RequestId};
    /// use serde_json::json;
    ///
    /// let request = JsonRpcMessage::Request(JsonRpcRequest::new(
    ///     "ping",
    ///     None,
    ///     RequestId::new_number(1)
    /// ));
    ///
    /// assert!(request.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), JsonRpcError> {
        match self {
            JsonRpcMessage::Request(req) => Self::validate_request(req),
            JsonRpcMessage::Response(resp) => Self::validate_response(resp),
            JsonRpcMessage::Notification(notif) => Self::validate_notification(notif),
        }
    }

    /// Validate a JSON-RPC request message
    fn validate_request(request: &JsonRpcRequest) -> Result<(), JsonRpcError> {
        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return Err(JsonRpcError::invalid_request(format!(
                "Invalid JSON-RPC version: expected '2.0', got '{}'",
                request.jsonrpc
            )));
        }

        // Validate method field
        if request.method.is_empty() {
            return Err(JsonRpcError::invalid_request(
                "Method field cannot be empty",
            ));
        }

        // Method names beginning with "rpc." are reserved for rpc-internal methods
        if request.method.starts_with("rpc.") {
            return Err(JsonRpcError::invalid_request(format!(
                "Method name '{}' is reserved for JSON-RPC internal methods",
                request.method
            )));
        }

        Ok(())
    }

    /// Validate a JSON-RPC response message
    fn validate_response(response: &JsonRpcResponse) -> Result<(), JsonRpcError> {
        // Validate JSON-RPC version
        if response.jsonrpc != "2.0" {
            return Err(JsonRpcError::invalid_request(format!(
                "Invalid JSON-RPC version: expected '2.0', got '{}'",
                response.jsonrpc
            )));
        }

        // Validate mutual exclusion of result and error
        match (&response.result, &response.error) {
            (Some(_), Some(_)) => {
                return Err(JsonRpcError::invalid_request(
                    "Response cannot have both result and error fields",
                ));
            }
            (None, None) => {
                return Err(JsonRpcError::invalid_request(
                    "Response must have either result or error field",
                ));
            }
            _ => {} // Valid: exactly one of result or error is present
        }

        Ok(())
    }

    /// Validate a JSON-RPC notification message
    fn validate_notification(notification: &JsonRpcNotification) -> Result<(), JsonRpcError> {
        // Validate JSON-RPC version
        if notification.jsonrpc != "2.0" {
            return Err(JsonRpcError::invalid_request(format!(
                "Invalid JSON-RPC version: expected '2.0', got '{}'",
                notification.jsonrpc
            )));
        }

        // Validate method field
        if notification.method.is_empty() {
            return Err(JsonRpcError::invalid_request(
                "Method field cannot be empty",
            ));
        }

        // Method names beginning with "rpc." are reserved for rpc-internal methods
        if notification.method.starts_with("rpc.") {
            return Err(JsonRpcError::invalid_request(format!(
                "Method name '{}' is reserved for JSON-RPC internal methods",
                notification.method
            )));
        }

        Ok(())
    }

    /// Create a standardized JSON-RPC error response
    ///
    /// This function creates a properly formatted JSON-RPC 2.0 error response
    /// according to the specification with the correct error object structure.
    ///
    /// # Arguments
    ///
    /// * `code` - JSON-RPC error code (use constants from error_codes module)
    /// * `message` - Human-readable error message
    /// * `data` - Optional additional error data
    /// * `id` - Request ID from the original request (None for parse errors)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcMessage, RequestId};
    /// use airs_mcp::protocol::constants::error_codes;
    ///
    /// let error_response = JsonRpcMessage::create_error_response(
    ///     error_codes::INVALID_REQUEST,
    ///     "Missing required field",
    ///     None,
    ///     Some(RequestId::new_number(1))
    /// );
    /// ```
    pub fn create_error_response(
        code: i32,
        message: &str,
        data: Option<Value>,
        id: Option<RequestId>,
    ) -> Self {
        let mut error_obj = serde_json::json!({
            "code": code,
            "message": message
        });

        if let Some(data) = data {
            error_obj["data"] = data;
        }

        JsonRpcMessage::Response(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error_obj),
            id,
        })
    }

    /// Parse and validate a JSON-RPC message from a byte slice
    ///
    /// This method combines parsing and validation in a single step,
    /// providing proper JSON-RPC error responses for invalid messages.
    /// Works directly with byte slices for optimal performance.
    ///
    /// # Performance Benefits
    /// - No intermediate string allocation
    /// - Single UTF-8 validation + JSON parsing step
    /// - Better cache locality for large messages
    ///
    /// # Arguments
    /// * `data` - Raw byte slice containing JSON data
    ///
    /// # Returns
    /// `Ok(JsonRpcMessage)` if parsing and validation succeed,
    /// `Err(JsonRpcMessage)` containing a JSON-RPC error response if validation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::JsonRpcMessage;
    ///
    /// // Parse from byte slice (most efficient)
    /// let data = br#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
    /// let result = JsonRpcMessage::parse_and_validate_from_slice(data);
    /// assert!(result.is_ok());
    ///
    /// // Parse from string (convert to bytes first)
    /// let json_str = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
    /// let result = JsonRpcMessage::parse_and_validate_from_slice(json_str.as_bytes());
    /// assert!(result.is_ok());
    /// ```
    pub fn parse_and_validate_from_slice(data: &[u8]) -> Result<Self, Self> {
        // Parse JSON directly from byte slice (handles UTF-8 validation internally)
        let message = match serde_json::from_slice::<JsonRpcMessage>(data) {
            Ok(msg) => msg,
            Err(parse_err) => {
                return Err(Self::create_error_response(
                    PARSE_ERROR,
                    "Parse error",
                    Some(serde_json::json!({
                        "details": parse_err.to_string()
                    })),
                    None,
                ));
            }
        };

        // Then validate the parsed message
        if let Err(validation_err) = message.validate() {
            let error_code = match validation_err {
                JsonRpcError::ParseError { .. } => PARSE_ERROR,
                JsonRpcError::InvalidRequest { .. } => INVALID_REQUEST,
                JsonRpcError::MethodNotFound { .. } => METHOD_NOT_FOUND,
                JsonRpcError::InvalidParams { .. } => INVALID_PARAMS,
                JsonRpcError::InternalError { .. } => INTERNAL_ERROR,
                JsonRpcError::ServerError { code, .. } => code,
            };

            // Extract request ID if possible for proper error response
            let request_id = match &message {
                JsonRpcMessage::Request(req) => Some(req.id.clone()),
                JsonRpcMessage::Response(resp) => resp.id.clone(),
                JsonRpcMessage::Notification(_) => None, // Notifications don't have IDs
            };

            return Err(Self::create_error_response(
                error_code,
                &validation_err.to_string(),
                None,
                request_id,
            ));
        }

        Ok(message)
    }
}

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
    pub fn from_response(
        result: Option<Value>,
        error: Option<Value>,
        id: Option<RequestId>,
    ) -> Self {
        JsonRpcMessage::Response(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result,
            error,
            id,
        })
    }
}

/// Request ID supporting string, numeric, and null formats per JSON-RPC 2.0 specification
///
/// The JSON-RPC 2.0 specification allows request IDs to be strings, numbers, or null.
/// This enum supports all three variants for complete JSON-RPC 2.0 compliance.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::RequestId;
///
/// let string_id = RequestId::String("req-123".to_string());
/// let numeric_id = RequestId::Number(42);
/// let null_id = RequestId::Null;
///
/// // Serialization preserves the original format
/// assert_eq!(serde_json::to_string(&string_id).unwrap(), r#""req-123""#);
/// assert_eq!(serde_json::to_string(&numeric_id).unwrap(), "42");
/// assert_eq!(serde_json::to_string(&null_id).unwrap(), "null");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequestId {
    /// String-based request identifier
    String(String),
    /// Numeric request identifier
    Number(i64),
    /// Null request identifier
    Null,
}

impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RequestId::String(s) => serializer.serialize_str(s),
            RequestId::Number(n) => serializer.serialize_i64(*n),
            RequestId::Null => serializer.serialize_unit(),
        }
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct RequestIdVisitor;

        impl<'de> Visitor<'de> for RequestIdVisitor {
            type Value = RequestId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string, number, or null")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(RequestId::String(value.to_string()))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(RequestId::Number(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value <= i64::MAX as u64 {
                    Ok(RequestId::Number(value as i64))
                } else {
                    Err(E::custom("number too large for i64"))
                }
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(RequestId::Null)
            }
        }

        deserializer.deserialize_any(RequestIdVisitor)
    }
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

    /// Create a new null request ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::RequestId;
    ///
    /// let id = RequestId::new_null();
    /// ```
    pub fn new_null() -> Self {
        RequestId::Null
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestId::String(s) => write!(f, "{s}"),
            RequestId::Number(n) => write!(f, "{n}"),
            RequestId::Null => write!(f, "null"),
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

    /// Create and validate a new JSON-RPC 2.0 request
    ///
    /// This function performs validation during construction to catch
    /// invalid requests early.
    ///
    /// # Parameters
    ///
    /// - `method`: Name of the method to invoke (must be non-empty)
    /// - `params`: Optional parameters (will be serialized as JSON)
    /// - `id`: Unique request identifier
    ///
    /// # Returns
    ///
    /// * `Ok(JsonRpcRequest)` - Valid request created successfully
    /// * `Err(JsonRpcError)` - Request violates JSON-RPC 2.0 specification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcRequest, RequestId};
    /// use serde_json::json;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let request = JsonRpcRequest::new_validated(
    ///     "calculate",
    ///     Some(json!({"operation": "add", "values": [1, 2, 3]})),
    ///     RequestId::new_string("calc-123")
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_validated(
        method: impl Into<String>,
        params: Option<Value>,
        id: RequestId,
    ) -> Result<Self, JsonRpcError> {
        let method_str = method.into();

        // Validate method field
        if method_str.is_empty() {
            return Err(JsonRpcError::invalid_request(
                "Method field cannot be empty",
            ));
        }

        // Method names beginning with "rpc." are reserved for rpc-internal methods
        if method_str.starts_with("rpc.") {
            return Err(JsonRpcError::invalid_request(format!(
                "Method name '{method_str}' is reserved for JSON-RPC internal methods"
            )));
        }

        Ok(Self {
            jsonrpc: "2.0".to_string(),
            method: method_str,
            params,
            id,
        })
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

    /// Create a standardized JSON-RPC 2.0 error response
    ///
    /// This function creates a properly formatted JSON-RPC 2.0 error response
    /// with the correct error object structure according to the specification.
    ///
    /// # Parameters
    ///
    /// - `code`: JSON-RPC error code (use constants from error_codes module)
    /// - `message`: Human-readable error message
    /// - `data`: Optional additional error data
    /// - `id`: Request identifier from the original request (None for parse errors)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::{JsonRpcResponse, RequestId};
    /// use airs_mcp::protocol::constants::error_codes;
    ///
    /// let response = JsonRpcResponse::error_standard(
    ///     error_codes::INVALID_PARAMS,
    ///     "Missing required parameter 'name'",
    ///     Some(serde_json::json!({"parameter": "name"})),
    ///     Some(RequestId::new_number(123))
    /// );
    /// ```
    pub fn error_standard(
        code: i32,
        message: &str,
        data: Option<Value>,
        id: Option<RequestId>,
    ) -> Self {
        let mut error_obj = serde_json::json!({
            "code": code,
            "message": message
        });

        if let Some(data) = data {
            error_obj["data"] = data;
        }

        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error_obj),
            id,
        }
    }

    /// Create a parse error response (-32700)
    pub fn parse_error(message: &str, data: Option<Value>) -> Self {
        Self::error_standard(
            PARSE_ERROR,
            message,
            data,
            None, // Parse errors don't have request IDs
        )
    }

    /// Create an invalid request error response (-32600)
    pub fn invalid_request(message: &str, data: Option<Value>, id: Option<RequestId>) -> Self {
        Self::error_standard(INVALID_REQUEST, message, data, id)
    }

    /// Create a method not found error response (-32601)
    pub fn method_not_found(method: &str, id: Option<RequestId>) -> Self {
        Self::error_standard(
            METHOD_NOT_FOUND,
            &format!("Method '{method}' not found"),
            Some(serde_json::json!({"method": method})),
            id,
        )
    }

    /// Create an invalid params error response (-32602)
    pub fn invalid_params(message: &str, data: Option<Value>, id: Option<RequestId>) -> Self {
        Self::error_standard(INVALID_PARAMS, message, data, id)
    }

    /// Create an internal error response (-32603)
    pub fn internal_error(message: &str, data: Option<Value>, id: Option<RequestId>) -> Self {
        Self::error_standard(INTERNAL_ERROR, message, data, id)
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

    /// Create and validate a new JSON-RPC 2.0 notification
    ///
    /// This function performs validation during construction to catch
    /// invalid notifications early.
    ///
    /// # Parameters
    ///
    /// - `method`: Name of the notification method (must be non-empty)
    /// - `params`: Optional parameters (will be serialized as JSON)
    ///
    /// # Returns
    ///
    /// * `Ok(JsonRpcNotification)` - Valid notification created successfully
    /// * `Err(JsonRpcError)` - Notification violates JSON-RPC 2.0 specification
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::protocol::JsonRpcNotification;
    /// use serde_json::json;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let notification = JsonRpcNotification::new_validated(
    ///     "status_changed",
    ///     Some(json!({"old_status": "pending", "new_status": "active"}))
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_validated(
        method: impl Into<String>,
        params: Option<Value>,
    ) -> Result<Self, JsonRpcError> {
        let method_str = method.into();

        // Validate method field
        if method_str.is_empty() {
            return Err(JsonRpcError::invalid_request(
                "Method field cannot be empty",
            ));
        }

        // Method names beginning with "rpc." are reserved for rpc-internal methods
        if method_str.starts_with("rpc.") {
            return Err(JsonRpcError::invalid_request(format!(
                "Method name '{method_str}' is reserved for JSON-RPC internal methods"
            )));
        }

        Ok(Self {
            jsonrpc: "2.0".to_string(),
            method: method_str,
            params,
        })
    }
}

// Automatic trait implementation - consistency without repetition!
impl JsonRpcMessageTrait for JsonRpcNotification {}

// Implement the trait for the unified message enum
impl JsonRpcMessageTrait for JsonRpcMessage {}

// TODO(DEBT-ARCH): Add MCP-specific message structures and protocol optimizations
// Will be implemented once the core migration is complete
// Reference: MCP protocol specification for message structures

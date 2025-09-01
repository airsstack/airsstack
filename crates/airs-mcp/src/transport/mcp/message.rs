//! MCP JSON-RPC Message Types
//!
//! Core message types aligned with the official Model Context Protocol (MCP) specification.
//! These structures match the official MCP TypeScript/Python SDK patterns exactly.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Core JSON-RPC message type aligned with MCP specification
///
/// This structure matches the official MCP specification's JSON-RPC message format,
/// supporting all message types: requests, responses, notifications, and errors.
///
/// The design is intentionally flat to match the MCP TypeScript/Python SDK patterns
/// and avoid the complexity of our current trait-based message hierarchy.
///
/// # JSON-RPC 2.0 Compliance
///
/// All messages include the required `jsonrpc: "2.0"` field as per specification.
/// The `id` field distinguishes between requests/responses (present) and notifications (absent).
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::mcp::JsonRpcMessage;
/// use serde_json::json;
///
/// // Request message
/// let request = JsonRpcMessage {
///     jsonrpc: "2.0".to_string(),
///     id: Some(json!(1)),
///     method: Some("initialize".to_string()),
///     params: Some(json!({"protocolVersion": "2024-11-05"})),
///     result: None,
///     error: None,
/// };
///
/// // Response message  
/// let response = JsonRpcMessage {
///     jsonrpc: "2.0".to_string(),
///     id: Some(json!(1)),
///     method: None,
///     params: None,
///     result: Some(json!({"protocolVersion": "2024-11-05"})),
///     error: None,
/// };
///
/// // Notification message
/// let notification = JsonRpcMessage {
///     jsonrpc: "2.0".to_string(),
///     id: None,
///     method: Some("initialized".to_string()),
///     params: None,
///     result: None,
///     error: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcMessage {
    /// JSON-RPC protocol version (always "2.0")
    pub jsonrpc: String,

    /// Message ID for correlation (present for requests/responses, absent for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonValue>,

    /// Method name for requests and notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Parameters for requests and notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<JsonValue>,

    /// Result for successful responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,

    /// Error for failed responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcMessage {
    /// Create a new request message
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `params` - Optional parameters
    /// * `id` - Request ID for correlation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::JsonRpcMessage;
    /// use serde_json::json;
    ///
    /// let request = JsonRpcMessage::new_request(
    ///     "ping",
    ///     None,
    ///     json!("ping-123")
    /// );
    ///
    /// assert_eq!(request.method.unwrap(), "ping");
    /// assert_eq!(request.id.unwrap(), json!("ping-123"));
    /// assert!(request.result.is_none());
    /// assert!(request.error.is_none());
    /// ```
    pub fn new_request(
        method: impl Into<String>,
        params: Option<JsonValue>,
        id: JsonValue,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Create a new success response message
    ///
    /// # Arguments
    ///
    /// * `result` - Response result data
    /// * `id` - Request ID for correlation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::JsonRpcMessage;
    /// use serde_json::json;
    ///
    /// let response = JsonRpcMessage::new_response(
    ///     json!({"status": "ok"}),
    ///     json!(1)
    /// );
    ///
    /// assert!(response.result.is_some());
    /// assert!(response.method.is_none());
    /// assert!(response.error.is_none());
    /// ```
    pub fn new_response(result: JsonValue, id: JsonValue) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    /// Create a new error response message
    ///
    /// # Arguments
    ///
    /// * `error` - Error details
    /// * `id` - Request ID for correlation (or null for parse errors)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::{JsonRpcMessage, JsonRpcError};
    /// use serde_json::json;
    ///
    /// let error = JsonRpcError {
    ///     code: -32601,
    ///     message: "Method not found".to_string(),
    ///     data: None,
    /// };
    ///
    /// let response = JsonRpcMessage::new_error(error, json!(1));
    ///
    /// assert!(response.error.is_some());
    /// assert!(response.result.is_none());
    /// ```
    pub fn new_error(error: JsonRpcError, id: JsonValue) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }

    /// Create a new notification message
    ///
    /// # Arguments
    ///
    /// * `method` - JSON-RPC method name
    /// * `params` - Optional parameters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::transport::mcp::JsonRpcMessage;
    /// use serde_json::json;
    ///
    /// let notification = JsonRpcMessage::new_notification(
    ///     "initialized",
    ///     Some(json!({"timestamp": "2024-01-01T00:00:00Z"}))
    /// );
    ///
    /// assert_eq!(notification.method.unwrap(), "initialized");
    /// assert!(notification.id.is_none());
    /// ```
    pub fn new_notification(method: impl Into<String>, params: Option<JsonValue>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some(method.into()),
            params,
            result: None,
            error: None,
        }
    }

    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        self.method.is_some() && self.id.is_some()
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        self.id.is_some()
            && self.method.is_none()
            && (self.result.is_some() || self.error.is_some())
    }

    /// Check if this is a notification message
    pub fn is_notification(&self) -> bool {
        self.method.is_some() && self.id.is_none()
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Standard JSON-RPC error structure
///
/// Follows the JSON-RPC 2.0 specification for error responses.
/// Standard error codes are defined in the specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code (integer)
    pub code: i64,

    /// Human-readable error message
    pub message: String,

    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

impl JsonRpcError {
    /// Standard JSON-RPC error codes
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;

    /// Create a new JSON-RPC error
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new JSON-RPC error with additional data
    pub fn with_data(code: i64, message: impl Into<String>, data: JsonValue) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(Self::PARSE_ERROR, "Parse error")
    }

    /// Create an invalid request error
    pub fn invalid_request() -> Self {
        Self::new(Self::INVALID_REQUEST, "Invalid Request")
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::with_data(
            Self::METHOD_NOT_FOUND,
            "Method not found",
            serde_json::json!({"method": method}),
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(details: &str) -> Self {
        Self::with_data(
            Self::INVALID_PARAMS,
            "Invalid params",
            serde_json::json!({"details": details}),
        )
    }

    /// Create an internal error
    pub fn internal_error(details: &str) -> Self {
        Self::with_data(
            Self::INTERNAL_ERROR,
            "Internal error",
            serde_json::json!({"details": details}),
        )
    }
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jsonrpc_message_request() {
        let request = JsonRpcMessage::new_request(
            "initialize",
            Some(json!({"protocolVersion": "2024-11-05"})),
            json!(1),
        );

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method.as_ref().unwrap(), "initialize");
        assert_eq!(request.id.as_ref().unwrap(), &json!(1));
        assert!(request.is_request());
        assert!(!request.is_response());
        assert!(!request.is_notification());
    }

    #[test]
    fn test_jsonrpc_message_response() {
        let response =
            JsonRpcMessage::new_response(json!({"protocolVersion": "2024-11-05"}), json!(1));

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.method.is_none());
        assert_eq!(response.id.as_ref().unwrap(), &json!(1));
        assert!(response.result.is_some());
        assert!(!response.is_request());
        assert!(response.is_response());
        assert!(!response.is_notification());
    }

    #[test]
    fn test_jsonrpc_message_notification() {
        let notification = JsonRpcMessage::new_notification(
            "initialized",
            Some(json!({"timestamp": "2024-01-01T00:00:00Z"})),
        );

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method.as_ref().unwrap(), "initialized");
        assert!(notification.id.is_none());
        assert!(!notification.is_request());
        assert!(!notification.is_response());
        assert!(notification.is_notification());
    }

    #[test]
    fn test_jsonrpc_message_error() {
        let error = JsonRpcError::method_not_found("unknown_method");
        let error_response = JsonRpcMessage::new_error(error, json!(1));

        assert_eq!(error_response.jsonrpc, "2.0");
        assert!(error_response.method.is_none());
        assert_eq!(error_response.id.as_ref().unwrap(), &json!(1));
        assert!(error_response.error.is_some());
        assert!(error_response.is_error());
        assert!(error_response.is_response());
    }

    #[test]
    fn test_jsonrpc_message_serialization() {
        let request =
            JsonRpcMessage::new_request("test", Some(json!({"key": "value"})), json!("test-123"));

        let json = request.to_json().unwrap();
        let parsed = JsonRpcMessage::from_json(&json).unwrap();

        assert_eq!(request, parsed);
    }

    #[test]
    fn test_jsonrpc_error_standard_codes() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }
}

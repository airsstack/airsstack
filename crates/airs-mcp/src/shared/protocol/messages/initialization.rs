//! MCP Protocol Initialization Messages
//!
//! This module provides message types for MCP protocol initialization,
//! including the initialize request/response and capability negotiation.

use serde::{Deserialize, Serialize};

use crate::base::jsonrpc::{JsonRpcMessage, JsonRpcRequest, RequestId};
use crate::shared::protocol::ProtocolResult;

use super::super::types::{ClientInfo, ProtocolVersion, ServerInfo};
use super::capabilities::{ClientCapabilities, ServerCapabilities};

/// Initialize request for MCP protocol handshake
///
/// Sent by the client to initiate the MCP protocol connection and negotiate
/// capabilities. This is the first message in the MCP protocol lifecycle.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::{
///     InitializeRequest, ProtocolVersion, ClientCapabilities, ClientInfo
/// };
///
/// let request = InitializeRequest::new(
///     ClientCapabilities::minimal(),
///     ClientInfo {
///         name: "example-client".to_string(),
///         version: "1.0.0".to_string(),
///     }
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeRequest {
    /// Protocol version being requested
    #[serde(rename = "protocolVersion")]
    pub protocol_version: ProtocolVersion,

    /// Client capabilities
    pub capabilities: ClientCapabilities,

    /// Client information
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Initialize response for MCP protocol handshake
///
/// Sent by the server in response to an initialize request, completing
/// the capability negotiation and providing server information.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::{
///     InitializeResponse, ProtocolVersion, ServerCapabilities, ServerInfo
/// };
///
/// let response = InitializeResponse::new(
///     ServerCapabilities::minimal(),
///     ServerInfo {
///         name: "example-server".to_string(),
///         version: "1.0.0".to_string(),
///     },
///     Some("Welcome to the example server!".to_string())
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeResponse {
    /// Protocol version being used
    #[serde(rename = "protocolVersion")]
    pub protocol_version: ProtocolVersion,

    /// Server capabilities
    pub capabilities: ServerCapabilities,

    /// Server information
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,

    /// Optional instructions for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

impl InitializeRequest {
    /// Create a new initialize request
    ///
    /// Uses the current protocol version by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{
    ///     InitializeRequest, ClientCapabilities, ClientInfo
    /// };
    ///
    /// let request = InitializeRequest::new(
    ///     ClientCapabilities::minimal(),
    ///     ClientInfo {
    ///         name: "my-client".to_string(),
    ///         version: "1.0.0".to_string(),
    ///     }
    /// );
    /// ```
    pub fn new(capabilities: ClientCapabilities, client_info: ClientInfo) -> Self {
        Self {
            protocol_version: ProtocolVersion::current(),
            capabilities,
            client_info,
        }
    }

    /// Create a new initialize request with a specific protocol version
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{
    ///     InitializeRequest, ProtocolVersion, ClientCapabilities, ClientInfo
    /// };
    ///
    /// let request = InitializeRequest::with_version(
    ///     ProtocolVersion::current(),
    ///     ClientCapabilities::minimal(),
    ///     ClientInfo {
    ///         name: "my-client".to_string(),
    ///         version: "1.0.0".to_string(),
    ///     }
    /// );
    /// ```
    pub fn with_version(
        protocol_version: ProtocolVersion,
        capabilities: ClientCapabilities,
        client_info: ClientInfo,
    ) -> Self {
        Self {
            protocol_version,
            capabilities,
            client_info,
        }
    }

    /// Convert to JSON-RPC request
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{
    ///     InitializeRequest, ClientCapabilities, ClientInfo
    /// };
    /// use airs_mcp::RequestId;
    ///
    /// let request = InitializeRequest::new(
    ///     ClientCapabilities::minimal(),
    ///     ClientInfo {
    ///         name: "my-client".to_string(),
    ///         version: "1.0.0".to_string(),
    ///     }
    /// );
    ///
    /// let jsonrpc_request = request.to_jsonrpc_request(
    ///     RequestId::new_string("init-001")
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_jsonrpc_request(&self, id: RequestId) -> ProtocolResult<JsonRpcRequest> {
        let params = serde_json::to_value(self).map_err(|_| {
            crate::shared::protocol::ProtocolError::InvalidProtocolVersion(
                "Failed to serialize initialize request".to_string(),
            )
        })?;

        Ok(JsonRpcRequest::new("initialize", Some(params), id))
    }
}

impl InitializeResponse {
    /// Create a new initialize response
    ///
    /// Uses the current protocol version by default.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{
    ///     InitializeResponse, ServerCapabilities, ServerInfo
    /// };
    ///
    /// let response = InitializeResponse::new(
    ///     ServerCapabilities::minimal(),
    ///     ServerInfo {
    ///         name: "my-server".to_string(),
    ///         version: "1.0.0".to_string(),
    ///     },
    ///     Some("Welcome to my server!".to_string())
    /// );
    /// ```
    pub fn new(
        capabilities: ServerCapabilities,
        server_info: ServerInfo,
        instructions: Option<String>,
    ) -> Self {
        Self {
            protocol_version: ProtocolVersion::current(),
            capabilities,
            server_info,
            instructions,
        }
    }

    /// Create a new initialize response with a specific protocol version
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{
    ///     InitializeResponse, ProtocolVersion, ServerCapabilities, ServerInfo
    /// };
    ///
    /// let response = InitializeResponse::with_version(
    ///     ProtocolVersion::current(),
    ///     ServerCapabilities::minimal(),
    ///     ServerInfo {
    ///         name: "my-server".to_string(),
    ///         version: "1.0.0".to_string(),
    ///     },
    ///     Some("Welcome!".to_string())
    /// );
    /// ```
    pub fn with_version(
        protocol_version: ProtocolVersion,
        capabilities: ServerCapabilities,
        server_info: ServerInfo,
        instructions: Option<String>,
    ) -> Self {
        Self {
            protocol_version,
            capabilities,
            server_info,
            instructions,
        }
    }

    /// Check if the server supports a specific capability
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::{
    ///     InitializeResponse, ServerCapabilities, ServerInfo
    /// };
    ///
    /// let response = InitializeResponse::new(
    ///     ServerCapabilities::with_resources(true, false),
    ///     ServerInfo {
    ///         name: "my-server".to_string(),
    ///         version: "1.0.0".to_string(),
    ///     },
    ///     None
    /// );
    ///
    /// assert!(response.has_resources());
    /// assert!(!response.has_tools());
    /// ```
    pub fn has_resources(&self) -> bool {
        self.capabilities.resources.is_some()
    }

    /// Check if the server supports tools
    pub fn has_tools(&self) -> bool {
        self.capabilities.tools.is_some()
    }

    /// Check if the server supports prompts
    pub fn has_prompts(&self) -> bool {
        self.capabilities.prompts.is_some()
    }

    /// Check if the server supports logging
    pub fn has_logging(&self) -> bool {
        self.capabilities.logging.is_some()
    }
}

// Implement JsonRpcMessage trait for integration with existing JSON-RPC system
impl JsonRpcMessage for InitializeRequest {}
impl JsonRpcMessage for InitializeResponse {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_request_creation() {
        let capabilities = ClientCapabilities::minimal();
        let client_info = ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        };

        let request = InitializeRequest::new(capabilities.clone(), client_info.clone());

        assert_eq!(request.protocol_version, ProtocolVersion::current());
        assert_eq!(request.capabilities, capabilities);
        assert_eq!(request.client_info, client_info);
    }

    #[test]
    fn test_initialize_request_with_version() {
        let version = ProtocolVersion::current();
        let capabilities = ClientCapabilities::minimal();
        let client_info = ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        };

        let request = InitializeRequest::with_version(
            version.clone(),
            capabilities.clone(),
            client_info.clone(),
        );

        assert_eq!(request.protocol_version, version);
        assert_eq!(request.capabilities, capabilities);
        assert_eq!(request.client_info, client_info);
    }

    #[test]
    fn test_initialize_response_creation() {
        let capabilities = ServerCapabilities::minimal();
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "1.0.0".to_string(),
        };
        let instructions = Some("Welcome to the test server!".to_string());

        let response = InitializeResponse::new(
            capabilities.clone(),
            server_info.clone(),
            instructions.clone(),
        );

        assert_eq!(response.protocol_version, ProtocolVersion::current());
        assert_eq!(response.capabilities, capabilities);
        assert_eq!(response.server_info, server_info);
        assert_eq!(response.instructions, instructions);
    }

    #[test]
    fn test_initialize_response_capability_checks() {
        // Test server with no capabilities
        let response = InitializeResponse::new(
            ServerCapabilities::minimal(),
            ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
            },
            None,
        );

        assert!(!response.has_resources());
        assert!(!response.has_tools());
        assert!(!response.has_prompts());
        assert!(!response.has_logging());

        // Test server with resources
        let response = InitializeResponse::new(
            ServerCapabilities::with_resources(true, false),
            ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
            },
            None,
        );

        assert!(response.has_resources());
        assert!(!response.has_tools());
        assert!(!response.has_prompts());
        assert!(!response.has_logging());
    }

    #[test]
    fn test_initialize_request_to_jsonrpc() {
        let request = InitializeRequest::new(
            ClientCapabilities::minimal(),
            ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        let jsonrpc_request = request
            .to_jsonrpc_request(RequestId::new_string("init-123"))
            .unwrap();

        assert_eq!(jsonrpc_request.method, "initialize");
        assert!(jsonrpc_request.params.is_some());
        assert_eq!(jsonrpc_request.id, RequestId::new_string("init-123"));
    }

    #[test]
    fn test_message_serialization() {
        let request = InitializeRequest::new(
            ClientCapabilities::with_sampling(),
            ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        let response = InitializeResponse::new(
            ServerCapabilities::with_resources(true, true),
            ServerInfo {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
            },
            Some("Welcome!".to_string()),
        );

        // Test that messages can be serialized and deserialized
        let request_json = request.to_json().unwrap();
        let response_json = response.to_json().unwrap();

        let request_deserialized: InitializeRequest = serde_json::from_str(&request_json).unwrap();
        let response_deserialized: InitializeResponse =
            serde_json::from_str(&response_json).unwrap();

        assert_eq!(request, request_deserialized);
        assert_eq!(response, response_deserialized);
    }
}

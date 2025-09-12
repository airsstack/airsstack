//! Core MCP Protocol Types and Domain-Specific Newtypes
//!
//! This module provides domain-specific newtypes and core protocol structures
//! with validation and proper encapsulation. These types are migrated from
//! the shared/protocol/types module as part of the module consolidation.
//!
//! # Architecture
//!
//! All types use private internal fields with controlled access through validated
//! constructors and accessor methods, ensuring type safety and preventing invalid
//! protocol messages at compile time.
//!
//! # Examples
//!
//! ```rust
//! use airs_mcp::protocol::{Uri, ProtocolVersion, ClientInfo};
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

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::protocol::errors::{ProtocolError, ProtocolResult};
use crate::protocol::{JsonRpcRequest, RequestId};
use crate::protocol::constants::methods;

/// Protocol version with validation and proper encapsulation
///
/// Represents an MCP protocol version in the format YYYY-MM-DD.
/// The internal string representation is private to ensure validation
/// and provide flexibility for future implementation changes.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::ProtocolVersion;
///
/// // Create current protocol version
/// let version = ProtocolVersion::current();
/// assert_eq!(version.as_str(), "2024-11-05");
///
/// // Create custom version with validation
/// let version = ProtocolVersion::new("2024-11-05")?;
/// assert_eq!(version.as_str(), "2024-11-05");
///
/// // Invalid version format fails
/// let result = ProtocolVersion::new("invalid");
/// assert!(result.is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProtocolVersion(String);

impl ProtocolVersion {
    /// Current supported protocol version
    pub const CURRENT: &'static str = "2024-11-05";
    
    /// Create a new protocol version with validation
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::InvalidProtocolVersion` if the version
    /// format is not YYYY-MM-DD.
    pub fn new(version: impl Into<String>) -> ProtocolResult<Self> {
        let version = version.into();
        if Self::is_valid_version(&version) {
            Ok(Self(version))
        } else {
            Err(ProtocolError::InvalidProtocolVersion(version))
        }
    }
    
    /// Create current protocol version
    ///
    /// This is guaranteed to be valid and will never fail.
    pub fn current() -> Self {
        Self(Self::CURRENT.to_string())
    }
    
    /// Get the version string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Check if this version is compatible with another version
    ///
    /// Currently implements exact version matching, but can be enhanced
    /// for semantic version compatibility in the future.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    
    fn is_valid_version(version: &str) -> bool {
        // Validate YYYY-MM-DD format
        if version.len() != 10 {
            return false;
        }
        
        let chars: Vec<char> = version.chars().collect();
        
        // Check format: YYYY-MM-DD
        chars.get(4) == Some(&'-') && 
        chars.get(7) == Some(&'-') &&
        chars[0..4].iter().all(|c| c.is_ascii_digit()) &&
        chars[5..7].iter().all(|c| c.is_ascii_digit()) &&
        chars[8..10].iter().all(|c| c.is_ascii_digit())
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// URI with validation and type safety
///
/// Represents a Uniform Resource Identifier with validation and utility methods.
/// The internal string representation is private to ensure validation.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::Uri;
///
/// // Valid URI construction
/// let uri = Uri::new("file:///path/to/file")?;
/// assert_eq!(uri.scheme(), Some("file"));
/// assert_eq!(uri.as_str(), "file:///path/to/file");
///
/// // Invalid URI fails validation
/// let result = Uri::new("not-a-uri");
/// assert!(result.is_err());
///
/// // Unchecked construction for trusted sources
/// let uri = Uri::new_unchecked("custom://internal");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Uri(String);

impl Uri {
    /// Create a new URI with validation
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::InvalidUri` if the URI format is invalid.
    pub fn new(uri: impl Into<String>) -> ProtocolResult<Self> {
        let uri = uri.into();
        if Self::is_valid_uri(&uri) {
            Ok(Self(uri))
        } else {
            Err(ProtocolError::InvalidUri(uri))
        }
    }
    
    /// Create URI without validation (for trusted sources)
    ///
    /// This should only be used when the URI is known to be valid,
    /// such as constants or internally generated URIs.
    pub fn new_unchecked(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }
    
    /// Get the URI string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Extract the URI scheme (e.g., "file", "http", "custom")
    pub fn scheme(&self) -> Option<&str> {
        self.0.split(':').next()
    }
    
    /// Check if this is a file URI
    pub fn is_file_uri(&self) -> bool {
        self.scheme() == Some("file")
    }
    
    /// Check if this is an HTTP/HTTPS URI
    pub fn is_http_uri(&self) -> bool {
        matches!(self.scheme(), Some("http") | Some("https"))
    }
    
    fn is_valid_uri(uri: &str) -> bool {
        // Basic URI validation - must have scheme and not be empty
        !uri.is_empty() && uri.contains(':') && !uri.starts_with(':')
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// MIME type with validation
///
/// Represents a MIME type with validation to ensure proper format.
/// The internal string representation is private to ensure validation.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::MimeType;
///
/// // Valid MIME type construction
/// let mime = MimeType::new("text/plain")?;
/// assert_eq!(mime.as_str(), "text/plain");
/// assert_eq!(mime.main_type(), "text");
/// assert_eq!(mime.sub_type(), "plain");
///
/// // Invalid MIME type fails validation
/// let result = MimeType::new("invalid");
/// assert!(result.is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MimeType(String);

impl MimeType {
    /// Create a new MIME type with validation
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::InvalidMimeType` if the MIME type format is invalid.
    pub fn new(mime_type: impl Into<String>) -> ProtocolResult<Self> {
        let mime_type = mime_type.into();
        if Self::is_valid_mime_type(&mime_type) {
            Ok(Self(mime_type))
        } else {
            Err(ProtocolError::InvalidMimeType(mime_type))
        }
    }
    
    /// Get the MIME type string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the main type (e.g., "text" from "text/plain")
    pub fn main_type(&self) -> &str {
        self.0.split('/').next().unwrap_or("")
    }
    
    /// Get the sub type (e.g., "plain" from "text/plain")
    pub fn sub_type(&self) -> &str {
        self.0.split('/').nth(1).unwrap_or("")
    }
    
    /// Check if this is a text MIME type
    pub fn is_text(&self) -> bool {
        self.main_type() == "text"
    }
    
    /// Check if this is an image MIME type
    pub fn is_image(&self) -> bool {
        self.main_type() == "image"
    }
    
    fn is_valid_mime_type(mime_type: &str) -> bool {
        // Basic MIME type validation: type/subtype
        if !mime_type.contains('/') || mime_type.starts_with('/') || mime_type.ends_with('/') {
            return false;
        }
        
        let parts: Vec<&str> = mime_type.split('/').collect();
        parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty()
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Base64 encoded data with validation
///
/// Represents base64 encoded data with validation to ensure proper encoding.
/// The internal string representation is private to ensure validation.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::Base64Data;
///
/// // Valid base64 construction
/// let data = Base64Data::new("SGVsbG8gV29ybGQ=")?;
/// assert_eq!(data.as_str(), "SGVsbG8gV29ybGQ=");
///
/// // Invalid base64 fails validation
/// let result = Base64Data::new("invalid!@#");
/// assert!(result.is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Base64Data(String);

impl Base64Data {
    /// Create new base64 data with validation
    ///
    /// # Errors
    ///
    /// Returns `ProtocolError::InvalidBase64Data` if the data is not valid base64.
    pub fn new(data: impl Into<String>) -> ProtocolResult<Self> {
        let data = data.into();
        if Self::is_valid_base64(&data) {
            Ok(Self(data))
        } else {
            Err(ProtocolError::InvalidBase64Data)
        }
    }
    
    /// Get the base64 string
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Get the length of the base64 string
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    /// Check if the base64 string is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    fn is_valid_base64(data: &str) -> bool {
        // Basic base64 validation - can be enhanced with proper base64 crate
        if data.is_empty() {
            return false;
        }
        
        // Check that all characters are valid base64 characters
        data.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
        }) && 
        // Check padding is only at the end
        !data.trim_end_matches('=').contains('=')
    }
}

impl fmt::Display for Base64Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Client information for protocol initialization
///
/// Contains information about the MCP client, including name and version.
/// This information is exchanged during the initialization handshake.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::ClientInfo;
///
/// let client_info = ClientInfo {
///     name: "example-client".to_string(),
///     version: "1.0.0".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClientInfo {
    /// Name of the client application
    pub name: String,
    /// Version of the client application
    pub version: String,
}

/// Server information for protocol initialization
///
/// Contains information about the MCP server, including name and version.
/// This information is exchanged during the initialization handshake.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::ServerInfo;
///
/// let server_info = ServerInfo {
///     name: "example-server".to_string(),
///     version: "1.0.0".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerInfo {
    /// Name of the server application
    pub name: String,
    /// Version of the server application
    pub version: String,
}

// TODO(DEBT-ARCH): Add content types from shared/protocol/types/content.rs
// Reference: ResourceContent, TextContent, BlobContent structures
// TODO(DEBT-ARCH): Add MCP message structures from shared/protocol/messages/
// Reference: InitializeRequest, InitializeResponse, capability definitions

// ==== Additional Types Restored from git for compilation ====

/// Multi-modal content for MCP protocol messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Content {
    /// Plain text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
        /// URI of the resource (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<Uri>,
        /// MIME type of the content
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<MimeType>,
    },

    /// Image content with base64 encoded data
    #[serde(rename = "image")]
    Image {
        /// Base64 encoded image data
        #[serde(rename = "data")]
        data: Base64Data,
        /// MIME type of the image
        #[serde(rename = "mimeType")]
        mime_type: MimeType,
        /// URI of the resource (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<Uri>,
    },

    /// Resource reference content
    #[serde(rename = "resource")]
    Resource {
        /// URI of the resource
        #[serde(rename = "uri")]
        resource: Uri,
        /// Optional text description of the resource
        text: Option<String>,
        /// Optional MIME type of the resource
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<MimeType>,
    },
}

impl Content {
    /// Create text content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            uri: None,
            mime_type: None,
        }
    }
    
    /// Create text content with URI
    pub fn text_with_uri(text: impl Into<String>, uri: impl Into<String>) -> Result<Self, String> {
        let uri_str = uri.into();
        let uri = Uri::new_unchecked(uri_str);
        Ok(Self::Text {
            text: text.into(),
            uri: Some(uri),
            mime_type: None,
        })
    }

    /// Extract text content if available
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text { text, .. } => Some(text),
            Content::Resource { text: Some(text), .. } => Some(text),
            _ => None,
        }
    }
}

/// Tool definition for MCP protocol
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Capability system definitions

/// Client capabilities for MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    pub experimental: Option<serde_json::Value>,
    pub sampling: Option<SamplingCapabilities>,
    pub roots: Option<RootsCapabilities>,
}

/// Server capabilities for MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ServerCapabilities {
    pub experimental: Option<serde_json::Value>,
    pub logging: Option<LoggingCapabilities>,
    pub prompts: Option<PromptCapabilities>,
    pub resources: Option<ResourceCapabilities>,
    pub tools: Option<ToolCapabilities>,
}

/// Sampling capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamplingCapabilities {}

/// Roots capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RootsCapabilities {
    pub list_changed: Option<bool>,
}

/// Logging capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingCapabilities {}

impl Default for LoggingCapabilities {
    fn default() -> Self {
        Self {}
    }
}

/// Prompt capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptCapabilities {
    pub list_changed: Option<bool>,
}

impl Default for PromptCapabilities {
    fn default() -> Self {
        Self {
            list_changed: Some(false),
        }
    }
}

/// Resource capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceCapabilities {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

impl Default for ResourceCapabilities {
    fn default() -> Self {
        Self {
            subscribe: Some(false),
            list_changed: Some(false),
        }
    }
}

/// Tool capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCapabilities {
    pub list_changed: Option<bool>,
}

impl Default for ToolCapabilities {
    fn default() -> Self {
        Self {
            list_changed: Some(false),
        }
    }
}

/// Represents a prompt template available from the server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    /// Unique identifier for the prompt
    pub name: String,
    /// Human-readable name for the prompt
    pub title: Option<String>,
    /// Optional description of the prompt's purpose
    pub description: Option<String>,
    /// Array of arguments this prompt accepts
    pub arguments: Vec<PromptArgument>,
}

/// Represents an argument for a prompt template
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// Name of the argument
    pub name: String,
    /// Description of the argument
    pub description: Option<String>,
    /// Whether this argument is required
    pub required: bool,
}

impl PromptArgument {
    /// Create a required argument
    pub fn required(name: impl Into<String>, description: Option<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            description: description.map(|d| d.into()),
            required: true,
        }
    }
    
    /// Create an optional argument
    pub fn optional(name: impl Into<String>, description: Option<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            description: description.map(|d| d.into()),
            required: false,
        }
    }
}

/// Prompt message content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptMessage {
    /// Role of the message sender
    pub role: String,
    /// Content of the message
    pub content: Content,
}

impl PromptMessage {
    /// Create a user message
    pub fn user(content: Content) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }
    
    /// Create an assistant message
    pub fn assistant(content: Content) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }
    
    /// Create a system message
    pub fn system(content: Content) -> Self {
        Self {
            role: "system".to_string(),
            content,
        }
    }
}

/// Represents a resource available from the server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    /// URI of the resource
    pub uri: Uri,
    /// Name of the resource
    pub name: String,
    /// Description of the resource
    pub description: Option<String>,
    /// MIME type of the resource
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<MimeType>,
}

/// Log level enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    /// Convert LogLevel to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info", 
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Critical => "critical",
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingConfig {
    /// Minimum log level to include
    pub level: LogLevel,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging config with specified level
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
    
    /// Get the minimum level (compatibility with old API)
    pub fn min_level(&self) -> &LogLevel {
        &self.level
    }
}

/// Initialize request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeRequest {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: ProtocolVersion,
    /// Capabilities requested by client
    pub capabilities: serde_json::Value,
    /// Client information
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

/// Initialize response message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeResponse {
    /// Protocol version
    #[serde(rename = "protocolVersion")]
    pub protocol_version: ProtocolVersion,
    /// Capabilities offered by server
    pub capabilities: serde_json::Value,
    /// Server information
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

impl InitializeResponse {
    /// Create a new initialize response
    pub fn new(
        capabilities: serde_json::Value,
        server_info: ServerInfo,
        _instructions: Option<String>, // instructions are handled elsewhere
    ) -> Self {
        Self {
            protocol_version: ProtocolVersion::current(),
            capabilities,
            server_info,
        }
    }
}

/// Set logging request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLoggingRequest {
    /// Logging level to set
    pub level: LogLevel,
}

impl SetLoggingRequest {
    /// Create a new set logging request
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

/// Get prompt request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptRequest {
    /// Name of the prompt to get
    pub name: String,
    /// Arguments for the prompt
    pub arguments: std::collections::HashMap<String, String>,
}

/// Read resource request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceRequest {
    /// URI of the resource to read
    pub uri: Uri,
}

/// Subscribe resource request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscribeResourceRequest {
    /// URI of the resource to subscribe to
    pub uri: Uri,
}

/// Unsubscribe resource request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribeResourceRequest {
    /// URI of the resource to unsubscribe from
    pub uri: Uri,
}

/// Call tool request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolRequest {
    /// Name of the tool to call
    pub name: String,
    /// Arguments for the tool
    pub arguments: serde_json::Value,
}

/// List resources request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    pub cursor: Option<String>,
}

impl ListResourcesRequest {
    /// Create a new list resources request
    pub fn new() -> Self {
        Self { cursor: None }
    }

    /// Create a new list resources request with cursor
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self { cursor: Some(cursor.into()) }
    }

    /// Convert to JSON-RPC request
    pub fn to_jsonrpc_request(&self, id: RequestId) -> Result<JsonRpcRequest, ProtocolError> {
        let params = serde_json::to_value(self)
            .map_err(|e| ProtocolError::Serialization { message: format!("Failed to serialize ListResourcesRequest: {e}") })?;
        
        Ok(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: methods::RESOURCES_LIST.to_string(),
            params: Some(params),
            id,
        })
    }
}

/// List resources response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResponse {
    pub resources: Vec<Resource>,
    pub next_cursor: Option<String>,
}

/// List prompts request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsRequest {
    pub cursor: Option<String>,
}

impl ListPromptsRequest {
    /// Create a new list prompts request
    pub fn new() -> Self {
        Self { cursor: None }
    }

    /// Create a new list prompts request with cursor
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self { cursor: Some(cursor.into()) }
    }

    /// Convert to JSON-RPC request
    pub fn to_jsonrpc_request(&self, id: RequestId) -> Result<JsonRpcRequest, ProtocolError> {
        let params = serde_json::to_value(self)
            .map_err(|e| ProtocolError::Serialization { message: format!("Failed to serialize ListPromptsRequest: {e}") })?;
        
        Ok(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: methods::PROMPTS_LIST.to_string(),
            params: Some(params),
            id,
        })
    }
}

/// List prompts response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResponse {
    pub prompts: Vec<Prompt>,
    pub next_cursor: Option<String>,
}

/// List tools request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsRequest {
    pub cursor: Option<String>,
}

impl ListToolsRequest {
    /// Create a new list tools request
    pub fn new() -> Self {
        Self { cursor: None }
    }

    /// Create a new list tools request with cursor
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self { cursor: Some(cursor.into()) }
    }

    /// Convert to JSON-RPC request
    pub fn to_jsonrpc_request(&self, id: RequestId) -> Result<JsonRpcRequest, ProtocolError> {
        let params = serde_json::to_value(self)
            .map_err(|e| ProtocolError::Serialization { message: format!("Failed to serialize ListToolsRequest: {e}") })?;
        
        Ok(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: methods::TOOLS_LIST.to_string(),
            params: Some(params),
            id,
        })
    }
}

/// List tools response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<Tool>,
    pub next_cursor: Option<String>,
}

/// Call tool response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResponse {
    pub content: Vec<Content>,
    pub is_error: Option<bool>,
}

/// Get prompt response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResponse {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

/// Read resource response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResponse {
    pub contents: Vec<Content>,
}

/// Set logging response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLoggingResponse {
    pub success: bool,
    pub message: Option<String>,
}

/// List resource templates response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourceTemplatesResponse {
    pub resource_templates: Vec<ResourceTemplate>,
    pub next_cursor: Option<String>,
}

/// Resource template for dynamic resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTemplate {
    pub uri_template: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

// Constructor implementations for request types
impl InitializeRequest {
    /// Create a new InitializeRequest with specific protocol version
    pub fn with_version(
        protocol_version: ProtocolVersion,
        capabilities: serde_json::Value,
        client_info: ClientInfo,
    ) -> Self {
        Self {
            protocol_version,
            capabilities,
            client_info,
        }
    }

    /// Convert to JSON-RPC request
    pub fn to_jsonrpc_request(&self, id: RequestId) -> Result<JsonRpcRequest, ProtocolError> {
        let params = serde_json::to_value(self)
            .map_err(|e| ProtocolError::Serialization { message: format!("Failed to serialize InitializeRequest: {e}") })?;
        
        Ok(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: methods::INITIALIZE.to_string(),
            params: Some(params),
            id,
        })
    }
}

impl ReadResourceRequest {
    /// Create a new ReadResourceRequest for a URI
    pub fn new(uri: String) -> Result<Self, crate::protocol::TransportError> {
        Ok(Self { uri: Uri::new_unchecked(uri) })
    }
}

impl SubscribeResourceRequest {
    /// Create a new SubscribeResourceRequest for a URI
    pub fn new(uri: String) -> Result<Self, crate::protocol::TransportError> {
        Ok(Self { uri: Uri::new_unchecked(uri) })
    }
}

impl CallToolRequest {
    /// Create a new CallToolRequest
    pub fn new(name: String, arguments: serde_json::Value) -> Self {
        Self { name, arguments }
    }
}

impl GetPromptRequest {
    /// Create a new GetPromptRequest
    pub fn new(name: String, arguments: HashMap<String, String>) -> Self {
        Self { name, arguments }
    }
}

// Constructor implementations for response types
impl CallToolResponse {
    /// Create a successful tool call response
    pub fn success(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: Some(false),
        }
    }
    
    /// Create an error tool call response
    pub fn error_text(error: String) -> Self {
        Self {
            content: vec![Content::text(error)],
            is_error: Some(true),
        }
    }
}

impl LoggingConfig {
    /// Create default logging configuration
    pub fn default() -> Self {
        Self {
            level: LogLevel::Info,
        }
    }
}

/// Core MCP server configuration required by all transports
///
/// This contains only the universal MCP requirements that every transport needs,
/// regardless of transport type (STDIO, HTTP, WebSocket, etc.). This configuration
/// defines the fundamental server identity, capabilities, and protocol compliance
/// that must be consistent across all transport implementations.
///
/// # Architecture
///
/// This struct represents the protocol-level server configuration that sits at the
/// foundation of the MCP transport abstraction. All transport-specific configurations
/// should embed this as their core configuration to ensure protocol compliance.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::protocol::types::{ServerConfig, ServerInfo, ServerCapabilities, ProtocolVersion};
///
/// // Default configuration
/// let config = ServerConfig::default();
///
/// // Custom configuration
/// let config = ServerConfig {
///     server_info: ServerInfo {
///         name: "my-mcp-server".to_string(),
///         version: "1.0.0".to_string(),
///     },
///     capabilities: ServerCapabilities::default(),
///     protocol_version: ProtocolVersion::current(),
///     instructions: Some("Custom server instructions".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ServerConfig {
    /// Server information to send during initialization
    pub server_info: ServerInfo,
    /// Server capabilities to advertise
    pub capabilities: ServerCapabilities,
    /// Protocol version to support
    pub protocol_version: ProtocolVersion,
    /// Optional instructions to provide to clients during initialization
    pub instructions: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_info: ServerInfo {
                name: "airs-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ServerCapabilities::default(),
            protocol_version: ProtocolVersion::current(),
            instructions: Some(
                "MCP server with configurable capabilities. Use appropriate authentication method."
                    .to_string(),
            ),
        }
    }
}

// ================================================================================
// MCP Response Types
// ================================================================================

/// Result of calling a tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolResult {
    /// Content returned by the tool
    pub content: Vec<Content>,
    /// Whether the tool call failed
    #[serde(default)]
    pub is_error: bool,
}

impl CallToolResult {
    /// Create a successful tool result
    pub fn success(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: false,
        }
    }

    /// Create an error tool result
    pub fn error(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: true,
        }
    }
}

/// Result of reading a resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceResult {
    /// Contents of the resource
    pub contents: Vec<Content>,
}

impl ReadResourceResult {
    /// Create a new resource read result
    pub fn new(contents: Vec<Content>) -> Self {
        Self { contents }
    }
}

/// Result of getting a prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptResult {
    /// Description of the prompt
    pub description: Option<String>,
    /// Prompt messages
    pub messages: Vec<PromptMessage>,
}

impl GetPromptResult {
    /// Create a new prompt result
    pub fn new(description: Option<String>, messages: Vec<PromptMessage>) -> Self {
        Self { description, messages }
    }
}

/// Result of listing resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourcesResult {
    /// List of resources
    pub resources: Vec<Resource>,
    /// Next page cursor if applicable
    pub next_cursor: Option<String>,
}

impl ListResourcesResult {
    /// Create a new resource list result
    pub fn new(resources: Vec<Resource>) -> Self {
        Self {
            resources,
            next_cursor: None,
        }
    }

    /// Create a new resource list result with pagination
    pub fn with_cursor(resources: Vec<Resource>, next_cursor: Option<String>) -> Self {
        Self {
            resources,
            next_cursor,
        }
    }
}

/// Result of listing tools
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListToolsResult {
    /// List of tools
    pub tools: Vec<Tool>,
    /// Next page cursor if applicable
    pub next_cursor: Option<String>,
}

impl ListToolsResult {
    /// Create a new tool list result
    pub fn new(tools: Vec<Tool>) -> Self {
        Self {
            tools,
            next_cursor: None,
        }
    }

    /// Create a new tool list result with pagination
    pub fn with_cursor(tools: Vec<Tool>, next_cursor: Option<String>) -> Self {
        Self {
            tools,
            next_cursor,
        }
    }
}

/// Result of listing prompts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListPromptsResult {
    /// List of prompts
    pub prompts: Vec<Prompt>,
    /// Next page cursor if applicable
    pub next_cursor: Option<String>,
}

impl ListPromptsResult {
    /// Create a new prompt list result
    pub fn new(prompts: Vec<Prompt>) -> Self {
        Self {
            prompts,
            next_cursor: None,
        }
    }

    /// Create a new prompt list result with pagination
    pub fn with_cursor(prompts: Vec<Prompt>, next_cursor: Option<String>) -> Self {
        Self {
            prompts,
            next_cursor,
        }
    }
}



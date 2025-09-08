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
use std::fmt;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use super::errors::{ProtocolError, ProtocolResult};

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

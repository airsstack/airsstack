//! Resource Messages for MCP Protocol
//!
//! This module provides message types for MCP resource operations including
//! listing, reading, subscribing to resources, and handling resource updates.

use serde::{Deserialize, Serialize};

use super::super::types::{Content, MimeType, Uri};
use crate::base::jsonrpc::message::JsonRpcMessage;

/// Represents a resource available from an MCP server
///
/// Resources are data sources that can be read by clients, such as files,
/// API endpoints, database records, or any other addressable data.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::{Resource, Uri};
///
/// let resource = Resource {
///     uri: Uri::new("file:///path/to/document.md")?,
///     name: "Project Documentation".to_string(),
///     description: Some("Main project documentation file".to_string()),
///     mime_type: None,
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    /// URI identifying the resource
    pub uri: Uri,
    /// Human-readable name for the resource
    pub name: String,
    /// Optional description of the resource
    pub description: Option<String>,
    /// Optional MIME type of the resource content
    #[serde(rename = "mimeType")]
    pub mime_type: Option<MimeType>,
}

impl Resource {
    /// Create a new resource
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Resource;
    ///
    /// let resource = Resource::new(
    ///     "file:///config.json",
    ///     "Configuration File",
    ///     Some("Application configuration"),
    ///     Some("application/json")
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        uri: impl Into<String>,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        mime_type: Option<impl Into<String>>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        let mime_type = match mime_type {
            Some(mt) => Some(MimeType::new(mt)?),
            None => None,
        };

        Ok(Self {
            uri: Uri::new(uri)?,
            name: name.into(),
            description: description.map(|d| d.into()),
            mime_type,
        })
    }

    /// Check if this resource has a specific scheme
    #[must_use]
    pub fn has_scheme(&self, scheme: &str) -> bool {
        self.uri.scheme() == Some(scheme)
    }

    /// Get the resource's file extension if it's a file URI
    #[must_use]
    pub fn file_extension(&self) -> Option<&str> {
        if self.has_scheme("file") {
            self.uri
                .as_str()
                .rsplit('.')
                .next()
                .filter(|ext| !ext.contains('/'))
        } else {
            None
        }
    }
}

/// Template for generating resource URIs with parameters
///
/// Uses RFC 6570 URI templates to allow dynamic resource generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceTemplate {
    /// RFC 6570 URI template string
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,
    /// Human-readable name for the template
    pub name: String,
    /// Optional description of the template
    pub description: Option<String>,
    /// Optional MIME type for resources generated from this template
    #[serde(rename = "mimeType")]
    pub mime_type: Option<MimeType>,
}

impl ResourceTemplate {
    /// Create a new resource template
    pub fn new(
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        mime_type: Option<impl Into<String>>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        let mime_type = match mime_type {
            Some(mt) => Some(MimeType::new(mt)?),
            None => None,
        };

        Ok(Self {
            uri_template: uri_template.into(),
            name: name.into(),
            description: description.map(|d| d.into()),
            mime_type,
        })
    }
}

/// Request to list available resources
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::ListResourcesRequest;
///
/// let request = ListResourcesRequest::new();
/// let request_with_cursor = ListResourcesRequest::with_cursor("next_page_token");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourcesRequest {
    /// Optional cursor for pagination
    pub cursor: Option<String>,
}

impl ListResourcesRequest {
    /// Create a new list resources request
    pub fn new() -> Self {
        Self { cursor: None }
    }

    /// Create a new list resources request with cursor
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self {
            cursor: Some(cursor.into()),
        }
    }
}

impl Default for ListResourcesRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRpcMessage for ListResourcesRequest {}

/// Response containing available resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourcesResponse {
    /// List of available resources
    pub resources: Vec<Resource>,
    /// Optional cursor for next page of results
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

impl ListResourcesResponse {
    /// Create a new list resources response
    pub fn new(resources: Vec<Resource>) -> Self {
        Self {
            resources,
            next_cursor: None,
        }
    }

    /// Create a new list resources response with pagination
    pub fn with_pagination(resources: Vec<Resource>, next_cursor: Option<String>) -> Self {
        Self {
            resources,
            next_cursor,
        }
    }

    /// Check if there are more resources available
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.next_cursor.is_some()
    }

    /// Get the number of resources in this response
    #[must_use]
    pub fn count(&self) -> usize {
        self.resources.len()
    }
}

impl JsonRpcMessage for ListResourcesResponse {}

/// Request to read a specific resource
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::ReadResourceRequest;
///
/// let request = ReadResourceRequest::new("file:///path/to/file.txt")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceRequest {
    /// URI of the resource to read
    pub uri: Uri,
}

impl ReadResourceRequest {
    /// Create a new read resource request
    pub fn new(uri: impl Into<String>) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self {
            uri: Uri::new(uri)?,
        })
    }
}

impl JsonRpcMessage for ReadResourceRequest {}

/// Response containing resource content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceResponse {
    /// Content of the resource
    pub contents: Vec<Content>,
}

impl ReadResourceResponse {
    /// Create a new read resource response
    pub fn new(contents: Vec<Content>) -> Self {
        Self { contents }
    }

    /// Create a response with a single text content
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            contents: vec![Content::text(text)],
        }
    }

    /// Create a response with a single image content
    pub fn image(
        data: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self {
            contents: vec![Content::image(data, mime_type)?],
        })
    }

    /// Add content to the response
    pub fn add_content(&mut self, content: Content) {
        self.contents.push(content);
    }

    /// Get the number of content items
    #[must_use]
    pub fn content_count(&self) -> usize {
        self.contents.len()
    }

    /// Check if the response is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}

impl JsonRpcMessage for ReadResourceResponse {}

/// Request to subscribe to resource changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscribeResourceRequest {
    /// URI of the resource to subscribe to
    pub uri: Uri,
}

impl SubscribeResourceRequest {
    /// Create a new subscribe resource request
    pub fn new(uri: impl Into<String>) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self {
            uri: Uri::new(uri)?,
        })
    }
}

impl JsonRpcMessage for SubscribeResourceRequest {}

/// Request to unsubscribe from resource changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribeResourceRequest {
    /// URI of the resource to unsubscribe from
    pub uri: Uri,
}

impl UnsubscribeResourceRequest {
    /// Create a new unsubscribe resource request
    pub fn new(uri: impl Into<String>) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self {
            uri: Uri::new(uri)?,
        })
    }
}

impl JsonRpcMessage for UnsubscribeResourceRequest {}

/// Notification sent when a subscribed resource changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUpdatedNotification {
    /// URI of the changed resource
    pub uri: Uri,
}

impl ResourceUpdatedNotification {
    /// Create a new resource updated notification
    pub fn new(uri: impl Into<String>) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self {
            uri: Uri::new(uri)?,
        })
    }
}

impl JsonRpcMessage for ResourceUpdatedNotification {}

/// Notification sent when the list of resources changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceListChangedNotification {}

impl ResourceListChangedNotification {
    /// Create a new resource list changed notification
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ResourceListChangedNotification {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRpcMessage for ResourceListChangedNotification {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::protocol::ProtocolError;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new(
            "file:///test.txt",
            "Test File",
            Some("A test file"),
            Some("text/plain"),
        )
        .unwrap();

        assert_eq!(resource.uri.as_str(), "file:///test.txt");
        assert_eq!(resource.name, "Test File");
        assert_eq!(resource.description, Some("A test file".to_string()));
        assert!(resource.mime_type.is_some());
        assert_eq!(resource.mime_type.unwrap().as_str(), "text/plain");
    }

    #[test]
    fn test_resource_invalid_uri() {
        let result = Resource::new("invalid", "Test", None::<String>, None::<String>);
        assert!(matches!(result, Err(ProtocolError::InvalidUri(_))));
    }

    #[test]
    fn test_resource_scheme_check() {
        let resource =
            Resource::new("file:///test.txt", "Test", None::<String>, None::<String>).unwrap();

        assert!(resource.has_scheme("file"));
        assert!(!resource.has_scheme("http"));
    }

    #[test]
    fn test_resource_file_extension() {
        let resource = Resource::new(
            "file:///path/test.txt",
            "Test",
            None::<String>,
            None::<String>,
        )
        .unwrap();

        assert_eq!(resource.file_extension(), Some("txt"));
    }

    #[test]
    fn test_list_resources_request() {
        let request = ListResourcesRequest::new();
        assert!(request.cursor.is_none());

        let request_with_cursor = ListResourcesRequest::with_cursor("token123");
        assert_eq!(request_with_cursor.cursor, Some("token123".to_string()));
    }

    #[test]
    fn test_list_resources_response() {
        let resources = vec![
            Resource::new(
                "file:///test1.txt",
                "Test 1",
                None::<String>,
                None::<String>,
            )
            .unwrap(),
            Resource::new(
                "file:///test2.txt",
                "Test 2",
                None::<String>,
                None::<String>,
            )
            .unwrap(),
        ];

        let response = ListResourcesResponse::new(resources.clone());
        assert_eq!(response.count(), 2);
        assert!(!response.has_more());

        let response_with_pagination =
            ListResourcesResponse::with_pagination(resources, Some("next_token".to_string()));
        assert!(response_with_pagination.has_more());
    }

    #[test]
    fn test_read_resource_request() {
        let request = ReadResourceRequest::new("file:///test.txt").unwrap();
        assert_eq!(request.uri.as_str(), "file:///test.txt");
    }

    #[test]
    fn test_read_resource_response() {
        let response = ReadResourceResponse::text("Hello, world!");
        assert_eq!(response.content_count(), 1);
        assert!(!response.is_empty());

        let mut response = ReadResourceResponse::new(vec![]);
        assert!(response.is_empty());

        response.add_content(Content::text("Added content"));
        assert_eq!(response.content_count(), 1);
    }

    #[test]
    fn test_subscribe_resource_request() {
        let request = SubscribeResourceRequest::new("file:///watch.txt").unwrap();
        assert_eq!(request.uri.as_str(), "file:///watch.txt");
    }

    #[test]
    fn test_resource_notifications() {
        let notification = ResourceUpdatedNotification::new("file:///changed.txt").unwrap();
        assert_eq!(notification.uri.as_str(), "file:///changed.txt");

        let _list_changed = ResourceListChangedNotification::new();
        // Should be constructible
        let _default = ResourceListChangedNotification::default();
    }

    #[test]
    fn test_resource_template() {
        let template = ResourceTemplate::new(
            "file:///docs/{category}/{id}",
            "Document Template",
            Some("Template for documents"),
            Some("text/markdown"),
        )
        .unwrap();

        assert_eq!(template.uri_template, "file:///docs/{category}/{id}");
        assert_eq!(template.name, "Document Template");
    }

    #[test]
    fn test_message_serialization() {
        // Test that all message types can be serialized and deserialized
        let request = ListResourcesRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ListResourcesRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, deserialized);

        let resource =
            Resource::new("file:///test.txt", "Test", None::<String>, None::<String>).unwrap();
        let response = ListResourcesResponse::new(vec![resource]);
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ListResourcesResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, deserialized);
    }
}

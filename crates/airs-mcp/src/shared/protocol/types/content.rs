//! Content System for MCP Protocol
//!
//! This module provides multi-modal content support for MCP messages,
//! including text, image, and resource content with proper type safety.

use std::fmt::Write;

use serde::{Deserialize, Serialize};

use super::common::{Base64Data, MimeType, Uri};

/// Multi-modal content for MCP protocol messages
///
/// Content represents different types of data that can be included in MCP messages,
/// supporting text, images, and resource references with proper type safety.
///
/// For resource responses (ReadResourceResponse), this maps to MCP schema types:
/// - Text -> TextResourceContents (with uri field)
/// - Image -> BlobResourceContents (with uri field)
/// - Resource -> EmbeddedResource
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::{Content, Uri, MimeType, Base64Data};
///
/// // Text content for resource responses - includes URI
/// let text_content = Content::Text {
///     text: "Hello, world!".to_string(),
///     uri: Some(Uri::new("file:///example.txt")?),
///     mime_type: Some(MimeType::new("text/plain")?),
/// };
///
/// // Image content with validation - includes URI for resource responses
/// let image_content = Content::Image {
///     data: Base64Data::new("SGVsbG8gV29ybGQ=")?,
///     mime_type: MimeType::new("image/png")?,
///     uri: Some(Uri::new("file:///image.png")?),
/// };
///
/// // Resource content with URI validation
/// let resource_content = Content::Resource {
///     resource: Uri::new("file:///path/to/resource")?,
///     text: Some("Resource description".to_string()),
///     mime_type: Some(MimeType::new("text/plain")?),
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Content {
    /// Plain text content - maps to TextResourceContents in resource responses
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
        /// URI of the resource (required for resource responses per MCP schema)
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<Uri>,
        /// MIME type of the content
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<MimeType>,
    },

    /// Image content with base64 encoded data - maps to BlobResourceContents in resource responses
    #[serde(rename = "image")]
    Image {
        /// Base64 encoded image data (renamed to 'blob' in resource responses)
        #[serde(rename = "data")]
        data: Base64Data,
        /// MIME type of the image (e.g., "image/png", "image/jpeg")
        #[serde(rename = "mimeType")]
        mime_type: MimeType,
        /// URI of the resource (required for resource responses per MCP schema)
        #[serde(skip_serializing_if = "Option::is_none")]
        uri: Option<Uri>,
    },

    /// Resource reference content - maps to EmbeddedResource
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Content;
    ///
    /// let content = Content::text("Hello, world!");
    /// ```
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            uri: None,
            mime_type: None,
        }
    }

    /// Create text content with URI (for resource responses)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Content;
    ///
    /// let content = Content::text_with_uri("Hello, world!", "file:///example.txt")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn text_with_uri(
        text: impl Into<String>,
        uri: impl Into<String>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self::Text {
            text: text.into(),
            uri: Some(Uri::new(uri)?),
            mime_type: None,
        })
    }

    /// Create text content with URI and MIME type (for resource responses)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Content;
    ///
    /// let content = Content::text_with_uri_and_mime_type(
    ///     "Hello, world!",
    ///     "file:///example.txt",
    ///     "text/plain"
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn text_with_uri_and_mime_type(
        text: impl Into<String>,
        uri: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self::Text {
            text: text.into(),
            uri: Some(Uri::new(uri)?),
            mime_type: Some(MimeType::new(mime_type)?),
        })
    }

    /// Create image content with validation
    ///
    /// # Errors
    ///
    /// Returns an error if the base64 data or MIME type is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Content;
    ///
    /// let content = Content::image("SGVsbG8gV29ybGQ=", "image/png")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn image(
        data: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self::Image {
            data: Base64Data::new(data)?,
            mime_type: MimeType::new(mime_type)?,
            uri: None,
        })
    }

    /// Create image content with URI (for resource responses)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Content;
    ///
    /// let content = Content::image_with_uri("SGVsbG8gV29ybGQ=", "image/png", "file:///image.png")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn image_with_uri(
        data: impl Into<String>,
        mime_type: impl Into<String>,
        uri: impl Into<String>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        Ok(Self::Image {
            data: Base64Data::new(data)?,
            mime_type: MimeType::new(mime_type)?,
            uri: Some(Uri::new(uri)?),
        })
    }

    /// Create resource content with validation
    ///
    /// # Errors
    ///
    /// Returns an error if the URI or MIME type is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::shared::protocol::Content;
    ///
    /// let content = Content::resource(
    ///     "file:///path/to/file",
    ///     Some("File description"),
    ///     Some("text/plain")
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn resource(
        uri: impl Into<String>,
        text: Option<impl Into<String>>,
        mime_type: Option<impl Into<String>>,
    ) -> Result<Self, crate::shared::protocol::ProtocolError> {
        let mime_type = match mime_type {
            Some(mt) => Some(MimeType::new(mt)?),
            None => None,
        };

        Ok(Self::Resource {
            resource: Uri::new(uri)?,
            text: text.map(|t| t.into()),
            mime_type,
        })
    }

    /// Check if this content is text
    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Check if this content is an image
    #[must_use]
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image { .. })
    }

    /// Check if this content is a resource
    #[must_use]
    pub fn is_resource(&self) -> bool {
        matches!(self, Self::Resource { .. })
    }

    /// Get the text content if this is text content
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text, .. } => Some(text),
            _ => None,
        }
    }

    /// Get the image data if this is image content
    #[must_use]
    pub fn as_image(&self) -> Option<(&Base64Data, &MimeType)> {
        match self {
            Self::Image {
                data, mime_type, ..
            } => Some((data, mime_type)),
            _ => None,
        }
    }

    /// Get the resource URI if this is resource content
    #[must_use]
    pub fn as_resource(&self) -> Option<&Uri> {
        match self {
            Self::Resource { resource, .. } => Some(resource),
            _ => None,
        }
    }

    /// Get the URI if present (for content with URI field)
    #[must_use]
    pub fn uri(&self) -> Option<&Uri> {
        match self {
            Self::Text { uri, .. } | Self::Image { uri, .. } => uri.as_ref(),
            Self::Resource { resource, .. } => Some(resource),
        }
    }

    /// Get the MIME type if present
    #[must_use]
    pub fn mime_type(&self) -> Option<&MimeType> {
        match self {
            Self::Text { mime_type, .. } => mime_type.as_ref(),
            Self::Image { mime_type, .. } => Some(mime_type),
            Self::Resource { mime_type, .. } => mime_type.as_ref(),
        }
    }

    /// Get a human-readable description of this content
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Text {
                text,
                uri,
                mime_type,
            } => {
                let mut desc = if text.len() <= 50 {
                    format!("Text: {text}")
                } else {
                    format!("Text: {}...", &text[..47])
                };
                if let Some(uri) = uri {
                    write!(desc, " [{}]", uri.as_str()).expect("String write should not fail");
                }
                if let Some(mime_type) = mime_type {
                    write!(desc, " ({})", mime_type.as_str())
                        .expect("String write should not fail");
                }
                desc
            }
            Self::Image { mime_type, uri, .. } => {
                let mut desc = format!("Image: {}", mime_type.as_str());
                if let Some(uri) = uri {
                    write!(desc, " [{}]", uri.as_str()).expect("String write should not fail");
                }
                desc
            }
            Self::Resource {
                resource,
                text,
                mime_type,
            } => {
                let mut desc = format!("Resource: {}", resource.as_str());
                if let Some(text) = text {
                    write!(desc, " ({text})").expect("String write should not fail");
                }
                if let Some(mime_type) = mime_type {
                    write!(desc, " [{}]", mime_type.as_str())
                        .expect("String write should not fail");
                }
                desc
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::protocol::ProtocolError;

    #[test]
    fn test_text_content() {
        let content = Content::text("Hello, world!");
        assert!(content.is_text());
        assert!(!content.is_image());
        assert!(!content.is_resource());
        assert_eq!(content.as_text(), Some("Hello, world!"));
        assert_eq!(content.description(), "Text: Hello, world!");
        assert_eq!(content.uri(), None);
        assert_eq!(content.mime_type(), None);
    }

    #[test]
    fn test_text_content_with_uri() {
        let content = Content::text_with_uri("Hello, world!", "file:///example.txt").unwrap();
        assert!(content.is_text());
        assert_eq!(content.as_text(), Some("Hello, world!"));
        assert_eq!(content.uri().unwrap().as_str(), "file:///example.txt");
        assert_eq!(content.mime_type(), None);
        assert_eq!(
            content.description(),
            "Text: Hello, world! [file:///example.txt]"
        );
    }

    #[test]
    fn test_text_content_with_uri_and_mime() {
        let content = Content::text_with_uri_and_mime_type(
            "Hello, world!",
            "file:///example.txt",
            "text/plain",
        )
        .unwrap();
        assert!(content.is_text());
        assert_eq!(content.as_text(), Some("Hello, world!"));
        assert_eq!(content.uri().unwrap().as_str(), "file:///example.txt");
        assert_eq!(content.mime_type().unwrap().as_str(), "text/plain");
        assert_eq!(
            content.description(),
            "Text: Hello, world! [file:///example.txt] (text/plain)"
        );
    }

    #[test]
    fn test_text_content_long() {
        let long_text = "A".repeat(100);
        let content = Content::text(&long_text);
        let desc = content.description();
        assert!(desc.starts_with("Text: "));
        assert!(desc.ends_with("..."));
        assert_eq!(desc.len(), 56); // "Text: " (6) + 47 chars + "..." (3) = 56
    }

    #[test]
    fn test_image_content_success() {
        let content = Content::image("SGVsbG8gV29ybGQ=", "image/png").unwrap();
        assert!(!content.is_text());
        assert!(content.is_image());
        assert!(!content.is_resource());

        let (data, mime_type) = content.as_image().unwrap();
        assert_eq!(data.as_str(), "SGVsbG8gV29ybGQ=");
        assert_eq!(mime_type.as_str(), "image/png");
        assert_eq!(content.description(), "Image: image/png");
        assert_eq!(content.uri(), None);
    }

    #[test]
    fn test_image_content_with_uri() {
        let content =
            Content::image_with_uri("SGVsbG8gV29ybGQ=", "image/png", "file:///image.png").unwrap();
        assert!(content.is_image());
        assert_eq!(content.uri().unwrap().as_str(), "file:///image.png");
        assert_eq!(
            content.description(),
            "Image: image/png [file:///image.png]"
        );
    }

    #[test]
    fn test_image_content_invalid_base64() {
        let result = Content::image("invalid!@#", "image/png");
        assert!(matches!(result, Err(ProtocolError::InvalidBase64Data)));
    }

    #[test]
    fn test_image_content_invalid_mime_type() {
        let result = Content::image("SGVsbG8gV29ybGQ=", "invalid");
        assert!(matches!(result, Err(ProtocolError::InvalidMimeType(_))));
    }

    #[test]
    fn test_resource_content_success() {
        let content = Content::resource(
            "file:///path/to/file",
            Some("Test file"),
            Some("text/plain"),
        )
        .unwrap();

        assert!(!content.is_text());
        assert!(!content.is_image());
        assert!(content.is_resource());

        let resource = content.as_resource().unwrap();
        assert_eq!(resource.as_str(), "file:///path/to/file");
        assert_eq!(content.uri().unwrap().as_str(), "file:///path/to/file");
        assert_eq!(content.mime_type().unwrap().as_str(), "text/plain");
        assert_eq!(
            content.description(),
            "Resource: file:///path/to/file (Test file) [text/plain]"
        );
    }

    #[test]
    fn test_resource_content_minimal() {
        let content =
            Content::resource("file:///path/to/file", None::<String>, None::<String>).unwrap();

        assert_eq!(content.description(), "Resource: file:///path/to/file");
        assert_eq!(content.uri().unwrap().as_str(), "file:///path/to/file");
        assert_eq!(content.mime_type(), None);
    }

    #[test]
    fn test_resource_content_invalid_uri() {
        let result = Content::resource("invalid-uri", None::<String>, None::<String>);
        assert!(matches!(result, Err(ProtocolError::InvalidUri(_))));
    }

    #[test]
    fn test_resource_content_invalid_mime_type() {
        let result = Content::resource("file:///path/to/file", None::<String>, Some("invalid"));
        assert!(matches!(result, Err(ProtocolError::InvalidMimeType(_))));
    }

    #[test]
    fn test_content_serialization() {
        // Test that content can be serialized and deserialized
        let contents = vec![
            Content::text("Hello"),
            Content::text_with_uri("Hello", "file:///test.txt").unwrap(),
            Content::image("SGVsbG8=", "image/png").unwrap(),
            Content::image_with_uri("SGVsbG8=", "image/png", "file:///image.png").unwrap(),
            Content::resource("file:///test", Some("Test"), Some("text/plain")).unwrap(),
        ];

        for content in contents {
            let json = serde_json::to_string(&content).unwrap();
            let deserialized: Content = serde_json::from_str(&json).unwrap();
            assert_eq!(content, deserialized);
        }
    }
}

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
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::{Content, Uri, MimeType, Base64Data};
///
/// // Text content
/// let text_content = Content::Text {
///     text: "Hello, world!".to_string(),
/// };
///
/// // Image content with validation
/// let image_content = Content::Image {
///     data: Base64Data::new("SGVsbG8gV29ybGQ=")?,
///     mime_type: MimeType::new("image/png")?,
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
    /// Plain text content
    #[serde(rename = "text")]
    Text {
        /// The text content
        text: String,
    },

    /// Image content with base64 encoded data
    #[serde(rename = "image")]
    Image {
        /// Base64 encoded image data
        data: Base64Data,
        /// MIME type of the image (e.g., "image/png", "image/jpeg")
        mime_type: MimeType,
    },

    /// Resource reference content
    #[serde(rename = "resource")]
    Resource {
        /// URI of the resource
        resource: Uri,
        /// Optional text description of the resource
        text: Option<String>,
        /// Optional MIME type of the resource
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
        Self::Text { text: text.into() }
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
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Get the image data if this is image content
    #[must_use]
    pub fn as_image(&self) -> Option<(&Base64Data, &MimeType)> {
        match self {
            Self::Image { data, mime_type } => Some((data, mime_type)),
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

    /// Get a human-readable description of this content
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Text { text } => {
                if text.len() <= 50 {
                    format!("Text: {text}")
                } else {
                    format!("Text: {}...", &text[..47])
                }
            }
            Self::Image { mime_type, .. } => {
                format!("Image: {}", mime_type.as_str())
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
            Content::image("SGVsbG8=", "image/png").unwrap(),
            Content::resource("file:///test", Some("Test"), Some("text/plain")).unwrap(),
        ];

        for content in contents {
            let json = serde_json::to_string(&content).unwrap();
            let deserialized: Content = serde_json::from_str(&json).unwrap();
            assert_eq!(content, deserialized);
        }
    }
}

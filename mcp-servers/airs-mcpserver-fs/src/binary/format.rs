//! File format detection and validation

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: Third-party crate imports
// (None needed for this module)

// Layer 3: Internal module imports
// (None needed yet)

/// File format detection using magic numbers and content analysis
#[derive(Debug)]
pub struct FormatDetector;

/// Supported file formats for binary processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    // Image formats
    Jpeg,
    Png,
    Gif,
    WebP,
    Tiff,
    Bmp,

    // Document formats
    Pdf,

    // Text formats
    Text,

    // Unknown/unsupported format
    Unknown,
}

impl FormatDetector {
    /// Create a new format detector
    pub fn new() -> Self {
        Self
    }

    /// Detect file format from file content (magic numbers)
    pub fn detect_from_bytes(&self, bytes: &[u8]) -> FileFormat {
        if let Some(kind) = infer::get(bytes) {
            match kind.mime_type() {
                "image/jpeg" => FileFormat::Jpeg,
                "image/png" => FileFormat::Png,
                "image/gif" => FileFormat::Gif,
                "image/webp" => FileFormat::WebP,
                "image/tiff" => FileFormat::Tiff,
                "image/bmp" => FileFormat::Bmp,
                "application/pdf" => FileFormat::Pdf,
                _ => FileFormat::Unknown,
            }
        } else {
            // Check if it's likely text content
            if bytes
                .iter()
                .all(|&b| b.is_ascii() || b.is_ascii_whitespace())
            {
                FileFormat::Text
            } else {
                FileFormat::Unknown
            }
        }
    }

    /// Detect file format from file extension (fallback method)
    pub fn detect_from_extension<P: AsRef<Path>>(&self, path: P) -> FileFormat {
        if let Some(extension) = path.as_ref().extension() {
            match extension.to_string_lossy().to_lowercase().as_str() {
                "jpg" | "jpeg" => FileFormat::Jpeg,
                "png" => FileFormat::Png,
                "gif" => FileFormat::Gif,
                "webp" => FileFormat::WebP,
                "tiff" | "tif" => FileFormat::Tiff,
                "bmp" => FileFormat::Bmp,
                "pdf" => FileFormat::Pdf,
                "txt" | "md" | "rs" | "py" | "js" | "html" | "css" | "json" | "toml" | "yml"
                | "yaml" => FileFormat::Text,
                _ => FileFormat::Unknown,
            }
        } else {
            FileFormat::Unknown
        }
    }

    /// Check if a format is supported for processing
    pub fn is_supported(&self, format: FileFormat) -> bool {
        !matches!(format, FileFormat::Unknown)
    }
}

impl Default for FormatDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FileFormat {
    /// Check if the format is an image
    pub fn is_image(&self) -> bool {
        matches!(
            self,
            FileFormat::Jpeg
                | FileFormat::Png
                | FileFormat::Gif
                | FileFormat::WebP
                | FileFormat::Tiff
                | FileFormat::Bmp
        )
    }

    /// Check if the format is a document
    pub fn is_document(&self) -> bool {
        matches!(self, FileFormat::Pdf)
    }

    /// Check if the format is text
    pub fn is_text(&self) -> bool {
        matches!(self, FileFormat::Text)
    }

    /// Get the MIME type for the format
    pub fn mime_type(&self) -> &'static str {
        match self {
            FileFormat::Jpeg => "image/jpeg",
            FileFormat::Png => "image/png",
            FileFormat::Gif => "image/gif",
            FileFormat::WebP => "image/webp",
            FileFormat::Tiff => "image/tiff",
            FileFormat::Bmp => "image/bmp",
            FileFormat::Pdf => "application/pdf",
            FileFormat::Text => "text/plain",
            FileFormat::Unknown => "application/octet-stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detector_creation() {
        let detector = FormatDetector::new();
        // Basic creation test - just verify the detector can be created
        assert!(std::mem::size_of_val(&detector) == std::mem::size_of::<FormatDetector>());
    }

    #[test]
    fn test_detect_from_extension() {
        let detector = FormatDetector::new();

        assert_eq!(
            detector.detect_from_extension("image.jpg"),
            FileFormat::Jpeg
        );
        assert_eq!(
            detector.detect_from_extension("document.pdf"),
            FileFormat::Pdf
        );
        assert_eq!(
            detector.detect_from_extension("script.rs"),
            FileFormat::Text
        );
        assert_eq!(
            detector.detect_from_extension("unknown.xyz"),
            FileFormat::Unknown
        );
    }

    #[test]
    fn test_file_format_properties() {
        assert!(FileFormat::Jpeg.is_image());
        assert!(!FileFormat::Jpeg.is_document());
        assert!(!FileFormat::Jpeg.is_text());

        assert!(FileFormat::Pdf.is_document());
        assert!(!FileFormat::Pdf.is_image());
        assert!(!FileFormat::Pdf.is_text());

        assert!(FileFormat::Text.is_text());
        assert!(!FileFormat::Text.is_image());
        assert!(!FileFormat::Text.is_document());
    }

    #[test]
    fn test_mime_types() {
        assert_eq!(FileFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(FileFormat::Pdf.mime_type(), "application/pdf");
        assert_eq!(FileFormat::Text.mime_type(), "text/plain");
        assert_eq!(FileFormat::Unknown.mime_type(), "application/octet-stream");
    }

    #[test]
    fn test_is_supported() {
        let detector = FormatDetector::new();

        assert!(detector.is_supported(FileFormat::Jpeg));
        assert!(detector.is_supported(FileFormat::Pdf));
        assert!(detector.is_supported(FileFormat::Text));
        assert!(!detector.is_supported(FileFormat::Unknown));
    }

    #[test]
    fn test_detect_from_bytes_text() {
        let detector = FormatDetector::new();
        let text_bytes = b"Hello, world!";

        assert_eq!(detector.detect_from_bytes(text_bytes), FileFormat::Text);
    }
}

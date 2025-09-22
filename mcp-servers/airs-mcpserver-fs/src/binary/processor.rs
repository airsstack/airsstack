//! Binary file processing coordinator

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: Third-party crate imports
use anyhow::Result;

// Layer 3: Internal module imports
use crate::binary::format::{FileFormat, FormatDetector};
use crate::config::settings::BinaryConfig;

/// Main binary file processing coordinator
/// Security Hardened: Binary processing disabled for security reasons
#[derive(Debug)]
pub struct BinaryProcessor {
    format_detector: FormatDetector,
    /// Configuration kept for API compatibility but binary processing is disabled
    #[allow(dead_code)]
    config: BinaryConfig,
}

/// Result of binary file processing
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Detected file format
    pub format: FileFormat,
    /// Size of processed data
    pub size: usize,
    /// Processing metadata (thumbnails, extracted text, etc.)
    pub metadata: ProcessingMetadata,
}

/// Metadata from binary processing
#[derive(Debug, Clone, Default)]
pub struct ProcessingMetadata {
    /// Thumbnail data for images (base64 encoded)
    pub thumbnail: Option<String>,
    /// Extracted text content
    pub text_content: Option<String>,
    /// Image dimensions (width, height)
    pub dimensions: Option<(u32, u32)>,
    /// Additional format-specific metadata
    pub properties: std::collections::HashMap<String, String>,
}

impl BinaryProcessor {
    /// Create a new binary processor with configuration
    pub fn new(config: BinaryConfig) -> Self {
        Self {
            format_detector: FormatDetector::new(),
            config,
        }
    }

    /// Process binary file data based on format and configuration
    /// SECURITY HARDENING: All binary processing is disabled for security
    pub async fn process_file_data(&self, data: &[u8], path: &Path) -> Result<ProcessingResult> {
        // Detect file format first
        let format = self.format_detector.detect_from_bytes(data);

        // SECURITY: Reject all binary file processing
        match format {
            FileFormat::Jpeg
            | FileFormat::Png
            | FileFormat::Gif
            | FileFormat::WebP
            | FileFormat::Tiff
            | FileFormat::Bmp => Err(anyhow::anyhow!(
                "Binary file processing disabled for security: {} (detected format: {:?})",
                path.display(),
                format
            )),
            FileFormat::Pdf => Err(anyhow::anyhow!(
                "PDF processing disabled for security: {} (detected format: {:?})",
                path.display(),
                format
            )),
            FileFormat::Text => {
                // Text files are allowed - basic processing only
                let result = ProcessingResult {
                    format,
                    size: data.len(),
                    metadata: ProcessingMetadata {
                        text_content: std::str::from_utf8(data).ok().map(|s| s.to_string()),
                        ..Default::default()
                    },
                };
                Ok(result)
            }
            FileFormat::Unknown => {
                // Unknown formats: check if they might be binary
                if data
                    .iter()
                    .any(|&b| b > 127 || (b < 32 && b != b'\n' && b != b'\r' && b != b'\t'))
                {
                    Err(anyhow::anyhow!(
                        "Unknown binary file processing disabled for security: {}",
                        path.display()
                    ))
                } else {
                    // Likely text content - allow basic processing
                    let result = ProcessingResult {
                        format,
                        size: data.len(),
                        metadata: ProcessingMetadata {
                            text_content: std::str::from_utf8(data).ok().map(|s| s.to_string()),
                            ..Default::default()
                        },
                    };
                    Ok(result)
                }
            }
        }
    }

    /// Check if a file format can be processed with current configuration
    /// SECURITY HARDENING: Only text files are allowed for processing
    pub fn can_process(&self, format: FileFormat) -> bool {
        match format {
            // All binary formats are disabled for security
            FileFormat::Jpeg
            | FileFormat::Png
            | FileFormat::Gif
            | FileFormat::WebP
            | FileFormat::Tiff
            | FileFormat::Bmp => false,
            FileFormat::Pdf => false, // PDF processing disabled for security
            FileFormat::Text => true, // Only text files are allowed
            FileFormat::Unknown => false, // Unknown formats rejected for security
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::settings::BinaryConfig;
    use std::path::PathBuf;

    fn create_test_config() -> BinaryConfig {
        BinaryConfig {
            max_file_size: 1024 * 1024,       // 1MB
            binary_processing_disabled: true, // Security hardening - always disabled
        }
    }

    #[test]
    fn test_binary_processor_creation() {
        let config = create_test_config();
        let processor = BinaryProcessor::new(config);
        assert!(processor.config.binary_processing_disabled);
    }

    #[tokio::test]
    async fn test_process_text_data() {
        let config = create_test_config();
        let processor = BinaryProcessor::new(config);

        let text_data = b"Hello, world!";
        let path = PathBuf::from("test.txt");

        let result = processor.process_file_data(text_data, &path).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.format, FileFormat::Text);
        assert_eq!(result.size, text_data.len());
        assert_eq!(
            result.metadata.text_content,
            Some("Hello, world!".to_string())
        );
    }

    #[tokio::test]
    async fn test_binary_file_rejection_over_size_limit() {
        let mut config = create_test_config();
        config.max_file_size = 10; // Very small limit
        let processor = BinaryProcessor::new(config);

        // Create binary data (simulating JPEG content)
        let large_data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG header
        let path = PathBuf::from("large.jpg");

        let result = processor.process_file_data(&large_data, &path).await;
        assert!(result.is_err());
        // Should fail due to binary restriction, not size limit
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Binary file processing disabled"));
    }

    #[test]
    fn test_can_process_security_policy() {
        let config = create_test_config();
        let processor = BinaryProcessor::new(config);

        // Security hardening: All binary formats should be rejected
        assert!(!processor.can_process(FileFormat::Jpeg));
        assert!(!processor.can_process(FileFormat::Pdf));
        assert!(!processor.can_process(FileFormat::Unknown));

        // Only text files are allowed
        assert!(processor.can_process(FileFormat::Text));
    }

    #[test]
    fn test_can_process_security_hardened() {
        let config = create_test_config();
        let processor = BinaryProcessor::new(config);

        // All binary formats should be rejected for security
        assert!(!processor.can_process(FileFormat::Jpeg));
        assert!(!processor.can_process(FileFormat::Png));
        assert!(!processor.can_process(FileFormat::Pdf));
        assert!(!processor.can_process(FileFormat::Unknown));

        // Only text files are allowed
        assert!(processor.can_process(FileFormat::Text));
    }
}

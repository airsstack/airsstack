//! Binary file processing coordinator

// Layer 1: Standard library imports
use std::path::Path;

// Layer 2: Third-party crate imports
use anyhow::Result;

// Layer 3: Internal module imports
use crate::binary::format::{FileFormat, FormatDetector};
use crate::config::settings::BinaryConfig;

/// Main binary file processing coordinator
#[derive(Debug)]
pub struct BinaryProcessor {
    format_detector: FormatDetector,
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
    pub async fn process_file_data(&self, data: &[u8], _path: &Path) -> Result<ProcessingResult> {
        // Check file size limits
        if data.len() > self.config.max_file_size as usize {
            return Err(anyhow::anyhow!(
                "File size ({} bytes) exceeds limit ({} bytes)",
                data.len(),
                self.config.max_file_size
            ));
        }

        // Detect file format
        let format = self.format_detector.detect_from_bytes(data);
        
        // Initialize result
        let mut result = ProcessingResult {
            format,
            size: data.len(),
            metadata: ProcessingMetadata::default(),
        };

        // Process based on format and configuration
        match format {
            FileFormat::Jpeg | FileFormat::Png | FileFormat::Gif | 
            FileFormat::WebP | FileFormat::Tiff | FileFormat::Bmp => {
                if self.config.enable_image_processing {
                    self.process_image_data(data, &mut result).await?;
                }
            },
            FileFormat::Pdf => {
                if self.config.enable_pdf_processing {
                    self.process_pdf_data(data, &mut result).await?;
                }
            },
            FileFormat::Text => {
                // Text files are processed directly without binary processing
                if let Ok(text) = std::str::from_utf8(data) {
                    result.metadata.text_content = Some(text.to_string());
                }
            },
            FileFormat::Unknown => {
                // Unknown formats are stored as-is
            },
        }

        Ok(result)
    }

    /// Process image data (placeholder implementation)
    async fn process_image_data(&self, _data: &[u8], result: &mut ProcessingResult) -> Result<()> {
        // TODO: Implement actual image processing in Phase 2
        // This would include:
        // - Loading image with `image` crate
        // - Extracting dimensions
        // - Generating thumbnails
        // - Reading EXIF metadata
        
        // Placeholder: just mark that image processing was attempted
        result.metadata.properties.insert(
            "processing_attempted".to_string(),
            "image".to_string(),
        );
        
        Ok(())
    }

    /// Process PDF data (placeholder implementation)
    async fn process_pdf_data(&self, _data: &[u8], result: &mut ProcessingResult) -> Result<()> {
        // TODO: Implement actual PDF processing in Phase 2
        // This would include:
        // - Text extraction from PDF
        // - Image extraction from PDF
        // - Metadata reading
        
        // Placeholder: just mark that PDF processing was attempted
        result.metadata.properties.insert(
            "processing_attempted".to_string(),
            "pdf".to_string(),
        );
        
        Ok(())
    }

    /// Check if a file format can be processed with current configuration
    pub fn can_process(&self, format: FileFormat) -> bool {
        match format {
            FileFormat::Jpeg | FileFormat::Png | FileFormat::Gif | 
            FileFormat::WebP | FileFormat::Tiff | FileFormat::Bmp => {
                self.config.enable_image_processing
            },
            FileFormat::Pdf => self.config.enable_pdf_processing,
            FileFormat::Text => true, // Text is always processable
            FileFormat::Unknown => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::BinaryConfig;
    use std::path::PathBuf;

    fn create_test_config() -> BinaryConfig {
        BinaryConfig {
            max_file_size: 1024 * 1024, // 1MB
            enable_image_processing: true,
            enable_pdf_processing: true,
        }
    }

    #[test]
    fn test_binary_processor_creation() {
        let config = create_test_config();
        let processor = BinaryProcessor::new(config);
        assert!(processor.config.enable_image_processing);
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
        assert_eq!(result.metadata.text_content, Some("Hello, world!".to_string()));
    }

    #[tokio::test]
    async fn test_file_size_limit() {
        let mut config = create_test_config();
        config.max_file_size = 10; // Very small limit
        let processor = BinaryProcessor::new(config);
        
        let large_data = vec![0u8; 100]; // Larger than limit
        let path = PathBuf::from("large.bin");
        
        let result = processor.process_file_data(&large_data, &path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds limit"));
    }

    #[test]
    fn test_can_process() {
        let config = create_test_config();
        let processor = BinaryProcessor::new(config);
        
        assert!(processor.can_process(FileFormat::Jpeg));
        assert!(processor.can_process(FileFormat::Pdf));
        assert!(processor.can_process(FileFormat::Text));
        assert!(!processor.can_process(FileFormat::Unknown));
    }

    #[test]
    fn test_can_process_with_disabled_features() {
        let mut config = create_test_config();
        config.enable_image_processing = false;
        config.enable_pdf_processing = false;
        let processor = BinaryProcessor::new(config);
        
        assert!(!processor.can_process(FileFormat::Jpeg));
        assert!(!processor.can_process(FileFormat::Pdf));
        assert!(processor.can_process(FileFormat::Text)); // Always processable
    }
}

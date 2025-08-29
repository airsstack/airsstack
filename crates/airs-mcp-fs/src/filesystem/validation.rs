//! Path validation and security checks

// Layer 1: Standard library imports
use std::path::{Path, PathBuf};

// Layer 2: Third-party crate imports
use anyhow::Result;
use path_clean::PathClean;
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;
use urlencoding::decode;

// Layer 3: Internal module imports
// (None needed yet)

/// Security validation errors that prevent information leakage
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Access denied")]
    AccessDenied,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Security policy violation")]
    PolicyViolation,
}

/// Path validation and security checks
#[derive(Debug)]
pub struct PathValidator {
    allowed_patterns: Vec<String>,
    denied_patterns: Vec<String>,
}

impl PathValidator {
    /// Create a new path validator with patterns
    pub fn new(allowed_patterns: Vec<String>, denied_patterns: Vec<String>) -> Self {
        Self {
            allowed_patterns,
            denied_patterns,
        }
    }

    /// Validate a path for security and access control with comprehensive security checks
    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, SecurityError> {
        let path_str = path.as_ref().to_string_lossy();

        // CRITICAL FIX 1: Null byte injection prevention
        if path_str.contains('\0') {
            return Err(SecurityError::InvalidInput);
        }

        // CRITICAL FIX 2: URL decode before validation to prevent encoding bypass
        let decoded_path = match decode(&path_str) {
            Ok(decoded) => decoded.to_string(),
            Err(_) => path_str.to_string(), // If decode fails, use original
        };

        // CRITICAL FIX 3: Additional null byte check after decoding
        if decoded_path.contains('\0') {
            return Err(SecurityError::InvalidInput);
        }

        // CRITICAL FIX 4: Unicode normalization to prevent Unicode bypass attacks
        let normalized_path: String = decoded_path.nfc().collect();

        // CRITICAL FIX 5: Control character filtering
        if normalized_path
            .chars()
            .any(|c| c.is_control() && c != '\t' && c != '\n' && c != '\r')
        {
            return Err(SecurityError::InvalidInput);
        }

        // CRITICAL FIX 6: Enhanced path traversal detection
        // Check for various traversal patterns before and after cleaning
        let dangerous_patterns = [
            "..",
            "..\\",
            "../",
            "..\\\\",
            "%2e%2e",
            "%2e%2e%2f",
            "%2e%2e%5c",
            "\u{2e}\u{2e}",
            "\u{ff0e}\u{ff0e}",
            "\\x2e\\x2e",
            "0x2e0x2e",
        ];

        let lower_path = normalized_path.to_lowercase();
        for pattern in &dangerous_patterns {
            if lower_path.contains(&pattern.to_lowercase()) {
                return Err(SecurityError::AccessDenied);
            }
        }

        // Convert to PathBuf and clean
        let path_buf = PathBuf::from(&normalized_path);
        let cleaned_path = path_buf.clean();

        // CRITICAL FIX 7: Post-cleaning traversal detection
        let cleaned_str = cleaned_path.to_string_lossy();
        if cleaned_str.contains("..") {
            return Err(SecurityError::AccessDenied);
        }

        // CRITICAL FIX 8: Additional security checks for absolute paths
        // For absolute paths, ensure they don't try to escape to sensitive system areas
        if cleaned_str.starts_with('/') {
            let sensitive_paths = ["/etc/", "/root/", "/sys/", "/proc/", "/dev/", "/boot/"];
            for sensitive in &sensitive_paths {
                if cleaned_str.starts_with(sensitive) {
                    return Err(SecurityError::AccessDenied);
                }
            }
        }

        // Check denied patterns first
        for pattern in &self.denied_patterns {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches_path(&cleaned_path) {
                    return Err(SecurityError::PolicyViolation);
                }
            }
        }

        // Check allowed patterns
        let mut allowed = false;
        for pattern in &self.allowed_patterns {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches_path(&cleaned_path) {
                    allowed = true;
                    break;
                }
            }
        }

        if !allowed {
            return Err(SecurityError::AccessDenied);
        }

        Ok(cleaned_path)
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        // SECURITY FIX: Secure defaults that allow reasonable development work
        Self::new(
            vec![
                "src/**".to_string(),
                "docs/**".to_string(),
                "examples/**".to_string(),
                "tests/**".to_string(),
                "*.md".to_string(),
                "*.txt".to_string(),
                "*.rs".to_string(),
                "**/*.rs".to_string(),
                "**/*.md".to_string(),
                "**/*.txt".to_string(),
                "**/tmp/**".to_string(), // Temp directories
                "/tmp/**".to_string(),   // System temp directory
            ],
            vec![
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/.env*".to_string(),
                "**/.*".to_string(),
                "**/*password*".to_string(),
                "**/*secret*".to_string(),
                "**/*key*".to_string(),
            ],
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_path_validator_creation() {
        let validator =
            PathValidator::new(vec!["src/**".to_string()], vec!["**/.git/**".to_string()]);
        assert_eq!(validator.allowed_patterns.len(), 1);
        assert_eq!(validator.denied_patterns.len(), 1);
    }

    #[test]
    fn test_valid_path() {
        let validator = PathValidator::default();
        let result = validator.validate_path("src/main.rs");
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_traversal_prevention() {
        let validator = PathValidator::default();
        let result = validator.validate_path("../../../etc/passwd");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::AccessDenied));
    }

    #[test]
    fn test_null_byte_injection_prevention() {
        let validator = PathValidator::default();

        // Test direct null byte
        let result = validator.validate_path("src/file\0.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::InvalidInput));

        // Test URL encoded null byte
        let result = validator.validate_path("src/file%00.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::InvalidInput));
    }

    #[test]
    fn test_unicode_normalization() {
        let validator = PathValidator::default();

        // Test Unicode dot-dot traversal
        let result = validator.validate_path("src/\u{2e}\u{2e}/../../etc/passwd");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::AccessDenied));
    }

    #[test]
    fn test_control_character_filtering() {
        let validator = PathValidator::default();

        // Test vertical tab (should be rejected)
        let result = validator.validate_path("src/file\x0B.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::InvalidInput));
    }

    #[test]
    fn test_denied_pattern() {
        let validator =
            PathValidator::new(vec!["**/*".to_string()], vec!["**/.git/**".to_string()]);
        let result = validator.validate_path(".git/config");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityError::PolicyViolation
        ));
    }

    #[test]
    fn test_allowed_pattern() {
        let validator = PathValidator::new(vec!["src/**".to_string()], vec![]);

        let valid_result = validator.validate_path("src/main.rs");
        assert!(valid_result.is_ok());

        let invalid_result = validator.validate_path("target/debug/app");
        assert!(invalid_result.is_err());
        assert!(matches!(
            invalid_result.unwrap_err(),
            SecurityError::AccessDenied
        ));
    }

    #[test]
    fn test_secure_defaults() {
        let validator = PathValidator::default();

        // Should allow safe files
        assert!(validator.validate_path("src/main.rs").is_ok());
        assert!(validator.validate_path("docs/README.md").is_ok());

        // Should deny dangerous patterns
        assert!(validator.validate_path(".env").is_err());
        assert!(validator.validate_path("target/debug/app").is_err());
        assert!(validator.validate_path(".git/config").is_err());
    }
}

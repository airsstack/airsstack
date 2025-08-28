//! Path validation and security checks

// Layer 1: Standard library imports
use std::path::{Path, PathBuf};

// Layer 2: Third-party crate imports
use anyhow::{anyhow, Result};
use path_clean::PathClean;

// Layer 3: Internal module imports
// (None needed yet)

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

    /// Validate a path for security and access control
    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = path.as_ref();

        // Clean and canonicalize the path to prevent traversal attacks
        let cleaned_path = path.clean();

        // Check for path traversal attempts
        if cleaned_path.to_string_lossy().contains("..") {
            return Err(anyhow!("Path traversal detected: {}", path.display()));
        }

        // Check denied patterns first
        for pattern in &self.denied_patterns {
            if glob::Pattern::new(pattern)?.matches_path(&cleaned_path) {
                return Err(anyhow!(
                    "Path denied by security policy: {}",
                    path.display()
                ));
            }
        }

        // Check allowed patterns
        let mut allowed = false;
        for pattern in &self.allowed_patterns {
            if glob::Pattern::new(pattern)?.matches_path(&cleaned_path) {
                allowed = true;
                break;
            }
        }

        if !allowed {
            return Err(anyhow!("Path not in allowed list: {}", path.display()));
        }

        Ok(cleaned_path)
    }
}

impl Default for PathValidator {
    fn default() -> Self {
        Self::new(
            vec!["**/*".to_string()], // Allow all by default
            vec![],                   // No denied patterns by default
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path traversal detected"));
    }

    #[test]
    fn test_denied_pattern() {
        let validator =
            PathValidator::new(vec!["**/*".to_string()], vec!["**/.git/**".to_string()]);
        let result = validator.validate_path(".git/config");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path denied by security policy"));
    }

    #[test]
    fn test_allowed_pattern() {
        let validator = PathValidator::new(vec!["src/**".to_string()], vec![]);

        let valid_result = validator.validate_path("src/main.rs");
        assert!(valid_result.is_ok());

        let invalid_result = validator.validate_path("target/debug/app");
        assert!(invalid_result.is_err());
        assert!(invalid_result
            .unwrap_err()
            .to_string()
            .contains("Path not in allowed list"));
    }
}

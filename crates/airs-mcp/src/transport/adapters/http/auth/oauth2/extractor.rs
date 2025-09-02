//! HTTP Request Data Extraction
//!
//! Utilities for extracting OAuth2-relevant data from HTTP requests,
//! including bearer tokens and MCP method names.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use super::error::HttpAuthError;

/// HTTP data extraction utilities
pub struct HttpExtractor;

impl HttpExtractor {
    /// Extract bearer token from HTTP Authorization header
    ///
    /// Supports standard "Bearer <token>" format with case-insensitive header matching.
    ///
    /// # Arguments
    /// * `headers` - HTTP headers from the request
    ///
    /// # Returns
    /// * Bearer token if found and valid format
    /// * HttpAuthError if missing, malformed, or invalid
    pub fn extract_bearer_token(
        headers: &HashMap<String, String>,
    ) -> Result<String, HttpAuthError> {
        // Try different case variations of Authorization header
        let auth_header = headers
            .get("Authorization")
            .or_else(|| headers.get("authorization"))
            .or_else(|| headers.get("AUTHORIZATION"))
            .ok_or_else(|| HttpAuthError::MissingHeader {
                header: "Authorization".to_string(),
            })?;

        // Parse Bearer token format
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if token.trim().is_empty() {
                return Err(HttpAuthError::MalformedAuth {
                    message: "Empty bearer token".to_string(),
                });
            }
            Ok(token.trim().to_string())
        } else if let Some(token) = auth_header.strip_prefix("bearer ") {
            // Handle lowercase bearer prefix
            if token.trim().is_empty() {
                return Err(HttpAuthError::MalformedAuth {
                    message: "Empty bearer token".to_string(),
                });
            }
            Ok(token.trim().to_string())
        } else {
            Err(HttpAuthError::MalformedAuth {
                message: format!("Expected 'Bearer <token>' format, got: {auth_header}"),
            })
        }
    }

    /// Extract method name from HTTP request path
    ///
    /// Uses path-based method extraction for MCP protocol compliance.
    /// Expects paths like "/mcp/tools/call" or "/api/v1/resources/list".
    ///
    /// # Arguments
    /// * `path` - HTTP request path
    ///
    /// # Returns
    /// * Method name extracted from path, or None if not extractable
    pub fn extract_method(path: &str) -> Option<String> {
        // Handle MCP-style paths: /mcp/tools/call -> tools/call
        if let Some(mcp_path) = path.strip_prefix("/mcp/") {
            return Some(mcp_path.to_string());
        }

        // Handle API-style paths: /api/v1/tools/call -> tools/call
        if let Some(api_path) = path.strip_prefix("/api/v1/") {
            return Some(api_path.to_string());
        }

        // Handle root-level paths: /tools/call -> tools/call
        if let Some(root_path) = path.strip_prefix('/') {
            if !root_path.is_empty() {
                return Some(root_path.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token_success() {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Bearer test_token_123".to_string(),
        );

        let result = HttpExtractor::extract_bearer_token(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_token_123");
    }

    #[test]
    fn test_extract_bearer_token_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert(
            "authorization".to_string(),
            "bearer test_token_123".to_string(),
        );

        let result = HttpExtractor::extract_bearer_token(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_token_123");
    }

    #[test]
    fn test_extract_bearer_token_missing_header() {
        let headers = HashMap::new();

        let result = HttpExtractor::extract_bearer_token(&headers);
        assert!(result.is_err());
        match result.unwrap_err() {
            HttpAuthError::MissingHeader { header } => {
                assert_eq!(header, "Authorization");
            }
            _ => panic!("Expected MissingHeader error"),
        }
    }

    #[test]
    fn test_extract_bearer_token_malformed() {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            "Basic dXNlcjpwYXNz".to_string(),
        );

        let result = HttpExtractor::extract_bearer_token(&headers);
        assert!(result.is_err());
        match result.unwrap_err() {
            HttpAuthError::MalformedAuth { message } => {
                assert!(message.contains("Expected 'Bearer <token>' format"));
            }
            _ => panic!("Expected MalformedAuth error"),
        }
    }

    #[test]
    fn test_extract_method_mcp_path() {
        assert_eq!(
            HttpExtractor::extract_method("/mcp/tools/call"),
            Some("tools/call".to_string())
        );
        assert_eq!(
            HttpExtractor::extract_method("/mcp/resources/list"),
            Some("resources/list".to_string())
        );
    }

    #[test]
    fn test_extract_method_api_path() {
        assert_eq!(
            HttpExtractor::extract_method("/api/v1/tools/call"),
            Some("tools/call".to_string())
        );
        assert_eq!(
            HttpExtractor::extract_method("/api/v1/resources/list"),
            Some("resources/list".to_string())
        );
    }

    #[test]
    fn test_extract_method_root_path() {
        assert_eq!(
            HttpExtractor::extract_method("/tools/call"),
            Some("tools/call".to_string())
        );
        assert_eq!(
            HttpExtractor::extract_method("/resources/list"),
            Some("resources/list".to_string())
        );
    }

    #[test]
    fn test_extract_method_empty_path() {
        assert_eq!(HttpExtractor::extract_method(""), None);
        assert_eq!(HttpExtractor::extract_method("/"), None);
    }
}

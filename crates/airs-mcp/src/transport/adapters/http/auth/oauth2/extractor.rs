//! HTTP Request Data Extraction
//!
//! Utilities for extracting OAuth2-relevant data from HTTP requests.
//! This module focuses on authentication data extraction (bearer tokens)
//! and delegates method extraction to the authorization layer.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use super::error::HttpAuthError;

/// HTTP authentication data extraction utilities
///
/// Provides utilities for extracting authentication data from HTTP requests.
/// Method extraction is handled by the authorization layer to maintain proper
/// separation of concerns and fix the OAuth2 JSON-RPC method extraction bug.
pub struct HttpExtractor;

impl HttpExtractor {
    /// Extract bearer token from HTTP Authorization header
    ///
    /// Supports standard "Bearer \<token\>" format with case-insensitive header matching.
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
}

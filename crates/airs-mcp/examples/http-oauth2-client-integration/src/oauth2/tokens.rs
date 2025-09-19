// Token management for OAuth2 tokens

// Standard library imports
use std::sync::{Arc, Mutex};

// Third-party crate imports
use chrono::{DateTime, Utc};

// Internal module imports
use crate::{OAuth2IntegrationError, TokenStore};

/// Token manager for handling OAuth2 token storage and validation
pub struct TokenManager {
    tokens: Arc<Mutex<Option<TokenStore>>>,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the current tokens
    pub fn set_tokens(&self, tokens: TokenStore) {
        let mut guard = self.tokens.lock().unwrap();
        *guard = Some(tokens);
    }

    /// Get the current tokens (if any)
    pub fn get_current_tokens(&self) -> Option<TokenStore> {
        let guard = self.tokens.lock().unwrap();
        guard.clone()
    }

    /// Check if we have valid, non-expired tokens
    pub fn has_valid_tokens(&self) -> bool {
        let guard = self.tokens.lock().unwrap();
        if let Some(tokens) = &*guard {
            // Check if token is not expired (with 60 second buffer)
            let now = Utc::now();
            let buffer = chrono::Duration::seconds(60);
            tokens.expires_at > (now + buffer)
        } else {
            false
        }
    }

    /// Check if the current access token is expired
    pub fn is_token_expired(&self) -> bool {
        let guard = self.tokens.lock().unwrap();
        if let Some(tokens) = &*guard {
            let now = Utc::now();
            tokens.expires_at <= now
        } else {
            true // No token is considered expired
        }
    }

    /// Get the remaining time until token expiration
    pub fn time_until_expiration(&self) -> Option<chrono::Duration> {
        let guard = self.tokens.lock().unwrap();
        if let Some(tokens) = &*guard {
            let now = Utc::now();
            if tokens.expires_at > now {
                Some(tokens.expires_at - now)
            } else {
                Some(chrono::Duration::zero())
            }
        } else {
            None
        }
    }

    /// Clear all stored tokens
    pub fn clear_tokens(&self) {
        let mut guard = self.tokens.lock().unwrap();
        *guard = None;
    }

    /// Get access token if valid, or return error
    pub fn get_valid_access_token(&self) -> Result<String, OAuth2IntegrationError> {
        if !self.has_valid_tokens() {
            return Err(OAuth2IntegrationError::TokenExpired);
        }

        let guard = self.tokens.lock().unwrap();
        if let Some(tokens) = &*guard {
            Ok(tokens.access_token.clone())
        } else {
            Err(OAuth2IntegrationError::AuthenticationRequired)
        }
    }

    /// Get refresh token if available
    pub fn get_refresh_token(&self) -> Option<String> {
        let guard = self.tokens.lock().unwrap();
        if let Some(tokens) = &*guard {
            tokens.refresh_token.clone()
        } else {
            None
        }
    }

    /// Check if we have a refresh token
    pub fn has_refresh_token(&self) -> bool {
        self.get_refresh_token().is_some()
    }

    /// Get token scope
    pub fn get_token_scope(&self) -> Option<String> {
        let guard = self.tokens.lock().unwrap();
        if let Some(tokens) = &*guard {
            Some(tokens.scope.clone())
        } else {
            None
        }
    }

    /// Validate that the token has the required scope
    pub fn has_required_scope(&self, required_scope: &str) -> bool {
        if let Some(current_scope) = self.get_token_scope() {
            // Simple scope validation - check if required scope is present
            // In production, this should handle scope hierarchies and wildcards
            current_scope.split_whitespace().any(|scope| scope == required_scope)
        } else {
            false
        }
    }

    /// Get token information for debugging
    pub fn get_token_info(&self) -> Option<TokenInfo> {
        let guard = self.tokens.lock().unwrap();
        if let Some(tokens) = &*guard {
            Some(TokenInfo {
                has_access_token: !tokens.access_token.is_empty(),
                has_refresh_token: tokens.refresh_token.is_some(),
                expires_at: tokens.expires_at,
                scope: tokens.scope.clone(),
                is_expired: self.is_token_expired(),
                time_until_expiration: self.time_until_expiration(),
            })
        } else {
            None
        }
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Token information for debugging and monitoring
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub has_access_token: bool,
    pub has_refresh_token: bool,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
    pub is_expired: bool,
    pub time_until_expiration: Option<chrono::Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tokens() -> TokenStore {
        TokenStore {
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            scope: "mcp:read mcp:write".to_string(),
        }
    }

    fn create_expired_tokens() -> TokenStore {
        TokenStore {
            access_token: "expired-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            scope: "mcp:read".to_string(),
        }
    }

    #[test]
    fn test_token_manager_creation() {
        let manager = TokenManager::new();
        assert!(!manager.has_valid_tokens());
        assert!(manager.get_current_tokens().is_none());
    }

    #[test]
    fn test_token_storage_and_retrieval() {
        let manager = TokenManager::new();
        let tokens = create_test_tokens();
        
        manager.set_tokens(tokens.clone());
        
        assert!(manager.has_valid_tokens());
        let retrieved = manager.get_current_tokens().unwrap();
        assert_eq!(retrieved.access_token, tokens.access_token);
        assert_eq!(retrieved.refresh_token, tokens.refresh_token);
    }

    #[test]
    fn test_expired_token_detection() {
        let manager = TokenManager::new();
        let expired_tokens = create_expired_tokens();
        
        manager.set_tokens(expired_tokens);
        
        assert!(!manager.has_valid_tokens());
        assert!(manager.is_token_expired());
    }

    #[test]
    fn test_scope_validation() {
        let manager = TokenManager::new();
        let tokens = create_test_tokens();
        
        manager.set_tokens(tokens);
        
        assert!(manager.has_required_scope("mcp:read"));
        assert!(manager.has_required_scope("mcp:write"));
        assert!(!manager.has_required_scope("admin:write"));
    }

    #[test]
    fn test_token_info() {
        let manager = TokenManager::new();
        let tokens = create_test_tokens();
        
        manager.set_tokens(tokens);
        
        let info = manager.get_token_info().unwrap();
        assert!(info.has_access_token);
        assert!(info.has_refresh_token);
        assert!(!info.is_expired);
        assert!(info.time_until_expiration.is_some());
    }

    #[test]
    fn test_clear_tokens() {
        let manager = TokenManager::new();
        let tokens = create_test_tokens();
        
        manager.set_tokens(tokens);
        assert!(manager.has_valid_tokens());
        
        manager.clear_tokens();
        assert!(!manager.has_valid_tokens());
        assert!(manager.get_current_tokens().is_none());
    }
}
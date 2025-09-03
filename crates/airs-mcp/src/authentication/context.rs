//! Generic Authentication Context
//!
//! Provides generic authentication context that supports any authentication
//! strategy with their own custom data structures.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};

// Layer 3: Internal module imports
use crate::authentication::metadata::AuthMetadata;
use crate::authentication::method::AuthMethod;

/// Generic authentication context supporting any authentication method
///
/// Uses compile-time generics for zero-cost abstractions following
/// workspace standard §6. Each authentication strategy defines its own data type.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::authentication::{AuthContext, AuthMethod};
///
/// // OAuth2 strategy defines its own data structure
/// #[derive(Debug, Clone)]
/// struct OAuth2Data {
///     user_id: String,
///     scopes: Vec<String>,
/// }
///
/// let oauth2_data = OAuth2Data {
///     user_id: "user123".to_string(),
///     scopes: vec!["read".to_string(), "write".to_string()],
/// };
///
/// let context = AuthContext::new(
///     AuthMethod::new("oauth2"),
///     oauth2_data,
/// );
///
/// // API Key strategy defines its own data structure
/// #[derive(Debug, Clone)]
/// struct ApiKeyData {
///     key_id: String,
///     permissions: Vec<String>,
/// }
///
/// let api_data = ApiKeyData {
///     key_id: "key_123".to_string(),
///     permissions: vec!["admin".to_string()],
/// };
///
/// let context = AuthContext::new(
///     AuthMethod::new("apikey"),
///     api_data,
/// );
/// ```
#[derive(Debug, Clone)]
pub struct AuthContext<T> {
    /// Authentication method identifier
    pub method: AuthMethod,
    
    /// Strategy-specific authentication data
    pub auth_data: T,
    
    /// Extensible metadata
    pub metadata: AuthMetadata,
    
    /// Request timestamp
    pub created_at: DateTime<Utc>,
    
    /// Optional expiration (strategy-dependent)
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Request ID for audit logging
    pub request_id: Option<String>,
}

impl<T> AuthContext<T> {
    /// Create new authentication context
    pub fn new(method: AuthMethod, auth_data: T) -> Self {
        Self {
            method,
            auth_data,
            metadata: AuthMetadata::new(),
            created_at: Utc::now(),
            expires_at: None,
            request_id: None,
        }
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Set request ID
    pub fn with_request_id<S: Into<String>>(mut self, request_id: S) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: AuthMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Add metadata attribute
    pub fn add_metadata<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata.insert(key, value);
        self
    }
    
    /// Check if authentication is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Transform auth data type (useful for conversions)
    pub fn map_data<U, F>(self, f: F) -> AuthContext<U>
    where
        F: FnOnce(T) -> U,
    {
        AuthContext {
            method: self.method,
            auth_data: f(self.auth_data),
            metadata: self.metadata,
            created_at: self.created_at,
            expires_at: self.expires_at,
            request_id: self.request_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestAuthData {
        user_id: String,
    }

    #[test]
    fn test_auth_context_creation() {
        let method = AuthMethod::new("test");
        let data = TestAuthData {
            user_id: "user123".to_string(),
        };
        
        let context = AuthContext::new(method.clone(), data.clone());
        
        assert_eq!(context.method, method);
        assert_eq!(context.auth_data, data);
        assert!(!context.is_expired());
    }

    #[test]
    fn test_auth_context_with_expiration() {
        let method = AuthMethod::new("test");
        let data = TestAuthData {
            user_id: "user123".to_string(),
        };
        let future_time = Utc::now() + chrono::Duration::hours(1);
        
        let context = AuthContext::new(method, data)
            .with_expiration(future_time);
        
        assert_eq!(context.expires_at, Some(future_time));
        assert!(!context.is_expired());
    }

    #[test]
    fn test_auth_context_metadata() {
        let method = AuthMethod::new("test");
        let data = TestAuthData {
            user_id: "user123".to_string(),
        };
        
        let context = AuthContext::new(method, data)
            .add_metadata("client_ip", "192.168.1.1")
            .add_metadata("user_agent", "test-agent");
        
        assert_eq!(context.metadata.get("client_ip"), Some(&"192.168.1.1".to_string()));
        assert_eq!(context.metadata.get("user_agent"), Some(&"test-agent".to_string()));
    }

    #[test]
    fn test_auth_context_map_data() {
        let method = AuthMethod::new("test");
        let data = TestAuthData {
            user_id: "user123".to_string(),
        };
        
        let context = AuthContext::new(method.clone(), data);
        let mapped = context.map_data(|d| d.user_id);
        
        assert_eq!(mapped.method, method);
        assert_eq!(mapped.auth_data, "user123");
    }
}

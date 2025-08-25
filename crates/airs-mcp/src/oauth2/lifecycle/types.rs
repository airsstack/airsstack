//! Token Lifecycle Management Types
//!
//! This module defines the core types used throughout the token lifecycle
//! management system, including cache entries, requests, responses, and events.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::oauth2::AuthContext;

/// Token cache key for identifying cached tokens
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenCacheKey {
    /// User identifier
    pub user_id: String,

    /// Client identifier
    pub client_id: String,

    /// Optional scope identifier for fine-grained caching
    pub scope: Option<String>,
}

impl TokenCacheKey {
    /// Create a new token cache key
    pub fn new(user_id: String, client_id: String) -> Self {
        Self {
            user_id,
            client_id,
            scope: None,
        }
    }

    /// Create a new token cache key with scope
    pub fn with_scope(user_id: String, client_id: String, scope: String) -> Self {
        Self {
            user_id,
            client_id,
            scope: Some(scope),
        }
    }
}

impl fmt::Display for TokenCacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scope {
            Some(scope) => write!(f, "{}:{}:{}", self.user_id, self.client_id, scope),
            None => write!(f, "{}:{}", self.user_id, self.client_id),
        }
    }
}

impl TokenCacheKey {
    /// Parse from string representation
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.splitn(3, ':').collect(); // Use splitn to limit splits
        match parts.len() {
            2 => Ok(Self::new(parts[0].to_string(), parts[1].to_string())),
            3 => Ok(Self::with_scope(
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(), // This will contain the rest including any colons
            )),
            _ => Err("Invalid token cache key format".to_string()),
        }
    }
}

/// Token cache entry containing authentication context and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCacheEntry {
    /// Authentication context
    pub auth_context: AuthContext,

    /// Cache entry creation time
    pub created_at: DateTime<Utc>,

    /// Last access time
    pub last_accessed: DateTime<Utc>,

    /// Number of times this entry has been accessed
    pub access_count: u64,

    /// Refresh token (if available)
    pub refresh_token: Option<String>,

    /// Cache entry metadata
    pub metadata: HashMap<String, String>,
}

impl TokenCacheEntry {
    /// Create a new cache entry
    pub fn new(auth_context: AuthContext) -> Self {
        let now = Utc::now();
        Self {
            auth_context,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            refresh_token: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new cache entry with refresh token
    pub fn with_refresh_token(auth_context: AuthContext, refresh_token: String) -> Self {
        let mut entry = Self::new(auth_context);
        entry.refresh_token = Some(refresh_token);
        entry
    }

    /// Update last accessed time and increment access count
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        self.auth_context.is_expired()
    }

    /// Check if the token should be refreshed
    pub fn should_refresh(&self, threshold: Duration) -> bool {
        if let Some(exp) = self.auth_context.expires_at {
            let now = Utc::now();
            let time_until_expiry = exp.signed_duration_since(now);
            time_until_expiry.to_std().unwrap_or(Duration::ZERO) < threshold
        } else {
            false
        }
    }

    /// Get cache entry age
    pub fn age(&self) -> Duration {
        let now = Utc::now();
        now.signed_duration_since(self.created_at)
            .to_std()
            .unwrap_or(Duration::ZERO)
    }

    /// Get time since last access
    pub fn time_since_last_access(&self) -> Duration {
        let now = Utc::now();
        now.signed_duration_since(self.last_accessed)
            .to_std()
            .unwrap_or(Duration::ZERO)
    }
}

/// Request for refreshing an access token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    /// The refresh token to use
    pub refresh_token: String,

    /// Client ID making the request
    pub client_id: String,

    /// Optional scope for the new token
    pub scope: Option<String>,

    /// Additional parameters for the refresh request
    pub additional_params: HashMap<String, String>,
}

impl RefreshTokenRequest {
    /// Create a new refresh token request
    pub fn new(refresh_token: String, client_id: String) -> Self {
        Self {
            refresh_token,
            client_id,
            scope: None,
            additional_params: HashMap::new(),
        }
    }

    /// Set the scope for the refresh request
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Add an additional parameter
    pub fn with_param(mut self, key: String, value: String) -> Self {
        self.additional_params.insert(key, value);
        self
    }
}

/// Response from a token refresh operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    /// New access token
    pub access_token: String,

    /// Token type (usually "Bearer")
    pub token_type: String,

    /// Token expiration time in seconds
    pub expires_in: Option<u64>,

    /// New refresh token (if provided)
    pub refresh_token: Option<String>,

    /// Granted scopes
    pub scope: Option<String>,

    /// Additional response data
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl RefreshTokenResponse {
    /// Create a new refresh token response
    pub fn new(access_token: String, token_type: String) -> Self {
        Self {
            access_token,
            token_type,
            expires_in: None,
            refresh_token: None,
            scope: None,
            additional_data: HashMap::new(),
        }
    }

    /// Calculate expiration time from expires_in
    pub fn calculate_expiration(&self) -> Option<DateTime<Utc>> {
        self.expires_in
            .map(|seconds| Utc::now() + chrono::Duration::seconds(seconds as i64))
    }
}

/// Token status information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenStatus {
    /// Token is valid and not expired
    Valid {
        expires_at: Option<DateTime<Utc>>,
        scopes: Vec<String>,
    },

    /// Token is expired but refresh is available
    Expired {
        expired_at: DateTime<Utc>,
        refresh_available: bool,
    },

    /// Token is invalid or revoked
    Invalid { reason: String },

    /// Token not found in cache
    NotFound,

    /// Token is in the process of being refreshed
    Refreshing { started_at: DateTime<Utc> },
}

impl TokenStatus {
    /// Check if the token is usable
    pub fn is_usable(&self) -> bool {
        matches!(self, TokenStatus::Valid { .. })
    }

    /// Check if the token can be refreshed
    pub fn can_refresh(&self) -> bool {
        matches!(
            self,
            TokenStatus::Expired {
                refresh_available: true,
                ..
            }
        )
    }

    /// Check if the token is being refreshed
    pub fn is_refreshing(&self) -> bool {
        matches!(self, TokenStatus::Refreshing { .. })
    }
}

/// Token lifecycle events for monitoring and logging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenLifecycleEvent {
    /// Token was created and cached
    TokenCreated {
        key: TokenCacheKey,
        expires_at: Option<DateTime<Utc>>,
        timestamp: DateTime<Utc>,
    },

    /// Token was successfully refreshed
    TokenRefreshed {
        key: TokenCacheKey,
        old_expires_at: Option<DateTime<Utc>>,
        new_expires_at: Option<DateTime<Utc>>,
        timestamp: DateTime<Utc>,
    },

    /// Token expired and was removed from cache
    TokenExpired {
        key: TokenCacheKey,
        expired_at: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },

    /// Token was manually invalidated
    TokenInvalidated {
        key: TokenCacheKey,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Token validation failed
    ValidationFailed {
        key: Option<TokenCacheKey>,
        error: String,
        timestamp: DateTime<Utc>,
    },

    /// Cache hit occurred
    CacheHit {
        key: TokenCacheKey,
        timestamp: DateTime<Utc>,
    },

    /// Cache miss occurred
    CacheMiss {
        key: TokenCacheKey,
        timestamp: DateTime<Utc>,
    },

    /// Cache maintenance operation
    CacheMaintenance {
        operation: String,
        affected_count: u64,
        timestamp: DateTime<Utc>,
    },
}

impl TokenLifecycleEvent {
    /// Create a token created event
    pub fn token_created(key: TokenCacheKey, expires_at: Option<DateTime<Utc>>) -> Self {
        Self::TokenCreated {
            key,
            expires_at,
            timestamp: Utc::now(),
        }
    }

    /// Create a token refreshed event
    pub fn token_refreshed(
        key: TokenCacheKey,
        old_expires_at: Option<DateTime<Utc>>,
        new_expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self::TokenRefreshed {
            key,
            old_expires_at,
            new_expires_at,
            timestamp: Utc::now(),
        }
    }

    /// Create a token expired event
    pub fn token_expired(key: TokenCacheKey, expired_at: DateTime<Utc>) -> Self {
        Self::TokenExpired {
            key,
            expired_at,
            timestamp: Utc::now(),
        }
    }

    /// Create a token invalidated event
    pub fn token_invalidated(key: TokenCacheKey, reason: String) -> Self {
        Self::TokenInvalidated {
            key,
            reason,
            timestamp: Utc::now(),
        }
    }

    /// Get the event timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::TokenCreated { timestamp, .. }
            | Self::TokenRefreshed { timestamp, .. }
            | Self::TokenExpired { timestamp, .. }
            | Self::TokenInvalidated { timestamp, .. }
            | Self::ValidationFailed { timestamp, .. }
            | Self::CacheHit { timestamp, .. }
            | Self::CacheMiss { timestamp, .. }
            | Self::CacheMaintenance { timestamp, .. } => *timestamp,
        }
    }
}

/// Token cache metrics for monitoring and observability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCacheMetrics {
    /// Total number of cached tokens
    pub total_tokens: u64,

    /// Number of expired tokens
    pub expired_tokens: u64,

    /// Number of cache hits
    pub cache_hits: u64,

    /// Number of cache misses
    pub cache_misses: u64,

    /// Number of successful refreshes
    pub successful_refreshes: u64,

    /// Number of failed refreshes
    pub failed_refreshes: u64,

    /// Average token age in seconds
    pub average_token_age_seconds: f64,

    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,

    /// Memory usage in bytes (if applicable)
    pub memory_usage_bytes: Option<u64>,

    /// Timestamp of metrics collection
    pub collected_at: DateTime<Utc>,
}

impl TokenCacheMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            collected_at: Utc::now(),
            ..Default::default()
        }
    }

    /// Calculate hit ratio from hits and misses
    pub fn calculate_hit_ratio(&mut self) {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            self.hit_ratio = self.cache_hits as f64 / total_requests as f64;
        } else {
            self.hit_ratio = 0.0;
        }
    }

    /// Update timestamp to current time
    pub fn update_timestamp(&mut self) {
        self.collected_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_auth_context(user_id: &str, client_id: &str) -> AuthContext {
        let jwt_claims = crate::oauth2::types::JwtClaims {
            sub: user_id.to_string(),
            aud: Some(client_id.to_string()),
            iss: Some("test-issuer".to_string()),
            exp: Some(chrono::Utc::now().timestamp() + 3600),
            nbf: None,
            iat: Some(chrono::Utc::now().timestamp()),
            jti: Some("token123".to_string()),
            scope: Some("read write".to_string()),
            scopes: None,
        };

        AuthContext::new(jwt_claims, vec!["read".to_string(), "write".to_string()])
    }

    #[test]
    fn test_token_cache_key_string_conversion() {
        let key = TokenCacheKey::new("user123".to_string(), "client456".to_string());
        let key_str = key.to_string();
        let parsed_key = TokenCacheKey::from_string(&key_str).unwrap();
        assert_eq!(key, parsed_key);
    }

    #[test]
    fn test_token_cache_key_with_scope() {
        let key = TokenCacheKey::with_scope(
            "user123".to_string(),
            "client456".to_string(),
            "read:tools".to_string(),
        );
        let key_str = key.to_string();
        let parsed_key = TokenCacheKey::from_string(&key_str).unwrap();
        assert_eq!(key, parsed_key);
    }

    #[test]
    fn test_token_status_usability() {
        let valid = TokenStatus::Valid {
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            scopes: vec!["read".to_string()],
        };
        assert!(valid.is_usable());

        let expired = TokenStatus::Expired {
            expired_at: Utc::now() - chrono::Duration::hours(1),
            refresh_available: true,
        };
        assert!(!expired.is_usable());
        assert!(expired.can_refresh());
    }

    #[test]
    fn test_cache_metrics_hit_ratio() {
        let mut metrics = TokenCacheMetrics::new();
        metrics.cache_hits = 80;
        metrics.cache_misses = 20;
        metrics.calculate_hit_ratio();
        assert_eq!(metrics.hit_ratio, 0.8);
    }

    #[test]
    fn test_token_cache_entry_expiration() {
        let mut auth_context = create_test_auth_context("user123", "client456");
        auth_context.expires_at = Some(Utc::now() - chrono::Duration::hours(1)); // Expired

        let entry = TokenCacheEntry::new(auth_context);
        assert!(entry.is_expired());
    }

    #[test]
    fn test_token_cache_entry_refresh_threshold() {
        let mut auth_context = create_test_auth_context("user123", "client456");
        auth_context.expires_at = Some(Utc::now() + chrono::Duration::minutes(2)); // Expires in 2 minutes

        let entry = TokenCacheEntry::new(auth_context);
        let threshold = Duration::from_secs(300); // 5 minutes
        assert!(entry.should_refresh(threshold));
    }

    #[test]
    fn test_refresh_token_request_builder() {
        let request = RefreshTokenRequest::new("refresh123".to_string(), "client456".to_string())
            .with_scope("read:tools".to_string())
            .with_param("custom".to_string(), "value".to_string());

        assert_eq!(request.refresh_token, "refresh123");
        assert_eq!(request.client_id, "client456");
        assert_eq!(request.scope, Some("read:tools".to_string()));
        assert_eq!(
            request.additional_params.get("custom"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_lifecycle_event_timestamps() {
        let key = TokenCacheKey::new("user123".to_string(), "client456".to_string());
        let before = Utc::now();
        let event = TokenLifecycleEvent::token_created(key, None);
        let after = Utc::now();

        let timestamp = event.timestamp();
        assert!(timestamp >= before && timestamp <= after);
    }
}

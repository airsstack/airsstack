//! Token Lifecycle Management Traits
//!
//! This module defines the core traits for token lifecycle management,
//! providing clean abstractions for token caching, refresh handling,
//! and lifecycle event management.

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};

// Layer 3: Internal module imports
use crate::oauth2::{AuthContext, OAuth2Error, OAuth2Result};
use super::types::{
    RefreshTokenRequest, RefreshTokenResponse, TokenCacheEntry, TokenCacheKey,
    TokenCacheMetrics, TokenLifecycleEvent, TokenStatus,
};

/// Trait for token cache providers - manages token storage and retrieval
#[async_trait::async_trait]
pub trait TokenCacheProvider: Send + Sync + std::fmt::Debug {
    /// Store a token in the cache
    async fn store(
        &self,
        key: TokenCacheKey,
        entry: TokenCacheEntry,
        ttl: Option<Duration>,
    ) -> OAuth2Result<()>;

    /// Retrieve a token from the cache
    async fn retrieve(&self, key: &TokenCacheKey) -> OAuth2Result<Option<TokenCacheEntry>>;

    /// Remove a token from the cache
    async fn remove(&self, key: &TokenCacheKey) -> OAuth2Result<bool>;

    /// Check if a token exists in the cache
    async fn exists(&self, key: &TokenCacheKey) -> OAuth2Result<bool>;

    /// Get token expiration time
    async fn get_expiration(&self, key: &TokenCacheKey) -> OAuth2Result<Option<DateTime<Utc>>>;

    /// Update token expiration time
    async fn update_expiration(
        &self,
        key: &TokenCacheKey,
        new_expiration: DateTime<Utc>,
    ) -> OAuth2Result<()>;

    /// Clear expired tokens from the cache
    async fn clear_expired(&self) -> OAuth2Result<u64>;

    /// Get cache statistics
    async fn get_metrics(&self) -> OAuth2Result<TokenCacheMetrics>;

    /// Get all cached token keys (for debugging/monitoring)
    async fn list_keys(&self) -> OAuth2Result<Vec<TokenCacheKey>>;
}

/// Token refresh provider trait
///
/// This trait handles the refresh token flow for obtaining new access tokens
/// before expiration, supporting various refresh strategies and external IdP integration.
#[async_trait]
pub trait TokenRefreshProvider: Send + Sync + std::fmt::Debug {
    /// Refresh an access token using a refresh token
    async fn refresh_token(
        &self,
        request: RefreshTokenRequest,
    ) -> OAuth2Result<RefreshTokenResponse>;

    /// Check if a token should be refreshed based on expiration
    async fn should_refresh(&self, auth_context: &AuthContext) -> OAuth2Result<bool>;

    /// Get the refresh threshold for early token renewal
    fn get_refresh_threshold(&self) -> Duration;

    /// Validate a refresh token
    async fn validate_refresh_token(&self, refresh_token: &str) -> OAuth2Result<bool>;

    /// Revoke a refresh token (for logout/cleanup)
    async fn revoke_refresh_token(&self, refresh_token: &str) -> OAuth2Result<()>;
}

/// Complete token lifecycle management trait
///
/// This trait combines caching and refresh functionality with lifecycle
/// event management for comprehensive token management.
#[async_trait]
pub trait TokenLifecycleProvider: Send + Sync + std::fmt::Debug {
    /// Get current token status for a user/client
    async fn get_token_status(&self, user_id: &str, client_id: &str) -> OAuth2Result<TokenStatus>;

    /// Store a new token with automatic expiration management
    async fn store_token(&self, auth_context: AuthContext) -> OAuth2Result<()>;

    /// Retrieve a valid token, refreshing if necessary
    async fn get_valid_token(
        &self,
        user_id: &str,
        client_id: &str,
    ) -> OAuth2Result<Option<AuthContext>>;

    /// Invalidate a token (for logout)
    async fn invalidate_token(&self, user_id: &str, client_id: &str) -> OAuth2Result<()>;

    /// Clean up expired tokens and perform maintenance
    async fn cleanup_expired_tokens(&self) -> OAuth2Result<u64>;

    /// Register a lifecycle event listener
    async fn on_lifecycle_event(
        &self,
        event: TokenLifecycleEvent,
    ) -> OAuth2Result<()>;

    /// Get token lifecycle metrics
    async fn get_lifecycle_metrics(&self) -> OAuth2Result<TokenCacheMetrics>;
}

/// Token validation provider trait
///
/// This trait provides token validation capabilities including
/// signature verification, claims validation, and scope checking.
#[async_trait]
pub trait TokenValidationProvider: Send + Sync + std::fmt::Debug {
    /// Validate a token and return authentication context
    async fn validate_token(&self, token: &str) -> OAuth2Result<AuthContext>;

    /// Check if a token is expired
    async fn is_token_expired(&self, token: &str) -> OAuth2Result<bool>;

    /// Extract token expiration time
    async fn get_token_expiration(&self, token: &str) -> OAuth2Result<DateTime<Utc>>;

    /// Validate token signature
    async fn validate_signature(&self, token: &str) -> OAuth2Result<bool>;

    /// Extract token claims without full validation
    async fn extract_claims(&self, token: &str) -> OAuth2Result<serde_json::Value>;
}

/// Token lifecycle event handler trait
///
/// This trait defines handlers for various token lifecycle events
/// for monitoring, logging, and custom business logic.
#[async_trait]
pub trait TokenLifecycleEventHandler: Send + Sync + std::fmt::Debug {
    /// Handle token creation event
    async fn on_token_created(&self, auth_context: &AuthContext) -> OAuth2Result<()>;

    /// Handle token refresh event
    async fn on_token_refreshed(
        &self,
        old_context: &AuthContext,
        new_context: &AuthContext,
    ) -> OAuth2Result<()>;

    /// Handle token expiration event
    async fn on_token_expired(&self, auth_context: &AuthContext) -> OAuth2Result<()>;

    /// Handle token invalidation event
    async fn on_token_invalidated(&self, auth_context: &AuthContext) -> OAuth2Result<()>;

    /// Handle token validation failure
    async fn on_validation_failed(&self, token: &str, error: &OAuth2Error) -> OAuth2Result<()>;

    /// Handle cache hit event
    async fn on_cache_hit(&self, key: &TokenCacheKey) -> OAuth2Result<()>;

    /// Handle cache miss event
    async fn on_cache_miss(&self, key: &TokenCacheKey) -> OAuth2Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trait_object_compatibility() {
        // Test that traits can be used as trait objects
        fn assert_object_safe<T: ?Sized>() {}
        
        assert_object_safe::<dyn TokenCacheProvider>();
        assert_object_safe::<dyn TokenRefreshProvider>();
        assert_object_safe::<dyn TokenLifecycleProvider>();
        assert_object_safe::<dyn TokenValidationProvider>();
        assert_object_safe::<dyn TokenLifecycleEventHandler>();
    }

    #[test]
    fn test_trait_send_sync() {
        // Test that traits are Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        
        assert_send_sync::<Box<dyn TokenCacheProvider>>();
        assert_send_sync::<Box<dyn TokenRefreshProvider>>();
        assert_send_sync::<Box<dyn TokenLifecycleProvider>>();
        assert_send_sync::<Box<dyn TokenValidationProvider>>();
        assert_send_sync::<Box<dyn TokenLifecycleEventHandler>>();
    }
}

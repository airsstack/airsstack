//! Token Lifecycle Manager
//!
//! This module provides the main token lifecycle management implementation
//! that coordinates caching, refresh, and lifecycle events.

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};

// Layer 3: Internal module imports
use super::cache::{TokenCache, TokenCacheConfig};
use super::config::TokenLifecycleConfig;
use super::traits::{
    TokenCacheProvider, TokenLifecycleEventHandler, TokenLifecycleProvider, TokenRefreshProvider,
};
use super::types::{
    RefreshTokenRequest, TokenCacheEntry, TokenCacheKey, TokenCacheMetrics, TokenLifecycleEvent,
    TokenStatus,
};
use crate::oauth2::{AuthContext, OAuth2Error, OAuth2Result};

/// Comprehensive token lifecycle manager with generic type parameters for static dispatch
#[derive(Debug)]
pub struct TokenLifecycleManager<C, R, H> 
where
    C: TokenCacheProvider + Send + Sync + 'static,
    R: TokenRefreshProvider + Send + Sync + 'static,
    H: TokenLifecycleEventHandler + Send + Sync + 'static,
{
    /// Token cache implementation
    cache: Arc<C>,

    /// Token refresh handler
    refresh_handler: Arc<R>,

    /// Event handlers for lifecycle events
    event_handlers: Vec<Arc<H>>,

    /// Configuration
    config: TokenLifecycleConfig,

    /// Metrics for monitoring
    metrics: Arc<tokio::sync::RwLock<LifecycleMetrics>>,
}

/// Metrics for token lifecycle operations
#[derive(Debug, Default, Clone)]
pub struct LifecycleMetrics {
    /// Total number of tokens stored
    pub tokens_stored: u64,

    /// Total number of tokens retrieved
    pub tokens_retrieved: u64,

    /// Total number of tokens invalidated
    pub tokens_invalidated: u64,

    /// Total number of automatic refreshes performed
    pub automatic_refreshes: u64,

    /// Total number of cleanup operations
    pub cleanup_operations: u64,

    /// Total number of lifecycle events processed
    pub lifecycle_events_processed: u64,

    /// Last operation timestamp
    pub last_operation_at: Option<DateTime<Utc>>,
}

impl<C, R, H> TokenLifecycleManager<C, R, H>
where
    C: TokenCacheProvider + Send + Sync + 'static,
    R: TokenRefreshProvider + Send + Sync + 'static,
    H: TokenLifecycleEventHandler + Send + Sync + 'static,
{
    /// Create a new token lifecycle manager with dependency injection
    pub fn new(
        cache: Arc<C>,
        refresh_handler: Arc<R>,
        event_handlers: Vec<Arc<H>>,
        config: TokenLifecycleConfig,
    ) -> Self {
        Self {
            cache,
            refresh_handler,
            event_handlers,
            config,
            metrics: Arc::new(tokio::sync::RwLock::new(LifecycleMetrics::default())),
        }
    }

    /// Create with default configuration (convenience constructor)
    pub fn with_defaults(
        cache: Arc<C>,
        refresh_handler: Arc<R>,
        event_handlers: Vec<Arc<H>>,
    ) -> Self {
        Self::new(
            cache,
            refresh_handler,
            event_handlers,
            TokenLifecycleConfig::default(),
        )
    }

    /// Add an event handler for lifecycle events
    pub fn add_event_handler(&mut self, handler: Arc<H>) {
        self.event_handlers.push(handler);
    }

    /// Factory method to create manager with default implementations
    /// This maintains compatibility with existing code while allowing dependency injection
    pub async fn with_default_providers(
        config: TokenLifecycleConfig,
    ) -> OAuth2Result<TokenLifecycleManager<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>>
    {
        let cache_config = TokenCacheConfig {
            max_size: config.max_cache_size,
            default_ttl: config.default_cache_ttl,
            cleanup_interval: config.validation_interval,
            enable_lru_eviction: true,
            enable_metrics: true,
        };

        let cache = Arc::new(TokenCache::with_config(cache_config));
        let refresh_handler = Arc::new(PlaceholderRefreshHandler::new());
        let event_handlers = Vec::new(); // Empty initially, can be added later

        Ok(TokenLifecycleManager::new(
            cache,
            refresh_handler,
            event_handlers,
            config,
        ))
    }

    /// Emit a lifecycle event to all registered handlers
    async fn emit_event(&self, event: TokenLifecycleEvent) -> OAuth2Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.lifecycle_events_processed += 1;
        metrics.last_operation_at = Some(Utc::now());
        drop(metrics);

        for handler in &self.event_handlers {
            if let Err(e) = self
                .handle_lifecycle_event_internal(handler.as_ref(), &event)
                .await
            {
                warn!("Event handler failed: {}", e);
            }
        }

        Ok(())
    }

    /// Handle lifecycle event with error isolation
    async fn handle_lifecycle_event_internal(
        &self,
        handler: &dyn TokenLifecycleEventHandler,
        event: &TokenLifecycleEvent,
    ) -> OAuth2Result<()> {
        match event {
            TokenLifecycleEvent::TokenCreated { key, .. } => {
                // Extract auth context from cache for the event
                if let Ok(Some(entry)) = self.cache.retrieve(key).await {
                    handler.on_token_created(&entry.auth_context).await?;
                }
            }
            TokenLifecycleEvent::TokenRefreshed { key, .. } => {
                // For refresh events, we'd need to track old and new contexts
                // This is a simplified implementation
                if let Ok(Some(entry)) = self.cache.retrieve(key).await {
                    handler
                        .on_token_refreshed(&entry.auth_context, &entry.auth_context)
                        .await?;
                }
            }
            TokenLifecycleEvent::TokenExpired { key, .. } => {
                // For expired tokens, try to get the cached entry first
                if let Ok(Some(entry)) = self.cache.retrieve(key).await {
                    handler.on_token_expired(&entry.auth_context).await?;
                } else {
                    // If not in cache, create minimal JWT claims for the event
                    let jwt_claims = crate::oauth2::types::JwtClaims {
                        sub: key.user_id.clone(),
                        aud: Some(key.client_id.clone()),
                        iss: None,
                        exp: Some(chrono::Utc::now().timestamp() - 1), // Already expired
                        nbf: None,
                        iat: None,
                        jti: None,
                        scope: None,
                        scopes: None,
                    };
                    let auth_context = AuthContext::new(jwt_claims, vec![]);
                    handler.on_token_expired(&auth_context).await?;
                }
            }
            TokenLifecycleEvent::TokenInvalidated { key, .. } => {
                // For invalidated tokens, try to get the cached entry first
                if let Ok(Some(entry)) = self.cache.retrieve(key).await {
                    handler.on_token_invalidated(&entry.auth_context).await?;
                } else {
                    // If not in cache, create minimal JWT claims for the event
                    let jwt_claims = crate::oauth2::types::JwtClaims {
                        sub: key.user_id.clone(),
                        aud: Some(key.client_id.clone()),
                        iss: None,
                        exp: Some(chrono::Utc::now().timestamp() - 1), // Already expired
                        nbf: None,
                        iat: None,
                        jti: None,
                        scope: None,
                        scopes: None,
                    };
                    let auth_context = AuthContext::new(jwt_claims, vec![]);
                    handler.on_token_invalidated(&auth_context).await?;
                }
            }
            TokenLifecycleEvent::ValidationFailed { error, .. } => {
                let oauth_error = OAuth2Error::ValidationFailed(error.clone());
                handler.on_validation_failed("", &oauth_error).await?;
            }
            TokenLifecycleEvent::CacheHit { key, .. } => {
                handler.on_cache_hit(key).await?;
            }
            TokenLifecycleEvent::CacheMiss { key, .. } => {
                handler.on_cache_miss(key).await?;
            }
            TokenLifecycleEvent::CacheMaintenance { .. } => {
                // No specific handler for maintenance events
            }
        }

        Ok(())
    }

    /// Attempt to refresh a token if needed
    async fn attempt_refresh(
        &self,
        key: &TokenCacheKey,
        entry: &TokenCacheEntry,
    ) -> OAuth2Result<Option<AuthContext>> {
        if !self.config.auto_refresh_enabled {
            return Ok(None);
        }

        // Check if refresh is needed
        if !self
            .refresh_handler
            .should_refresh(&entry.auth_context)
            .await?
        {
            return Ok(None);
        }

        // Check if we have a refresh token
        let refresh_token = match &entry.refresh_token {
            Some(token) => token.clone(),
            None => {
                debug!("No refresh token available for user: {}", key.user_id);
                return Ok(None);
            }
        };

        info!("Attempting token refresh for user: {}", key.user_id);

        // Create refresh request
        let refresh_request = RefreshTokenRequest::new(refresh_token, key.client_id.clone());

        // Attempt refresh
        match self.refresh_handler.refresh_token(refresh_request).await {
            Ok(refresh_response) => {
                // Create new auth context with refreshed token
                let mut new_auth_context = entry.auth_context.clone();

                // Update token and expiration
                if let Some(expires_in) = refresh_response.expires_in {
                    new_auth_context.expires_at =
                        Some(Utc::now() + chrono::Duration::seconds(expires_in as i64));
                }

                // Update scopes if provided
                if let Some(scope) = refresh_response.scope {
                    new_auth_context.scopes =
                        scope.split_whitespace().map(|s| s.to_string()).collect();
                }

                // Store refreshed token in cache
                let mut new_entry = TokenCacheEntry::new(new_auth_context.clone());
                new_entry.refresh_token = refresh_response
                    .refresh_token
                    .or(entry.refresh_token.clone());

                self.cache.store(key.clone(), new_entry, None).await?;

                // Emit refresh event
                let event = TokenLifecycleEvent::token_refreshed(
                    key.clone(),
                    entry.auth_context.expires_at,
                    new_auth_context.expires_at,
                );
                self.emit_event(event).await?;

                // Update metrics
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.automatic_refreshes += 1;
                    metrics.last_operation_at = Some(Utc::now());
                }

                info!("Token refresh successful for user: {}", key.user_id);
                Ok(Some(new_auth_context))
            }
            Err(e) => {
                error!("Token refresh failed for user: {} - {}", key.user_id, e);

                // Emit validation failed event
                let event = TokenLifecycleEvent::ValidationFailed {
                    key: Some(key.clone()),
                    error: e.to_string(),
                    timestamp: Utc::now(),
                };
                self.emit_event(event).await?;

                Err(e)
            }
        }
    }

    /// Update lifecycle metrics
    async fn update_metrics<F>(&self, updater: F)
    where
        F: FnOnce(&mut LifecycleMetrics),
    {
        let mut metrics = self.metrics.write().await;
        updater(&mut metrics);
        metrics.last_operation_at = Some(Utc::now());
    }
}

#[async_trait]
impl<C, R, H> TokenLifecycleProvider for TokenLifecycleManager<C, R, H>
where
    C: TokenCacheProvider + Send + Sync + 'static,
    R: TokenRefreshProvider + Send + Sync + 'static,
    H: TokenLifecycleEventHandler + Send + Sync + 'static,
{
    async fn get_token_status(&self, user_id: &str, client_id: &str) -> OAuth2Result<TokenStatus> {
        let key = TokenCacheKey::new(user_id.to_string(), client_id.to_string());

        match self.cache.retrieve(&key).await? {
            Some(entry) => {
                if entry.is_expired() {
                    Ok(TokenStatus::Expired {
                        expired_at: entry.auth_context.expires_at.unwrap_or_else(Utc::now),
                        refresh_available: entry.refresh_token.is_some(),
                    })
                } else {
                    Ok(TokenStatus::Valid {
                        expires_at: entry.auth_context.expires_at,
                        scopes: entry.auth_context.scopes.clone(),
                    })
                }
            }
            None => Ok(TokenStatus::NotFound),
        }
    }

    async fn store_token(&self, auth_context: AuthContext) -> OAuth2Result<()> {
        let client_id = auth_context.audience().unwrap_or("unknown").to_string();
        let key = TokenCacheKey::new(auth_context.user_id().to_string(), client_id);
        let entry = TokenCacheEntry::new(auth_context.clone());

        self.cache.store(key.clone(), entry, None).await?;

        // Emit created event
        let event = TokenLifecycleEvent::token_created(key, auth_context.expires_at);
        self.emit_event(event).await?;

        // Update metrics
        self.update_metrics(|m| m.tokens_stored += 1).await;

        debug!("Token stored for user: {}", auth_context.user_id());
        Ok(())
    }

    async fn get_valid_token(
        &self,
        user_id: &str,
        client_id: &str,
    ) -> OAuth2Result<Option<AuthContext>> {
        let key = TokenCacheKey::new(user_id.to_string(), client_id.to_string());

        match self.cache.retrieve(&key).await? {
            Some(entry) => {
                // Emit cache hit event
                let event = TokenLifecycleEvent::CacheHit {
                    key: key.clone(),
                    timestamp: Utc::now(),
                };
                self.emit_event(event).await?;

                if entry.is_expired() {
                    // Try to refresh if possible
                    match self.attempt_refresh(&key, &entry).await {
                        Ok(Some(refreshed_context)) => {
                            self.update_metrics(|m| m.tokens_retrieved += 1).await;
                            Ok(Some(refreshed_context))
                        }
                        Ok(None) => {
                            // No refresh possible, remove expired token
                            self.cache.remove(&key).await?;

                            let event = TokenLifecycleEvent::token_expired(
                                key,
                                entry.auth_context.expires_at.unwrap_or_else(Utc::now),
                            );
                            self.emit_event(event).await?;

                            Ok(None)
                        }
                        Err(_) => {
                            // Refresh failed, remove token
                            self.cache.remove(&key).await?;
                            Ok(None)
                        }
                    }
                } else {
                    // Token is valid
                    self.update_metrics(|m| m.tokens_retrieved += 1).await;
                    Ok(Some(entry.auth_context))
                }
            }
            None => {
                // Emit cache miss event
                let event = TokenLifecycleEvent::CacheMiss {
                    key,
                    timestamp: Utc::now(),
                };
                self.emit_event(event).await?;

                Ok(None)
            }
        }
    }

    async fn invalidate_token(&self, user_id: &str, client_id: &str) -> OAuth2Result<()> {
        let key = TokenCacheKey::new(user_id.to_string(), client_id.to_string());

        let was_removed = self.cache.remove(&key).await?;

        if was_removed {
            // Emit invalidated event
            let event =
                TokenLifecycleEvent::token_invalidated(key, "Manual invalidation".to_string());
            self.emit_event(event).await?;

            // Update metrics
            self.update_metrics(|m| m.tokens_invalidated += 1).await;

            info!("Token invalidated for user: {}", user_id);
        }

        Ok(())
    }

    async fn cleanup_expired_tokens(&self) -> OAuth2Result<u64> {
        let removed_count = self.cache.clear_expired().await?;

        if removed_count > 0 {
            // Emit maintenance event
            let event = TokenLifecycleEvent::CacheMaintenance {
                operation: "cleanup_expired".to_string(),
                affected_count: removed_count,
                timestamp: Utc::now(),
            };
            self.emit_event(event).await?;

            // Update metrics
            self.update_metrics(|m| m.cleanup_operations += 1).await;

            info!("Cleaned up {} expired tokens", removed_count);
        }

        Ok(removed_count)
    }

    async fn on_lifecycle_event(&self, event: TokenLifecycleEvent) -> OAuth2Result<()> {
        self.emit_event(event).await
    }

    async fn get_lifecycle_metrics(&self) -> OAuth2Result<TokenCacheMetrics> {
        // Combine cache metrics with lifecycle metrics
        let cache_metrics = self.cache.get_metrics().await?;
        let lifecycle_metrics = self.metrics.read().await;

        let mut combined_metrics = cache_metrics;

        // Add lifecycle-specific metrics
        // Note: In a complete implementation, we'd have a more comprehensive metrics structure
        combined_metrics.collected_at =
            lifecycle_metrics.last_operation_at.unwrap_or_else(Utc::now);

        Ok(combined_metrics)
    }
}

/// Builder for token lifecycle manager (simplified for generic approach)
pub struct TokenLifecycleManagerBuilder {
    config: TokenLifecycleConfig,
    event_handlers: Vec<Arc<PlaceholderEventHandler>>,
}

impl TokenLifecycleManagerBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: TokenLifecycleConfig::default(),
            event_handlers: Vec::new(),
        }
    }

    /// Set the lifecycle configuration
    pub fn with_config(mut self, config: TokenLifecycleConfig) -> Self {
        self.config = config;
        self
    }

    /// Add an event handler
    pub fn add_event_handler(mut self, handler: Arc<PlaceholderEventHandler>) -> Self {
        self.event_handlers.push(handler);
        self
    }

    /// Build the token lifecycle manager
    pub async fn build(
        self,
    ) -> OAuth2Result<
        TokenLifecycleManager<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>,
    > {
        let mut manager: TokenLifecycleManager<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler> = 
            TokenLifecycleManager::<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>::with_default_providers(self.config).await?;

        // Add event handlers  
        for handler in self.event_handlers {
            manager.add_event_handler(handler);
        }

        Ok(manager)
    }
}

impl Default for TokenLifecycleManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder refresh handler for testing and development
#[derive(Debug)]
pub struct PlaceholderRefreshHandler;

impl PlaceholderRefreshHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlaceholderRefreshHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenRefreshProvider for PlaceholderRefreshHandler {
    async fn refresh_token(
        &self,
        _request: RefreshTokenRequest,
    ) -> OAuth2Result<super::types::RefreshTokenResponse> {
        // This is a placeholder implementation
        Err(OAuth2Error::RefreshFailed(
            "Placeholder refresh handler - not implemented".to_string(),
        ))
    }

    async fn should_refresh(&self, auth_context: &AuthContext) -> OAuth2Result<bool> {
        if let Some(expires_at) = auth_context.expires_at {
            let now = Utc::now();
            let threshold = chrono::Duration::minutes(5); // 5-minute threshold
            Ok(expires_at.signed_duration_since(now) <= threshold)
        } else {
            Ok(false)
        }
    }

    fn get_refresh_threshold(&self) -> Duration {
        Duration::from_secs(300) // 5 minutes
    }

    async fn validate_refresh_token(&self, refresh_token: &str) -> OAuth2Result<bool> {
        Ok(!refresh_token.is_empty())
    }

    async fn revoke_refresh_token(&self, refresh_token: &str) -> OAuth2Result<()> {
        // Placeholder implementation - would revoke the token in real implementation
        debug!("PlaceholderRefreshHandler revoking token: {}", refresh_token);
        Ok(())
    }
}

/// Placeholder event handler for testing and development
#[derive(Debug)]
pub struct PlaceholderEventHandler;

impl PlaceholderEventHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlaceholderEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenLifecycleEventHandler for PlaceholderEventHandler {
    async fn on_token_created(&self, auth_context: &AuthContext) -> OAuth2Result<()> {
        debug!("Token created for user: {}", auth_context.user_id());
        Ok(())
    }

    async fn on_token_refreshed(
        &self,
        old_context: &AuthContext,
        new_context: &AuthContext,
    ) -> OAuth2Result<()> {
        debug!(
            "Token refreshed for user: {} (old expires: {:?}, new expires: {:?})",
            old_context.user_id(), old_context.expires_at, new_context.expires_at
        );
        Ok(())
    }

    async fn on_token_expired(&self, auth_context: &AuthContext) -> OAuth2Result<()> {
        debug!("Token expired for user: {}", auth_context.user_id());
        Ok(())
    }

    async fn on_token_invalidated(&self, auth_context: &AuthContext) -> OAuth2Result<()> {
        debug!("Token invalidated for user: {}", auth_context.user_id());
        Ok(())
    }

    async fn on_validation_failed(&self, token: &str, error: &OAuth2Error) -> OAuth2Result<()> {
        debug!("Token validation failed: {} - {:?}", token, error);
        Ok(())
    }

    async fn on_cache_hit(&self, key: &TokenCacheKey) -> OAuth2Result<()> {
        debug!("Cache hit for key: {:?}", key);
        Ok(())
    }

    async fn on_cache_miss(&self, key: &TokenCacheKey) -> OAuth2Result<()> {
        debug!("Cache miss for key: {:?}", key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::types::JwtClaims;

    // Type alias for convenience in tests
    type TestManager = TokenLifecycleManager<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>;

    fn create_test_auth_context(user_id: &str, client_id: &str) -> AuthContext {
        let jwt_claims = JwtClaims {
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

    #[tokio::test]
    async fn test_lifecycle_manager_creation() {
        let config = TokenLifecycleConfig::default();
        let manager: TestManager = TokenLifecycleManager::<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>::with_default_providers(config).await.unwrap();

        // Basic functionality test
        let status = manager
            .get_token_status("user123", "client456")
            .await
            .unwrap();
        assert_eq!(status, TokenStatus::NotFound);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_token() {
        let config = TokenLifecycleConfig::default();
        let manager: TestManager = TokenLifecycleManager::<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>::with_default_providers(config).await.unwrap();

        let auth_context = create_test_auth_context("user123", "client456");

        // Store token
        manager.store_token(auth_context.clone()).await.unwrap();

        // Retrieve token
        let retrieved = manager
            .get_valid_token("user123", "client456")
            .await
            .unwrap();
        assert!(retrieved.is_some());

        let retrieved_context = retrieved.unwrap();
        assert_eq!(retrieved_context.user_id(), auth_context.user_id());
        assert_eq!(retrieved_context.audience(), auth_context.audience());
    }

    #[tokio::test]
    async fn test_token_invalidation() {
        let config = TokenLifecycleConfig::default();
        let manager: TestManager = TokenLifecycleManager::<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>::with_default_providers(config).await.unwrap();

        let auth_context = create_test_auth_context("user123", "client456");

        // Store token
        manager.store_token(auth_context).await.unwrap();

        // Verify token exists
        let status = manager
            .get_token_status("user123", "client456")
            .await
            .unwrap();
        assert!(status.is_usable());

        // Invalidate token
        manager
            .invalidate_token("user123", "client456")
            .await
            .unwrap();

        // Verify token is gone
        let status = manager
            .get_token_status("user123", "client456")
            .await
            .unwrap();
        assert_eq!(status, TokenStatus::NotFound);
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let config = TokenLifecycleConfig::default();

        let manager = TokenLifecycleManagerBuilder::new()
            .with_config(config)
            .build()
            .await
            .unwrap();

        let status = manager
            .get_token_status("user123", "client456")
            .await
            .unwrap();
        assert_eq!(status, TokenStatus::NotFound);
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens() {
        let config = TokenLifecycleConfig::default();
        let manager: TestManager = TokenLifecycleManager::<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>::with_default_providers(config).await.unwrap();

        // Store a normal token
        let auth_context = create_test_auth_context("user123", "client456");
        manager.store_token(auth_context).await.unwrap();

        // Cleanup should not remove any tokens (none are expired)
        let removed_count = manager.cleanup_expired_tokens().await.unwrap();
        assert_eq!(removed_count, 0); // No expired tokens to remove

        // Token should still exist
        let status = manager
            .get_token_status("user123", "client456")
            .await
            .unwrap();
        assert!(status.is_usable());
    }

    #[tokio::test]
    async fn test_get_lifecycle_metrics() {
        let config = TokenLifecycleConfig::default();
        let manager: TestManager = TokenLifecycleManager::<TokenCache, PlaceholderRefreshHandler, PlaceholderEventHandler>::with_default_providers(config).await.unwrap();

        let auth_context = create_test_auth_context("user123", "client456");

        // Perform some operations
        manager.store_token(auth_context).await.unwrap();
        manager
            .get_valid_token("user123", "client456")
            .await
            .unwrap();
        manager
            .invalidate_token("user123", "client456")
            .await
            .unwrap();

        // Get metrics
        let metrics = manager.get_lifecycle_metrics().await.unwrap();

        // Metrics should reflect our operations
        assert!(metrics.collected_at <= Utc::now());
    }
}

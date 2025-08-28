//! Token Cache Implementation
//!
//! This module provides in-memory and persistent token cache implementations
//! with automatic expiration, LRU eviction, and comprehensive metrics.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

// Layer 3: Internal module imports
use super::traits::TokenCacheProvider;
use super::types::{TokenCacheEntry, TokenCacheKey, TokenCacheMetrics};
use crate::oauth2::{OAuth2Error, OAuth2Result};

/// Configuration for token cache behavior
#[derive(Debug, Clone)]
pub struct TokenCacheConfig {
    /// Maximum number of cached tokens
    pub max_size: usize,

    /// Default TTL for cached tokens
    pub default_ttl: Duration,

    /// Cleanup interval for expired tokens
    pub cleanup_interval: Duration,

    /// Enable LRU eviction when cache is full
    pub enable_lru_eviction: bool,

    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for TokenCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            default_ttl: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            enable_lru_eviction: true,
            enable_metrics: true,
        }
    }
}

/// Internal cache entry with metadata
#[derive(Debug, Clone)]
pub(super) struct CacheEntryWithMetadata {
    /// The actual cache entry
    entry: TokenCacheEntry,

    /// Expiration time for this cache entry
    expires_at: Option<DateTime<Utc>>,

    /// Last access time for LRU eviction
    last_accessed: DateTime<Utc>,

    /// Number of times this entry has been accessed
    access_count: u64,
}

impl CacheEntryWithMetadata {
    fn new(entry: TokenCacheEntry, ttl: Option<Duration>) -> Self {
        let now = Utc::now();
        let expires_at = ttl.map(|duration| now + chrono::Duration::from_std(duration).unwrap());

        Self {
            entry,
            expires_at,
            last_accessed: now,
            access_count: 0,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    fn mark_accessed(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
        self.entry.mark_accessed();
    }
}

/// In-memory token cache with LRU eviction and automatic cleanup
#[derive(Debug)]
pub struct TokenCache {
    /// Cache configuration
    config: TokenCacheConfig,

    /// Cache storage
    pub(super) cache: Arc<RwLock<HashMap<TokenCacheKey, CacheEntryWithMetadata>>>,

    /// Cache metrics
    metrics: Arc<RwLock<TokenCacheMetrics>>,

    /// Background cleanup task handle
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl TokenCache {
    /// Create a new token cache with default configuration
    pub fn new() -> Self {
        Self::with_config(TokenCacheConfig::default())
    }

    /// Create a new token cache with custom configuration
    pub fn with_config(config: TokenCacheConfig) -> Self {
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let metrics = Arc::new(RwLock::new(TokenCacheMetrics::new()));

        let mut token_cache = Self {
            config,
            cache,
            metrics,
            cleanup_handle: None,
        };

        // Start background cleanup task
        token_cache.start_cleanup_task();

        token_cache
    }

    /// Start the background cleanup task
    fn start_cleanup_task(&mut self) {
        if self.config.cleanup_interval.is_zero() {
            return;
        }

        let cache = Arc::clone(&self.cache);
        let metrics = Arc::clone(&self.metrics);
        let interval = self.config.cleanup_interval;

        self.cleanup_handle = Some(tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                match Self::cleanup_expired_internal(&cache, &metrics).await {
                    Ok(removed_count) => {
                        if removed_count > 0 {
                            debug!("Cleaned up {} expired tokens", removed_count);
                        }
                    }
                    Err(e) => {
                        error!("Token cache cleanup failed: {}", e);
                    }
                }
            }
        }));
    }

    /// Internal cleanup function for background task
    async fn cleanup_expired_internal(
        cache: &Arc<RwLock<HashMap<TokenCacheKey, CacheEntryWithMetadata>>>,
        metrics: &Arc<RwLock<TokenCacheMetrics>>,
    ) -> OAuth2Result<u64> {
        let expired_keys: Vec<TokenCacheKey> = {
            let cache_read = cache.read().await;
            cache_read
                .iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(key, _)| key.clone())
                .collect()
        };

        if expired_keys.is_empty() {
            return Ok(0);
        }

        let removed_count = expired_keys.len() as u64;

        {
            let mut cache_write = cache.write().await;
            for key in expired_keys {
                cache_write.remove(&key);
            }
        }

        // Update metrics
        {
            let mut metrics_write = metrics.write().await;
            metrics_write.expired_tokens += removed_count;
            metrics_write.update_timestamp();
        }

        Ok(removed_count)
    }

    /// Evict least recently used entries if cache is full
    async fn evict_lru_if_needed(&self) -> OAuth2Result<()> {
        if !self.config.enable_lru_eviction {
            return Ok(());
        }

        let cache_size = {
            let cache_read = self.cache.read().await;
            cache_read.len()
        };

        if cache_size < self.config.max_size {
            return Ok(());
        }

        // Find the least recently used entry
        let lru_key = {
            let cache_read = self.cache.read().await;
            cache_read
                .iter()
                .min_by_key(|(_, entry)| entry.last_accessed)
                .map(|(key, _)| key.clone())
        };

        if let Some(key) = lru_key {
            let mut cache_write = self.cache.write().await;
            cache_write.remove(&key);
            warn!("Evicted LRU token from cache: {:?}", key);
        }

        Ok(())
    }

    /// Update cache metrics
    async fn update_metrics(&self, hit: bool) {
        if !self.config.enable_metrics {
            return;
        }

        let mut metrics = self.metrics.write().await;

        if hit {
            metrics.cache_hits += 1;
        } else {
            metrics.cache_misses += 1;
        }

        metrics.calculate_hit_ratio();
        metrics.update_timestamp();
    }

    /// Calculate cache statistics
    async fn calculate_cache_stats(&self) -> OAuth2Result<TokenCacheMetrics> {
        let cache = self.cache.read().await;
        let mut metrics = self.metrics.read().await.clone();

        metrics.total_tokens = cache.len() as u64;
        metrics.expired_tokens = cache.values().filter(|entry| entry.is_expired()).count() as u64;

        // Calculate average token age
        if !cache.is_empty() {
            let total_age: Duration = cache
                .values()
                .map(|entry| {
                    Utc::now()
                        .signed_duration_since(entry.entry.created_at)
                        .to_std()
                        .unwrap_or(Duration::ZERO)
                })
                .sum();

            metrics.average_token_age_seconds = total_age.as_secs_f64() / cache.len() as f64;
        }

        metrics.update_timestamp();
        Ok(metrics)
    }
}

impl Default for TokenCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TokenCache {
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
    }
}

#[async_trait]
impl TokenCacheProvider for TokenCache {
    async fn store(
        &self,
        key: TokenCacheKey,
        entry: TokenCacheEntry,
        ttl: Option<Duration>,
    ) -> OAuth2Result<()> {
        // Use configured default TTL if none provided
        let effective_ttl = ttl.or(Some(self.config.default_ttl));

        // Evict LRU entries if needed
        self.evict_lru_if_needed().await?;

        let cache_entry = CacheEntryWithMetadata::new(entry, effective_ttl);

        {
            let mut cache = self.cache.write().await;
            cache.insert(key, cache_entry);
        }

        debug!("Stored token in cache");
        Ok(())
    }

    async fn retrieve(&self, key: &TokenCacheKey) -> OAuth2Result<Option<TokenCacheEntry>> {
        let mut cache = self.cache.write().await;

        if let Some(cache_entry) = cache.get_mut(key) {
            if cache_entry.is_expired() {
                // Remove expired entry
                cache.remove(key);
                self.update_metrics(false).await;
                return Ok(None);
            }

            // Mark as accessed and return
            cache_entry.mark_accessed();
            self.update_metrics(true).await;
            Ok(Some(cache_entry.entry.clone()))
        } else {
            self.update_metrics(false).await;
            Ok(None)
        }
    }

    async fn remove(&self, key: &TokenCacheKey) -> OAuth2Result<bool> {
        let mut cache = self.cache.write().await;
        Ok(cache.remove(key).is_some())
    }

    async fn exists(&self, key: &TokenCacheKey) -> OAuth2Result<bool> {
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(key) {
            Ok(!entry.is_expired())
        } else {
            Ok(false)
        }
    }

    async fn get_expiration(&self, key: &TokenCacheKey) -> OAuth2Result<Option<DateTime<Utc>>> {
        let cache = self.cache.read().await;

        Ok(cache.get(key).and_then(|entry| entry.expires_at))
    }

    async fn update_expiration(
        &self,
        key: &TokenCacheKey,
        new_expiration: DateTime<Utc>,
    ) -> OAuth2Result<()> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(key) {
            entry.expires_at = Some(new_expiration);
            Ok(())
        } else {
            Err(OAuth2Error::TokenNotFound)
        }
    }

    async fn clear_expired(&self) -> OAuth2Result<u64> {
        Self::cleanup_expired_internal(&self.cache, &self.metrics).await
    }

    async fn get_metrics(&self) -> OAuth2Result<TokenCacheMetrics> {
        self.calculate_cache_stats().await
    }

    async fn list_keys(&self) -> OAuth2Result<Vec<TokenCacheKey>> {
        let cache = self.cache.read().await;
        Ok(cache
            .iter()
            .filter(|(_, entry)| !entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect())
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::oauth2::types::JwtClaims;
    use crate::oauth2::AuthContext;

    async fn create_test_cache() -> TokenCache {
        let config = TokenCacheConfig {
            max_size: 10,
            default_ttl: Duration::from_secs(3600),
            cleanup_interval: Duration::from_secs(1),
            enable_lru_eviction: true,
            enable_metrics: true,
        };
        TokenCache::with_config(config)
    }

    fn create_test_entry() -> TokenCacheEntry {
        // Create test JWT claims
        let jwt_claims = JwtClaims {
            sub: "user123".to_string(),
            aud: Some("client456".to_string()),
            iss: Some("test-issuer".to_string()),
            exp: Some(chrono::Utc::now().timestamp() + 3600),
            nbf: None,
            iat: Some(chrono::Utc::now().timestamp()),
            jti: Some("token123".to_string()),
            scope: Some("read write".to_string()),
            scopes: None,
        };

        let auth_context =
            AuthContext::new(jwt_claims, vec!["read".to_string(), "write".to_string()]);
        TokenCacheEntry::with_refresh_token(auth_context, "test_token".to_string())
    }

    #[tokio::test]
    async fn test_cache_store_and_retrieve() {
        let cache = create_test_cache().await;
        let key = TokenCacheKey::new("user123".to_string(), "client456".to_string());
        let entry = create_test_entry();

        // Store
        cache.store(key.clone(), entry.clone(), None).await.unwrap();

        // Retrieve
        let retrieved = cache.retrieve(&key).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_entry = retrieved.unwrap();
        assert_eq!(
            retrieved_entry.auth_context.user_id(),
            entry.auth_context.user_id()
        );
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = create_test_cache().await;
        let key = TokenCacheKey::new("user123".to_string(), "client456".to_string());
        let entry = create_test_entry();

        // Store with very short TTL
        cache
            .store(key.clone(), entry, Some(Duration::from_millis(1)))
            .await
            .unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should return None for expired entry
        let retrieved = cache.retrieve(&key).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_removal() {
        let cache = create_test_cache().await;
        let key = TokenCacheKey::new("user123".to_string(), "client456".to_string());
        let entry = create_test_entry();

        // Store
        cache.store(key.clone(), entry, None).await.unwrap();

        // Verify exists
        assert!(cache.exists(&key).await.unwrap());

        // Remove
        let removed = cache.remove(&key).await.unwrap();
        assert!(removed);

        // Verify doesn't exist
        assert!(!cache.exists(&key).await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = create_test_cache().await;
        let key = TokenCacheKey::new("user123".to_string(), "client456".to_string());
        let entry = create_test_entry();

        // Store and retrieve to generate metrics
        cache.store(key.clone(), entry, None).await.unwrap();
        cache.retrieve(&key).await.unwrap(); // Hit
        cache
            .retrieve(&TokenCacheKey::new(
                "other".to_string(),
                "client".to_string(),
            ))
            .await
            .unwrap(); // Miss

        let metrics = cache.get_metrics().await.unwrap();
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 1);
        assert_eq!(metrics.hit_ratio, 0.5);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let mut config = TokenCacheConfig::default();
        config.max_size = 2; // Very small cache
        config.enable_lru_eviction = true;

        let cache = TokenCache::with_config(config);

        // Fill cache to capacity
        let key1 = TokenCacheKey::new("user1".to_string(), "client1".to_string());
        let key2 = TokenCacheKey::new("user2".to_string(), "client2".to_string());
        let key3 = TokenCacheKey::new("user3".to_string(), "client3".to_string());

        cache
            .store(key1.clone(), create_test_entry(), None)
            .await
            .unwrap();
        cache
            .store(key2.clone(), create_test_entry(), None)
            .await
            .unwrap();

        // Access key1 to make it more recently used
        cache.retrieve(&key1).await.unwrap();

        // Add key3, should evict key2 (LRU)
        cache
            .store(key3.clone(), create_test_entry(), None)
            .await
            .unwrap();

        // key1 and key3 should exist, key2 should be evicted
        assert!(cache.exists(&key1).await.unwrap());
        assert!(!cache.exists(&key2).await.unwrap());
        assert!(cache.exists(&key3).await.unwrap());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let cache = create_test_cache().await;

        let key1 = TokenCacheKey::new("user1".to_string(), "client1".to_string());
        let key2 = TokenCacheKey::new("user2".to_string(), "client2".to_string());

        cache
            .store(key1.clone(), create_test_entry(), None)
            .await
            .unwrap();
        cache
            .store(key2.clone(), create_test_entry(), None)
            .await
            .unwrap();

        let keys = cache.list_keys().await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&key1));
        assert!(keys.contains(&key2));
    }

    #[tokio::test]
    async fn test_clear_expired() {
        let cache = create_test_cache().await;

        let key1 = TokenCacheKey::new("user1".to_string(), "client1".to_string());
        let key2 = TokenCacheKey::new("user2".to_string(), "client2".to_string());

        // Store one entry with a negative TTL (already expired)
        // We'll manually create the cache entry with past expiration
        {
            let mut cache_map = cache.cache.write().await;
            let expired_entry = CacheEntryWithMetadata {
                entry: create_test_entry(),
                expires_at: Some(Utc::now() - chrono::Duration::seconds(1)), // Expired 1 second ago
                last_accessed: Utc::now(),
                access_count: 0,
            };
            cache_map.insert(key1.clone(), expired_entry);

            // Store valid entry normally
            drop(cache_map); // Release lock before calling store
            cache
                .store(
                    key2.clone(),
                    create_test_entry(),
                    Some(Duration::from_secs(3600)),
                )
                .await
                .unwrap();
        }

        // Clear expired
        let removed_count = cache.clear_expired().await.unwrap();
        assert_eq!(removed_count, 1);

        // Only key2 should remain
        assert!(!cache.exists(&key1).await.unwrap());
        assert!(cache.exists(&key2).await.unwrap());
    }
}

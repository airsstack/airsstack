//! Token Lifecycle Configuration
//!
//! Configuration types for token lifecycle management including cache strategies,
//! refresh policies, and validation intervals.

// Layer 1: Standard library imports
use std::time::Duration;

/// Token lifecycle configuration
#[derive(Debug, Clone)]
pub struct TokenLifecycleConfig {
    /// Default token cache TTL (time to live)
    pub default_cache_ttl: Duration,

    /// Maximum token cache size
    pub max_cache_size: usize,

    /// Refresh token before expiration threshold
    pub refresh_threshold: Duration,

    /// Maximum number of refresh attempts
    pub max_refresh_attempts: u32,

    /// Token validation interval
    pub validation_interval: Duration,

    /// Enable automatic token refresh
    pub auto_refresh_enabled: bool,

    /// Token cache strategy
    pub cache_strategy: TokenCacheStrategy,
}

/// Token cache strategy
#[derive(Debug, Clone)]
pub enum TokenCacheStrategy {
    /// In-memory cache only
    Memory,
    /// Redis-backed cache
    Redis { connection_url: String },
    /// Multi-tier cache (memory + Redis)
    Hybrid {
        memory_size: usize,
        redis_url: String,
    },
}

impl Default for TokenLifecycleConfig {
    fn default() -> Self {
        Self {
            default_cache_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 10000,
            refresh_threshold: Duration::from_secs(300), // 5 minutes
            max_refresh_attempts: 3,
            validation_interval: Duration::from_secs(60), // 1 minute
            auto_refresh_enabled: true,
            cache_strategy: TokenCacheStrategy::Memory,
        }
    }
}

impl TokenLifecycleConfig {
    /// Create a new token lifecycle configuration builder
    pub fn builder() -> TokenLifecycleConfigBuilder {
        TokenLifecycleConfigBuilder::new()
    }
}

/// Builder for token lifecycle configuration
#[derive(Debug, Default)]
pub struct TokenLifecycleConfigBuilder {
    config: TokenLifecycleConfig,
}

impl TokenLifecycleConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: TokenLifecycleConfig::default(),
        }
    }

    /// Set default cache TTL
    pub fn default_cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.default_cache_ttl = ttl;
        self
    }

    /// Set maximum cache size
    pub fn max_cache_size(mut self, size: usize) -> Self {
        self.config.max_cache_size = size;
        self
    }

    /// Set refresh threshold
    pub fn refresh_threshold(mut self, threshold: Duration) -> Self {
        self.config.refresh_threshold = threshold;
        self
    }

    /// Set maximum refresh attempts
    pub fn max_refresh_attempts(mut self, attempts: u32) -> Self {
        self.config.max_refresh_attempts = attempts;
        self
    }

    /// Set validation interval
    pub fn validation_interval(mut self, interval: Duration) -> Self {
        self.config.validation_interval = interval;
        self
    }

    /// Enable or disable automatic refresh
    pub fn auto_refresh_enabled(mut self, enabled: bool) -> Self {
        self.config.auto_refresh_enabled = enabled;
        self
    }

    /// Set cache strategy
    pub fn cache_strategy(mut self, strategy: TokenCacheStrategy) -> Self {
        self.config.cache_strategy = strategy;
        self
    }

    /// Build the configuration
    pub fn build(self) -> TokenLifecycleConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TokenLifecycleConfig::default();

        assert_eq!(config.default_cache_ttl, Duration::from_secs(3600));
        assert_eq!(config.max_cache_size, 10000);
        assert_eq!(config.refresh_threshold, Duration::from_secs(300));
        assert_eq!(config.max_refresh_attempts, 3);
        assert_eq!(config.validation_interval, Duration::from_secs(60));
        assert!(config.auto_refresh_enabled);

        // Check cache strategy is Memory
        match config.cache_strategy {
            TokenCacheStrategy::Memory => {}
            _ => panic!("Expected Memory cache strategy"),
        }
    }

    #[test]
    fn test_config_builder() {
        let config = TokenLifecycleConfig::builder()
            .default_cache_ttl(Duration::from_secs(7200))
            .max_cache_size(5000)
            .refresh_threshold(Duration::from_secs(600))
            .max_refresh_attempts(5)
            .validation_interval(Duration::from_secs(120))
            .auto_refresh_enabled(false)
            .cache_strategy(TokenCacheStrategy::Redis {
                connection_url: "redis://localhost:6379".to_string(),
            })
            .build();

        assert_eq!(config.default_cache_ttl, Duration::from_secs(7200));
        assert_eq!(config.max_cache_size, 5000);
        assert_eq!(config.refresh_threshold, Duration::from_secs(600));
        assert_eq!(config.max_refresh_attempts, 5);
        assert_eq!(config.validation_interval, Duration::from_secs(120));
        assert!(!config.auto_refresh_enabled);

        // Check cache strategy is Redis
        match config.cache_strategy {
            TokenCacheStrategy::Redis { connection_url } => {
                assert_eq!(connection_url, "redis://localhost:6379");
            }
            _ => panic!("Expected Redis cache strategy"),
        }
    }

    #[test]
    fn test_cache_strategy_hybrid() {
        let config = TokenLifecycleConfig::builder()
            .cache_strategy(TokenCacheStrategy::Hybrid {
                memory_size: 1000,
                redis_url: "redis://localhost:6379".to_string(),
            })
            .build();

        match config.cache_strategy {
            TokenCacheStrategy::Hybrid {
                memory_size,
                redis_url,
            } => {
                assert_eq!(memory_size, 1000);
                assert_eq!(redis_url, "redis://localhost:6379");
            }
            _ => panic!("Expected Hybrid cache strategy"),
        }
    }

    #[test]
    fn test_builder_fluent_api() {
        let builder = TokenLifecycleConfigBuilder::new();
        let config = builder
            .default_cache_ttl(Duration::from_secs(1800))
            .max_cache_size(2000)
            .build();

        assert_eq!(config.default_cache_ttl, Duration::from_secs(1800));
        assert_eq!(config.max_cache_size, 2000);
        // Other values should be defaults
        assert_eq!(config.refresh_threshold, Duration::from_secs(300));
        assert_eq!(config.max_refresh_attempts, 3);
    }
}

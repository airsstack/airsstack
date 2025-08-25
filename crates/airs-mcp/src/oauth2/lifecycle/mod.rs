//! OAuth 2.1 Token Lifecycle Management
//!
//! This module provides comprehensive token lifecycle management including:
//! - Refresh token handling with automatic renewal
//! - Secure token caching with expiration management
//! - Token lifecycle events and monitoring
//! - Production-ready token storage and retrieval

// Sub-modules
pub mod cache;
pub mod config;
pub mod manager;
pub mod refresh;
pub mod traits;
pub mod types;

// Re-exports for public API
pub use cache::{TokenCache, TokenCacheConfig};
pub use config::{TokenCacheStrategy, TokenLifecycleConfig, TokenLifecycleConfigBuilder};
pub use manager::{TokenLifecycleManager, TokenLifecycleManagerBuilder};
pub use refresh::{RefreshTokenConfig, RefreshTokenHandler, RefreshTokenStrategy};
pub use traits::{
    TokenCacheProvider, TokenLifecycleEventHandler, TokenLifecycleProvider, TokenRefreshProvider,
};
pub use types::{
    RefreshTokenRequest, RefreshTokenResponse, TokenCacheEntry, TokenCacheKey, TokenCacheMetrics,
    TokenLifecycleEvent, TokenStatus,
};

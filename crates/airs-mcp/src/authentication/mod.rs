//! Multi-Method Authentication for MCP Protocol
//!
//! This module provides a unified authentication framework supporting multiple
//! authentication methods while maintaining 100% backward compatibility with
//! the existing OAuth2 implementation.
//!
//! # Supported Authentication Methods
//!
//! - **OAuth 2.1**: JWT token validation with JWKS support (existing implementation)
//! - **API Key**: Header-based or query parameter API key authentication
//! - **Basic Auth**: Username/password HTTP basic authentication
//! - **Custom**: Pluggable custom authentication strategies
//!
//! # Architecture
//!
//! The authentication system uses the Strategy pattern to support multiple
//! authentication methods through a unified interface:
//!
//! ```rust,no_run
//! use airs_mcp::authentication::{AuthenticationManager, AuthMethod};
//! use airs_mcp::authentication::strategies::{OAuth2Strategy, ApiKeyStrategy};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create authentication manager with multiple strategies
//! let auth_manager = AuthenticationManager::builder()
//!     .add_strategy(AuthMethod::OAuth2, OAuth2Strategy::new(oauth2_config)?)
//!     .add_strategy(AuthMethod::ApiKey, ApiKeyStrategy::new(api_key_config)?)
//!     .primary_method(AuthMethod::OAuth2)
//!     .fallback_method(AuthMethod::ApiKey)
//!     .build()?;
//!
//! // Use in HTTP middleware
//! // let app = Router::new()
//! //     .layer(authentication_middleware_layer(auth_manager));
//! # Ok(())
//! # }
//! ```
//!
//! # Backward Compatibility
//!
//! Existing OAuth2 usage continues to work unchanged:
//!
//! ```rust,no_run
//! // Existing code still works
//! use airs_mcp::oauth2::{OAuth2Config, AuthContext};
//!
//! // New enhanced context automatically supports all auth methods
//! use airs_mcp::authentication::AuthContext as EnhancedAuthContext;
//! ```
//!
//! # Features
//!
//! ## Strategy Pattern
//! - Pluggable authentication methods
//! - Primary and fallback strategy chains
//! - Custom strategy implementation support
//!
//! ## Unified Context
//! - Enhanced AuthContext supporting all methods
//! - Consistent metadata and audit logging
//! - Method-specific claims and attributes
//!
//! ## Configuration Management
//! - TOML/YAML configuration support
//! - Environment variable integration
//! - Runtime strategy switching

pub mod config;
pub mod context;
pub mod error;
pub mod manager;
pub mod middleware;
pub mod strategies;

// Core type re-exports
pub use config::{AuthMethodConfig, AuthenticationConfig};
pub use context::{AuthContext, AuthMetadata, AuthMethod, Claims};
pub use error::{AuthenticationError, AuthenticationResult};
pub use manager::{AuthenticationManager, AuthenticationManagerBuilder};
pub use middleware::authentication_middleware_layer;

// Strategy re-exports
pub use strategies::{ApiKeyStrategy, BasicAuthStrategy, CustomStrategy, OAuth2Strategy, Strategy};

// Backward compatibility re-exports (delegate to oauth2 module)
pub use crate::oauth2::{
    AuthContext as LegacyAuthContext, AuthMetadata as LegacyAuthMetadata,
    OAuth2Config as LegacyOAuth2Config, OAuth2Error as LegacyOAuth2Error,
};

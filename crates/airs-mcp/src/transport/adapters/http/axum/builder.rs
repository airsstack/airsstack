//! AxumHttpServer Builder
//!
//! This module provides a builder pattern for AxumHttpServer that enables:
//! - Parameter-free Default implementation
//! - Engine self-configuration capabilities
//! - Progressive complexity from beginner to expert usage
//!
//! The builder complements the generic HttpTransportBuilder by handling
//! Axum-specific configuration and setup.

use std::sync::Arc;

use crate::transport::adapters::http::axum::AxumHttpServer;
use crate::transport::adapters::http::config::HttpTransportConfig;
use crate::transport::adapters::http::connection_manager::HttpConnectionManager;
use crate::transport::error::TransportError;

/// Builder for AxumHttpServer with progressive configuration options
///
/// This builder enables engine self-configuration by providing sensible defaults
/// and allowing step-by-step customization. It supports the progressive developer
/// experience tiers defined in Phase 5 architecture.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::axum::AxumHttpServerBuilder;
///
/// // Tier 1: Complete beginner (parameter-free)
/// let server = AxumHttpServerBuilder::default().build();
///
/// // Tier 2: Basic customization
/// let server = AxumHttpServerBuilder::default()
///     .max_connections(500)
///     .bind_port(8080)
///     .build();
///
/// // Tier 3: Advanced configuration
/// let config = HttpTransportConfig::new()
///     .max_connections(2000)
///     .session_timeout(Duration::from_secs(600));
///
/// let server = AxumHttpServerBuilder::default()
///     .with_config(config)
///     .build();
///
/// // Tier 4: Expert control (fully custom components)
/// let connection_manager = Arc::new(HttpConnectionManager::new(
///     5000,
///     custom_health_config
/// ));
///
/// let server = AxumHttpServerBuilder::default()
///     .with_connection_manager(connection_manager)
///     .with_config(custom_config)
///     .build();
/// ```
#[derive(Debug)]
pub struct AxumHttpServerBuilder {
    config: Option<HttpTransportConfig>,
    connection_manager: Option<Arc<HttpConnectionManager>>,
}

impl Default for AxumHttpServerBuilder {
    /// Create a builder with no parameters required
    ///
    /// This enables the parameter-free Default implementation pattern
    /// required by Phase 5.2. All dependencies will be created with
    /// sensible defaults when build() is called.
    fn default() -> Self {
        Self {
            config: None,
            connection_manager: None,
        }
    }
}

impl AxumHttpServerBuilder {
    /// Create a new builder (same as Default::default())
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom configuration
    ///
    /// If not called, a default HttpTransportConfig will be used.
    pub fn with_config(mut self, config: HttpTransportConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set a custom connection manager
    ///
    /// If not called, a default HttpConnectionManager will be created.
    pub fn with_connection_manager(
        mut self,
        connection_manager: Arc<HttpConnectionManager>,
    ) -> Self {
        self.connection_manager = Some(connection_manager);
        self
    }

    /// Set maximum connections (convenience method for Tier 2 usage)
    ///
    /// This creates or modifies the internal config to set max_connections.
    /// Provides simpler API for common configuration needs.
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        let config = self.config.unwrap_or_else(HttpTransportConfig::new);
        self.config = Some(config.max_connections(max_connections));
        self
    }

    /// Set bind port (convenience method for Tier 2 usage)
    ///
    /// This creates or modifies the internal config to change the bind port
    /// while keeping the host as 127.0.0.1.
    pub fn bind_port(mut self, port: u16) -> Self {
        let config = self.config.unwrap_or_else(HttpTransportConfig::new);
        let bind_address = format!("127.0.0.1:{}", port)
            .parse()
            .expect("Valid bind address");
        self.config = Some(config.bind_address(bind_address));
        self
    }

    /// Set session timeout (convenience method for Tier 2 usage)
    ///
    /// This creates or modifies the internal config to set session timeout.
    pub fn session_timeout(mut self, timeout: std::time::Duration) -> Self {
        let config = self.config.unwrap_or_else(HttpTransportConfig::new);
        self.config = Some(config.session_timeout(timeout));
        self
    }

    /// Build the AxumHttpServer
    ///
    /// This method creates all missing dependencies with sensible defaults:
    /// - HttpTransportConfig::new() if no config provided
    /// - HttpConnectionManager::with_defaults() if no connection manager provided
    ///
    /// # Errors
    ///
    /// Returns TransportError if server construction fails due to invalid
    /// configuration or resource allocation issues.
    pub fn build(self) -> Result<AxumHttpServer, TransportError> {
        let config = self.config.unwrap_or_else(HttpTransportConfig::new);

        let connection_manager = self
            .connection_manager
            .unwrap_or_else(|| Arc::new(HttpConnectionManager::with_defaults()));

        AxumHttpServer::from_parts(connection_manager, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_default_builder() {
        let builder = AxumHttpServerBuilder::default();
        assert!(builder.config.is_none());
        assert!(builder.connection_manager.is_none());
    }

    #[test]
    fn test_builder_with_config() {
        let config = HttpTransportConfig::new().max_connections(500);
        let builder = AxumHttpServerBuilder::default().with_config(config.clone());

        assert!(builder.config.is_some());
        assert_eq!(builder.config.unwrap().max_connections, 500);
    }

    #[test]
    fn test_convenience_methods() {
        let builder = AxumHttpServerBuilder::default()
            .max_connections(2000)
            .bind_port(9090)
            .session_timeout(Duration::from_secs(300));

        let config = builder.config.unwrap();
        assert_eq!(config.max_connections, 2000);
        assert_eq!(config.bind_address.port(), 9090);
        assert_eq!(config.session_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_build_with_defaults() {
        let result = AxumHttpServerBuilder::default().build();
        assert!(result.is_ok(), "Should build successfully with defaults");
    }

    #[test]
    fn test_build_with_custom_config() {
        let config = HttpTransportConfig::new().max_connections(1500);
        let result = AxumHttpServerBuilder::default().with_config(config).build();

        assert!(
            result.is_ok(),
            "Should build successfully with custom config"
        );
    }

    #[test]
    fn test_progressive_complexity_tiers() {
        // Tier 1: Complete beginner
        let _server1 = AxumHttpServerBuilder::default().build();

        // Tier 2: Basic customization
        let _server2 = AxumHttpServerBuilder::default()
            .max_connections(500)
            .bind_port(8080)
            .build();

        // Tier 3: Advanced configuration
        let config = HttpTransportConfig::new()
            .max_connections(2000)
            .session_timeout(Duration::from_secs(600));

        let _server3 = AxumHttpServerBuilder::default().with_config(config).build();

        // Tier 4: Expert control
        let connection_manager = Arc::new(HttpConnectionManager::with_defaults());
        let _server4 = AxumHttpServerBuilder::default()
            .with_connection_manager(connection_manager)
            .with_config(HttpTransportConfig::new())
            .build();

        // All tiers should compile and work
    }
}

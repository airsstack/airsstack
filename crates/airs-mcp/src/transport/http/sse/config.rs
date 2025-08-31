//! HTTP SSE Transport Configuration
//!
//! This module provides configuration structures for the HTTP SSE Transport,
//! extending the HTTP Streamable Transport foundation with SSE-specific settings.
//!
//! ⚠️  **DEPRECATION NOTICE**: This transport is provided for ecosystem
//! transition support. New implementations should use HTTP Streamable transport.

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::time::Duration;

use super::constants::{DEFAULT_MESSAGES_ENDPOINT, DEFAULT_SSE_ENDPOINT};
use crate::transport::http::config::HttpTransportConfig;

/// HTTP SSE Transport Configuration
///
/// Extends the base HTTP configuration with SSE-specific settings while
/// leveraging the shared infrastructure foundation.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::http::sse::{HttpSseConfig, MigrationMode};
/// use std::time::Duration;
///
/// // Default SSE configuration with deprecation warnings
/// let config = HttpSseConfig::new();
///
/// // Custom SSE configuration with migration guidance
/// let config = HttpSseConfig::new()
///     .sse_endpoint_path("/events")
///     .messages_endpoint_path("/api/messages")
///     .migration_mode(MigrationMode::Active);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct HttpSseConfig {
    /// Base HTTP transport configuration (shared infrastructure)
    pub base_config: HttpTransportConfig,

    /// SSE endpoint configuration
    pub sse_endpoint: SseEndpointConfig,

    /// Messages endpoint path
    pub messages_endpoint: String,

    /// Deprecation settings and warnings
    pub deprecation: DeprecationConfig,

    /// Migration assistance mode
    pub migration_mode: MigrationMode,
}

/// SSE-specific endpoint configuration
#[derive(Debug, Clone, PartialEq)]
pub struct SseEndpointConfig {
    /// SSE endpoint path (default: "/sse")
    pub path: String,

    /// Event heartbeat interval for client connection maintenance
    pub heartbeat_interval: Duration,

    /// Maximum events buffered per session
    pub max_event_buffer: usize,

    /// Event retry interval suggested to clients
    pub retry_interval: Duration,

    /// Maximum number of concurrent SSE connections
    pub max_sse_connections: usize,

    /// SSE connection timeout
    pub connection_timeout: Duration,
}

/// Deprecation configuration and warnings
#[derive(Debug, Clone, PartialEq)]
pub struct DeprecationConfig {
    /// Enable deprecation warnings in HTTP responses
    pub warnings_enabled: bool,

    /// Planned sunset date for SSE transport (if known)
    pub sunset_date: Option<DateTime<Utc>>,

    /// Warning frequency to avoid response spam
    pub warning_frequency: Duration,

    /// Warning escalation based on proximity to sunset
    pub escalate_warnings: bool,
}

/// Migration assistance mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MigrationMode {
    /// Silent operation, no migration assistance
    Silent,
    /// Passive hints in headers/responses
    Passive,
    /// Active migration suggestions and tools
    Active,
    /// Aggressive migration promotion with performance comparisons
    Aggressive,
}

impl HttpSseConfig {
    /// Create new SSE configuration with sensible defaults
    ///
    /// Default values optimized for legacy compatibility:
    /// - Base HTTP config with conservative settings
    /// - SSE endpoint at "/sse"
    /// - Messages endpoint at "/messages"
    /// - Passive migration mode with deprecation warnings
    /// - 30-second heartbeat interval
    /// - Maximum 1000 SSE connections (lower than HTTP Streamable)
    pub fn new() -> Self {
        let mut base_config = HttpTransportConfig::new();

        // Apply conservative SSE-specific defaults
        base_config.max_connections = 1000; // Lower than HTTP Streamable
        base_config.session_timeout = Duration::from_secs(1800); // 30 minutes
        base_config.request_timeout = Duration::from_secs(60); // Longer for SSE

        Self {
            base_config,
            sse_endpoint: SseEndpointConfig::new(),
            messages_endpoint: DEFAULT_MESSAGES_ENDPOINT.to_string(),
            deprecation: DeprecationConfig::new(),
            migration_mode: MigrationMode::Passive,
        }
    }

    /// Set SSE endpoint path
    pub fn sse_endpoint_path(mut self, path: impl Into<String>) -> Self {
        self.sse_endpoint.path = path.into();
        self
    }

    /// Set messages endpoint path
    pub fn messages_endpoint_path(mut self, path: impl Into<String>) -> Self {
        self.messages_endpoint = path.into();
        self
    }

    /// Set migration assistance mode
    pub fn migration_mode(mut self, mode: MigrationMode) -> Self {
        self.migration_mode = mode;
        self
    }

    /// Set planned sunset date for deprecation timeline
    pub fn sunset_date(mut self, date: Option<DateTime<Utc>>) -> Self {
        self.deprecation.sunset_date = date;
        self
    }

    /// Enable or disable deprecation warnings
    pub fn deprecation_warnings(mut self, enabled: bool) -> Self {
        self.deprecation.warnings_enabled = enabled;
        self
    }

    /// Set SSE heartbeat interval
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.sse_endpoint.heartbeat_interval = interval;
        self
    }

    /// Set maximum SSE connections
    pub fn max_sse_connections(mut self, max: usize) -> Self {
        self.sse_endpoint.max_sse_connections = max;
        self
    }

    /// Set maximum event buffer size per session
    pub fn max_event_buffer(mut self, max: usize) -> Self {
        self.sse_endpoint.max_event_buffer = max;
        self
    }

    /// Configure for high-performance legacy scenarios
    ///
    /// Adjusts settings for maximum performance within SSE constraints
    pub fn high_performance(mut self) -> Self {
        self.base_config.max_connections = 1000; // Still conservative
        self.base_config.max_concurrent_requests = 20;
        self.sse_endpoint.max_sse_connections = 800;
        self.sse_endpoint.heartbeat_interval = Duration::from_secs(15);
        self.sse_endpoint.max_event_buffer = 1000;
        self
    }

    /// Configure for migration encouragement
    ///
    /// Enables active migration assistance and performance comparisons
    pub fn encourage_migration(mut self) -> Self {
        self.migration_mode = MigrationMode::Active;
        self.deprecation.warnings_enabled = true;
        self.deprecation.escalate_warnings = true;
        self
    }
}

impl Default for HttpSseConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SseEndpointConfig {
    /// Create new SSE endpoint configuration with defaults
    pub fn new() -> Self {
        Self {
            path: DEFAULT_SSE_ENDPOINT.to_string(),
            heartbeat_interval: Duration::from_secs(30),
            max_event_buffer: 100,
            retry_interval: Duration::from_secs(3),
            max_sse_connections: 1000,
            connection_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for SseEndpointConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DeprecationConfig {
    /// Create new deprecation configuration with defaults
    pub fn new() -> Self {
        Self {
            warnings_enabled: true,
            sunset_date: None,
            warning_frequency: Duration::from_secs(300), // 5 minutes
            escalate_warnings: false,
        }
    }

    /// Determine current deprecation phase based on sunset date
    pub fn current_phase(&self) -> DeprecationPhase {
        let Some(sunset_date) = self.sunset_date else {
            return DeprecationPhase::Active;
        };

        let now = Utc::now();
        let time_until_sunset = sunset_date.signed_duration_since(now);

        if time_until_sunset.num_days() > 365 {
            DeprecationPhase::PreAnnouncement
        } else if time_until_sunset.num_days() > 180 {
            DeprecationPhase::InitialDeprecation
        } else if time_until_sunset.num_days() > 90 {
            DeprecationPhase::ActiveMigration
        } else if time_until_sunset.num_days() > 30 {
            DeprecationPhase::FinalNotice
        } else if time_until_sunset.num_days() > 0 {
            DeprecationPhase::ImmediateSunset
        } else {
            DeprecationPhase::Sunset
        }
    }
}

impl Default for DeprecationConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Deprecation phase for timeline management
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeprecationPhase {
    /// Before official deprecation announcement
    PreAnnouncement,
    /// Initial deprecation period (passive warnings)
    InitialDeprecation,
    /// Active migration period (active assistance)
    ActiveMigration,
    /// Final notice period (aggressive warnings)
    FinalNotice,
    /// Immediate sunset (urgent migration required)
    ImmediateSunset,
    /// Transport discontinued
    Sunset,
    /// No sunset date set (ongoing active status)
    Active,
}

impl MigrationMode {
    /// Check if migration assistance is enabled
    pub fn is_active(&self) -> bool {
        matches!(self, MigrationMode::Active | MigrationMode::Aggressive)
    }

    /// Check if warnings should be displayed
    pub fn show_warnings(&self) -> bool {
        !matches!(self, MigrationMode::Silent)
    }
}

// Extension methods for HttpTransportConfig
impl HttpTransportConfig {
    /// Convert HTTP Streamable config to SSE config with defaults
    ///
    /// This method provides a convenient way to transition configuration
    /// from HTTP Streamable to SSE while maintaining compatible settings.
    pub fn to_sse_config(self) -> HttpSseConfig {
        HttpSseConfig {
            base_config: self,
            sse_endpoint: SseEndpointConfig::new(),
            messages_endpoint: DEFAULT_MESSAGES_ENDPOINT.to_string(),
            deprecation: DeprecationConfig::new(),
            migration_mode: MigrationMode::Passive,
        }
    }

    /// Create SSE config with migration encouragement
    ///
    /// Suitable for scenarios where active migration to HTTP Streamable
    /// should be promoted while maintaining SSE compatibility.
    pub fn to_sse_config_with_migration_encouragement(self) -> HttpSseConfig {
        self.to_sse_config().encourage_migration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_config_defaults() {
        let config = HttpSseConfig::new();

        assert_eq!(config.sse_endpoint.path, DEFAULT_SSE_ENDPOINT);
        assert_eq!(config.messages_endpoint, DEFAULT_MESSAGES_ENDPOINT);
        assert_eq!(config.migration_mode, MigrationMode::Passive);
        assert!(config.deprecation.warnings_enabled);
    }

    #[test]
    fn test_sse_config_builder() {
        let config = HttpSseConfig::new()
            .sse_endpoint_path("/events")
            .messages_endpoint_path("/api/messages")
            .migration_mode(MigrationMode::Active)
            .heartbeat_interval(Duration::from_secs(15));

        assert_eq!(config.sse_endpoint.path, "/events");
        assert_eq!(config.messages_endpoint, "/api/messages");
        assert_eq!(config.migration_mode, MigrationMode::Active);
        assert_eq!(
            config.sse_endpoint.heartbeat_interval,
            Duration::from_secs(15)
        );
    }

    #[test]
    fn test_deprecation_phases() {
        let mut config = DeprecationConfig::new();

        // Test with no sunset date
        assert_eq!(config.current_phase(), DeprecationPhase::Active);

        // Test with future sunset date (200 days = InitialDeprecation phase)
        config.sunset_date = Some(Utc::now() + chrono::Duration::days(200));
        assert_eq!(config.current_phase(), DeprecationPhase::InitialDeprecation);

        // Test with closer sunset date (120 days = ActiveMigration phase)
        config.sunset_date = Some(Utc::now() + chrono::Duration::days(120));
        assert_eq!(config.current_phase(), DeprecationPhase::ActiveMigration);

        // Test with imminent sunset (15 days = ImmediateSunset phase)
        config.sunset_date = Some(Utc::now() + chrono::Duration::days(15));
        assert_eq!(config.current_phase(), DeprecationPhase::ImmediateSunset);
    }

    #[test]
    fn test_migration_mode_behavior() {
        assert!(MigrationMode::Active.is_active());
        assert!(MigrationMode::Aggressive.is_active());
        assert!(!MigrationMode::Passive.is_active());
        assert!(!MigrationMode::Silent.is_active());

        assert!(MigrationMode::Passive.show_warnings());
        assert!(!MigrationMode::Silent.show_warnings());
    }

    #[test]
    fn test_http_config_to_sse_conversion() {
        let http_config = HttpTransportConfig::new()
            .max_connections(5000)
            .session_timeout(Duration::from_secs(600));

        let sse_config = http_config.to_sse_config();

        assert_eq!(sse_config.base_config.max_connections, 5000);
        assert_eq!(
            sse_config.base_config.session_timeout,
            Duration::from_secs(600)
        );
        assert_eq!(sse_config.migration_mode, MigrationMode::Passive);
    }
}

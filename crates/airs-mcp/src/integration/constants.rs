//! Integration Layer Constants
//!
//! This module re-exports protocol constants for backward compatibility
//! and adds integration-specific constants.

// Re-export protocol constants
pub use crate::protocol::constants::*;

/// Default configuration values for MCP client
pub mod defaults {
    /// Default client name
    pub const CLIENT_NAME: &str = "airs-mcp-client";

    /// Default timeout in seconds
    pub const TIMEOUT_SECONDS: u64 = 30;

    /// Default maximum retry attempts
    pub const MAX_RETRIES: u32 = 3;

    /// Default initial retry delay in milliseconds
    pub const INITIAL_RETRY_DELAY_MS: u64 = 100;

    /// Default maximum retry delay in seconds
    pub const MAX_RETRY_DELAY_SECONDS: u64 = 30;

    /// Default maximum reconnection attempts
    pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;

    /// Default initial reconnection delay in seconds
    pub const INITIAL_RECONNECT_DELAY_SECONDS: u64 = 1;

    /// Default maximum reconnection delay in seconds
    pub const MAX_RECONNECT_DELAY_SECONDS: u64 = 60;

    /// Default auto-retry setting
    pub const AUTO_RETRY: bool = true;

    /// Default auto-reconnect setting  
    pub const AUTO_RECONNECT: bool = false;
}

/// Integration-specific defaults
pub mod integration_defaults {
    /// Default client configuration timeout
    pub const CLIENT_TIMEOUT_MS: u64 = 30_000;

    /// Default maximum pending requests
    pub const MAX_PENDING_REQUESTS: usize = 1000;

    /// Default retry delay in milliseconds
    pub const RETRY_DELAY_MS: u64 = 1000;

    /// Default strict validation setting
    pub const STRICT_VALIDATION: bool = true;

    /// Default log operations setting
    pub const LOG_OPERATIONS: bool = false;
}

//! Integration Layer Constants
//!
//! This module re-exports protocol constants for backward compatibility
//! and adds integration-specific constants.

// Re-export protocol constants
pub use crate::protocol::constants::*;

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
//! API Key HTTP Authentication Module
//!
//! This module provides HTTP-specific adapter for API key authentication,
//! following the same pattern as OAuth2 authentication.

pub mod adapter;

// Re-exports for convenience
pub use adapter::{ApiKeyConfig, ApiKeyStrategyAdapter};

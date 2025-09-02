//! HTTP Authentication Adapters
//!
//! This module provides authentication adapters for HTTP transport,
//! bridging various authentication strategies to HTTP request handling.

pub mod oauth2;

// Re-export main types for convenience
pub use oauth2::{HttpAuthError, HttpAuthRequest, HttpExtractor, OAuth2StrategyAdapter};

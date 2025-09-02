//! OAuth2 Authentication Strategy
//!
//! This module provides OAuth2 authentication strategy implementation
//! that integrates with the existing OAuth2 validation infrastructure.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports

pub mod request;
pub mod strategy;

// Re-exports
pub use request::{OAuth2AuthRequest, OAuth2Request};
pub use strategy::OAuth2Strategy;

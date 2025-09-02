//! Authentication Strategies
//!
//! This module contains different authentication strategy implementations
//! following the AuthenticationStrategy trait pattern.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports

pub mod oauth2;

// Re-exports for convenience
pub use oauth2::{OAuth2AuthRequest, OAuth2Request, OAuth2Strategy};

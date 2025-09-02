//! OAuth2 HTTP Authentication Module
//!
//! This module provides OAuth2 authentication components specifically
//! designed for HTTP transport integration.

pub mod adapter;
pub mod error;
pub mod extractor;

pub use adapter::{HttpAuthRequest, OAuth2StrategyAdapter};
pub use error::HttpAuthError;
pub use extractor::HttpExtractor;

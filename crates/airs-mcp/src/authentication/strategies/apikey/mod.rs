//! API Key Authentication Strategy
//!
//! This module provides authentication via API keys, supporting multiple
//! common API key patterns used in MCP servers.

pub mod strategy;
pub mod types;
pub mod validator;

// Re-exports for convenience
pub use strategy::ApiKeyStrategy;
pub use types::{ApiKeyRequest, ApiKeySource};
pub use validator::{ApiKeyAuthData, ApiKeyValidator, InMemoryApiKeyValidator};

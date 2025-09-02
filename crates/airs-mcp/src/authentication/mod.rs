//! Multi-Method Authentication for MCP Protocol
//!
//! This module provides a unified authentication framework supporting multiple
//! authentication methods while maintaining 100% backward compatibility with
//! the existing OAuth2 implementation.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports

pub mod context;
pub mod error;
pub mod manager;
pub mod metadata;
pub mod method;
pub mod request;
pub mod strategy;

// Re-exports
pub use context::AuthContext;
pub use error::{AuthError, AuthResult};
pub use manager::{AuthenticationManager, ManagerConfig, DEFAULT_AUTH_TIMEOUT, DEFAULT_ENABLE_AUDIT_LOGGING};
pub use metadata::AuthMetadata;
pub use method::AuthMethod;
pub use request::AuthRequest;
pub use strategy::AuthenticationStrategy;

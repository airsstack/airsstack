//! Authentication Strategy Trait
//!
//! Async interface for authentication strategies with zero-cost abstractions.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::authentication::context::AuthContext;
use crate::authentication::error::AuthResult;
use crate::authentication::method::AuthMethod;
use crate::authentication::request::AuthRequest;

/// Generic authentication strategy interface
///
/// Uses async_trait for clean async methods and pure generic interface
/// following workspace standard §6. Each strategy defines its own data types.
#[async_trait]
pub trait AuthenticationStrategy<T, D>: Send + Sync + 'static
where
    T: Send + Sync,
    D: Send + Sync + 'static,
{
    /// Authentication method this strategy handles
    fn method(&self) -> AuthMethod;

    /// Authenticate a request using this strategy
    async fn authenticate(&self, request: &impl AuthRequest<T>) -> AuthResult<AuthContext<D>>;

    /// Validate an existing authentication context
    async fn validate(&self, context: &AuthContext<D>) -> AuthResult<bool>;
}

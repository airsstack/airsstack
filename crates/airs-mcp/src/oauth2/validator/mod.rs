//! OAuth2 Validator Module
//!
//! This module provides a unified, trait-based OAuth2 validation system following
//! workspace standards for zero-cost abstractions and performance optimization.
//!
//! ## Architecture
//!
//! The module follows composition over inheritance using generic trait constraints:
//! - `Jwt`: JWT token validation (no "Validator" suffix per naming convention)
//! - `Scope`: OAuth scope validation for MCP method authorization
//! - `Validator<J, S>`: Main composition object with zero-cost trait dispatch
//! - `ValidatorBuilder`: Type-safe builder pattern for construction
//!
//! ## Workspace Standards Compliance
//!
//! - **§1**: Generic type usage with trait bounds for zero-cost abstractions
//! - **§2**: No unnecessary `'static` lifetime constraints
//! - **§3**: Stack allocation preferred, no unnecessary `Box<T>`
//! - **§4**: Trait design with associated types for flexibility

// Module declarations
pub mod builder;
pub mod jwt;
pub mod scope;
#[allow(clippy::module_inception)]
pub mod validator;

// Re-exports for public API
pub use builder::{create_default_validator, create_validator_with_mappings, ValidatorBuilder};
pub use jwt::{Jwt, JwtValidator};
pub use scope::{Scope, ScopeValidator};
pub use validator::Validator;

// Re-export common types for convenience
pub use crate::oauth2::{
    context::AuthContext,
    error::{OAuth2Error, OAuth2Result},
    types::JwtClaims,
};

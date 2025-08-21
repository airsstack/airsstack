//! OAuth 2.1 Authentication for MCP Protocol
//!
//! This module provides enterprise-grade OAuth 2.1 authentication with:
//! - RFC 9728: Protected Resource Metadata
//! - RFC 7636: PKCE (Proof Key for Code Exchange)
//! - RFC 8707: Resource Indicators
//! - MCP 2025-06-18: Protocol integration
//!
//! # Architecture
//!
//! The OAuth 2.1 implementation follows a middleware-based architecture that
//! integrates seamlessly with existing HTTP transports:
//!
//! ```rust,no_run
//! use airs_mcp::oauth2::{OAuth2Config, oauth2_middleware_layer};
//! use axum::{Router, routing::post};
//! use url::Url;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Configure OAuth 2.1
//! let config = OAuth2Config::builder()
//!     .jwks_url(Url::parse("https://auth.example.com/.well-known/jwks.json")?)
//!     .audience("mcp-server".to_string())
//!     .issuer("https://auth.example.com".to_string())
//!     .build()?;
//!
//! // Create router with OAuth middleware (placeholder implementation)
//! let _app: Router = Router::new()
//!     .route("/mcp", post(handle_mcp_request));
//!     // .layer(oauth2_middleware_layer(config)); // Will be implemented in Phase 1, Step 2
//! # Ok(())
//! # }
//! # async fn handle_mcp_request() -> &'static str { "MCP Handler" }
//! ```
//!
//! # Features
//!
//! ## JWT Token Validation
//! - Automatic JWKS key retrieval and caching
//! - RS256 signature validation
//! - Audience and issuer verification
//! - Token expiration and not-before claims validation
//!
//! ## Scope-Based Access Control
//! - MCP method to OAuth scope mapping
//! - Operation-specific scope checking
//! - Support for custom scope mappings
//!
//! ## Request Context
//! - Authenticated request context with user information
//! - Scope information for fine-grained access control
//! - Audit logging support
//!
//! ## Standards Compliance
//! - RFC 6750: Bearer Token Usage
//! - RFC 9728: Protected Resource Metadata (Phase 1, Step 3)
//! - RFC 7636: PKCE Support (Phase 2)
//! - RFC 8707: Resource Indicators (Phase 2)

// Sub-modules
pub mod config;
pub mod context;
pub mod error;
pub mod jwt_validator;
pub mod metadata;
pub mod middleware;
pub mod scope_validator;

// Re-exports for public API
pub use config::{OAuth2Config, ScopeMapping};
pub use context::{AuditLogEntry, AuthContext, AuthContextExt, AuthMetadata};
pub use error::{OAuth2Error, OAuth2Result};
pub use jwt_validator::{JwksResponse, JwtClaims, JwtValidator};
pub use metadata::{oauth_metadata_handler, ProtectedResourceMetadata};
pub use middleware::{oauth2_middleware_handler, oauth2_middleware_layer, OAuth2Middleware};
pub use scope_validator::ScopeValidator;

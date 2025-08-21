//! OAuth 2.1 Middleware Module
//!
//! This module provides a comprehensive trait-based middleware architecture for OAuth 2.1
//! authentication and authorization. It separates core OAuth logic from HTTP framework
//! specifics, enabling clean integration with different web frameworks.
//!
//! # Architecture
//!
//! The middleware module follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 HTTP Framework Layer                        │
//! │              (Axum, Warp, Tide, etc.)                     │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │              OAuth Middleware Traits                       │
//! │   (OAuthMiddleware, OAuthRequestProcessor, etc.)          │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │             Framework-Agnostic Core                        │
//! │         (Authentication, Authorization Logic)              │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │               OAuth Foundation                              │
//! │      (JWT Validation, Scope Checking, Config)             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage Examples
//!
//! ## Basic Usage with Axum
//!
//! ```rust,no_run
//! use airs_mcp::oauth2::{
//!     OAuth2Config,
//!     middleware::{OAuth2MiddlewareCore, axum::AxumOAuth2Middleware},
//! };
//! use axum::{Router, routing::post};
//! use tower::Layer;
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
//! // Create and apply middleware
//! let middleware = AxumOAuth2Middleware::new(config)?;
//! let app = Router::<()>::new()
//!     .route("/mcp", post(handle_mcp_request))
//!     .layer(middleware);
//! # Ok(())
//! # }
//! # async fn handle_mcp_request() -> &'static str { "MCP Handler" }
//! ```
//!
//! ## Custom Framework Integration
//!
//! ```rust,no_run
//! use airs_mcp::oauth2::{
//!     middleware::{traits::*, OAuth2MiddlewareCore},
//!     error::OAuth2Error,
//! };
//! use async_trait::async_trait;
//!
//! // Custom framework request/response types
//! struct MyRequest;
//! struct MyResponse;
//! struct MyNext;
//!
//! // Implement OAuth traits for your framework
//! struct MyFrameworkOAuthMiddleware {
//!     core: OAuth2MiddlewareCore,
//! }
//!
//! #[async_trait]
//! impl OAuthMiddleware for MyFrameworkOAuthMiddleware {
//!     type Request = MyRequest;
//!     type Response = MyResponse;
//!     type Next = MyNext;
//!     type Error = OAuth2Error;
//!
//!     async fn handle_oauth(
//!         &self,
//!         request: Self::Request,
//!         next: Self::Next,
//!     ) -> Result<Self::Response, Self::Error> {
//!         // Your framework-specific implementation
//!         todo!()
//!     }
//! }
//! ```

// Public modules
pub mod core;
pub mod traits;
pub mod types;
pub mod utils;

// Framework-specific implementations
pub mod axum;

// Re-exports for public API
pub use core::OAuth2MiddlewareCore;
pub use traits::{
    AuthenticationProvider, FullOAuthMiddleware, OAuthMiddleware, OAuthMiddlewareConfig,
    OAuthRequestProcessor, OAuthResponseBuilder,
};
pub use types::{
    AuthenticationResult, CorsConfig, HttpError, MiddlewareConfig, MiddlewareContext,
    MiddlewareError, MiddlewareRequest, MiddlewareResponse, MiddlewareResult, ProcessingResult,
};
pub use utils::{create_default_core, MiddlewareBuilder};

// Framework-specific re-exports
pub use self::axum::{oauth2_middleware_layer, AxumOAuth2Middleware};

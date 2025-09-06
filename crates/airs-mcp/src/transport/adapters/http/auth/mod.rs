//! HTTP Authentication Adapters
//!
//! This module provides authentication adapters for HTTP transport,
//! bridging various authentication strategies to HTTP request handling.

pub mod apikey;
pub mod axum_middleware;
pub mod jsonrpc_authorization;
pub mod middleware;
pub mod oauth2;

// Re-export main types for convenience
pub use apikey::{ApiKeyConfig, ApiKeyStrategyAdapter};
pub use axum_middleware::{AxumHttpAuthLayer, AxumHttpAuthMiddleware, AxumHttpAuthService};
pub use jsonrpc_authorization::{
    JsonRpcAuthorizationLayer, JsonRpcAuthorizationLayerBuilder, JsonRpcHttpRequest,
    OAuth2JsonRpcAuthorizationLayer, jsonrpc_authorization_middleware,
};
pub use middleware::{HttpAuthConfig, HttpAuthMiddleware, HttpAuthRequest as MiddlewareHttpAuthRequest, HttpAuthStrategyAdapter};
pub use oauth2::{HttpAuthError, HttpAuthRequest, HttpExtractor, OAuth2StrategyAdapter};

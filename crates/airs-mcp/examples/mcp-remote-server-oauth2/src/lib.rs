//! OAuth2 MCP Remote Server Library
//!
//! This library demonstrates OAuth2 authentication integration with AirsStack's
//! MCP Transport infrastructure. It shows how to:
//!
//! - Use AxumHttpServer with OAuth2StrategyAdapter
//! - Set up OAuth2Strategy with JWT validation
//! - Integrate with MCP providers (resources, tools, prompts)
//! - Create test infrastructure for OAuth2 token validation

pub mod auth;
pub mod testing;
pub mod config;

// Re-export commonly used types for convenience
pub use auth::{keys::TestKeys, tokens::TokenConfig, setup::OAuth2Setup};
pub use testing::{jwks::MockJwksServer, endpoints::TokenEndpoints};
pub use config::server::ServerConfig;

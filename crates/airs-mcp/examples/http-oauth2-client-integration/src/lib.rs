//! HTTP OAuth2 Client Integration Library
//!
//! This library provides a complete OAuth2 authorization code flow implementation
//! with PKCE support, integrating with MCP (Model Context Protocol) operations.

// Module declarations
pub mod cli;
pub mod integration;
pub mod mcp_client;
pub mod oauth2;
pub mod oauth2_client;

// Re-export all public types and functions from modules
pub use cli::{args, Config};
pub use integration::FlowOrchestrator;
pub use mcp_client::{McpOperations, McpSession};
pub use oauth2::{
    create_code_challenge, extract_jwt_claims_unverified, generate_random_string,
    is_valid_jwt_format, AuthorizationCode, McpServerConfig, OAuth2ClientConfig, OAuth2Error,
    OAuth2Flow, OAuth2IntegrationError, OAuth2ServerConfig, PkceChallenge, PkceGenerator,
    TokenClaims, TokenManager, TokenResponse, TokenStore,
};
pub use oauth2_client::{
    simulate_automatic_authorization, simulate_interactive_authorization,
    TokenManager as OAuth2TokenManager,
};

// OAuth2 client implementation with PKCE support

pub mod config;
pub mod errors;
pub mod flow;
pub mod pkce;
pub mod tokens;
pub mod types;
pub mod utils;

// Re-export all types and functions
pub use config::{McpServerConfig, OAuth2ClientConfig, OAuth2ServerConfig};
pub use errors::OAuth2IntegrationError;
pub use flow::OAuth2Flow;
pub use pkce::PkceGenerator;
pub use tokens::TokenManager;
pub use types::{AuthorizationCode, OAuth2Error, PkceChallenge, TokenClaims, TokenResponse, TokenStore};
pub use utils::{create_code_challenge, extract_jwt_claims_unverified, generate_random_string, is_valid_jwt_format};

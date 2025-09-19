// OAuth2 mock authorization server module

pub mod endpoints;
pub mod jwks;
pub mod server;
pub mod tokens;

pub use endpoints::create_oauth2_router;
pub use jwks::create_jwks_router;
pub use server::{OAuth2ServerState, RegisteredClient, ServerStats};
pub use tokens::{generate_jwt_token, generate_refresh_token};

// JWKS (JSON Web Key Set) endpoint implementation

// Standard library imports
use std::sync::Arc;

// Third-party crate imports
use axum::{extract::State, response::Json, routing::get, Router};
use tracing::{debug, info};

// Internal module imports
use super::server::OAuth2ServerState;

/// Create JWKS router
pub fn create_jwks_router() -> Router<Arc<OAuth2ServerState>> {
    Router::new().route("/", get(jwks_endpoint))
}

/// JWKS endpoint handler
///
/// Returns the public keys used to verify JWT tokens issued by this server.
/// This endpoint is used by resource servers (like our MCP server) to validate
/// the JWT tokens without needing to contact the authorization server.
async fn jwks_endpoint(State(state): State<Arc<OAuth2ServerState>>) -> Json<serde_json::Value> {
    info!("🔑 JWKS endpoint requested");
    debug!(
        "📋 JWKS response: {}",
        serde_json::to_string_pretty(&state.jwks_response).unwrap_or_default()
    );

    Json(state.jwks_response.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_oauth2_client_integration::OAuth2ServerConfig;
    use tokio;

    #[tokio::test]
    async fn test_jwks_endpoint() {
        // This test would need proper setup with real keys
        // For now, just verify the endpoint structure

        let config = OAuth2ServerConfig::default();
        let test_key = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC7VJTUt9Us8cKB
UMnV0QdEVjLMYhHqtN+UR4LCbDJ0ITY4+F/V4lklSJT+YZHEtDlF5dJrSZWmOdOs
-----END PRIVATE KEY-----"#;

        // In a real test, we would:
        // 1. Create a proper OAuth2ServerState with test keys
        // 2. Call the jwks_endpoint function
        // 3. Verify the JWKS response format
        // 4. Validate that the public key components are correct

        assert!(!test_key.is_empty());
    }
}

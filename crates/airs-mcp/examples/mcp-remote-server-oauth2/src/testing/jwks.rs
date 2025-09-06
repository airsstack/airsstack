//! Mock JWKS Server for OAuth2 Testing
//!
//! This module provides a mock JWKS server that serves the public keys
//! for JWT token validation by AirsStack's OAuth2Strategy.

use axum::{extract::State, response::Json, routing::get, Router};
use serde_json::Value;
use tokio::net::TcpListener;
use tracing::{info, warn};

use crate::auth::keys::TestKeys;

/// Mock JWKS server state
#[derive(Clone)]
pub struct MockJwksState {
    pub jwks_response: Value,
}

/// JWKS endpoint handler - serves public keys for JWT validation
async fn jwks_endpoint(State(state): State<MockJwksState>) -> Json<Value> {
    Json(state.jwks_response.clone())
}

/// Mock JWKS server for testing OAuth2 JWT validation
pub struct MockJwksServer;

impl MockJwksServer {
    /// Start mock JWKS server for JWT validation testing
    ///
    /// This server provides the JWKS endpoint that AirsStack's OAuth2Strategy
    /// will use to fetch public keys for JWT signature verification.
    pub async fn start(test_keys: TestKeys) -> Result<(), Box<dyn std::error::Error>> {
        let state = MockJwksState {
            jwks_response: test_keys.jwks_response,
        };

        let app = Router::new()
            .route("/.well-known/jwks.json", get(jwks_endpoint))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:3002").await?;
        info!("🔑 Mock JWKS server started on http://127.0.0.1:3002");
        info!("📋 JWKS endpoint: http://127.0.0.1:3002/.well-known/jwks.json");

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                warn!("JWKS server error: {}", e);
            }
        });

        Ok(())
    }
}

// OAuth2 Mock Authorization Server
// Provides complete OAuth2 authorization endpoints for testing

// Standard library imports
use std::net::SocketAddr;
use std::sync::Arc;

// Third-party crate imports
use axum::Router;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;
use tracing_subscriber;

// Internal module imports
use http_oauth2_client_integration::{OAuth2ServerConfig, OAuth2IntegrationError};

mod server;
mod endpoints;
mod tokens;
mod jwks;

use server::OAuth2ServerState;

/// CLI arguments for the OAuth2 mock authorization server
#[derive(Parser, Debug)]
#[command(name = "http-oauth2-mock-server")]
#[command(about = "OAuth2 Mock Authorization Server for MCP Integration Testing")]
struct Args {
    /// Server host to bind to
    #[arg(long, default_value = "localhost")]
    host: String,

    /// Server port to bind to
    #[arg(long, default_value = "3002")]
    port: u16,

    /// OAuth2 issuer URL
    #[arg(long, default_value = "http://localhost:3002")]
    issuer: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), OAuth2IntegrationError> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .init();

    info!("🚀 Starting OAuth2 Mock Authorization Server");
    info!("🌐 Server: {}:{}", args.host, args.port);
    info!("🔒 Issuer: {}", args.issuer);

    // Load server configuration
    let config = OAuth2ServerConfig {
        server_host: args.host.clone(),
        server_port: args.port,
        issuer: args.issuer,
        jwks_endpoint: format!("http://{}:{}/jwks", args.host, args.port),
        private_key_pem: load_private_key()?,
    };

    // Initialize server state
    let state = Arc::new(OAuth2ServerState::new(config).await?);

    // Create the application router
    let app = create_app_router(state);

    // Bind to the address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .map_err(|_| OAuth2IntegrationError::Configuration {
            message: "Invalid host/port combination".to_string(),
        })?;

    let listener = TcpListener::bind(&addr).await
        .map_err(|e| OAuth2IntegrationError::Configuration {
            message: format!("Failed to bind to {}: {}", addr, e),
        })?;

    info!("✅ OAuth2 Authorization Server listening on {}", addr);
    info!("🔑 Available endpoints:");
    info!("   - GET  /jwks                 (JWKS public keys)");
    info!("   - GET  /authorize            (OAuth2 authorization)");
    info!("   - POST /token                (OAuth2 token exchange)");
    info!("   - GET  /.well-known/openid-configuration (OpenID discovery)");
    info!("   - GET  /health               (Health check)");

    // Start the server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| OAuth2IntegrationError::Configuration {
            message: format!("Server error: {}", e),
        })?;

    info!("👋 OAuth2 Authorization Server shutdown complete");
    Ok(())
}

/// Create the application router with all endpoints
fn create_app_router(state: Arc<OAuth2ServerState>) -> Router {
    Router::new()
        .nest("/", endpoints::create_oauth2_router())
        .nest("/jwks", jwks::create_jwks_router())
        .with_state(state)
}

/// Load the private key for JWT signing
fn load_private_key() -> Result<String, OAuth2IntegrationError> {
    // Try to load from test_keys directory
    let key_path = "test_keys/private_key.pem";
    
    if std::path::Path::new(key_path).exists() {
        std::fs::read_to_string(key_path)
            .map_err(|e| OAuth2IntegrationError::Configuration {
                message: format!("Failed to read private key from {}: {}", key_path, e),
            })
    } else {
        Err(OAuth2IntegrationError::Configuration {
            message: format!(
                "Private key not found at {}. Please run './scripts/setup_keys.sh' or './scripts/setup_keys.py' first.",
                key_path
            ),
        })
    }
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("📡 Received Ctrl+C signal, shutting down gracefully");
        },
        _ = terminate => {
            info!("📡 Received terminate signal, shutting down gracefully");
        },
    }
}
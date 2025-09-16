//! OAuth2 Integration Example for MCP Inspector Compatibility
//!
//! This example demonstrates OAuth2 authorization code flow with PKCE support for MCP Inspector compatibility.
//! It create    // 1. Start Mock JWKS Server (for JWT validation)
//! on the same port as the MCP endpoint:
//!
//! 1. **Proxy Server (Port 3002)**: Public-facing server that routes requests
//! 2. **MCP Server (Port 3001)**: OAuth2-protected MCP operations  
//! 3. **Custom Routes Server (Port 3003)**: OAuth2 discovery, authorization, and dev tools
//!
//! The proxy intelligently routes requests:
//! - `/mcp/*` → MCP Server (OAuth2-protected MCP operations)
//! - `/*` → Custom Routes Server (OAuth2 discovery, authorization, dev tools)

// Layer 1: Standard library imports
use std::{path::PathBuf, sync::Arc, time::Duration};

// Layer 2: Third-party crate imports
use clap::Parser;
use tokio::{
    signal,
    task::{JoinError, JoinHandle},
    time::sleep,
};
use tracing::{info, warn};

// Layer 3: Internal module imports
mod auth_flow;
mod config;
mod jwks;
mod proxy;
mod server;
mod tokens;

use airs_mcp::{
    authentication::strategies::oauth2::OAuth2Strategy,
    integration::McpServer,
    oauth2::validator::{Jwt, Scope, Validator},
    transport::adapters::http::{
        auth::{middleware::HttpAuthConfig, oauth2::OAuth2StrategyAdapter},
        axum::AxumHttpServer,
        config::HttpTransportConfig,
        connection_manager::{HealthCheckConfig, HttpConnectionManager},
        HttpTransportBuilder,
    },
};

use config::create_oauth2_config;
use jwks::start_mock_jwks_server;
use proxy::start_proxy_server;
use server::create_test_environment;
use tokens::TestKeys;

/// Command line arguments for the OAuth2 MCP integration server
#[derive(Parser, Debug)]
#[command(
    name = "oauth2-integration",
    about = "OAuth2 MCP Integration Server for MCP Inspector Compatibility",
    long_about = "This server implements a three-server proxy architecture that enables \
                  MCP Inspector compatibility by providing OAuth2 discovery endpoints \
                  on the same port as the MCP endpoint."
)]
struct Args {
    /// Private key file for JWT signing (PEM format)
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Path to private key file in PEM format"
    )]
    private_key: Option<PathBuf>,

    /// Proxy server bind address (public-facing)
    #[arg(
        long,
        default_value = "127.0.0.1:3002",
        help = "Proxy server bind address (public endpoint)"
    )]
    proxy_addr: String,

    /// MCP server bind address (internal)
    #[arg(
        long,
        default_value = "127.0.0.1:3001",
        help = "MCP server bind address (OAuth2-protected MCP operations)"
    )]
    mcp_addr: String,

    /// Custom routes server bind address (internal)
    #[arg(
        long,
        default_value = "127.0.0.1:3003",
        help = "Custom routes server bind address (OAuth2 discovery and authorization)"
    )]
    custom_routes_addr: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize comprehensive logging for OAuth2 debugging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| {
                    "oauth2_mcp_integration=debug,airs_mcp=debug,oauth2=debug,jsonwebtoken=debug,axum=info".to_string()
                })
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting OAuth2 MCP Integration Server with Three-Server Architecture");

    // Print server configuration
    print_server_info(&args);

    // Generate test keys for JWT operations
    let test_keys = TestKeys::generate()?;

    // Create OAuth2 configuration
    let oauth2_config = Arc::new(create_oauth2_config()?);

    // Start all three servers in background tasks
    let servers = start_three_server_architecture(args, test_keys, oauth2_config).await?;

    // Wait for shutdown signal
    info!("=== 🏁 SERVERS STARTED - WAITING FOR SHUTDOWN SIGNAL ===");
    wait_for_shutdown().await;

    // Graceful shutdown
    info!("🛑 Received shutdown signal, stopping all servers...");
    shutdown_servers(servers).await?;

    info!("✅ All servers stopped gracefully");
    Ok(())
}

/// Print comprehensive server configuration information
fn print_server_info(args: &Args) {
    info!("=== 🔧 SERVER CONFIGURATION ===");
    info!("📡 Proxy Server (Public): http://{}", args.proxy_addr);
    info!("🔒 MCP Server (Internal): http://{}", args.mcp_addr);
    info!(
        "🔐 Custom Routes (Internal): http://{}",
        args.custom_routes_addr
    );
    info!("");
    info!("=== 🌐 THREE-SERVER ARCHITECTURE ===");
    info!(
        "1. 📡 Proxy Server ({}): Routes all incoming requests",
        args.proxy_addr
    );
    info!("   • /mcp/* → MCP Server (OAuth2-protected operations)");
    info!("   • /* → Custom Routes Server (discovery, auth, dev tools)");
    info!("");
    info!(
        "2. 🔒 MCP Server ({}): OAuth2-protected MCP operations",
        args.mcp_addr
    );
    info!("   • Requires valid OAuth2 Bearer token");
    info!("   • Provides MCP protocol endpoints");
    info!("");
    info!(
        "3. 🔐 Custom Routes Server ({}): OAuth2 infrastructure",
        args.custom_routes_addr
    );
    info!("   • /.well-known/oauth-authorization-server (discovery)");
    info!("   • /authorize (authorization endpoint)");
    info!("   • /token (token endpoint)");
    info!("   • /dev/* (development tools)");
    info!("");
    info!("=== � MCP INSPECTOR COMPATIBILITY ===");
    info!("• Inspector connects to: http://{}", args.proxy_addr);
    info!(
        "• Discovery endpoint: http://{}/.well-known/oauth-authorization-server",
        args.proxy_addr
    );
    info!(
        "• MCP endpoint: http://{}/mcp (OAuth2-protected)",
        args.proxy_addr
    );
    info!(
        "• Authorization endpoint: http://{}/authorize",
        args.proxy_addr
    );
    info!("• Token endpoint: http://{}/token", args.proxy_addr);
    info!("");
}

/// Server handle for tracking running servers
#[derive(Debug)]
struct ServerHandle {
    name: String,
    handle: JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
}

/// Start all three servers in the architecture
async fn start_three_server_architecture(
    args: Args,
    test_keys: TestKeys,
    oauth2_config: Arc<airs_mcp::oauth2::config::OAuth2Config>,
) -> Result<Vec<ServerHandle>, Box<dyn std::error::Error>> {
    let mut servers = Vec::new();

    // 1. Start Mock JWKS Server (for JWT validation)
    info!("� Starting Mock JWKS Server...");
    start_mock_jwks_server(test_keys.clone(), oauth2_config.as_ref().clone()).await?;

    // 2. Start MCP Server (OAuth2-protected MCP operations)
    info!("🔒 Starting MCP Server on {}...", args.mcp_addr);
    let mcp_handle = start_mcp_server(
        args.mcp_addr.clone(),
        test_keys.clone(),
        oauth2_config.clone(),
    )
    .await?;
    servers.push(ServerHandle {
        name: "MCP Server".to_string(),
        handle: mcp_handle,
    });

    // 3. Start Custom Routes Server (OAuth2 discovery and authorization)
    info!(
        "🔐 Starting Custom Routes Server on {}...",
        args.custom_routes_addr
    );
    let custom_routes_handle =
        start_custom_routes_server(args.custom_routes_addr.clone(), test_keys.clone()).await?;
    servers.push(ServerHandle {
        name: "Custom Routes Server".to_string(),
        handle: custom_routes_handle,
    });

    // 4. Start Proxy Server (public-facing router)
    info!("� Starting Proxy Server on {}...", args.proxy_addr);
    let proxy_handle = start_proxy_server_task(
        args.proxy_addr.clone(),
        format!("http://{}", args.mcp_addr),
        format!("http://{}", args.custom_routes_addr),
    )
    .await?;
    servers.push(ServerHandle {
        name: "Proxy Server".to_string(),
        handle: proxy_handle,
    });

    // Allow servers to initialize
    sleep(Duration::from_millis(1000)).await;

    info!("✅ All three servers started successfully!");
    Ok(servers)
}

/// Start the MCP server (OAuth2-protected MCP operations)
async fn start_mcp_server(
    bind_addr: String,
    _test_keys: TestKeys,
    oauth2_config: Arc<airs_mcp::oauth2::config::OAuth2Config>,
) -> Result<
    JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    Box<dyn std::error::Error>,
> {
    let handle = tokio::spawn(async move {
        // Create OAuth2 validators and strategy
        let jwt_validator = Jwt::new((*oauth2_config).clone())?;
        let scope_validator = Scope::with_default_mappings();
        let validator = Validator::new(jwt_validator, scope_validator);
        let oauth2_strategy = OAuth2Strategy::new(validator);
        let oauth2_adapter = OAuth2StrategyAdapter::new(oauth2_strategy);

        // Create HTTP authentication configuration
        let auth_config = HttpAuthConfig {
            include_error_details: true,
            auth_realm: "OAuth2 MCP Integration Server".to_string(),
            request_timeout_secs: 30,
            skip_paths: vec![
                "/health".to_string(),
                "/status".to_string(),
                "/metrics".to_string(),
                "/info".to_string(),
            ],
        };

        // Create HTTP transport configuration for MCP server
        let socket_addr: std::net::SocketAddr = bind_addr
            .parse()
            .map_err(|e| format!("Failed to parse bind address '{}': {}", bind_addr, e))?;
        let http_config = HttpTransportConfig::new().bind_address(socket_addr);

        // Create connection manager
        let connection_manager = HttpConnectionManager::new(1000, HealthCheckConfig::default());

        // Create test environment and MCP handlers
        let (handlers, _temp_dir_guard) = create_test_environment()
            .await
            .map_err(|e| format!("Failed to create test environment: {}", e))?;

        // Create the OAuth2-enabled MCP server using AxumHttpServer
        let mut engine = AxumHttpServer::from_parts(Arc::new(connection_manager), http_config)?
            .with_oauth2_authorization(oauth2_adapter, auth_config);

        // Register the MCP handlers
        engine.register_custom_mcp_handler(handlers);

        // Build HTTP transport
        let transport = HttpTransportBuilder::with_engine(engine)?
            .bind(bind_addr.parse()?)
            .await?
            .build()
            .await?;

        // Create MCP server
        let mcp_server = McpServer::new(transport);

        info!("🔒 MCP Server ready on {}", bind_addr);

        // Use tokio::select to ensure temp_dir stays alive until server is cancelled
        // The _temp_dir_guard will be automatically dropped when this function exits
        let result = tokio::select! {
            result = mcp_server.start() => {
                // Server finished normally
                result.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
            _ = tokio::signal::ctrl_c() => {
                // Server was cancelled
                info!("🛑 MCP Server cancelled");
                Ok(())
            }
        };

        // _temp_dir_guard is automatically dropped here when the function returns
        result
    });

    Ok(handle)
}

/// Start the custom routes server (OAuth2 discovery and authorization)
async fn start_custom_routes_server(
    bind_addr: String,
    test_keys: TestKeys,
) -> Result<
    JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    Box<dyn std::error::Error>,
> {
    let handle = tokio::spawn(async move {
        // Create custom routes server with OAuth2 endpoints
        let app = auth_flow::create_oauth2_routes_app(test_keys)?;

        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

        info!("🔐 Custom Routes Server ready on {}", bind_addr);

        axum::serve(listener, app).await?;
        Ok(()) as Result<(), Box<dyn std::error::Error + Send + Sync>>
    });

    Ok(handle)
}

/// Start the proxy server (public-facing router)
async fn start_proxy_server_task(
    bind_addr: String,
    mcp_server_url: String,
    custom_routes_url: String,
) -> Result<
    JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    Box<dyn std::error::Error>,
> {
    let handle = tokio::spawn(async move {
        if let Err(e) = start_proxy_server(&bind_addr, mcp_server_url, custom_routes_url).await {
            return Err(format!("Proxy server error: {}", e).into());
        }
        Ok(()) as Result<(), Box<dyn std::error::Error + Send + Sync>>
    });

    Ok(handle)
}

/// Wait for shutdown signal (Ctrl+C)
async fn wait_for_shutdown() {
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
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}

/// Gracefully shutdown all servers
async fn shutdown_servers(servers: Vec<ServerHandle>) -> Result<(), Box<dyn std::error::Error>> {
    for server in servers {
        info!("🛑 Stopping {}...", server.name);

        // Abort the server task
        server.handle.abort();

        // Wait for the task to complete or get result if already finished
        match server.handle.await {
            Ok(result) => match result {
                Ok(()) => info!("✅ {} stopped gracefully", server.name),
                Err(e) => warn!("⚠️ {} stopped with error: {}", server.name, e),
            },
            Err(JoinError { .. }) => {
                // Task was aborted, which is expected during shutdown
                info!("✅ {} stopped (aborted)", server.name);
            }
        }
    }

    Ok(())
}

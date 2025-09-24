//! HTTP API Key Client Integration
//!
//! This example demonstrates how to create an HTTP MCP client with API key authentication.
//! It includes a lightweight mock server for testing and supports multiple authentication methods.

// Layer 1: Standard library imports
use std::env;

// Layer 2: Third-party crate imports
use clap::{Parser, Subcommand};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

// Layer 3: Internal module imports
mod client;
mod config;

use client::HttpMcpClient;
use config::ClientConfig;

/// HTTP MCP Client CLI
#[derive(Parser)]
#[command(name = "http-apikey-client")]
#[command(about = "HTTP MCP client with API key authentication support")]
#[command(version)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Use mock server for testing
    #[arg(long)]
    mock: bool,

    /// Use production server (Phase 4.4)
    #[arg(long)]
    production: bool,

    /// Client command to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run interactive demo
    Demo,
    /// List available tools
    ListTools,
    /// Call a specific tool
    CallTool {
        /// Tool name to call
        name: String,
        /// Tool arguments as JSON string
        #[arg(short, long)]
        args: Option<String>,
    },
    /// List available resources
    ListResources,
    /// Read a specific resource
    ReadResource {
        /// Resource URI to read
        uri: String,
    },
    /// Test connection to server
    TestConnection,
    /// Validate configuration
    ValidateConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    // Initialize tracing
    let level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting HTTP MCP Client Integration");

    // Load configuration based on mode
    let config = if cli.mock || env::var("USE_MOCK").is_ok() {
        info!("Using mock server configuration");
        ClientConfig::for_mock_server()
    } else if cli.production || env::var("USE_PRODUCTION").is_ok() {
        info!("Using production server configuration (Phase 4.4)");
        ClientConfig::for_production_server()
    } else {
        info!("Loading configuration from environment");
        ClientConfig::from_env()
    };

    info!("Client configuration: {:?}", config);

    // Execute command
    match cli.command {
        Some(Commands::Demo) => run_demo(config).await,
        Some(Commands::ListTools) => run_list_tools(config).await,
        Some(Commands::CallTool { name, args }) => run_call_tool(config, name, args).await,
        Some(Commands::ListResources) => run_list_resources(config).await,
        Some(Commands::ReadResource { uri }) => run_read_resource(config, uri).await,
        Some(Commands::TestConnection) => run_test_connection(config).await,
        Some(Commands::ValidateConfig) => run_validate_config(config).await,
        None => run_demo(config).await, // Default to demo
    }
}

async fn run_demo(config: ClientConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Running interactive demo");

    match HttpMcpClient::new(&config).await {
        Ok(mut client) => {
            if let Err(e) = client.run_demo().await {
                error!("Demo failed: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Failed to create client: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_list_tools(
    config: ClientConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = HttpMcpClient::new(&config).await?;
    client.initialize().await?;

    let tools = client.list_tools().await?;
    println!("Available tools:");
    for tool in tools {
        println!("  - {tool}");
    }

    Ok(())
}

async fn run_call_tool(
    config: ClientConfig,
    name: String,
    args: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = HttpMcpClient::new(&config).await?;
    client.initialize().await?;

    let arguments = if let Some(args_str) = args {
        Some(serde_json::from_str(&args_str)?)
    } else {
        None
    };

    let result = client.call_tool(&name, arguments).await?;
    println!("Tool result: {result}");

    Ok(())
}

async fn run_list_resources(
    config: ClientConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = HttpMcpClient::new(&config).await?;
    client.initialize().await?;

    let resources = client.list_resources().await?;
    println!("Available resources:");
    for resource in resources {
        println!("  - {resource}");
    }

    Ok(())
}

async fn run_read_resource(
    config: ClientConfig,
    uri: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = HttpMcpClient::new(&config).await?;
    client.initialize().await?;

    let content = client.read_resource(&uri).await?;
    println!("Resource content: {content}");

    Ok(())
}

async fn run_test_connection(
    config: ClientConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Testing connection to server: {}", config.server_url);

    match HttpMcpClient::new(&config).await {
        Ok(mut client) => match client.initialize().await {
            Ok(()) => {
                println!("✓ Connection successful!");
                println!("✓ Authentication working");
                println!("✓ MCP session initialized");
            }
            Err(e) => {
                println!("✗ Connection failed during initialization: {e}");
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("✗ Failed to create client: {e}");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_validate_config(
    config: ClientConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Configuration validation:");
    println!("  Server URL: {}", config.server_url);
    println!("  Auth Method: {:?}", config.auth_method);
    println!("  API Key: [REDACTED]");
    println!("  Timeout: {:?}", config.timeout);
    println!("  Mock Mode: {}", config.mock_mode);

    // Basic validation
    if config.server_url.is_empty() {
        println!("✗ Server URL is empty");
        std::process::exit(1);
    }

    if config.api_key.is_empty() {
        println!("✗ API key is empty");
        std::process::exit(1);
    }

    println!("✓ Configuration appears valid");
    Ok(())
}

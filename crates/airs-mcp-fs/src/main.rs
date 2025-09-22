//! AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol
//!
//! Binary entry point for the MCP server that provides secure filesystem operations
//! for Claude Desktop and other MCP-compatible AI tools.

// Layer 1: Standard library imports
use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info};

// Layer 3: Internal module imports
// Layer 3a: AIRS foundation crates (prioritized)
use airs_mcp::integration::McpServer;
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;

// Layer 3b: Local crate modules
use airs_mcp_fs::mcp::FilesystemMessageHandler;
use airs_mcp_fs::{DefaultFilesystemMcpServer, Settings};

#[derive(Parser)]
#[command(name = "airs-mcp-fs")]
#[command(about = "AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate example configuration files
    GenerateConfig {
        /// Output directory for configuration files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Environment to generate config for
        #[arg(short, long, default_value = "development")]
        env: String,

        /// Whether to overwrite existing files
        #[arg(long)]
        force: bool,
    },
    /// Run the MCP server (default when no command specified)
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // ⚠️ DEPRECATION WARNING - Display migration notice
    eprintln!("\n⚠️  DEPRECATION NOTICE: airs-mcp-fs has moved!");
    eprintln!("   New location: mcp-servers/airs-mcpserver-fs");
    eprintln!("   New binary: airs-mcpserver-fs");
    eprintln!("   Migration guide: https://github.com/airsstack/airsstack/blob/main/mcp-servers/airs-mcpserver-fs/MIGRATION.md");
    eprintln!("   Legacy support ends: December 31, 2025\n");

    // Determine if we're running as MCP server (serve command or no command)
    let is_mcp_server = matches!(
        cli.command.as_ref().unwrap_or(&Commands::Serve),
        Commands::Serve
    );

    // Initialize structured logging with environment filter support
    if is_mcp_server {
        // For MCP server: log to files to keep STDIO clean
        let log_dir = std::env::var("AIRS_MCP_FS_LOG_DIR").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{home}/.local/share/airs-mcp-fs/logs")
        });

        // Create log directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("Warning: Failed to create log directory {log_dir}: {e}");
        }

        let log_file = format!("{log_dir}/airs-mcp-fs.log");
        let file_appender = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Warning: Failed to open log file {log_file}: {e}, logging disabled");
                // If we can't create log file, disable logging completely for MCP mode
                tracing_subscriber::fmt()
                    .with_env_filter("off")
                    .with_writer(std::io::sink)
                    .init();
                return match cli.command.unwrap_or(Commands::Serve) {
                    Commands::GenerateConfig { output, env, force } => {
                        generate_config(output, &env, force).await
                    }
                    Commands::Serve => run_server().await,
                };
            }
        };

        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "airs_mcp_fs=info".into()),
            )
            .with_ansi(false)
            .with_writer(file_appender)
            .init();
    } else {
        // For CLI commands: use standard console output with colors
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "airs_mcp_fs=info".into()),
            )
            .init();
    }

    match cli.command.unwrap_or(Commands::Serve) {
        Commands::GenerateConfig { output, env, force } => {
            generate_config(output, &env, force).await
        }
        Commands::Serve => run_server().await,
    }
}

async fn generate_config(output_dir: PathBuf, env: &str, force: bool) -> Result<()> {
    info!("🔧 Generating configuration files for environment: {}", env);

    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
        info!("📁 Created directory: {}", output_dir.display());
    }

    let config_content = match env {
        "development" => include_str!("../examples/config/development.toml"),
        "staging" => include_str!("../examples/config/staging.toml"),
        "production" => include_str!("../examples/config/production.toml"),
        _ => include_str!("../examples/config/config.toml"),
    };

    let config_file = output_dir.join(format!("{env}.toml"));

    if config_file.exists() && !force {
        error!(
            "❌ Configuration file already exists: {}",
            config_file.display()
        );
        error!("   Use --force to overwrite or choose a different output directory");
        process::exit(1);
    }

    fs::write(&config_file, config_content)?;
    info!("✅ Generated configuration: {}", config_file.display());

    // Also copy the base config.toml if generating development
    if env == "development" {
        let base_config = output_dir.join("config.toml");
        if !base_config.exists() || force {
            let base_content = include_str!("../examples/config/config.toml");
            fs::write(&base_config, base_content)?;
            info!("✅ Generated base configuration: {}", base_config.display());
        }
    }

    info!("🎉 Configuration generation complete!");
    info!("📖 Next steps:");
    info!(
        "   1. Edit {} to customize your settings",
        config_file.display()
    );
    info!("   2. Update allowed_paths for your specific use case");
    info!("   3. Run: cargo run --bin airs-mcp-fs serve");
    info!("   4. See CONFIG_GUIDE.md for detailed setup instructions");

    Ok(())
}

async fn run_server() -> Result<()> {
    info!(
        "🚀 Starting AIRS MCP-FS server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration settings
    let settings = match Settings::load() {
        Ok(settings) => {
            info!("✅ Configuration loaded successfully");
            settings
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            error!("💡 Try running: airs-mcp-fs generate-config");
            process::exit(1);
        }
    };

    // Create STDIO transport for Claude Desktop integration
    info!("🔌 Creating STDIO transport with MessageHandler integration");

    // Initialize filesystem MCP server
    let filesystem_server = match DefaultFilesystemMcpServer::with_default_handlers(settings).await
    {
        Ok(server) => {
            info!("✅ Filesystem MCP server initialized with security manager");
            Arc::new(server)
        }
        Err(e) => {
            error!("❌ Failed to initialize filesystem server: {}", e);
            process::exit(1);
        }
    };

    // Create MessageHandler wrapper for the server
    let message_handler = Arc::new(FilesystemMessageHandler::new(filesystem_server));
    info!("✅ MessageHandler wrapper created");

    // Create and configure STDIO transport with handler
    let transport = match StdioTransportBuilder::new()
        .with_message_handler(message_handler)
        .build()
        .await
    {
        Ok(transport) => {
            info!("✅ STDIO transport created with MessageHandler integration");
            transport
        }
        Err(e) => {
            error!("❌ Failed to create STDIO transport: {}", e);
            process::exit(1);
        }
    };

    // Wrap transport in high-level McpServer for lifecycle management
    let server = McpServer::new(transport);
    info!("✅ MCP server wrapper created");

    info!("🚀 Starting AIRS MCP-FS server");
    info!("📋 Available capabilities:");
    info!("   • Tools: read_file, write_file, list_directory");
    info!("   • Security: Path validation, approval workflows, audit logging");
    info!("   • Transport: STDIO integration for Claude Desktop");
    info!("");
    info!("💡 Usage:");
    info!("   Connect via Claude Desktop MCP client configuration");
    info!("   Send JSON-RPC requests to stdin, receive responses on stdout");

    // Start the server - this will run indefinitely until interrupted
    if let Err(e) = server.start().await {
        error!("❌ Failed to start MCP server: {}", e);
        process::exit(1);
    }

    info!("✅ AIRS MCP-FS server shutdown complete");
    Ok(())
}

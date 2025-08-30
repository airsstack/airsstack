//! AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol
//!
//! Binary entry point for the MCP server that provides secure filesystem operations
//! for Claude Desktop and other MCP-compatible AI tools.

// Layer 1: Standard library imports
use std::fs;
use std::path::PathBuf;
use std::process;

// Layer 2: Third-party crate imports
use airs_mcp::integration::mcp::McpServerBuilder;
use airs_mcp::transport::StdioTransport;
use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info};

// Layer 3: Internal module imports
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
                eprintln!(
                    "Warning: Failed to open log file {log_file}: {e}, logging disabled"
                );
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
    let transport = match StdioTransport::new().await {
        Ok(transport) => {
            info!("✅ STDIO transport initialized for Claude Desktop");
            transport
        }
        Err(e) => {
            error!("❌ Failed to create STDIO transport: {}", e);
            process::exit(1);
        }
    };

    // Initialize filesystem MCP server
    let filesystem_server = match DefaultFilesystemMcpServer::with_default_handlers(settings).await
    {
        Ok(server) => {
            info!("✅ Filesystem MCP server initialized with security manager");
            server
        }
        Err(e) => {
            error!("❌ Failed to initialize filesystem server: {}", e);
            process::exit(1);
        }
    };

    // Build complete MCP server with STDIO transport and filesystem tools
    let mcp_server = match McpServerBuilder::new()
        .server_info("airs-mcp-fs", env!("CARGO_PKG_VERSION"))
        .with_tool_provider(filesystem_server)
        .build(transport)
        .await
    {
        Ok(server) => {
            info!("✅ Complete MCP server built successfully");
            info!("📋 Available tools: read_file, write_file, list_directory");
            info!("🔗 Ready for Claude Desktop connections via STDIO");
            server
        }
        Err(e) => {
            error!("❌ Failed to build MCP server: {}", e);
            process::exit(1);
        }
    };

    // Start the server event loop
    info!("🎯 Starting MCP server event loop...");
    if let Err(e) = mcp_server.run().await {
        error!("💥 MCP server runtime error: {}", e);
        process::exit(1);
    }

    info!("🛑 AIRS MCP-FS server shutdown completed");
    Ok(())
}

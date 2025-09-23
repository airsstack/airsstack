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
use airs_mcp::protocol::Transport;
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;

// Layer 3b: Local crate modules
use airs_mcpserver_fs::mcp::FilesystemMessageHandler;
use airs_mcpserver_fs::{DefaultFilesystemMcpServer, Settings};

#[derive(Parser)]
#[command(name = "airs-mcpserver-fs")]
#[command(about = "AIRS MCP-FS: Security-first filesystem bridge for Model Context Protocol")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup AIRS MCP server directory structure
    Setup {
        /// Custom configuration directory (default: ~/.airs-mcpserver-fs/config)
        #[arg(long, help = "Custom configuration directory")]
        config_dir: Option<PathBuf>,

        /// Custom logs directory (default: ~/.airs-mcpserver-fs/logs)
        #[arg(long, help = "Custom logs directory")]
        logs_dir: Option<PathBuf>,

        /// Whether to overwrite existing directories
        #[arg(long)]
        force: bool,
    },
    /// Generate example configuration files
    Config {
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
    Serve {
        /// Custom configuration directory (overrides AIRS_MCPSERVER_FS_CONFIG_DIR)
        #[arg(long, help = "Custom configuration directory")]
        config_dir: Option<PathBuf>,

        /// Custom logs directory (overrides AIRS_MCPSERVER_FS_LOG_DIR)
        #[arg(long, help = "Custom logs directory")]
        logs_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine if we're running as MCP server (serve command or no command)
    let is_mcp_server = matches!(
        cli.command.as_ref().unwrap_or(&Commands::Serve {
            config_dir: None,
            logs_dir: None
        }),
        Commands::Serve { .. }
    );

    // Extract logs directory for server command
    let logs_dir_override = if let Some(Commands::Serve { logs_dir, .. }) = &cli.command {
        logs_dir.clone()
    } else if matches!(cli.command, None) {
        None // Default serve command
    } else {
        None // Not a serve command
    };

    // Initialize structured logging with environment filter support
    if is_mcp_server {
        // For MCP server: log to files to keep STDIO clean
        let log_dir = if let Some(custom_logs_dir) = logs_dir_override {
            custom_logs_dir.to_string_lossy().to_string()
        } else {
            std::env::var("AIRS_MCPSERVER_FS_LOG_DIR")
                .or_else(|_| std::env::var("AIRS_MCP_FS_LOG_DIR")) // Backward compatibility
                .unwrap_or_else(|_| {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    format!("{home}/.airs-mcpserver-fs/logs")
                })
        };

        // Create log directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("Warning: Failed to create log directory {log_dir}: {e}");
        }

        let log_file = format!("{log_dir}/airs-mcpserver-fs.log");
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
                return match cli.command.unwrap_or(Commands::Serve {
                    config_dir: None,
                    logs_dir: None,
                }) {
                    Commands::Setup {
                        config_dir,
                        logs_dir,
                        force,
                    } => setup_directories(config_dir, logs_dir, force).await,
                    Commands::Config { output, env, force } => {
                        generate_config(output, &env, force).await
                    }
                    Commands::Serve {
                        config_dir,
                        logs_dir,
                    } => run_server(config_dir, logs_dir).await,
                };
            }
        };

        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "airs_mcpserver_fs=info".into()),
            )
            .with_ansi(false)
            .with_writer(file_appender)
            .init();
    } else {
        // For CLI commands: use standard console output with colors
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "airs_mcpserver_fs=info".into()),
            )
            .init();
    }

    match cli.command.unwrap_or(Commands::Serve {
        config_dir: None,
        logs_dir: None,
    }) {
        Commands::Setup {
            config_dir,
            logs_dir,
            force,
        } => setup_directories(config_dir, logs_dir, force).await,
        Commands::Config { output, env, force } => generate_config(output, &env, force).await,
        Commands::Serve {
            config_dir,
            logs_dir,
        } => run_server(config_dir, logs_dir).await,
    }
}

async fn setup_directories(
    config_dir: Option<PathBuf>,
    logs_dir: Option<PathBuf>,
    force: bool,
) -> Result<()> {
    info!("🏗️ Setting up AIRS MCP Server directory structure");

    // Determine directories to create
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let default_base = format!("{}/.airs-mcpserver-fs", home);

    let config_path =
        config_dir.unwrap_or_else(|| PathBuf::from(format!("{}/config", default_base)));
    let logs_path = logs_dir.unwrap_or_else(|| PathBuf::from(format!("{}/logs", default_base)));

    info!("📁 Configuration directory: {}", config_path.display());
    info!("📁 Logs directory: {}", logs_path.display());

    // Check if directories already exist
    if config_path.exists() && !force {
        error!(
            "❌ Configuration directory already exists: {}",
            config_path.display()
        );
        error!("   Use --force to overwrite or choose a different directory");
        process::exit(1);
    }

    if logs_path.exists() && !force {
        error!("❌ Logs directory already exists: {}", logs_path.display());
        error!("   Use --force to overwrite or choose a different directory");
        process::exit(1);
    }

    // Create directories
    if let Err(e) = fs::create_dir_all(&config_path) {
        error!("❌ Failed to create configuration directory: {}", e);
        process::exit(1);
    }
    info!(
        "✅ Created configuration directory: {}",
        config_path.display()
    );

    if let Err(e) = fs::create_dir_all(&logs_path) {
        error!("❌ Failed to create logs directory: {}", e);
        process::exit(1);
    }
    info!("✅ Created logs directory: {}", logs_path.display());

    // Create a sample configuration file
    let config_file = config_path.join("development.toml");
    if !config_file.exists() || force {
        let config_content = include_str!("../examples/config/development.toml");
        if let Err(e) = fs::write(&config_file, config_content) {
            error!("❌ Failed to create sample configuration: {}", e);
        } else {
            info!("✅ Created sample configuration: {}", config_file.display());
        }
    }

    info!("🎉 Setup complete!");
    info!("📖 Next steps:");
    info!(
        "   1. Edit {} to customize your settings",
        config_file.display()
    );
    info!(
        "   2. Set AIRS_MCPSERVER_FS_CONFIG_DIR={}",
        config_path.display()
    );
    info!("   3. Run: airs-mcpserver-fs serve");
    info!("   4. See CONFIGURATION.md for detailed setup instructions");

    Ok(())
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
    info!("   3. Run: airs-mcpserver-fs serve");
    info!("   4. See CONFIGURATION.md for detailed setup instructions");

    Ok(())
}

async fn run_server(config_dir: Option<PathBuf>, _logs_dir: Option<PathBuf>) -> Result<()> {
    info!(
        "🚀 Starting AIRS MCP-FS server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration settings with custom config directory if provided
    let settings = if let Some(custom_config_dir) = config_dir {
        info!(
            "📁 Using custom configuration directory: {}",
            custom_config_dir.display()
        );

        // Temporarily set the environment variable for the configuration loader
        std::env::set_var("AIRS_MCPSERVER_FS_CONFIG_DIR", &custom_config_dir);

        // Create configuration loader with custom directory
        use airs_mcpserver_fs::config::ConfigurationLoader;
        let loader = ConfigurationLoader::new().with_config_dir(custom_config_dir);
        let (settings, source_info) = loader.load().map_err(|e| {
            error!(
                "❌ Failed to load configuration from custom directory: {}",
                e
            );
            e
        })?;

        // Log configuration source information
        info!(
            "📋 Configuration loaded from {} environment",
            source_info.environment
        );
        if !source_info.files.is_empty() {
            info!("   Configuration files: {:?}", source_info.files);
        }

        settings
    } else {
        // Use standard configuration loading
        match Settings::load() {
            Ok(settings) => {
                info!("✅ Configuration loaded successfully");
                settings
            }
            Err(e) => {
                error!("❌ Failed to load configuration: {}", e);
                error!("💡 Try running: airs-mcpserver-fs config");
                process::exit(1);
            }
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
    let mut transport = match StdioTransportBuilder::new()
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

    info!("🚀 Starting AIRS MCP-FS server");
    info!("📋 Available capabilities:");
    info!("   • Tools: read_file, write_file, list_directory");
    info!("   • Security: Path validation, approval workflows, audit logging");
    info!("   • Transport: STDIO integration for Claude Desktop");
    info!("");
    info!("💡 Usage:");
    info!("   Connect via Claude Desktop MCP client configuration");
    info!("   Send JSON-RPC requests to stdin, receive responses on stdout");

    // Start the transport - this begins reading from stdin in background
    if let Err(e) = transport.start().await {
        error!("❌ Failed to start STDIO transport: {}", e);
        process::exit(1);
    }

    info!("✅ STDIO transport started, ready for MCP communication");
    info!("� Server is now listening on stdin for JSON-RPC messages");

    // Wait for transport completion (blocks until stdin EOF or error)
    // This is the key fix - wait for the background stdin reader to complete
    if let Err(e) = transport.wait_for_completion().await {
        error!("❌ Transport error during operation: {}", e);
        process::exit(1);
    }

    info!("✅ AIRS MCP-FS server shutdown complete");
    Ok(())
}

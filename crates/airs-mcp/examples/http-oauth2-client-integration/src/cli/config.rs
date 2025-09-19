//! Configuration structure and validation for OAuth2 client integration

// Layer 2: Third-party crate imports
use clap::ArgMatches;
use tracing::info;

/// Configuration for OAuth2 client integration
#[derive(Debug, Clone)]
pub struct Config {
    pub auth_server_url: String,
    pub mcp_server_url: String,
    pub client_id: String,
    pub scope: String,
    pub interactive: bool,
}

impl Config {
    /// Create configuration from command line arguments
    pub fn from_args(matches: &ArgMatches) -> Self {
        let config = Self {
            auth_server_url: matches.get_one::<String>("auth-server").unwrap().clone(),
            mcp_server_url: matches.get_one::<String>("mcp-server").unwrap().clone(),
            client_id: matches.get_one::<String>("client-id").unwrap().clone(),
            scope: matches.get_one::<String>("scope").unwrap().clone(),
            interactive: matches.get_flag("interactive"),
        };

        config.log_configuration();
        config
    }

    /// Log the current configuration
    fn log_configuration(&self) {
        info!("📋 Configuration:");
        info!("  Auth Server: {}", self.auth_server_url);
        info!("  MCP Server: {}", self.mcp_server_url);
        info!("  Client ID: {}", self.client_id);
        info!("  Scope: {}", self.scope);
        info!("  Interactive: {}", self.interactive);
    }
}

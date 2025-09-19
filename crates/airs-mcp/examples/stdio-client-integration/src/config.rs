// Layer 1: Standard library imports
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (none for config)

/// Configuration for the STDIO MCP client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Command to execute for the MCP server
    pub server_command: String,

    /// Arguments to pass to the server command
    pub server_args: Vec<String>,

    /// Timeout for individual requests
    pub request_timeout: Duration,

    /// Environment variables to set for the server process
    pub server_env: HashMap<String, String>,

    /// Working directory for the server process
    pub server_working_dir: Option<PathBuf>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_command: "stdio-mcp-server".to_string(),
            server_args: vec![],
            request_timeout: Duration::from_secs(30),
            server_env: HashMap::new(),
            server_working_dir: None,
        }
    }
}

impl ClientConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(command) = env::var("MCP_SERVER_COMMAND") {
            config.server_command = command;
        }

        if let Ok(args) = env::var("MCP_SERVER_ARGS") {
            config.server_args = args.split_whitespace().map(|s| s.to_string()).collect();
        }

        if let Ok(timeout_str) = env::var("MCP_REQUEST_TIMEOUT") {
            if let Ok(timeout_secs) = timeout_str.parse::<u64>() {
                config.request_timeout = Duration::from_secs(timeout_secs);
            }
        }

        if let Ok(working_dir) = env::var("MCP_SERVER_WORKING_DIR") {
            config.server_working_dir = Some(PathBuf::from(working_dir));
        }

        config
    }

    /// Create a config for running against the mock server
    pub fn for_mock_server() -> Self {
        Self {
            server_command: "./target/debug/stdio-mock-server".to_string(),
            server_args: vec![],
            request_timeout: Duration::from_secs(10),
            server_env: HashMap::new(),
            server_working_dir: None,
        }
    }
}

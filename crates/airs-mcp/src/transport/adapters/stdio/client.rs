//! STDIO Transport Client Implementation
//!
//! This module provides a clean client-side STDIO transport implementation using the
//! TransportClient trait for direct request-response communication with MCP servers
//! running as child processes.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::timeout;
use tracing::warn;

// Layer 3: Internal module imports
use crate::protocol::{JsonRpcRequest, JsonRpcResponse, TransportClient, TransportError};

/// STDIO Transport Client for communicating with MCP servers via child processes
///
/// This client implementation spawns a child process and communicates with it
/// using stdin/stdout following the JSON-RPC protocol. It provides a clean
/// request-response interface without the complexity of event-driven patterns.
///
/// # Architecture
///
/// ```text
/// StdioTransportClient -> Child Process (MCP Server)
///     |                       |
///     |- stdin (requests) ->  |- reads requests
///     |                       |- processes requests  
///     |<- stdout (responses) <|- writes responses
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::stdio::{StdioTransportClient, StdioTransportClientBuilder};
/// use airs_mcp::protocol::{TransportClient, JsonRpcRequest, RequestId};
/// use serde_json::json;
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Build and configure the client
/// let mut client = StdioTransportClientBuilder::new()
///     .command("python")
///     .args(vec!["-m".to_string(), "my_mcp_server".to_string()])
///     .timeout(Duration::from_secs(30))
///     .build()
///     .await?;
///
/// // Send a request and get response directly
/// let request = JsonRpcRequest::new(
///     "initialize",
///     Some(json!({"capabilities": {}})),
///     RequestId::new_string("init-1")
/// );
///
/// let response = client.call(request).await?;
/// println!("Response: {:?}", response);
///
/// client.close().await?;
/// # Ok(())
/// # }
/// ```
pub struct StdioTransportClient {
    /// The spawned child process
    child: Child,

    /// Writer for sending requests to child's stdin
    stdin: BufWriter<ChildStdin>,

    /// Reader for receiving responses from child's stdout
    stdout: BufReader<ChildStdout>,

    /// Client configuration
    config: StdioClientConfig,
}

/// Configuration for StdioTransportClient
///
/// Contains settings for process spawning, communication timeouts,
/// and environment management.
#[derive(Debug, Clone)]
pub struct StdioClientConfig {
    /// Command to execute for the MCP server
    pub command: String,

    /// Arguments to pass to the command
    pub args: Vec<String>,

    /// Timeout for individual request-response cycles
    pub timeout: Duration,

    /// Environment variables to set for the child process
    pub env_vars: HashMap<String, String>,

    /// Working directory for the child process
    pub working_dir: Option<std::path::PathBuf>,

    /// Buffer size for stdin/stdout operations
    pub buffer_size: usize,
}

impl Default for StdioClientConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            timeout: Duration::from_secs(30),
            env_vars: HashMap::new(),
            working_dir: None,
            buffer_size: 8192,
        }
    }
}

#[async_trait]
impl TransportClient for StdioTransportClient {
    type Error = TransportError;

    async fn call(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse, TransportError> {
        // Check if child process is still running
        if let Ok(Some(exit_status)) = self.child.try_wait() {
            return Err(TransportError::not_ready(format!(
                "Child process exited with status: {exit_status:?}"
            )));
        }

        // Serialize the request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| TransportError::Serialization { source: e })?;

        // Send request with timeout
        let send_future = async {
            self.stdin.write_all(request_json.as_bytes()).await?;
            self.stdin.write_all(b"\n").await?;
            self.stdin.flush().await?;
            Ok::<(), std::io::Error>(())
        };

        timeout(self.config.timeout, send_future)
            .await
            .map_err(|_| TransportError::request_timeout(self.config.timeout))?
            .map_err(|e| TransportError::Io { source: e })?;

        // Read response with timeout
        let mut response_line = String::new();
        let read_future = self.stdout.read_line(&mut response_line);

        timeout(self.config.timeout, read_future)
            .await
            .map_err(|_| TransportError::request_timeout(self.config.timeout))?
            .map_err(|e| TransportError::Io { source: e })?;

        // Parse the response
        if response_line.trim().is_empty() {
            return Err(TransportError::invalid_response(
                "Received empty response from server",
            ));
        }

        let response: JsonRpcResponse =
            serde_json::from_str(response_line.trim()).map_err(|e| {
                TransportError::invalid_response(format!("Failed to parse JSON response: {e}"))
            })?;

        Ok(response)
    }

    fn is_ready(&self) -> bool {
        // Note: try_wait() requires mutable access, but is_ready() takes &self
        // We'll use a simple heuristic - if stdin is closed, assume not ready
        // For a more accurate check, the caller should use call() which will
        // detect process termination
        true // Assume ready unless call() detects otherwise
    }

    fn transport_type(&self) -> &'static str {
        "stdio"
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        // Flush and close stdin to signal the child process to exit gracefully
        if let Err(e) = self.stdin.flush().await {
            // Log the error but continue with termination
            warn!(%e, "Failed to flush stdin during close");
        }

        // The stdin will be closed when the BufWriter is dropped
        // but we can also send EOF explicitly by closing the underlying handle

        // Wait for the child process to exit with a timeout
        let wait_future = self.child.wait();
        match timeout(self.config.timeout, wait_future).await {
            Ok(Ok(_exit_status)) => {
                // Process exited cleanly
                Ok(())
            }
            Ok(Err(e)) => {
                // Error waiting for process
                Err(TransportError::Io { source: e })
            }
            Err(_) => {
                // Timeout waiting for process to exit, force kill
                if let Err(e) = self.child.kill().await {
                    return Err(TransportError::Io { source: e });
                }
                // Wait for kill to complete
                if let Err(e) = self.child.wait().await {
                    return Err(TransportError::Io { source: e });
                }
                Ok(())
            }
        }
    }
}

/// Builder for StdioTransportClient
///
/// Provides a fluent interface for configuring and creating StdioTransportClient instances.
/// The builder pattern ensures all required configuration is provided before creating the client.
///
/// # Examples
///
/// ```rust,no_run
/// use airs_mcp::transport::adapters::stdio::StdioTransportClientBuilder;
/// use std::time::Duration;
/// use std::collections::HashMap;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut client = StdioTransportClientBuilder::new()
///     .command("python")
///     .args(vec!["-m".to_string(), "my_mcp_server".to_string()])
///     .timeout(Duration::from_secs(60))
///     .env_var("PYTHONPATH", "/path/to/modules")
///     .working_dir("/path/to/working/dir")
///     .buffer_size(16384)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct StdioTransportClientBuilder {
    config: StdioClientConfig,
}

impl StdioTransportClientBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: StdioClientConfig::default(),
        }
    }

    /// Set the command to execute for the MCP server
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute (e.g., "python", "node", "/path/to/executable")
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.config.command = command.into();
        self
    }

    /// Set the arguments to pass to the command
    ///
    /// # Arguments
    ///
    /// * `args` - Vector of arguments to pass to the command
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.config.args = args;
        self
    }

    /// Add a single argument to the command
    ///
    /// # Arguments
    ///
    /// * `arg` - Single argument to add
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.config.args.push(arg.into());
        self
    }

    /// Set the timeout for request-response cycles
    ///
    /// # Arguments
    ///
    /// * `timeout` - Duration to wait for request-response completion
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set an environment variable for the child process
    ///
    /// # Arguments
    ///
    /// * `key` - Environment variable name
    /// * `value` - Environment variable value
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env_vars.insert(key.into(), value.into());
        self
    }

    /// Set multiple environment variables for the child process
    ///
    /// # Arguments
    ///
    /// * `env_vars` - HashMap of environment variables
    pub fn env_vars(mut self, env_vars: HashMap<String, String>) -> Self {
        self.config.env_vars = env_vars;
        self
    }

    /// Set the working directory for the child process
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to the working directory
    pub fn working_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.config.working_dir = Some(dir.into());
        self
    }

    /// Set the buffer size for stdin/stdout operations
    ///
    /// # Arguments
    ///
    /// * `size` - Buffer size in bytes
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.config.buffer_size = size;
        self
    }

    /// Build the StdioTransportClient
    ///
    /// This method spawns the child process and sets up the communication channels.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No command was specified
    /// - Failed to spawn the child process
    /// - Failed to set up stdin/stdout communication
    pub async fn build(self) -> Result<StdioTransportClient, TransportError> {
        if self.config.command.is_empty() {
            return Err(TransportError::not_ready(
                "No command specified for STDIO transport client",
            ));
        }

        // Spawn the child process
        let mut command = Command::new(&self.config.command);
        command
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()); // Let stderr pass through for debugging

        // Set environment variables
        for (key, value) in &self.config.env_vars {
            command.env(key, value);
        }

        // Set working directory if specified
        if let Some(ref dir) = self.config.working_dir {
            command.current_dir(dir);
        }

        let mut child = command.spawn().map_err(|e| TransportError::Connection {
            message: format!(
                "Failed to spawn child process '{command}': {e}",
                command = self.config.command
            ),
        })?;

        // Take stdin and stdout from the child process
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TransportError::Connection {
                message: "Failed to get stdin handle from child process".to_string(),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TransportError::Connection {
                message: "Failed to get stdout handle from child process".to_string(),
            })?;

        // Wrap in buffered readers/writers
        let stdin = BufWriter::new(stdin);
        let stdout = BufReader::new(stdout);

        Ok(StdioTransportClient {
            child,
            stdin,
            stdout,
            config: self.config,
        })
    }
}

impl Default for StdioTransportClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

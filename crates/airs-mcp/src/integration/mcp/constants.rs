//! MCP Protocol Constants
//!
//! This module defines constants for MCP protocol method names and other
//! standardized values to ensure consistency and reduce typos.

/// MCP protocol method names
pub mod methods {
    /// Initialization method
    pub const INITIALIZE: &str = "initialize";
    /// Initialized notification
    pub const INITIALIZED: &str = "initialized";

    /// Resource-related methods
    pub const RESOURCES_LIST: &str = "resources/list";
    pub const RESOURCES_READ: &str = "resources/read";
    pub const RESOURCES_SUBSCRIBE: &str = "resources/subscribe";
    pub const RESOURCES_UNSUBSCRIBE: &str = "resources/unsubscribe";
    pub const RESOURCES_TEMPLATES_LIST: &str = "resources/templates/list";

    /// Tool-related methods
    pub const TOOLS_LIST: &str = "tools/list";
    pub const TOOLS_CALL: &str = "tools/call";

    /// Prompt-related methods
    pub const PROMPTS_LIST: &str = "prompts/list";
    pub const PROMPTS_GET: &str = "prompts/get";

    /// Logging-related methods
    pub const LOGGING_SET_LEVEL: &str = "logging/setLevel";
}

/// JSON-RPC error codes
pub mod error_codes {
    /// Parse error - Invalid JSON was received by the server
    pub const PARSE_ERROR: i32 = -32700;

    /// Invalid Request - The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32600;

    /// Method not found - The method does not exist / is not available
    pub const METHOD_NOT_FOUND: i32 = -32601;

    /// Invalid params - Invalid method parameter(s)
    pub const INVALID_PARAMS: i32 = -32602;

    /// Internal error - Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Server error - Reserved for implementation-defined server-errors
    pub const SERVER_ERROR_START: i32 = -32099;
    pub const SERVER_ERROR_END: i32 = -32000;
}

/// Protocol version constants
pub mod protocol {
    /// Current MCP protocol version
    pub const CURRENT_VERSION: &str = "2024-11-05";

    /// JSON-RPC version
    pub const JSONRPC_VERSION: &str = "2.0";
}

/// Default configuration values
pub mod defaults {
    /// Default client name
    pub const CLIENT_NAME: &str = "airs-mcp-client";

    /// Default server name
    pub const SERVER_NAME: &str = "airs-mcp-server";

    /// Default timeout in seconds
    pub const TIMEOUT_SECONDS: u64 = 30;

    /// Default max retries
    pub const MAX_RETRIES: u32 = 3;

    /// Default strict validation setting
    pub const STRICT_VALIDATION: bool = true;

    /// Default log operations setting
    pub const LOG_OPERATIONS: bool = false;
}

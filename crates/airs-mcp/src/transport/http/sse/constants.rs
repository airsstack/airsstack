//! HTTP SSE Transport Constants
//!
//! This module defines constants for SSE transport endpoints, headers, and other
//! string literals to ensure consistency and prevent typos across the codebase.
//!
//! This follows the workspace-level technical standard of using constants
//! for all endpoint paths and HTTP-related string literals.

/// Default SSE endpoint path for Server-Sent Events streaming
pub const DEFAULT_SSE_ENDPOINT: &str = "/sse";

/// Default messages endpoint path for JSON request/response
pub const DEFAULT_MESSAGES_ENDPOINT: &str = "/messages";

/// HTTP headers used by SSE transport
pub mod headers {
    /// Deprecation warning header
    pub const TRANSPORT_DEPRECATED: &str = "X-Transport-Deprecated";

    /// Migration guidance header
    pub const MIGRATION_AVAILABLE: &str = "X-Migration-Available";

    /// SSE last event ID header
    pub const LAST_EVENT_ID: &str = "Last-Event-ID";

    /// Session ID header for correlation
    pub const SESSION_ID: &str = "X-Session-ID";
}

/// SSE event types
pub mod events {
    /// Standard message event type
    pub const MESSAGE: &str = "message";

    /// Heartbeat/keep-alive event type
    pub const HEARTBEAT: &str = "heartbeat";

    /// Error event type
    pub const ERROR: &str = "error";

    /// Migration suggestion event type
    pub const MIGRATION_HINT: &str = "migration-hint";
}

/// Content types used by SSE transport
pub mod content_types {
    /// Server-Sent Events content type
    pub const EVENT_STREAM: &str = "text/event-stream";

    /// JSON content type for messages endpoint
    pub const JSON: &str = "application/json";
}

/// Cache control values for SSE
pub mod cache_control {
    /// No-cache directive for SSE responses
    pub const NO_CACHE: &str = "no-cache";
}

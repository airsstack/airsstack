# STDIO Transport Implementation Reference - Complete Working Example

**Created:** 2025-09-13  
**Purpose:** Reference for TASK-031 Transport Builder Architectural Consistency  
**Source:** Complete STDIO implementation that correctly follows ADR-011 patterns

## Overview

This document contains the complete STDIO transport implementation that correctly implements the `TransportBuilder<()>` pattern. This serves as the architectural reference for implementing the missing `TransportBuilder<HttpContext>` for HTTP transport in TASK-031.

## Key Architecture Principles Demonstrated

1. **Pre-configured Pattern**: Handler set via `TransportBuilder::with_message_handler()`
2. **Build-time Validation**: `build()` fails if no handler is set
3. **Safe Transport Creation**: Transport created with handler already configured
4. **No Dangerous Methods**: No public `set_message_handler()` after construction
5. **Event-driven Architecture**: Uses `MessageHandler<()>` for clean separation of concerns

---

## 1. Module Structure (`mod.rs`)

```rust
//! STDIO Transport Adapter
//!
//! This module provides STDIO transport configuration and implementation
//! following ADR-011 Transport Configuration Separation Architecture.

pub mod config;
pub mod transport;

pub use config::StdioTransportConfig;
pub use transport::{StdioTransport, StdioTransportBuilder};

// Type aliases for convenience (as per Phase 5.5.5 requirements)
/// Type alias for STDIO message handlers using unit context
pub type StdioMessageHandler = dyn crate::protocol::MessageHandler<()>;

/// Type alias for STDIO message context
pub type StdioMessageContext = crate::protocol::MessageContext<()>;
```

## 2. Configuration Implementation (`config.rs`)

```rust
//! STDIO Transport Configuration
//!
//! This module provides configuration structures for the STDIO transport
//! following ADR-011 Transport Configuration Separation Architecture.

use std::path::PathBuf;

use crate::protocol::transport::TransportConfig;
use crate::protocol::types::{ServerCapabilities, ServerConfig};

/// STDIO-specific transport configuration
///
/// This configuration structure contains both universal MCP requirements
/// (via ServerConfig) and STDIO-specific settings optimized for standard
/// input/output communication patterns.
///
/// # Design Principles
///
/// - **Buffer Management**: STDIO needs specific buffering for performance
/// - **Flush Control**: Important for interactive STDIO sessions  
/// - **Log Separation**: Logs must go to file, not stdout (would corrupt MCP protocol)
/// - **Validation Strictness**: STDIO-specific validation levels
#[derive(Debug, Clone)]
pub struct StdioTransportConfig {
    /// Universal MCP requirements (transport-agnostic)
    server_config: Option<ServerConfig>,

    /// Buffer size for stdin/stdout operations
    ///
    /// Larger buffers improve performance for bulk operations,
    /// smaller buffers reduce latency for interactive sessions.
    /// Default: 8192 bytes
    buffer_size: usize,

    /// Whether to flush stdout after each response
    ///
    /// Essential for interactive STDIO sessions to ensure responses
    /// are immediately visible to the client.
    /// Default: true
    flush_on_response: bool,

    /// STDIO-specific validation strictness
    ///
    /// When enabled, performs additional validation on JSON-RPC
    /// messages before processing. Useful for debugging.
    /// Default: false
    strict_validation: bool,

    /// Whether to log operations (to file, not stdout!)
    ///
    /// Logging to stdout would corrupt the MCP protocol stream,
    /// so logs must be written to a separate file.
    /// Default: false
    log_operations: bool,

    /// Log file path (separate from stdout)
    ///
    /// Required when log_operations is true. Must not be stdout/stderr
    /// to avoid corrupting the MCP protocol stream.
    /// Default: None
    log_file_path: Option<PathBuf>,
}

impl StdioTransportConfig {
    /// Create a new STDIO transport configuration with sensible defaults
    ///
    /// Default values are optimized for typical MCP STDIO workloads:
    /// - 8KB buffer size for good performance/latency balance
    /// - Flush on response for interactive sessions
    /// - Non-strict validation for performance
    /// - No logging to avoid stdout corruption
    pub fn new() -> Self {
        Self {
            server_config: None,
            buffer_size: 8192,
            flush_on_response: true,
            strict_validation: false,
            log_operations: false,
            log_file_path: None,
        }
    }

    /// Set buffer size for stdin/stdout operations
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set whether to flush stdout after each response
    pub fn flush_on_response(mut self, flush: bool) -> Self {
        self.flush_on_response = flush;
        self
    }

    /// Set STDIO-specific validation strictness
    pub fn strict_validation(mut self, strict: bool) -> Self {
        self.strict_validation = strict;
        self
    }

    /// Enable logging with specified file path
    pub fn with_log_file_path(mut self, path: Option<PathBuf>) -> Self {
        self.log_operations = path.is_some();
        self.log_file_path = path;
        self
    }

    /// Enable/disable operation logging
    pub fn with_log_operations(mut self, log: bool) -> Self {
        self.log_operations = log;
        self
    }

    // Getters for STDIO-specific configuration

    /// Get buffer size
    pub fn get_buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Get flush on response setting
    pub fn get_flush_on_response(&self) -> bool {
        self.flush_on_response
    }

    /// Get strict validation setting
    pub fn get_strict_validation(&self) -> bool {
        self.strict_validation
    }

    /// Get logging operations setting
    pub fn get_log_operations(&self) -> bool {
        self.log_operations
    }

    /// Get log file path
    pub fn get_log_file_path(&self) -> Option<&PathBuf> {
        self.log_file_path.as_ref()
    }
}

impl Default for StdioTransportConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of TransportConfig trait for STDIO transport
///
/// This provides the standardized interface for MCP server configuration
/// management while allowing STDIO-specific capability modifications.
impl TransportConfig for StdioTransportConfig {
    fn set_server_config(&mut self, server_config: ServerConfig) {
        self.server_config = Some(server_config);
    }

    fn server_config(&self) -> Option<&ServerConfig> {
        self.server_config.as_ref()
    }

    fn effective_capabilities(&self) -> ServerCapabilities {
        if let Some(server_cfg) = &self.server_config {
            let mut caps = server_cfg.capabilities.clone();

            // STDIO-specific capability modifications
            // Remove experimental features that don't work well over STDIO
            caps.experimental = None;

            // STDIO doesn't support certain streaming patterns
            if let Some(ref mut resources) = caps.resources {
                // Disable subscription for STDIO (interactive polling is better)
                resources.subscribe = Some(false);
            }

            caps
        } else {
            // Default capabilities if no server config set
            ServerCapabilities::default()
        }
    }
}
```

## 3. Core Transport Implementation (`transport.rs`)

### Imports and Type Definitions

```rust
//! STDIO Transport Implementation
//!
//! This module provides a modern STDIO transport implementation using the
//! unified protocol module Transport trait for event-driven message handling.

// Layer 1: Standard library imports
use std::fmt::Debug;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;

// Layer 3: Internal module imports
use crate::protocol::{
    JsonRpcMessage, MessageContext, MessageHandler, Transport, TransportBuilder, TransportError,
};

/// Type alias for STDIO message context (no transport-specific data)
pub type StdioMessageContext = MessageContext<()>;
```

### StdioTransport Structure

```rust
/// Modern STDIO transport implementation
///
/// This transport reads JSON-RPC messages from stdin and writes responses to stdout,
/// using the event-driven Transport trait for clean separation of concerns.
///
/// # Architecture
///
/// ```text
/// stdin -> StdioTransport -> MessageHandler -> stdout
///          (event-driven)   (protocol logic)  (responses)
/// ```
pub struct StdioTransport {
    /// Event-driven message handler (STDIO uses no transport-specific context)
    message_handler: Option<Arc<dyn MessageHandler<()>>>,

    /// Shutdown signal broadcaster
    shutdown_tx: Option<broadcast::Sender<()>>,

    /// Session context (STDIO is single-session)
    session_id: String,

    /// Connection state
    is_running: bool,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            message_handler: None,
            shutdown_tx: None,
            session_id: "stdio-session".to_string(),
            is_running: false,
        }
    }

    /// Create transport with custom session ID
    pub fn with_session_id(session_id: String) -> Self {
        Self {
            message_handler: None,
            shutdown_tx: None,
            session_id,
            is_running: false,
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for StdioTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTransport")
            .field(
                "message_handler",
                &self
                    .message_handler
                    .as_ref()
                    .map(|_| "Arc<dyn MessageHandler<()>>"),
            )
            .field(
                "shutdown_tx",
                &self.shutdown_tx.as_ref().map(|_| "broadcast::Sender<()>"),
            )
            .field("session_id", &self.session_id)
            .field("is_running", &self.is_running)
            .finish()
    }
}
```

### Transport Trait Implementation

```rust
#[async_trait]
impl Transport for StdioTransport {
    type Error = TransportError;

    /// Start the transport and begin event-driven message processing
    ///
    /// This spawns a background task that reads from stdin and processes
    /// messages through the configured MessageHandler.
    async fn start(&mut self) -> Result<(), Self::Error> {
        if self.is_running {
            return Err(TransportError::Connection {
                message: "Transport already running".to_string(),
            });
        }

        // With pre-configured pattern, handler should always be set
        let handler = self.message_handler.as_ref()
            .ok_or_else(|| TransportError::Connection {
                message: "No message handler configured. Use StdioTransportBuilder for pre-configured setup.".to_string(),
            })?
            .clone();

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let session_id = self.session_id.clone();

        // Spawn stdin reader task
        tokio::spawn(async move {
            stdin_reader_loop(handler, session_id, shutdown_rx).await;
        });

        self.is_running = true;
        Ok(())
    }

    /// Close the transport and clean up resources
    async fn close(&mut self) -> Result<(), Self::Error> {
        if !self.is_running {
            return Ok(());
        }

        // Signal shutdown
        if let Some(shutdown_tx) = &self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }

        self.is_running = false;
        self.shutdown_tx = None;

        Ok(())
    }

    /// Send a JSON-RPC message through stdout
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<(), Self::Error> {
        // Serialize message to JSON
        let json = serde_json::to_string(message)
            .map_err(|e| TransportError::Serialization { source: e })?;

        // Write to stdout with newline delimiter
        let mut stdout = tokio::io::stdout();
        stdout
            .write_all(json.as_bytes())
            .await
            .map_err(|e| TransportError::Io { source: e })?;
        stdout
            .write_all(b"\n")
            .await
            .map_err(|e| TransportError::Io { source: e })?;
        stdout
            .flush()
            .await
            .map_err(|e| TransportError::Io { source: e })?;

        Ok(())
    }

    /// Get the current session ID
    fn session_id(&self) -> Option<String> {
        Some(self.session_id.clone())
    }

    /// Set session context for the transport
    fn set_session_context(&mut self, session_id: Option<String>) {
        self.session_id = session_id.unwrap_or_else(|| "stdio-session".to_string());
    }

    /// Check if the transport is currently connected
    fn is_connected(&self) -> bool {
        self.is_running
    }

    /// Get the transport type identifier
    fn transport_type(&self) -> &'static str {
        "stdio"
    }
}
```

### Background Event Loop

```rust
/// Background task that reads from stdin and processes messages
///
/// This function runs the main event loop for STDIO transport, reading
/// line-delimited JSON messages from stdin and dispatching them to the
/// configured MessageHandler.
async fn stdin_reader_loop(
    handler: Arc<dyn MessageHandler<()>>,
    session_id: String,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        tokio::select! {
            // Handle shutdown signal
            _ = shutdown_rx.recv() => {
                handler.handle_close().await;
                break;
            }

            // Read from stdin
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        // EOF reached, stdin closed
                        handler.handle_close().await;
                        break;
                    }
                    Ok(_) => {
                        // Process the line
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            match serde_json::from_str::<JsonRpcMessage>(trimmed) {
                                Ok(message) => {
                                    let context = MessageContext::new(session_id.clone());
                                    handler.handle_message(message, context).await;
                                }
                                Err(e) => {
                                    let error = TransportError::Serialization { source: e };
                                    handler.handle_error(error).await;
                                }
                            }
                        }
                        line.clear();
                    }
                    Err(e) => {
                        let error = TransportError::Io { source: e };
                        handler.handle_error(error).await;
                        break;
                    }
                }
            }
        }
    }
}
```

## 4. **THE CRITICAL PART - TransportBuilder Implementation**

**This is what HTTP is missing and what TASK-031 must implement!**

### StdioTransportBuilder Structure

```rust
/// Builder for creating pre-configured STDIO transports
///
/// This builder implements the pre-configured transport pattern where
/// transports are created with their message handlers already set,
/// eliminating the dangerous `set_message_handler()` pattern.
pub struct StdioTransportBuilder {
    /// Message handler for the transport (set via with_message_handler)
    message_handler: Option<Arc<dyn MessageHandler<()>>>,
}

impl StdioTransportBuilder {
    /// Create a new STDIO transport builder
    pub fn new() -> Self {
        Self {
            message_handler: None,
        }
    }
}

impl Default for StdioTransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for StdioTransportBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StdioTransportBuilder")
            .field(
                "message_handler",
                &self
                    .message_handler
                    .as_ref()
                    .map(|_| "Arc<dyn MessageHandler<()>>"),
            )
            .finish()
    }
}
```

### **THE MISSING IMPLEMENTATION** - TransportBuilder Trait

```rust
// THIS IS THE PATTERN HTTP MUST FOLLOW!
impl TransportBuilder<()> for StdioTransportBuilder {
    type Transport = StdioTransport;
    type Error = TransportError;

    /// Set the message handler for the transport
    ///
    /// This is the key method that implements the pre-configured pattern.
    /// The handler must be set before building the transport.
    fn with_message_handler(mut self, handler: Arc<dyn MessageHandler<()>>) -> Self {
        self.message_handler = Some(handler);
        self
    }

    /// Build the transport with the configured message handler
    ///
    /// This creates a fully configured transport that is ready to start.
    /// The transport will have its message handler pre-configured.
    async fn build(self) -> Result<Self::Transport, Self::Error> {
        let handler = self
            .message_handler
            .ok_or_else(|| TransportError::Connection {
                message: "Message handler must be set before building transport".to_string(),
            })?;

        Ok(StdioTransport {
            message_handler: Some(handler),  // ✅ PRE-CONFIGURED!
            shutdown_tx: None,
            session_id: "stdio-session".to_string(),
            is_running: false,
        })
    }
}
```

## 5. Complete Usage Example

```rust
use airs_mcp::protocol::{MessageHandler, JsonRpcMessage, MessageContext, TransportError, TransportBuilder, Transport};
use airs_mcp::transport::adapters::stdio::StdioTransportBuilder;
use async_trait::async_trait;
use std::sync::Arc;

struct EchoHandler;

#[async_trait]
impl MessageHandler<()> for EchoHandler {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<()>) {
        println!("Received: {:?}", message);
        // Echo logic would go here
    }

    async fn handle_error(&self, error: TransportError) {
        eprintln!("Transport error: {}", error);
    }

    async fn handle_close(&self) {
        println!("Transport closed");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = Arc::new(EchoHandler);
    
    // ✅ THE SAFE PRE-CONFIGURED PATTERN (follows ADR-011)
    let mut transport = StdioTransportBuilder::new()
        .with_message_handler(handler)     // ✅ Handler set at build time
        .build().await?;                   // ✅ Transport created with handler
    
    transport.start().await?;              // ✅ Safe to start
    transport.close().await?;              // ✅ Clean shutdown
    Ok(())
}
```

## 6. Comprehensive Test Suite

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;

    // Mock handler for testing
    struct MockHandler {
        messages: Arc<Mutex<Vec<JsonRpcMessage>>>,
        errors: Arc<Mutex<Vec<String>>>,
        close_called: Arc<AtomicBool>,
        message_count: Arc<AtomicUsize>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
                errors: Arc::new(Mutex::new(Vec::new())),
                close_called: Arc::new(AtomicBool::new(false)),
                message_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn get_messages(&self) -> Vec<JsonRpcMessage> {
            self.messages.lock().unwrap().clone()
        }

        fn get_errors(&self) -> Vec<String> {
            self.errors.lock().unwrap().clone()
        }

        fn was_close_called(&self) -> bool {
            self.close_called.load(Ordering::Acquire)
        }

        fn message_count(&self) -> usize {
            self.message_count.load(Ordering::Acquire)
        }
    }

    #[async_trait]
    impl MessageHandler<()> for MockHandler {
        async fn handle_message(&self, message: JsonRpcMessage, _context: StdioMessageContext) {
            self.messages.lock().unwrap().push(message);
            self.message_count.fetch_add(1, Ordering::Release);
        }

        async fn handle_error(&self, error: TransportError) {
            self.errors.lock().unwrap().push(error.to_string());
        }

        async fn handle_close(&self) {
            self.close_called.store(true, Ordering::Release);
        }
    }

    #[tokio::test]
    async fn test_transport_creation() {
        let transport = StdioTransport::new();
        assert_eq!(transport.transport_type(), "stdio");
        assert!(!transport.is_connected());
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));
    }

    #[tokio::test]
    async fn test_transport_builder() {
        let handler = Arc::new(MockHandler::new());

        // ✅ Test successful build with handler
        let transport_result = StdioTransportBuilder::new()
            .with_message_handler(handler.clone())
            .build()
            .await;

        assert!(transport_result.is_ok());
        let transport = transport_result.unwrap();
        assert_eq!(transport.transport_type(), "stdio");
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));

        // ❌ Test builder without handler fails
        let builder_without_handler = StdioTransportBuilder::new();
        let result = builder_without_handler.build().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Message handler must be set"));
    }

    #[tokio::test]
    async fn test_transport_lifecycle() {
        let handler = Arc::new(MockHandler::new());

        // Test creating transport with pre-configured pattern
        let transport_result = StdioTransportBuilder::new()
            .with_message_handler(handler.clone())
            .build()
            .await;

        assert!(transport_result.is_ok());
        let mut transport = transport_result.unwrap();

        // Test initial state
        assert!(!transport.is_connected());

        // Test that transport without handler (created with new()) fails to start
        let mut basic_transport = StdioTransport::new();
        let start_result = basic_transport.start().await;
        assert!(start_result.is_err());
        assert!(start_result
            .unwrap_err()
            .to_string()
            .contains("No message handler configured"));

        // Test close when not running
        let result = transport.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_session_management() {
        let mut transport = StdioTransport::new();

        // Test default session
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));

        // Test custom session
        transport.set_session_context(Some("custom-session".to_string()));
        assert_eq!(transport.session_id(), Some("custom-session".to_string()));

        // Test None becomes default
        transport.set_session_context(None);
        assert_eq!(transport.session_id(), Some("stdio-session".to_string()));
    }

    #[tokio::test]
    async fn test_send_message() {
        let mut transport = StdioTransport::new();

        // Create a test message
        let message = JsonRpcMessage::from_notification("test_method", None);

        // Test send (this will write to actual stdout in test environment)
        let result = transport.send(&message).await;
        assert!(result.is_ok());
    }
}
```

---

## TASK-031 Implementation Requirements

**For HTTP to match STDIO architectural consistency, HTTP must implement:**

### 1. **Add Handler Storage to HttpTransportBuilder**
```rust
pub struct HttpTransportBuilder<E: HttpEngine> {
    engine: E,
    message_handler: Option<Arc<dyn MessageHandler<HttpContext>>>,  // ← ADD THIS
}
```

### 2. **Implement TransportBuilder<HttpContext> for HttpTransportBuilder**
```rust
impl<E: HttpEngine> TransportBuilder<HttpContext> for HttpTransportBuilder<E> {
    type Transport = HttpTransport<E>;
    type Error = TransportError;
    
    fn with_message_handler(mut self, handler: Arc<dyn MessageHandler<HttpContext>>) -> Self {
        self.message_handler = Some(handler);
        self
    }
    
    async fn build(self) -> Result<Self::Transport, Self::Error> {
        let handler = self.message_handler
            .ok_or_else(|| TransportError::Protocol { 
                message: "Message handler must be set before building HTTP transport".to_string() 
            })?;
        
        let mut transport = HttpTransport::new(self.engine);
        transport.set_message_handler(handler);  // Pre-configure
        Ok(transport)
    }
}
```

### 3. **Update HttpTransport to Store Handler**
```rust
pub struct HttpTransport<E: HttpEngine> {
    engine: E,
    message_handler: Option<Arc<dyn MessageHandler<HttpContext>>>,  // ← ADD THIS
    session_id: Option<String>,
    is_connected: bool,
}
```

### 4. **Remove Dangerous Methods**
```rust
// ❌ REMOVE THIS - violates ADR-011
// pub fn register_mcp_handler(&mut self, handler: E::Handler) {
//     self.engine.register_mcp_handler(handler);
// }
```

This reference ensures TASK-031 implements the exact same safe pattern that STDIO already demonstrates correctly.
//! STDIO Transport Adapter
//!
//! This module provides STDIO transport configuration and implementation
//! following ADR-011 Transport Configuration Separation Architecture.

pub mod client;
pub mod config;
pub mod transport;

pub use client::{StdioTransportClient, StdioTransportClientBuilder};
pub use config::StdioTransportConfig;
pub use transport::{StdioTransport, StdioTransportBuilder};

// Type aliases for convenience (as per Phase 5.5.5 requirements)
/// Type alias for STDIO message handlers using unit context
pub type StdioMessageHandler = dyn crate::protocol::MessageHandler<()>;

/// Type alias for STDIO message context
pub type StdioMessageContext = crate::protocol::MessageContext<()>;

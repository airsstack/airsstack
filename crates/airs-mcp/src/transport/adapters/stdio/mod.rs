//! STDIO Transport Adapter
//!
//! This module provides STDIO transport configuration and implementation
//! following ADR-011 Transport Configuration Separation Architecture.

pub mod config;
pub mod transport;

pub use config::StdioTransportConfig;
pub use transport::{StdioTransport, StdioTransportBuilder};

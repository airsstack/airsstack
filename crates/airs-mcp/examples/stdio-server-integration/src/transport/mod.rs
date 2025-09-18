//! Transport Integration Module
//!
//! This module handles STDIO transport integration using the proper
//! transport layer architecture.

pub mod stdio;

pub use stdio::create_stdio_transport;
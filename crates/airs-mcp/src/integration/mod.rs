//! Integration Layer
//!
//! This module provides high-level abstractions for JSON-RPC communication
//! by integrating the correlation manager and transport layers into a unified
//! client interface.
//!
//! # Architecture
//!
//! The integration layer consists of:
//! - `JsonRpcClient`: High-level client for making JSON-RPC calls
//! - `Handler`: Trait for processing incoming notifications and requests
//! - `Router`: Message routing and handler registration system
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use airs_mcp::integration::JsonRpcClient;
//! use airs_mcp::transport::StdioTransport;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create transport and client
//!     let transport = StdioTransport::new().await?;
//!     let mut client = JsonRpcClient::new(transport).await?;
//!     
//!     // Make a method call
//!     let response = client.call("ping", Some(json!({"message": "hello"}))).await?;
//!     println!("Response: {:?}", response);
//!     
//!     // Send a notification
//!     client.notify("status", Some(json!({"status": "ready"}))).await?;
//!     
//!     // Clean shutdown
//!     client.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod handler;
pub mod router;

// Re-export main types for convenience
pub use client::JsonRpcClient;
pub use error::{IntegrationError, IntegrationResult};
pub use handler::{Handler, NotificationHandler, RequestHandler};
pub use router::{MessageRouter, RouteConfig};

//! Correlation Module
//!
//! This module provides request/response correlation for bidirectional JSON-RPC communication
//! in the Model Context Protocol implementation.
//!
//! ## Architecture
//!
//! - `manager.rs` - Core CorrelationManager implementation
//! - `types.rs` - Type definitions for correlation system
//! - `error.rs` - Error types and result definitions
//! - `tests.rs` - Comprehensive test suite
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! // This example will work once Phase 4 implementation is complete
//! use airs_mcp::correlation::{CorrelationManager, CorrelationConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut manager = CorrelationManager::new(CorrelationConfig::default());
//!     manager.start().await?;
//!     
//!     // Register a request
//!     let (id, receiver) = manager.register_request(
//!         serde_json::json!({"method": "ping"}),
//!         None
//!     ).await?;
//!     
//!     // Correlate response (would be done by transport layer)
//!     manager.correlate_response(id, serde_json::json!({"result": "pong"}))?;
//!     
//!     // Await the response
//!     let response = receiver.await??;
//!     println!("Response: {}", response);
//!     
//!     manager.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod manager;
pub mod types;

// Re-export main types for convenience
pub use error::{CorrelationError, CorrelationResult, RequestId};
pub use manager::{CorrelationConfig, CorrelationManager};
pub use types::{PendingRequest, RequestIdGenerator};

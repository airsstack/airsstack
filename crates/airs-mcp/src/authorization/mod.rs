//! Zero-Cost Generic Authorization Framework
//!
//! This module implements a zero-cost authorization architecture where:
//! - Authorization policies are compile-time generic types (no `dyn` patterns)
//! - Each auth/authz combination creates a unique server type at compile time
//! - NoAuth development mode compiles to zero overhead
//! - All authorization contexts are stack-allocated (no heap allocation)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   HTTP Layer    │    │  JSON-RPC Layer │    │   MCP Layer     │
//! │ • Bearer Token  │───▶│ • Parse Message │───▶│ • Method Auth   │
//! │ • Authentication│    │ • Extract Method│    │ • Scope Check   │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! # Zero-Cost Examples
//!
//! ```rust,ignore
//! // Development server - completely optimized away
//! type DevServer = McpServer<NoAuthAdapter, NoAuthorizationPolicy, NoAuthContext>;
//!
//! // OAuth2 server - specific concrete type
//! type OAuth2Server = McpServer<OAuth2StrategyAdapter, ScopeBasedPolicy, OAuth2AuthContext>;
//! ```

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports

pub mod context;
pub mod error;
pub mod extractor;
pub mod middleware;
pub mod policy;

// Re-exports for zero-cost generic architecture
pub use context::{AuthzContext, NoAuthContext, OAuth2AuthContext, ApiKeyAuthContext};
pub use error::{AuthzError, AuthzResult};
pub use extractor::{MethodExtractor, JsonRpcMethodExtractor, HttpPathMethodExtractor};
pub use middleware::AuthorizationMiddleware;
pub use policy::{AuthorizationPolicy, NoAuthorizationPolicy, ScopeBasedPolicy, BinaryAuthorizationPolicy};

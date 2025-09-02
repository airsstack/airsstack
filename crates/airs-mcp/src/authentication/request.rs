//! Authentication Request Abstraction
//!
//! Minimal interface focused only on custom attributes.
//! Authentication strategies extract specific data they need.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
// (none for this module)

/// Minimal authentication request interface
///
/// Provides only custom attributes. Each authentication strategy
/// is responsible for extracting the specific data it needs.
/// This keeps the interface minimal and maximally flexible.
pub trait AuthRequest<T>: Send + Sync {
    /// Get custom request attribute
    ///
    /// Strategies use this to extract whatever data they need:
    /// - HTTP strategies: headers, query params, method, path, body, etc.
    /// - gRPC strategies: metadata, method name, etc.
    /// - WebSocket strategies: origin, protocol, etc.
    fn custom_attribute(&self, key: &str) -> Option<String>;
    
    /// Get all custom attributes
    fn custom_attributes(&self) -> HashMap<String, String>;
    
    /// Get the underlying request object for advanced access
    fn inner(&self) -> &T;
}

//! MCP Capability Definitions
//!
//! This module provides capability structures for MCP protocol negotiation,
//! allowing clients and servers to declare and negotiate supported features.

use serde::{Deserialize, Serialize};

/// Client capabilities for MCP protocol negotiation
///
/// Declares the capabilities supported by an MCP client during initialization.
/// These capabilities determine which protocol features the client can handle.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::ClientCapabilities;
///
/// let capabilities = ClientCapabilities {
///     experimental: None,
///     sampling: None,
///     roots: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClientCapabilities {
    /// Experimental capabilities (implementation-specific)
    pub experimental: Option<serde_json::Value>,

    /// Sampling capabilities (server-initiated AI requests)
    pub sampling: Option<SamplingCapabilities>,

    /// Root directory access capabilities
    pub roots: Option<RootsCapabilities>,
}

/// Server capabilities for MCP protocol negotiation
///
/// Declares the capabilities supported by an MCP server during initialization.
/// These capabilities determine which protocol features the server provides.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::{ServerCapabilities, ResourceCapabilities};
///
/// let capabilities = ServerCapabilities {
///     experimental: None,
///     logging: None,
///     prompts: None,
///     resources: Some(ResourceCapabilities {
///         subscribe: Some(true),
///         list_changed: Some(false),
///     }),
///     tools: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ServerCapabilities {
    /// Experimental capabilities (implementation-specific)
    pub experimental: Option<serde_json::Value>,

    /// Logging capabilities
    pub logging: Option<LoggingCapabilities>,

    /// Prompt template capabilities
    pub prompts: Option<PromptCapabilities>,

    /// Resource management capabilities
    pub resources: Option<ResourceCapabilities>,

    /// Tool execution capabilities
    pub tools: Option<ToolCapabilities>,
}

/// Resource management capabilities
///
/// Declares which resource-related features are supported by the server.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::ResourceCapabilities;
///
/// let capabilities = ResourceCapabilities {
///     subscribe: Some(true),    // Supports resource subscriptions
///     list_changed: Some(true), // Supports change notifications
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceCapabilities {
    /// Whether resource subscriptions are supported
    pub subscribe: Option<bool>,

    /// Whether resource list change notifications are supported
    pub list_changed: Option<bool>,
}

/// Tool execution capabilities
///
/// Declares tool-related features supported by the server.
/// Currently a placeholder for future tool-specific capabilities.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::ToolCapabilities;
///
/// let capabilities = ToolCapabilities {};
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCapabilities {
    // Future tool-specific capabilities will be added here
}

/// Prompt template capabilities
///
/// Declares prompt-related features supported by the server.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::PromptCapabilities;
///
/// let capabilities = PromptCapabilities {
///     list_changed: Some(true), // Supports prompt list change notifications
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptCapabilities {
    /// Whether prompt list change notifications are supported
    pub list_changed: Option<bool>,
}

/// Logging capabilities
///
/// Declares logging-related features supported by the server.
/// Currently a placeholder for future logging-specific capabilities.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::LoggingCapabilities;
///
/// let capabilities = LoggingCapabilities {};
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoggingCapabilities {
    // Future logging-specific capabilities will be added here
}

/// Sampling capabilities (client-side)
///
/// Declares sampling-related features supported by the client.
/// Sampling allows servers to request AI model interactions through the client.
/// Currently a placeholder for future sampling-specific capabilities.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::SamplingCapabilities;
///
/// let capabilities = SamplingCapabilities {};
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SamplingCapabilities {
    // Future sampling-specific capabilities will be added here
}

/// Root directory access capabilities (client-side)
///
/// Declares root directory access features supported by the client.
/// Currently a placeholder for future root access capabilities.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::shared::protocol::RootsCapabilities;
///
/// let capabilities = RootsCapabilities {};
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RootsCapabilities {
    // Future root access capabilities will be added here
}

impl ClientCapabilities {
    /// Create minimal client capabilities
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Create client capabilities with sampling support
    pub fn with_sampling() -> Self {
        Self {
            sampling: Some(SamplingCapabilities {}),
            ..Default::default()
        }
    }

    /// Create client capabilities with roots support
    pub fn with_roots() -> Self {
        Self {
            roots: Some(RootsCapabilities {}),
            ..Default::default()
        }
    }
}

impl ServerCapabilities {
    /// Create minimal server capabilities
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Create server capabilities with resource support
    pub fn with_resources(subscribe: bool, list_changed: bool) -> Self {
        Self {
            resources: Some(ResourceCapabilities {
                subscribe: Some(subscribe),
                list_changed: Some(list_changed),
            }),
            ..Default::default()
        }
    }

    /// Create server capabilities with tool support
    pub fn with_tools() -> Self {
        Self {
            tools: Some(ToolCapabilities {}),
            ..Default::default()
        }
    }

    /// Create server capabilities with prompt support
    pub fn with_prompts(list_changed: bool) -> Self {
        Self {
            prompts: Some(PromptCapabilities {
                list_changed: Some(list_changed),
            }),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_capabilities_default() {
        let capabilities = ClientCapabilities::default();
        assert!(capabilities.experimental.is_none());
        assert!(capabilities.sampling.is_none());
        assert!(capabilities.roots.is_none());
    }

    #[test]
    fn test_client_capabilities_with_sampling() {
        let capabilities = ClientCapabilities::with_sampling();
        assert!(capabilities.sampling.is_some());
        assert!(capabilities.roots.is_none());
    }

    #[test]
    fn test_server_capabilities_default() {
        let capabilities = ServerCapabilities::default();
        assert!(capabilities.experimental.is_none());
        assert!(capabilities.logging.is_none());
        assert!(capabilities.prompts.is_none());
        assert!(capabilities.resources.is_none());
        assert!(capabilities.tools.is_none());
    }

    #[test]
    fn test_server_capabilities_with_resources() {
        let capabilities = ServerCapabilities::with_resources(true, false);
        assert!(capabilities.resources.is_some());

        let resources = capabilities.resources.unwrap();
        assert_eq!(resources.subscribe, Some(true));
        assert_eq!(resources.list_changed, Some(false));
    }

    #[test]
    fn test_capability_serialization() {
        let client_capabilities = ClientCapabilities::with_sampling();
        let server_capabilities = ServerCapabilities::with_resources(true, true);

        // Test that capabilities can be serialized and deserialized
        let client_json = serde_json::to_string(&client_capabilities).unwrap();
        let server_json = serde_json::to_string(&server_capabilities).unwrap();

        let client_deserialized: ClientCapabilities = serde_json::from_str(&client_json).unwrap();
        let server_deserialized: ServerCapabilities = serde_json::from_str(&server_json).unwrap();

        assert_eq!(client_capabilities, client_deserialized);
        assert_eq!(server_capabilities, server_deserialized);
    }
}

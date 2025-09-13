//! Generic Builder for AxumMcpRequestHandler
//!
//! This module provides a type-safe builder implementation using progressive type
//! refinement. The builder allows zero-cost construction of MCP request handlers
//! with optional providers while maintaining compile-time type safety.

// Layer 1: Standard library imports
// (none needed for this module)

// Layer 2: Third-party crate imports (none needed)

// Layer 3: Internal module imports
use crate::integration::LoggingHandler;
use crate::providers::{PromptProvider, ResourceProvider, ToolProvider};

use super::defaults::{NoLoggingHandler, NoPromptProvider, NoResourceProvider, NoToolProvider};
use super::mcp_request_handler::AxumMcpRequestHandler;

/// Builder for AxumMcpRequestHandler with progressive type refinement
///
/// This builder uses progressive type refinement to provide a type-safe,
/// ergonomic API for constructing MCP request handlers. It follows the
/// workspace pattern §5 for builder design with zero-cost abstractions.
///
/// # Type Parameters
///
/// * `R` - Resource provider type (defaults to NoResourceProvider)
/// * `T` - Tool provider type (defaults to NoToolProvider)
/// * `P` - Prompt provider type (defaults to NoPromptProvider)
/// * `L` - Logging handler type (defaults to NoLoggingHandler)
///
/// # Examples
///
/// ```rust
/// use airs_mcp::transport::adapters::http::mcp_request_handler_builder::AxumMcpRequestHandlerBuilder;
///
/// // Start with defaults
/// let handler = AxumMcpRequestHandlerBuilder::new()
///     .build();
/// ```
pub struct AxumMcpRequestHandlerBuilder<R, T, P, L> {
    resource_provider: Option<R>,
    tool_provider: Option<T>,
    prompt_provider: Option<P>,
    logging_handler: Option<L>,
}

impl
    AxumMcpRequestHandlerBuilder<
        NoResourceProvider,
        NoToolProvider,
        NoPromptProvider,
        NoLoggingHandler,
    >
{
    /// Create a new builder with default (no-op) provider types
    ///
    /// This constructor starts with zero-cost default providers,
    /// allowing progressive type refinement as providers are added.
    pub fn new() -> Self {
        Self {
            resource_provider: None,
            tool_provider: None,
            prompt_provider: None,
            logging_handler: None,
        }
    }
}

impl Default
    for AxumMcpRequestHandlerBuilder<
        NoResourceProvider,
        NoToolProvider,
        NoPromptProvider,
        NoLoggingHandler,
    >
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, T, P, L> AxumMcpRequestHandlerBuilder<R, T, P, L> {
    /// Set the resource provider, refining the builder type
    ///
    /// This method demonstrates progressive type refinement - the builder's
    /// type evolves to include the new resource provider type while preserving
    /// the existing tool, prompt, and logging types.
    ///
    /// # Type Parameters
    ///
    /// * `NewR` - The new resource provider type
    ///
    /// # Arguments
    ///
    /// * `provider` - Resource provider implementation
    pub fn with_resource_provider<NewR>(
        self,
        provider: NewR,
    ) -> AxumMcpRequestHandlerBuilder<NewR, T, P, L>
    where
        NewR: ResourceProvider + Send + Sync + 'static,
    {
        AxumMcpRequestHandlerBuilder {
            resource_provider: Some(provider),
            tool_provider: self.tool_provider,
            prompt_provider: self.prompt_provider,
            logging_handler: self.logging_handler,
        }
    }

    /// Set the tool provider, refining the builder type
    ///
    /// # Type Parameters
    ///
    /// * `NewT` - The new tool provider type
    ///
    /// # Arguments
    ///
    /// * `provider` - Tool provider implementation
    pub fn with_tool_provider<NewT>(
        self,
        provider: NewT,
    ) -> AxumMcpRequestHandlerBuilder<R, NewT, P, L>
    where
        NewT: ToolProvider + Send + Sync + 'static,
    {
        AxumMcpRequestHandlerBuilder {
            resource_provider: self.resource_provider,
            tool_provider: Some(provider),
            prompt_provider: self.prompt_provider,
            logging_handler: self.logging_handler,
        }
    }

    /// Set the prompt provider, refining the builder type
    ///
    /// # Type Parameters
    ///
    /// * `NewP` - The new prompt provider type
    ///
    /// # Arguments
    ///
    /// * `provider` - Prompt provider implementation
    pub fn with_prompt_provider<NewP>(
        self,
        provider: NewP,
    ) -> AxumMcpRequestHandlerBuilder<R, T, NewP, L>
    where
        NewP: PromptProvider + Send + Sync + 'static,
    {
        AxumMcpRequestHandlerBuilder {
            resource_provider: self.resource_provider,
            tool_provider: self.tool_provider,
            prompt_provider: Some(provider),
            logging_handler: self.logging_handler,
        }
    }

    /// Set the logging handler, refining the builder type
    ///
    /// # Type Parameters
    ///
    /// * `NewL` - The new logging handler type
    ///
    /// # Arguments
    ///
    /// * `handler` - Logging handler implementation
    pub fn with_logging_handler<NewL>(
        self,
        handler: NewL,
    ) -> AxumMcpRequestHandlerBuilder<R, T, P, NewL>
    where
        NewL: LoggingHandler + Send + Sync + 'static,
    {
        AxumMcpRequestHandlerBuilder {
            resource_provider: self.resource_provider,
            tool_provider: self.tool_provider,
            prompt_provider: self.prompt_provider,
            logging_handler: Some(handler),
        }
    }

    /// Build the final AxumMcpRequestHandler with type constraints enforced
    ///
    /// This method applies the necessary trait constraints only at build time,
    /// following the progressive type refinement pattern from workspace standards §5.
    /// The constraints ensure all provider types implement the required traits.
    pub fn build(self) -> AxumMcpRequestHandler<R, T, P, L>
    where
        R: ResourceProvider + Send + Sync + 'static,
        T: ToolProvider + Send + Sync + 'static,
        P: PromptProvider + Send + Sync + 'static,
        L: LoggingHandler + Send + Sync + 'static,
    {
        AxumMcpRequestHandler::new(
            self.resource_provider,
            self.tool_provider,
            self.prompt_provider,
            self.logging_handler,
        )
    }
}

// Convenience methods for common provider combinations

impl<R, T, P, L> AxumMcpRequestHandlerBuilder<R, T, P, L> {
    /// Remove the current resource provider, setting it back to NoResourceProvider
    pub fn without_resource_provider(
        self,
    ) -> AxumMcpRequestHandlerBuilder<NoResourceProvider, T, P, L> {
        AxumMcpRequestHandlerBuilder {
            resource_provider: None,
            tool_provider: self.tool_provider,
            prompt_provider: self.prompt_provider,
            logging_handler: self.logging_handler,
        }
    }

    /// Remove the current tool provider, setting it back to NoToolProvider
    pub fn without_tool_provider(self) -> AxumMcpRequestHandlerBuilder<R, NoToolProvider, P, L> {
        AxumMcpRequestHandlerBuilder {
            resource_provider: self.resource_provider,
            tool_provider: None,
            prompt_provider: self.prompt_provider,
            logging_handler: self.logging_handler,
        }
    }

    /// Remove the current prompt provider, setting it back to NoPromptProvider
    pub fn without_prompt_provider(
        self,
    ) -> AxumMcpRequestHandlerBuilder<R, T, NoPromptProvider, L> {
        AxumMcpRequestHandlerBuilder {
            resource_provider: self.resource_provider,
            tool_provider: self.tool_provider,
            prompt_provider: None,
            logging_handler: self.logging_handler,
        }
    }

    /// Remove the current logging handler, setting it back to NoLoggingHandler
    pub fn without_logging_handler(
        self,
    ) -> AxumMcpRequestHandlerBuilder<R, T, P, NoLoggingHandler> {
        AxumMcpRequestHandlerBuilder {
            resource_provider: self.resource_provider,
            tool_provider: self.tool_provider,
            prompt_provider: self.prompt_provider,
            logging_handler: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_type_evolution() {
        // Start with default types
        let builder = AxumMcpRequestHandlerBuilder::new();

        // Verify we can build with defaults
        let _handler = builder.build();

        // Test type evolution (this should compile)
        let builder = AxumMcpRequestHandlerBuilder::new();

        // Each step refines the type, proving progressive type refinement works
        let _builder_with_resource = builder.with_resource_provider(NoResourceProvider);

        // Could continue refining types here...
    }

    #[test]
    fn test_builder_default_construction() {
        let _handler1 = AxumMcpRequestHandlerBuilder::new().build();
        let _handler2 = AxumMcpRequestHandlerBuilder::default().build();

        // This test verifies:
        // 1. Default builder construction compiles correctly
        // 2. Progressive type refinement works with default types
        // 3. Zero-cost abstraction constraints are satisfied at compile time
        // The fact that this compiles proves our type system is working correctly
    }
}

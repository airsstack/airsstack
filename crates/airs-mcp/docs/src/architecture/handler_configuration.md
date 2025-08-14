# MCP Handler Configuration Architecture

## Problem Addressed

The original `AxumHttpServer` implementation had a significant design flaw: it created empty MCP handlers with no mechanism to configure them with actual provider implementations. This violated the principle of "make invalid states unrepresentable" and created poor developer experience.

## Solution: Multi-Pattern Handler Configuration

We've implemented a comprehensive handler configuration system with three approaches:

### 1. Direct Handler Configuration
```rust
let mcp_handlers = Arc::new(McpHandlers {
    resource_provider: Some(Arc::new(MyResourceProvider)),
    tool_provider: Some(Arc::new(MyToolProvider)),
    prompt_provider: Some(Arc::new(MyPromptProvider)),
    logging_handler: Some(Arc::new(MyLoggingHandler)),
    config: McpServerConfig::default(),
});

let server = AxumHttpServer::new(
    connection_manager,
    session_manager,
    jsonrpc_processor,
    mcp_handlers,
    config,
).await?;
```

### 2. Builder Pattern (Recommended)
```rust
let handlers_builder = McpHandlersBuilder::new()
    .with_resource_provider(Arc::new(MyResourceProvider))
    .with_tool_provider(Arc::new(MyToolProvider))
    .with_prompt_provider(Arc::new(MyPromptProvider))
    .with_logging_handler(Arc::new(MyLoggingHandler))
    .with_config(McpServerConfig::default());

let server = AxumHttpServer::with_handlers(
    connection_manager,
    session_manager,
    jsonrpc_processor,
    handlers_builder,
    config,
).await?;
```

### 3. Empty Handlers (Testing/Development)
```rust
let server = AxumHttpServer::new_with_empty_handlers(
    connection_manager,
    session_manager,
    jsonrpc_processor,
    config,
).await?;
```

## Key Design Principles Applied

1. **Explicit Configuration**: No hidden defaults or magical behavior
2. **Type Safety**: Compiler enforces proper handler configuration
3. **Flexibility**: Multiple patterns for different use cases
4. **Backward Compatibility**: Existing tests work unchanged
5. **Fail-Fast**: Missing handlers return clear "method not found" errors
6. **Fluent Interface**: Builder pattern provides clean, readable configuration

## Architecture Benefits

### For Production Use
- **Clear Ownership**: Explicit handler injection makes dependencies obvious
- **Testability**: Easy to inject mock implementations for testing
- **Incremental Rollout**: Can configure only needed providers
- **Error Handling**: Unconfigured handlers gracefully return JSON-RPC errors

### For Development/Testing
- **Quick Setup**: `new_with_empty_handlers()` for infrastructure testing
- **Partial Implementation**: Can implement one provider at a time
- **Clear Boundaries**: Interface segregation between different MCP capabilities

## Error Handling Strategy

When MCP handlers are not configured:
- Returns JSON-RPC error `-32601` (Method not found)
- Provides clear error message: "No resource provider configured"
- Allows server to continue operating for other functionality

## Example Usage Patterns

### Development Workflow
1. Start with `new_with_empty_handlers()` to test HTTP infrastructure
2. Implement one provider at a time using builder pattern
3. Use partial configuration during incremental development
4. Graduate to full configuration for production deployment

### Testing Strategy
- Unit tests: Use empty handlers to isolate HTTP server logic
- Integration tests: Use mock providers for MCP protocol testing
- End-to-end tests: Use real providers for complete workflow validation

## Future Extensibility

The builder pattern makes it easy to add new provider types:
```rust
// Future extension example
let handlers_builder = McpHandlersBuilder::new()
    .with_resource_provider(resource_provider)
    .with_tool_provider(tool_provider)
    .with_prompt_provider(prompt_provider)
    .with_logging_handler(logging_handler)
    .with_notification_handler(notification_handler)  // Future addition
    .with_custom_provider(custom_provider)             // Future addition
    .with_config(config);
```

## Technical Debt Prevention

This design proactively addresses several potential issues:
- **Null Object Pattern**: Prevents runtime null pointer exceptions
- **Dependency Injection**: Makes testing and mocking straightforward
- **Configuration Validation**: Compile-time checking of handler setup
- **Interface Segregation**: Providers implement only needed capabilities

## Conclusion

The multi-pattern handler configuration system transforms the `AxumHttpServer` from a static infrastructure component into a flexible, configurable MCP server foundation. It maintains excellent performance characteristics while providing the configurability needed for real-world deployments.

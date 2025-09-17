# Example Module Architecture Standard

**Created**: 2025-09-17  
**Status**: Technical Standard  
**Scope**: All MCP Integration Examples  
**Reference**: Derived from STDIO Server Integration Example Architecture

## Overview

This document establishes the canonical module architecture pattern for all MCP integration examples in the AIRS workspace. This architecture ensures consistency, maintainability, and proper separation of concerns across all example implementations.

## Standard Module Structure

All MCP integration examples MUST follow this exact module structure:

```
src/
├── lib.rs                     # Central module integration and re-exports
├── main.rs                    # Entry point (simplified, imports via lib.rs)  
├── handlers/
│   ├── mod.rs                 # Handler module exports
│   └── mcp_handler.rs         # MCP message handler + transport integration
├── providers/
│   ├── mod.rs                 # Provider module exports  
│   └── setup.rs               # Provider creation and test environment setup
├── transport/
│   ├── mod.rs                 # Transport module exports
│   └── [transport_type].rs    # Transport-specific integration (stdio.rs, http.rs, etc.)
└── utilities.rs               # Utility functions (logging, configuration, helpers)
```

## Module Responsibilities

### 1. `lib.rs` - Central Integration Point

**Purpose**: Central module integration and public API surface  
**Responsibilities**:
- Module declarations for all sub-modules
- Re-export main types for clean external usage
- Provide unified API for `main.rs` to import everything
- Documentation for the example crate

**Template**:
```rust
//! [Example Name] Integration Library
//! 
//! Provides modular components for [transport type] MCP integration
//! with proper transport layer architecture.

pub mod handlers;
pub mod providers; 
pub mod transport;
pub mod utilities;

// Re-export main types for convenience
pub use handlers::[HandlerType];
pub use providers::create_test_environment;
pub use transport::create_[transport_type]_transport;
pub use utilities::init_logging;
```

### 2. `main.rs` - Entry Point (Simplified)

**Purpose**: Application entry point with minimal logic  
**Responsibilities**:
- Import everything through `lib.rs`
- Initialize logging and configuration
- Coordinate high-level application flow
- Handle graceful shutdown

**Anti-patterns**:
- ❌ Direct implementation of MCP protocol logic
- ❌ Provider setup and configuration
- ❌ Transport-specific code
- ❌ Utility function definitions

**Template**:
```rust
//! [Example Name] Integration Example
//!
//! Demonstrates MCP integration using [transport type] transport
//! with proper transport layer architecture.

// Import everything through lib.rs
use [example_crate_name]::{
    [HandlerType],
    create_test_environment,
    create_[transport_type]_transport,
    init_logging,
};
use std::sync::Arc;
use airs_mcp::protocol::Transport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    
    // Create test environment and handler
    let (handler, _temp_dir) = create_test_environment().await?;
    let handler = Arc::new(handler);
    
    // Create and start transport
    let mut transport = create_[transport_type]_transport(handler).await?;
    transport.start().await?;
    
    Ok(())
}
```

### 3. `handlers/` - MCP Protocol Logic

**Purpose**: MCP message handling and protocol implementation  
**Responsibilities**:
- Implement MCP 2024-11-05 protocol methods
- Integrate with transport layer via `MessageHandler` trait
- Coordinate between providers for request processing
- Handle error mapping and response formatting

**File Structure**:
- `mod.rs`: Module exports and documentation
- `mcp_handler.rs`: Main MCP message handler implementation

**Key Implementation Requirements**:
```rust
// Must implement MessageHandler for transport integration
#[async_trait]
impl MessageHandler<()> for [HandlerType] {
    async fn handle_message(&self, message: JsonRpcMessage, context: MessageContext<()>) {
        // Route messages to MCP protocol handlers
    }
    
    async fn handle_error(&self, error: TransportError) {
        // Handle transport-level errors
    }
    
    async fn handle_close(&self) {
        // Handle graceful shutdown
    }
}
```

### 4. `providers/` - Provider Setup and Management

**Purpose**: MCP provider creation and test environment setup  
**Responsibilities**:
- Create and configure all MCP providers (tools, resources, prompts, logging)
- Set up test environments and sample data
- Handle provider lifecycle management
- Abstract provider configuration from main application

**File Structure**:
- `mod.rs`: Module exports
- `setup.rs`: Provider creation and test environment functions

**Key Functions**:
```rust
pub async fn create_test_environment() -> Result<
    ([HandlerType], Option<TempDir>), 
    Box<dyn std::error::Error>
> {
    // Set up test directory and files
    // Create all providers
    // Return configured handler
}
```

### 5. `transport/` - Transport Layer Integration

**Purpose**: Transport-specific integration and configuration  
**Responsibilities**:
- Wrap transport builders with example-specific logic
- Handle transport lifecycle (creation, start, shutdown)
- Provide transport-specific configuration patterns
- Abstract transport complexity from main application

**File Structure**:
- `mod.rs`: Module exports
- `[transport_type].rs`: Transport-specific implementation

**Key Functions**:
```rust
pub async fn create_[transport_type]_transport(
    handler: Arc<[HandlerType]>
) -> Result<[TransportType], TransportError> {
    [TransportType]Builder::new()
        .with_message_handler(handler)
        .build()
        .await
}
```

### 6. `utilities.rs` - Utility Functions

**Purpose**: Shared utility functions and helpers  
**Responsibilities**:
- Logging initialization and configuration
- Environment variable handling
- Common helper functions
- Configuration utilities

**Common Functions**:
```rust
pub fn init_logging() {
    // Tracing/logging setup
}

pub fn load_config() -> Result<Config, ConfigError> {
    // Environment-based configuration
}
```

## Transport Integration Requirements

### MessageHandler Implementation

All MCP handlers MUST implement the `MessageHandler<()>` trait for transport integration:

```rust
use async_trait::async_trait;
use airs_mcp::protocol::{MessageHandler, MessageContext, TransportError, JsonRpcMessage};

#[async_trait]
impl MessageHandler<()> for [HandlerType] {
    async fn handle_message(&self, message: JsonRpcMessage, _context: MessageContext<()>) {
        match message {
            JsonRpcMessage::Request(request) => {
                let response = self.process_mcp_request(&request).await;
                // Send response via transport
                let response_json = serde_json::to_string(&JsonRpcMessage::Response(response)).unwrap();
                println!("{}", response_json);
                tokio::io::stdout().flush().await.ok();
            }
            _ => {
                // Handle other message types if needed
            }
        }
    }

    async fn handle_error(&self, error: TransportError) {
        error!("Transport error: {}", error);
    }

    async fn handle_close(&self) {
        info!("Transport closed gracefully");
    }
}
```

### Transport Builder Pattern

All examples MUST use the transport builder pattern for proper integration:

```rust
pub async fn create_[transport_type]_transport(
    handler: Arc<[HandlerType]>
) -> Result<[TransportType], TransportError> {
    [TransportType]Builder::new()
        .with_message_handler(handler)
        // Transport-specific configuration
        .build()
        .await
}
```

## Workspace Standards Compliance

All examples MUST comply with workspace standards:

### Import Organization (§2.1)
```rust
// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports  
use async_trait::async_trait;
use serde_json::json;

// Layer 3: Internal module imports
use airs_mcp::protocol::{MessageHandler, Transport};
```

### Module Architecture (§4.3)
- `mod.rs` files contain ONLY module declarations and re-exports
- NO implementation code in `mod.rs` files
- Clean separation of concerns between modules

### Zero Warning Policy
- All examples MUST compile with zero warnings
- `cargo check --workspace`, `cargo clippy --workspace`, `cargo test --workspace`

### Dependency Management (§5.1)
- AIRS foundation crates prioritized at top of workspace dependencies
- Proper workspace dependency inheritance

## Testing Requirements

### Test Structure
Each example MUST include comprehensive testing:

```
tests/
├── README.md                  # Test documentation
├── requirements.txt           # Python test dependencies
├── setup.sh                   # Virtual environment setup
├── run_tests.py              # Test runner
├── test_[example]_basic.py    # Basic functionality tests
├── test_[example]_comprehensive.py  # Comprehensive test suite
└── test_[example]_integration.py    # Integration tests
```

### Test Categories
1. **Basic Tests**: Core functionality (ping, initialize, basic operations)
2. **Comprehensive Tests**: Full protocol compliance, error handling, edge cases
3. **Integration Tests**: End-to-end scenarios with realistic workflows

## Documentation Requirements

### Example Documentation
Each example MUST include:
- Clear README.md with setup and usage instructions
- Comprehensive module documentation in `lib.rs`
- Function-level documentation for all public APIs
- Usage examples in documentation

### Architecture Documentation
- Transport integration patterns
- Provider configuration examples
- Error handling strategies
- Performance considerations

## Benefits of This Architecture

### 1. Consistency
- Standardized structure across all examples
- Predictable organization for developers
- Easy navigation and understanding

### 2. Maintainability
- Clear separation of concerns
- Modular design enables focused changes
- Reduced coupling between components

### 3. Testability
- Isolated modules enable targeted testing
- Mock-friendly interfaces
- Clear dependency injection points

### 4. Reusability
- Modules can be used independently
- Transport patterns apply across examples
- Provider setup patterns are reusable

### 5. Extensibility
- Easy to add new transports
- Provider system is pluggable
- Handler logic is transport-agnostic

## Implementation Checklist

When creating new MCP integration examples:

- [ ] Create module structure following exact pattern
- [ ] Implement `MessageHandler<()>` trait in handlers
- [ ] Use transport builder pattern for integration
- [ ] Follow workspace standards (imports, warnings, dependencies)
- [ ] Create comprehensive test suite
- [ ] Document architecture and usage patterns
- [ ] Verify zero-warning compilation
- [ ] Test transport integration end-to-end

## Future Considerations

This architecture standard will evolve as we:
- Add new transport types (WebSocket, gRPC, etc.)
- Implement more complex provider patterns
- Enhance testing and validation frameworks
- Optimize performance and resource usage

All changes to this standard require:
- Discussion and approval in architecture review
- Update to this document
- Migration plan for existing examples
- Validation across all affected examples
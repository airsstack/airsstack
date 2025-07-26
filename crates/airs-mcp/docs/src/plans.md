# Implementation Technical Plans

## Project Structure & Module Organization

### Cargo Workspace Architecture

```toml
# Root Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/airs-mcp",
    "crates/airs-mcp-macros",  # Procedural macros for tool generation
    "examples/basic-server",
    "examples/basic-client", 
    "examples/claude-integration",
    "benchmarks",
    "tools/protocol-tester",   # Protocol compliance validation
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Rstlix0x0 <rstlix.dev@gmail.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/rstlix0x0/airs"
rust-version = "1.88.0"

[workspace.dependencies]
# Core async runtime
tokio = { version = "1.40", features = ["full"] }
tokio-util = "0.7"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Networking and HTTP
reqwest = { version = "0.12", features = ["json", "stream"] }
url = "2.5"

# Concurrency
dashmap = "6.0"
parking_lot = "0.12"

# Utilities
uuid = { version = "1.10", features = ["v4", "serde"] }
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# Security
oauth2 = { version = "4.4", optional = true }
rustls = { version = "0.23", optional = true }
ring = "0.17"

# Testing
proptest = "1.4"
criterion = "0.5"
```

### Core Library Structure

```
crates/airs-mcp/
├── Cargo.toml
├── src/
│   ├── lib.rs                          # Public API exports
│   │
│   ├── shared/                         # 🟠 Shared Data Types
│   │   ├── mod.rs
│   │   ├── common.rs                   # Common utilities and types
│   │   ├── protocol/                   # Protocol-specific data types
│   │   │   ├── mod.rs
│   │   │   ├── jsonrpc.rs              # JSON-RPC 2.0 message types
│   │   │   ├── batch.rs                # Batch operation types
│   │   │   ├── capabilities.rs         # Capability negotiation types
│   │   │   ├── progress.rs             # Progress tracking types
│   │   │   ├── cancellation.rs         # Cancellation types
│   │   │   ├── completion.rs           # Autocompletion types
│   │   │   ├── pagination.rs           # Pagination types
│   │   │   └── metadata.rs             # Protocol metadata types
│   │   ├── server/                     # Server feature data types
│   │   │   ├── mod.rs
│   │   │   ├── resource.rs             # Resource data structures
│   │   │   ├── tool.rs                 # Tool data structures
│   │   │   └── prompt.rs               # Prompt data structures
│   │   ├── client/                     # Client feature data types
│   │   │   ├── mod.rs
│   │   │   └── sampling.rs             # Sampling data structures
│   │   ├── security/                   # Security data types
│   │   │   ├── mod.rs
│   │   │   ├── oauth21.rs              # OAuth 2.1 + PKCE types
│   │   │   ├── approval.rs             # Human-in-the-loop types
│   │   │   └── audit.rs                # Audit logging types
│   │   ├── errors.rs                   # Comprehensive error types
│   │   └── result.rs                   # Result type aliases
│   │
│   ├── base/                           # 🟢 JSON-RPC 2.0 Foundation
│   │   ├── mod.rs
│   │   ├── core/                       # Core abstractions
│   │   │   ├── mod.rs
│   │   │   ├── message_processor.rs    # Message processing traits
│   │   │   ├── transport.rs            # Transport abstractions
│   │   │   ├── session.rs              # Session management traits
│   │   │   └── protocol.rs             # Protocol extension traits
│   │   ├── jsonrpc/                    # JSON-RPC 2.0 implementation
│   │   │   ├── mod.rs
│   │   │   ├── message.rs              # Core message processing
│   │   │   ├── request.rs              # Request handling with correlation
│   │   │   ├── response.rs             # Response processing
│   │   │   ├── notification.rs         # Notification handling
│   │   │   ├── batch.rs                # Batch operation support
│   │   │   ├── error_codes.rs          # Error code management
│   │   │   ├── validation.rs           # Message validation
│   │   │   └── correlation.rs          # Request-response correlation
│   │   ├── transport/                  # Transport implementations
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs               # Transport trait definitions
│   │   │   ├── stdio.rs                # STDIO transport
│   │   │   ├── streamable_http.rs      # HTTP transport (2025-03-26)
│   │   │   ├── session.rs              # Session management
│   │   │   └── factory.rs              # Transport factory
│   │   └── protocol/                   # MCP protocol extensions
│   │       ├── mod.rs
│   │       ├── progress.rs             # Progress tracking implementation
│   │       ├── cancellation.rs         # Cancellation support
│   │       ├── completion.rs           # Autocompletion engine
│   │       └── pagination.rs           # Pagination implementation
│   │
│   ├── lifecycle/                      # 🟢 Connection Lifecycle
│   │   ├── mod.rs
│   │   ├── core/                       # Lifecycle abstractions
│   │   │   ├── mod.rs
│   │   │   ├── state_machine.rs        # State machine traits
│   │   │   ├── constraints.rs          # Protocol constraint traits
│   │   │   ├── capabilities.rs         # Capability traits
│   │   │   └── connection.rs           # Connection management traits
│   │   ├── state/                      # State machine implementation
│   │   │   ├── mod.rs
│   │   │   ├── machine.rs              # Three-phase state machine
│   │   │   ├── transitions.rs          # State transition logic
│   │   │   └── constraints.rs          # Protocol constraint enforcement
│   │   ├── capabilities/               # Capability negotiation
│   │   │   ├── mod.rs
│   │   │   ├── negotiator.rs           # Negotiation logic
│   │   │   ├── validator.rs            # Capability validation
│   │   │   └── registry.rs             # Capability registry
│   │   └── connection/                 # Connection management
│   │       ├── mod.rs
│   │       ├── manager.rs              # Connection lifecycle management
│   │       ├── pool.rs                 # Connection pooling
│   │       ├── health.rs               # Health monitoring
│   │       └── recovery.rs             # Connection recovery
│   │
│   ├── server/                         # 🟢 Server Implementation
│   │   ├── mod.rs
│   │   ├── core/                       # Server abstractions
│   │   │   ├── mod.rs
│   │   │   ├── resource_provider.rs    # Resource provider traits
│   │   │   ├── tool_executor.rs        # Tool executor traits
│   │   │   ├── prompt_manager.rs       # Prompt manager traits
│   │   │   ├── subscription.rs         # Subscription traits
│   │   │   └── handler.rs              # Request handler traits
│   │   ├── resources/                  # Resource system
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs             # Resource provider implementation
│   │   │   ├── templates.rs            # URI template engine (RFC 6570)
│   │   │   ├── subscriptions.rs        # Real-time subscriptions
│   │   │   ├── content.rs              # Content handling (binary/text)
│   │   │   ├── uri_schemes.rs          # Custom URI scheme support
│   │   │   ├── pagination.rs           # Resource pagination
│   │   │   └── registry.rs             # Resource registry
│   │   ├── tools/                      # Tool system
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs             # Tool execution engine
│   │   │   ├── safety.rs               # Safety annotation system
│   │   │   ├── approval.rs             # Human-in-the-loop approval
│   │   │   ├── validation.rs           # JSON Schema validation
│   │   │   ├── results.rs              # Multi-modal result handling
│   │   │   ├── schema.rs               # Schema management
│   │   │   └── registry.rs             # Tool registry
│   │   ├── prompts/                    # Prompt system
│   │   │   ├── mod.rs
│   │   │   ├── template.rs             # Template engine
│   │   │   ├── completion.rs           # Autocompletion provider
│   │   │   ├── parameters.rs           # Parameter handling
│   │   │   ├── multimodal.rs           # Multi-modal content
│   │   │   ├── embedding.rs            # Resource embedding
│   │   │   └── registry.rs             # Prompt registry
│   │   └── builder.rs                  # Server builder pattern
│   │
│   ├── client/                         # 🟢 Client Implementation
│   │   ├── mod.rs
│   │   ├── core/                       # Client abstractions
│   │   │   ├── mod.rs
│   │   │   ├── sampling.rs             # Sampling traits
│   │   │   ├── capability_provider.rs  # Capability provider traits
│   │   │   └── handler.rs              # Request handler traits
│   │   ├── sampling/                   # Sampling system
│   │   │   ├── mod.rs
│   │   │   ├── requester.rs            # Sampling request handling
│   │   │   ├── approval.rs             # Double approval workflow
│   │   │   ├── model_preferences.rs    # Model preference handling
│   │   │   ├── context.rs              # Context management
│   │   │   ├── parameters.rs           # Parameter processing
│   │   │   └── response.rs             # Response handling
│   │   ├── capabilities/               # Client capabilities
│   │   │   ├── mod.rs
│   │   │   ├── root_access.rs          # Root directory access
│   │   │   ├── subscriptions.rs        # Subscription support
│   │   │   └── notifications.rs        # Notification handling
│   │   └── builder.rs                  # Client builder pattern
│   │
│   ├── security/                       # 🟢 Security Layer
│   │   ├── mod.rs
│   │   ├── core/                       # Security abstractions
│   │   │   ├── mod.rs
│   │   │   ├── authenticator.rs        # Authentication traits
│   │   │   ├── authorizer.rs           # Authorization traits
│   │   │   ├── approval.rs             # Approval workflow traits
│   │   │   └── audit.rs                # Audit logging traits
│   │   ├── authentication/             # Authentication implementations
│   │   │   ├── mod.rs
│   │   │   ├── oauth21_pkce.rs         # OAuth 2.1 + PKCE
│   │   │   ├── dynamic_client.rs       # RFC7591 client registration
│   │   │   ├── metadata.rs             # RFC8414 metadata discovery
│   │   │   ├── token_manager.rs        # Token management
│   │   │   ├── stdio_env.rs            # STDIO environment auth
│   │   │   └── transport_auth.rs       # Transport-specific auth
│   │   ├── authorization/              # Authorization implementations
│   │   │   ├── mod.rs
│   │   │   ├── capability_checker.rs   # Capability-based authorization
│   │   │   ├── permission_engine.rs    # Permission management
│   │   │   └── risk_assessment.rs      # Risk-based authorization
│   │   ├── approval/                   # Human-in-the-loop workflows
│   │   │   ├── mod.rs
│   │   │   ├── sampling_approval.rs    # Sampling approval workflow
│   │   │   ├── tool_approval.rs        # Tool execution approval
│   │   │   ├── workflow.rs             # Approval workflow engine
│   │   │   └── policy.rs               # Approval policy management
│   │   └── audit/                      # Audit and compliance
│   │       ├── mod.rs
│   │       ├── operation_logger.rs     # Operation logging
│   │       ├── security_events.rs      # Security event tracking
│   │       ├── compliance.rs           # Compliance validation
│   │       └── reporting.rs            # Audit reporting
│   │
│   ├── utils/                          # 🟠 Utilities
│   │   ├── mod.rs
│   │   ├── id_generator.rs             # ID generation utilities
│   │   ├── time.rs                     # Time utilities
│   │   ├── config.rs                   # Configuration utilities
│   │   └── testing.rs                  # Testing utilities
│   │
│   └── prelude.rs                      # Convenience imports
│
├── tests/                              # Integration tests
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── jsonrpc_compliance.rs       # JSON-RPC 2.0 compliance tests
│   │   ├── mcp_protocol.rs             # MCP protocol tests
│   │   ├── server_features.rs          # Server feature tests
│   │   ├── client_features.rs          # Client feature tests
│   │   ├── security.rs                 # Security tests
│   │   └── interop.rs                  # Interoperability tests
│   └── fixtures/                       # Test data and fixtures
│
├── benches/                            # Performance benchmarks
│   ├── message_processing.rs
│   ├── request_correlation.rs
│   ├── transport_performance.rs
│   └── end_to_end.rs
│
├── examples/                           # Usage examples
│   ├── basic_server.rs
│   ├── basic_client.rs
│   ├── advanced_server.rs
│   └── claude_integration.rs
│
└── docs/                               # Additional documentation
    ├── protocol_compliance.md
    ├── security_guide.md
    ├── performance_guide.md
    └── integration_guide.md
```

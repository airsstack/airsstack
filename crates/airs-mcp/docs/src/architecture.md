# System Architecture Design

## High-Level System Architecture

### Architectural Principles

```rust,ignore
// Core architectural decisions
pub struct ArchitecturalPrinciples {
    protocol_first: bool,           // Protocol compliance drives all decisions
    async_native: bool,             // Built for tokio async runtime
    type_safe: bool,                // Compile-time correctness where possible
    zero_unsafe: bool,              // No unsafe code blocks
    modular_monolith: bool,         // Clear domain boundaries, single artifact
    transport_agnostic: bool,       // Pluggable transport layer
}
```

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        AIRS MCP Library                         │
├─────────────────────────────────────────────────────────────────┤
│  Public API Layer                                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   McpServer     │  │   McpClient     │  │ TransportBuilder│  │
│  │   Builder       │  │   Builder       │  │                 │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Core Protocol Layer                                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  JSON-RPC 2.0   │  │   Lifecycle     │  │   Capability    │  │
│  │   Foundation    │  │   Manager       │  │   Negotiator    │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Feature Implementation Layer                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │    Resources    │  │      Tools      │  │     Prompts     │  │
│  │   (Server)      │  │   (Server)      │  │   (Server)      │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │    Sampling     │  │   Root Access   │  │  Subscriptions  │  │
│  │   (Client)      │  │   (Client)      │  │   (Client)      │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Security & Authorization Layer                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  OAuth 2.1 +    │  │  Human-in-Loop  │  │   Audit &       │  │
│  │     PKCE        │  │   Approval      │  │   Logging       │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Transport Abstraction Layer                                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ STDIO Transport │  │ HTTP Transport  │  │ Custom Transport│  │
│  │                 │  │  (Streamable)   │  │   Framework     │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

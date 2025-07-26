# Domain Architecture & Component Boundaries

## Domain Separation Strategy

```rust,ignore
// Clear domain boundaries with dependency inversion
pub mod domains {
    pub mod base {           // JSON-RPC 2.0 + Transport foundation
        pub mod jsonrpc;     // Message types and processing
        pub mod transport;   // Transport abstraction and implementations
        pub mod protocol;    // MCP protocol extensions
    }
    
    pub mod lifecycle {      // Connection and state management
        pub mod state_machine; // Three-phase lifecycle enforcement
        pub mod capabilities;  // Capability negotiation
        pub mod connection;    // Connection management
    }
    
    pub mod server {         // Server-side MCP features
        pub mod resources;   // Resource management
        pub mod tools;       // Tool execution with safety
        pub mod prompts;     // Prompt templates and completion
    }
    
    pub mod client {         // Client-side MCP features
        pub mod sampling;    // Server-initiated AI requests
        pub mod capabilities; // Client capability provision
    }
    
    pub mod security {       // Cross-cutting security concerns
        pub mod auth;        // Authentication mechanisms
        pub mod authz;       // Authorization and permissions
        pub mod audit;       // Audit logging and compliance
    }
}
```

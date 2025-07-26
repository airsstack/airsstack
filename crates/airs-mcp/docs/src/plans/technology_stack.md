# Technology Stack Decisions & Trade-offs

## Core Dependencies Rationale

```toml
[dependencies]
# === Core Async Runtime ===
tokio = { version = "1.40", features = ["full"] }
# Decision: Full tokio features for maximum flexibility
# Trade-off: Larger binary size vs development simplicity
# Rationale: MCP requires complex async patterns, full feature set justified

tokio-util = "0.7"
# Decision: Additional tokio utilities for codec and framing
# Rationale: Streamable HTTP transport requires advanced async utilities

# === Serialization Stack ===
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
# Decision: Standard Rust serialization with JSON focus
# Trade-off: JSON-only vs multi-format support
# Rationale: MCP is JSON-RPC based, other formats add complexity without benefit

# === Concurrent Data Structures ===
dashmap = "6.0"
# Decision: Concurrent HashMap for request correlation tracking
# Trade-off: Memory overhead vs lock-free concurrent access
# Rationale: Request correlation is performance-critical with high concurrency

parking_lot = "0.12"
# Decision: Faster mutex implementation than std
# Trade-off: External dependency vs performance
# Rationale: Used in non-critical paths, performance gain worth dependency

# === Networking ===
reqwest = { version = "0.12", features = ["json", "stream"] }
# Decision: High-level HTTP client with streaming support
# Trade-off: Heavy dependency vs implementation complexity
# Rationale: HTTP transport requires SSE support, reqwest provides this cleanly

url = "2.5"
# Decision: URL parsing and manipulation
# Rationale: Essential for URI template processing and HTTP transport

# === Security (Feature-gated) ===
oauth2 = { version = "4.4", optional = true }
# Decision: Mature OAuth 2.1 implementation
# Trade-off: External dependency vs security implementation complexity
# Rationale: OAuth 2.1 + PKCE is complex, tested implementation preferred

rustls = { version = "0.23", optional = true }
# Decision: Pure Rust TLS implementation
# Trade-off: Binary size vs security and consistency
# Rationale: Avoids OpenSSL dependency, consistent across platforms

# === Utilities ===
uuid = { version = "1.10", features = ["v4", "serde"] }
# Decision: UUID generation for request IDs
# Rationale: Guarantees unique IDs across distributed systems

thiserror = "1.0"
# Decision: Ergonomic error handling
# Rationale: Complex error hierarchy requires good error ergonomics

tracing = "0.1"
tracing-subscriber = "0.3"
# Decision: Structured logging framework
# Trade-off: Complexity vs observability
# Rationale: Production deployment requires comprehensive observability
```

## Feature Flag Strategy

```toml
[features]
default = ["stdio-transport", "client", "server"]

# Transport features
stdio-transport = []
http-transport = ["oauth2", "rustls", "reqwest/stream"]

# Component features  
server = []
client = ["server"]  # Client includes server for bidirectional communication

# Security features
oauth21 = ["oauth2", "rustls"]
audit-logging = ["tracing-subscriber/json"]

# Development features
testing-utils = ["proptest"]
benchmarking = ["criterion"]

# Compliance features
strict-compliance = []  # Enables additional protocol validation
performance-optimized = []  # Enables performance optimizations that may reduce compliance checking
```

## MSRV (Minimum Supported Rust Version) Policy

```toml
// Cargo.toml
rust-version = "1.84.0"

// Justification for 1.84.0:
// - Required for latest tokio async patterns
// - Improved const generics support
// - Better error handling with Result::inspect methods
// - Performance improvements in HashMap/BTreeMap
// - Security improvements in standard library
```

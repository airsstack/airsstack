## Protocol Architecture Patterns

### Connection Lifecycle: Three-Phase State Machine

```rust,ignore
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionPhase {
    Initialization,  // Capability negotiation only
    Operation,       // Full feature access
    Shutdown,        // Cleanup and termination
}

pub struct ProtocolConstraints {
    initialization_methods: HashSet<&'static str>, // "initialize", "ping"
    operation_methods: HashSet<&'static str>,      // All methods based on capabilities
    shutdown_methods: HashSet<&'static str>,       // Limited cleanup methods
}
```

Phase Transition Rules:

```
Initialization → Operation: After successful "initialize" + "initialized" exchange
Operation → Shutdown: On connection close or explicit shutdown
No reverse transitions allowed
```

### Capability Negotiation System

```rust,ignore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    // Server feature capabilities
    pub resources: Option<ResourceCapabilities>,
    pub tools: Option<ToolCapabilities>, 
    pub prompts: Option<PromptCapabilities>,
    pub logging: Option<LoggingCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct ClientCapabilities {
    // Client feature capabilities
    pub sampling: Option<SamplingCapabilities>,
    pub roots: Option<RootsCapabilities>,
}

// Runtime capability validation
impl Connection {
    pub fn can_use_feature(&self, feature: &str) -> bool {
        self.negotiated_capabilities
            .as_ref()
            .map(|caps| caps.supports_feature(feature))
            .unwrap_or(false)
    }
}
```

### Transport Abstraction Layer

```rust,ignore
#[async_trait]
pub trait BidirectionalTransport: Send + Sync {
    // Core messaging
    async fn send_message(&self, message: JsonRpcMessage) -> Result<(), TransportError>;
    async fn receive_message(&self) -> Result<JsonRpcMessage, TransportError>;
    
    // Session management  
    fn session_id(&self) -> Option<&str>;
    async fn close(&self) -> Result<(), TransportError>;
    
    // Resumability (HTTP transport)
    fn last_event_id(&self) -> Option<&str>;
    async fn resume_from(&self, event_id: &str) -> Result<(), TransportError>;
}
```

Transport Implementations:

- STDIO Transport: Process-based communication for local servers
- Streamable HTTP Transport: HTTP with Server-Sent Events for remote servers
- Custom Transports: Extensible for future transport mechanisms

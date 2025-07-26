# Data Flow Architecture

## Message Flow Patterns

### Client → Server Request Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │    │  Transport  │    │  Protocol   │    │   Server    │
│     API     │    │    Layer    │    │   State     │    │  Handler    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │                   │
       │ send_request()    │                   │                   │
       ├──────────────────►│                   │                   │
       │                   │ validate_phase()  │                   │
       │                   ├──────────────────►│                   │
       │                   │                   │ route_message()   │
       │                   │                   ├──────────────────►│
       │                   │                   │                   │
       │                   │                   │ ◄─────────────────┤
       │                   │ ◄─────────────────┤    response       │
       │ ◄─────────────────┤    response       │                   │
```

### Server → Client Request Flow (Sampling)

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Server    │    │  Protocol   │    │  Transport  │    │   Client    │
│   Handler   │    │   State     │    │    Layer    │    │  Handler    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
       │                    │                   │                   │
       │ request_sampling() │                   │                   │
       ├──────────────────► │                   │                   │
       │                    │ validate_capability│                  │
       │                    │                   │ send_request()    │
       │                    ├──────────────────►│                   │
       │                    │                   ├──────────────────►│
       │                    │                   │                   │
       │                    │                   │ ◄─────────────────┤
       │                    │ ◄─────────────────┤   approval_flow   │
       │ ◄───────────────── ┤                   │                   │
```

### Error Propagation Flow

```rust,ignore
// Hierarchical error handling with context preservation
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    #[error("Feature error: {0}")]
    Feature(#[from] FeatureError),
}

// Error context preservation through the stack
impl From<ValidationError> for ProtocolError {
    fn from(err: ValidationError) -> Self {
        ProtocolError::InvalidMessage {
            reason: err.to_string(),
            phase: err.context.phase,
            method: err.context.method,
        }
    }
}
```

# Core Component Design

## JSON-RPC 2.0 Foundation Layer

```rust,ignore
// Message processing engine
pub struct JsonRpcProcessor {
    request_tracker: RequestTracker,
    message_validator: MessageValidator,
    error_handler: ErrorHandler,
}

impl JsonRpcProcessor {
    pub async fn process_message(
        &self,
        message: JsonRpcMessage,
        context: &ProcessingContext,
    ) -> Result<Option<JsonRpcMessage>, ProcessingError> {
        // Validation → Processing → Response correlation
        let validated = self.message_validator.validate(message, context)?;
        let response = self.route_message(validated, context).await?;
        Ok(response)
    }
}

// Bidirectional request correlation
pub struct RequestTracker {
    pending_outgoing: DashMap<RequestId, oneshot::Sender<JsonRpcResponse>>,
    timeout_handles: DashMap<RequestId, AbortHandle>,
    id_generator: Box<dyn IdGenerator>,
}

impl RequestTracker {
    pub async fn send_request(
        &self,
        request: JsonRpcRequest,
        transport: &dyn BidirectionalTransport,
    ) -> Result<JsonRpcResponse, RequestError> {
        let (response_tx, response_rx) = oneshot::channel();
        let timeout = self.spawn_timeout_handler(&request.id);
        
        self.pending_outgoing.insert(request.id.clone(), response_tx);
        self.timeout_handles.insert(request.id.clone(), timeout);
        
        transport.send_message(JsonRpcMessage::V2_0(
            JsonRpcV2Message::Request(request)
        )).await?;
        
        response_rx.await.map_err(RequestError::from)
    }
}
```

## Transport Abstraction Layer

```rust,ignore
// Transport trait with session management
#[async_trait]
pub trait BidirectionalTransport: Send + Sync {
    // Core messaging interface
    async fn send_message(&self, message: JsonRpcMessage) -> Result<(), TransportError>;
    async fn receive_message(&self) -> Result<JsonRpcMessage, TransportError>;
    
    // Connection lifecycle
    async fn connect(&self) -> Result<(), TransportError>;
    async fn close(&self) -> Result<(), TransportError>;
    fn is_connected(&self) -> bool;
    
    // Session management (for HTTP transport)
    fn session_id(&self) -> Option<&str>;
    fn last_event_id(&self) -> Option<&str>;
    async fn resume_from(&self, event_id: &str) -> Result<(), TransportError>;
}

// STDIO transport implementation
pub struct StdioTransport {
    child_process: Arc<Mutex<Child>>,
    stdin_writer: Arc<Mutex<ChildStdin>>,
    stdout_reader: Arc<Mutex<BufReader<ChildStdout>>>,
    connection_state: Arc<RwLock<ConnectionState>>,
}

// HTTP transport implementation  
pub struct HttpTransport {
    client: reqwest::Client,
    endpoint: Url,
    session_id: Option<String>,
    last_event_id: Option<String>,
    auth_token: Arc<RwLock<Option<AccessToken>>>,
}
```

## Protocol State Machine

```rust,ignore
// Three-phase state machine with constraint enforcement
#[derive(Debug, Clone)]
pub struct ProtocolStateMachine {
    current_phase: ConnectionPhase,
    negotiated_capabilities: Option<NegotiatedCapabilities>,
    constraints: ProtocolConstraints,
}

impl ProtocolStateMachine {
    pub fn can_send_message(&self, method: &str, direction: MessageDirection) -> bool {
        match (self.current_phase, direction) {
            (ConnectionPhase::Initialization, MessageDirection::ClientToServer) => {
                matches!(method, "initialize" | "ping")
            }
            (ConnectionPhase::Initialization, MessageDirection::ServerToClient) => {
                matches!(method, "ping" | "logging/message")
            }
            (ConnectionPhase::Operation, _) => {
                self.negotiated_capabilities
                    .as_ref()
                    .map(|caps| caps.supports_method(method))
                    .unwrap_or(false)
            }
            (ConnectionPhase::Shutdown, _) => {
                matches!(method, "ping")
            }
        }
    }
    
    pub fn transition_to(&mut self, phase: ConnectionPhase) -> Result<(), StateError> {
        match (self.current_phase, phase) {
            (ConnectionPhase::Initialization, ConnectionPhase::Operation) => {
                if self.negotiated_capabilities.is_some() {
                    self.current_phase = phase;
                    Ok(())
                } else {
                    Err(StateError::InvalidTransition)
                }
            }
            (_, ConnectionPhase::Shutdown) => {
                self.current_phase = phase;
                Ok(())
            }
            _ => Err(StateError::InvalidTransition),
        }
    }
}
```

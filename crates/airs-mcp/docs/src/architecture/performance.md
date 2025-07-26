# Performance Architecture

## Memory Management Strategy

```rust,ignore
// Zero-copy message processing where possible
pub struct ZeroCopyProcessor {
    message_pool: ObjectPool<JsonRpcMessage>,
    buffer_pool: ObjectPool<Vec<u8>>,
}

impl ZeroCopyProcessor {
    pub fn deserialize_message(&self, bytes: &[u8]) -> Result<JsonRpcMessage, DeserializationError> {
        // Use memory-mapped deserialization where possible
        // Fall back to owned deserialization for complex cases
        if let Ok(message) = self.try_zero_copy_deserialize(bytes) {
            Ok(message)
        } else {
            serde_json::from_slice(bytes).map_err(DeserializationError::from)
        }
    }
}

// Request correlation with automatic cleanup
pub struct LifecycleAwareRequestTracker {
    pending: DashMap<RequestId, PendingRequest>,
    cleanup_interval: Duration,
    max_request_lifetime: Duration,
}

struct PendingRequest {
    sender: oneshot::Sender<JsonRpcResponse>,
    created_at: Instant,
    timeout_handle: AbortHandle,
}

impl LifecycleAwareRequestTracker {
    pub async fn start_cleanup_task(&self) {
        let mut interval = tokio::time::interval(self.cleanup_interval);
        loop {
            interval.tick().await;
            self.cleanup_expired_requests().await;
        }
    }
    
    async fn cleanup_expired_requests(&self) {
        let now = Instant::now();
        let expired: Vec<_> = self.pending
            .iter()
            .filter(|entry| now.duration_since(entry.value().created_at) > self.max_request_lifetime)
            .map(|entry| entry.key().clone())
            .collect();
        
        for request_id in expired {
            if let Some((_, request)) = self.pending.remove(&request_id) {
                request.timeout_handle.abort();
                let _ = request.sender.send(JsonRpcResponse::error(
                    request_id,
                    JsonRpcError::request_timeout(),
                ));
            }
        }
    }
}
```

## Concurrency Architecture

```rust,ignore
// Actor-like pattern for connection management
pub struct ConnectionActor {
    receiver: mpsc::UnboundedReceiver<ConnectionCommand>,
    transport: Box<dyn BidirectionalTransport>,
    state_machine: ProtocolStateMachine,
    message_processor: SecureMessageProcessor,
}

pub enum ConnectionCommand {
    SendMessage {
        message: JsonRpcMessage,
        response_sender: oneshot::Sender<Result<JsonRpcMessage, ProcessingError>>,
    },
    ProcessIncomingMessage {
        message: JsonRpcMessage,
    },
    Shutdown,
}

impl ConnectionActor {
    pub async fn run(mut self) {
        while let Some(command) = self.receiver.recv().await {
            match command {
                ConnectionCommand::SendMessage { message, response_sender } => {
                    let result = self.handle_outgoing_message(message).await;
                    let _ = response_sender.send(result);
                }
                ConnectionCommand::ProcessIncomingMessage { message } => {
                    if let Err(e) = self.handle_incoming_message(message).await {
                        log::error!("Error processing incoming message: {}", e);
                    }
                }
                ConnectionCommand::Shutdown => break,
            }
        }
    }
}
```

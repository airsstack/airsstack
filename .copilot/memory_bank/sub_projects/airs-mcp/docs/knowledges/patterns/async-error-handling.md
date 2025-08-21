# Async Error Handling Patterns

**Category**: Patterns  
**Complexity**: Medium  
**Last Updated**: 2025-08-21  
**Maintainer**: Core Development Team

## Overview
**What is this knowledge about?**

This document captures the async error handling patterns used throughout airs-mcp for consistent, robust error management in asynchronous Rust code. These patterns ensure predictable error propagation, proper resource cleanup, and maintainable error handling across transport layers, protocol handling, and application logic.

**Why this knowledge is important**: Consistent error handling patterns reduce debugging time, improve reliability, and ensure proper resource management in async contexts.

**Who should read this**: All developers working on airs-mcp, especially those implementing new async components or debugging error scenarios.

## Context & Background
**When and why was this approach chosen?**

The async error handling patterns were established during Phase 2 development when implementing HTTP transport functionality. The patterns address several async-specific challenges:

- **Resource Cleanup**: Ensuring proper cleanup when async operations fail
- **Error Propagation**: Maintaining error context across async boundaries
- **Cancellation Safety**: Handling task cancellation without resource leaks
- **Timeout Handling**: Consistent timeout behavior across different async operations

**Problems this approach solves**:
- Inconsistent error handling across async boundaries
- Resource leaks when async operations fail or are cancelled
- Lost error context in complex async call chains
- Unclear error recovery strategies

**Alternative approaches considered**:
- **Panic-based Error Handling**: Rejected due to poor error recovery and debugging experience
- **Result Chaining without Context**: Rejected due to loss of error context in complex scenarios
- **Callback-based Error Handling**: Rejected due to complexity and callback hell

## Technical Details
**How does this work?**

### Core Error Types

```rust
// Base transport error with context preservation
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection failed: {source}")]
    ConnectionFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
        context: String,
    },
    
    #[error("Send operation failed: {message}")]
    SendFailed { message: String },
    
    #[error("Receive operation failed: {message}")]
    ReceiveFailed { message: String },
    
    #[error("Transport timeout after {duration:?}")]
    Timeout { duration: Duration },
    
    #[error("Transport not connected")]
    NotConnected,
}

// HTTP-specific error extension
#[derive(Debug, thiserror::Error)]
pub enum HttpTransportError {
    #[error("HTTP request failed: {status_code}")]
    RequestFailed { status_code: u16, body: String },
    
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },
    
    #[error("Serialization failed")]
    SerializationFailed(#[from] serde_json::Error),
}
```

### Error Context Pattern

```rust
// Context-preserving error conversion
impl From<reqwest::Error> for TransportError {
    fn from(error: reqwest::Error) -> Self {
        TransportError::ConnectionFailed {
            source: Box::new(error),
            context: "HTTP client request failed".to_string(),
        }
    }
}

// Context enhancement helper
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T, TransportError>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context(self, context: &str) -> Result<T, TransportError> {
        self.map_err(|error| TransportError::ConnectionFailed {
            source: Box::new(error),
            context: context.to_string(),
        })
    }
}
```

### Timeout and Cancellation Pattern

```rust
use tokio::time::{timeout, Duration};

// Timeout wrapper for async operations
pub async fn with_timeout<T>(
    operation: impl Future<Output = Result<T, TransportError>>,
    duration: Duration,
) -> Result<T, TransportError> {
    match timeout(duration, operation).await {
        Ok(result) => result,
        Err(_) => Err(TransportError::Timeout { duration }),
    }
}

// Cancellation-safe resource management
pub struct TransportGuard {
    cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl TransportGuard {
    pub fn new<F>(cleanup: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            cleanup: Some(Box::new(cleanup)),
        }
    }
}

impl Drop for TransportGuard {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}
```

## Code Examples
**Practical implementation examples**

### Basic Error Handling Pattern
```rust
impl HttpClientTransport {
    pub async fn send(&self, data: Vec<u8>) -> Result<(), TransportError> {
        // Check connection state first
        if !self.is_connected() {
            return Err(TransportError::NotConnected);
        }
        
        // Apply timeout to the entire operation
        with_timeout(
            self.send_internal(data),
            self.config.send_timeout,
        ).await
    }
    
    async fn send_internal(&self, data: Vec<u8>) -> Result<(), TransportError> {
        // Setup cleanup guard for partial operations
        let _guard = TransportGuard::new(|| {
            // Cleanup any partial state on failure
            log::debug!("Cleaning up failed send operation");
        });
        
        // Perform HTTP request with context
        let response = self.client
            .post(&self.target_url.as_ref().unwrap().to_string())
            .body(data)
            .send()
            .await
            .with_context("Failed to send HTTP request")?;
            
        // Handle HTTP-specific errors
        if !response.status().is_success() {
            return Err(TransportError::SendFailed {
                message: format!("HTTP {}: {}", 
                    response.status(), 
                    response.text().await.unwrap_or_default()
                ),
            });
        }
        
        // Success - guard cleanup not needed
        std::mem::forget(_guard);
        Ok(())
    }
}
```

### Error Recovery Pattern
```rust
impl HttpClientTransport {
    pub async fn send_with_retry(&self, data: Vec<u8>) -> Result<(), TransportError> {
        let mut attempts = 0;
        let max_attempts = self.config.max_retries;
        
        loop {
            match self.send(data.clone()).await {
                Ok(()) => return Ok(()),
                Err(TransportError::Timeout { .. }) if attempts < max_attempts => {
                    attempts += 1;
                    log::warn!("Send timeout, retrying attempt {}/{}", attempts, max_attempts);
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2_u64.pow(attempts));
                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(TransportError::ConnectionFailed { .. }) if attempts < max_attempts => {
                    attempts += 1;
                    log::warn!("Connection failed, attempting reconnection {}/{}", attempts, max_attempts);
                    
                    // Attempt reconnection
                    if let Err(reconnect_error) = self.reconnect().await {
                        log::error!("Reconnection failed: {}", reconnect_error);
                    }
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }
}
```

### Stream Error Handling
```rust
use futures::stream::{Stream, StreamExt};

impl HttpClientTransport {
    pub fn receive_stream(&self) -> impl Stream<Item = Result<Vec<u8>, TransportError>> {
        let queue = Arc::clone(&self.message_queue);
        
        async_stream::stream! {
            loop {
                // Non-blocking queue check with proper error handling
                match self.try_receive_message().await {
                    Ok(Some(message)) => yield Ok(message),
                    Ok(None) => {
                        // No message available, wait briefly
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }
                    Err(error) => {
                        log::error!("Stream receive error: {}", error);
                        yield Err(error);
                        break;
                    }
                }
            }
        }
    }
}
```

## Performance Characteristics
**How does this perform?**

### Error Handling Overhead
- **Error Creation**: O(1) for most error types, O(k) for errors with context strings
- **Error Propagation**: Zero-cost abstractions - no runtime overhead for `?` operator
- **Context Enhancement**: Small allocation overhead for context strings

### Memory Usage
- **Error Storage**: Minimal - errors are typically short-lived
- **Context Strings**: String allocation for error context (can be optimized with static strings)
- **Cleanup Guards**: Minimal stack allocation for RAII cleanup

### Timeout Performance
- **Timeout Overhead**: Uses Tokio's efficient timer wheel implementation
- **Cancellation**: Zero-cost cancellation using Tokio's task cancellation
- **Resource Cleanup**: RAII pattern ensures cleanup occurs in Drop implementation

## Trade-offs & Limitations
**What are the constraints and compromises?**

### Design Trade-offs
- **Context vs Performance**: Error context strings improve debugging but add allocation overhead
- **Type Safety vs Flexibility**: Strongly-typed errors improve safety but require more enum variants
- **Retry Logic vs Complexity**: Automatic retry improves reliability but adds complexity

### Current Limitations
- **Error Context Allocation**: Context strings allocate memory on error creation
- **Retry Strategy**: Simple exponential backoff may not be optimal for all network conditions
- **Error Serialization**: Errors are not serializable for network transmission

### Async-Specific Considerations
- **Cancellation Safety**: Must ensure no resource leaks when tasks are cancelled
- **Send/Sync Requirements**: Errors must be Send + Sync for use across async boundaries
- **Future Polling**: Error handling must not block future polling

## Dependencies
**What does this rely on?**

### External Crates
- **thiserror**: Declarative error type derivation
- **tokio**: Async runtime and timeout utilities
- **futures**: Stream abstractions and async utilities
- **async-stream**: Stream creation macros

### Internal Modules
- **base**: Common error types and utilities
- **shared**: Protocol-specific error definitions

### Configuration Requirements
- **Timeout Configuration**: All timeout values configurable through config structs
- **Retry Configuration**: Retry counts and backoff strategies configurable

## Testing Strategy
**How is this tested?**

### Error Scenario Testing
```rust
#[tokio::test]
async fn test_timeout_error_handling() {
    let config = HttpTransportConfig {
        send_timeout: Duration::from_millis(1), // Very short timeout
        ..Default::default()
    };
    
    let transport = HttpClientTransport::new(
        "http://httpbin.org/delay/5".parse().unwrap(), // 5 second delay
        config
    ).await.unwrap();
    
    let result = transport.send(b"test".to_vec()).await;
    
    assert!(matches!(result, Err(TransportError::Timeout { .. })));
}

#[tokio::test]
async fn test_error_context_preservation() {
    let transport = create_failing_transport().await;
    
    let result = transport.send(b"test".to_vec()).await;
    
    match result {
        Err(TransportError::ConnectionFailed { source, context }) => {
            assert!(context.contains("HTTP client request failed"));
            assert!(source.to_string().contains("connection refused"));
        }
        _ => panic!("Expected ConnectionFailed error with context"),
    }
}
```

### Resource Cleanup Testing
```rust
#[tokio::test]
async fn test_cleanup_guard_on_cancellation() {
    let cleanup_called = Arc::new(AtomicBool::new(false));
    let cleanup_flag = Arc::clone(&cleanup_called);
    
    let task = tokio::spawn(async move {
        let _guard = TransportGuard::new(move || {
            cleanup_flag.store(true, Ordering::SeqCst);
        });
        
        // Simulate long-running operation
        tokio::time::sleep(Duration::from_secs(10)).await;
    });
    
    // Cancel the task
    task.abort();
    let _ = task.await;
    
    // Verify cleanup was called
    assert!(cleanup_called.load(Ordering::SeqCst));
}
```

## Common Pitfalls
**What should developers watch out for?**

### Async Error Handling Mistakes
- **Blocking in Error Handlers**: Never use blocking I/O in async error handling code
- **Ignoring Cancellation**: Always ensure proper cleanup when tasks are cancelled
- **Error Context Loss**: Use `.with_context()` to preserve error information across async boundaries
- **Resource Leaks**: Use RAII patterns (TransportGuard) for cleanup in async contexts

### Debugging Tips
- **Error Context**: Always include relevant context when creating errors
- **Timeout Debugging**: Log timeout values and operation durations for debugging
- **Error Chain**: Use `source()` method to traverse error chains for root cause analysis
- **Async Stack Traces**: Use `#[track_caller]` for better async stack traces

### Performance Considerations
- **Error Creation Cost**: Don't create errors in hot paths for flow control
- **Context String Allocation**: Use static strings for error context when possible
- **Retry Overhead**: Implement exponential backoff to avoid overwhelming failing services

## Related Knowledge
**What else should I read?**

### Related Architecture Documents
- **architecture/transport-layer-design.md**: Transport layer architecture that uses these error patterns
- **integration/mcp-protocol-compliance.md**: Protocol error handling requirements

### Performance Analysis
- **performance/error-handling-benchmarks.md**: Performance characteristics of different error handling approaches

### Security Considerations
- **security/error-information-disclosure.md**: Guidelines for error messages that don't leak sensitive information

## Evolution History
**How has this changed over time?**

### Major Revisions
- **2025-08-14**: Initial async error handling patterns established
  - Implemented TransportError hierarchy with context preservation
  - Added timeout and cancellation safety patterns
  - Established retry and recovery strategies

### Future Evolution Plans
- **Structured Logging**: Enhanced error context with structured logging integration
- **Metrics Integration**: Error rate and timeout metrics for monitoring
- **Error Serialization**: Serializable errors for distributed error handling
- **Custom Error Types**: Domain-specific error types for different components

## Examples in Codebase
**Where can I see this in action?**

### Reference Implementations
- **crates/airs-mcp/src/transport/http/client.rs**: Complete async error handling in HTTP client
- **crates/airs-mcp/src/transport/http/error.rs**: Error type definitions and conversions
- **crates/airs-mcp/src/base/error.rs**: Base error types and utilities

### Test Files
- **crates/airs-mcp/tests/error_handling_integration.rs**: Comprehensive error scenario testing
- **crates/airs-mcp/src/transport/http/tests.rs**: Unit tests for specific error patterns

### Example Applications
- **crates/airs-mcp/examples/error_handling_examples.rs**: Error handling pattern demonstrations
- **crates/airs-mcp/examples/simple-mcp-client/**: Real-world error handling usage

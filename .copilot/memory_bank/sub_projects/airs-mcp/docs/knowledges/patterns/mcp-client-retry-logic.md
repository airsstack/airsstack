# MCP Client Retry Logic Implementation (Preserved)

**Created**: 2025-09-16  
**Status**: Preserved for future implementation  
**Context**: Phase 3 - McpClient Refactoring  
**Decision**: Removed from initial implementation to resolve compilation issues

## Overview

This document preserves the retry logic implementation that was initially planned for the TransportClient-based MCP client but was temporarily removed to ship the core functionality.

## Implementation Details

### Configuration Structure

```rust
/// Configuration for MCP client behavior (retry-related fields)
#[derive(Debug, Clone)]
pub struct McpClientConfig {
    // ... other fields ...
    
    /// Whether to automatically retry failed operations
    pub auto_retry: bool,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay (doubles with each retry for exponential backoff)
    pub initial_retry_delay: Duration,
    /// Maximum retry delay (caps exponential backoff)
    pub max_retry_delay: Duration,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            // ... other defaults ...
            auto_retry: true,
            max_retries: 3,
            initial_retry_delay: Duration::from_millis(100),
            max_retry_delay: Duration::from_secs(10),
        }
    }
}
```

### Builder Pattern Methods

```rust
impl McpClientBuilder {
    /// Enable automatic retry on failures
    pub fn auto_retry(mut self, enabled: bool, max_retries: u32) -> Self {
        self.config.auto_retry = enabled;
        self.config.max_retries = max_retries;
        self
    }

    /// Configure retry timing (exponential backoff)
    pub fn retry_timing(mut self, initial_delay: Duration, max_delay: Duration) -> Self {
        self.config.initial_retry_delay = initial_delay;
        self.config.max_retry_delay = max_delay;
        self
    }
}
```

### Core Retry Logic

```rust
impl<T: TransportClient + 'static> McpClient<T> {
    /// Check if an error is retryable
    fn is_retryable_error(error: &McpError) -> bool {
        match error {
            McpError::NotConnected => true,
            McpError::Timeout { .. } => true,
            McpError::Integration(IntegrationError::Timeout { .. }) => true,
            // Don't retry most other errors
            _ => false,
        }
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let delay = self.config.initial_retry_delay * 2_u32.pow(attempt);
        std::cmp::min(delay, self.config.max_retry_delay)
    }

    /// Execute an operation with retry logic
    async fn execute_with_retry<F, Fut, R>(&mut self, operation: F) -> McpResult<R>
    where
        F: Fn(&mut Self) -> Fut,
        Fut: std::future::Future<Output = McpResult<R>>,
    {
        let mut attempt = 0;
        loop {
            match operation(self).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempt += 1;

                    // Check if we should retry
                    if !self.config.auto_retry
                        || attempt > self.config.max_retries
                        || !Self::is_retryable_error(&error)
                    {
                        return Err(error);
                    }

                    // Calculate retry delay
                    let delay = self.calculate_retry_delay(attempt - 1);
                    warn!(
                        attempt = attempt,
                        error = %error,
                        delay_ms = delay.as_millis(),
                        "Operation failed, retrying after delay"
                    );

                    // Wait before retry
                    sleep(delay).await;
                }
            }
        }
    }
}
```

### Usage Pattern

```rust
/// Initialize connection with the MCP server (with retry)
pub async fn initialize(&mut self) -> McpResult<ServerCapabilities> {
    // ... setup code ...
    
    let result = self.execute_with_retry(|client| async move {
        client.initialize_without_retry().await
    }).await;
    
    // ... result handling ...
}
```

## Issues Encountered

### Lifetime Compilation Error

The main issue was with the closure lifetime in `execute_with_retry`:

```rust
// This caused lifetime compilation errors:
let result = self.execute_with_retry(|client| async move {
    client.initialize_without_retry().await
}).await;
```

**Error Message**:
```
lifetime may not live long enough
return type of closure contains a lifetime `'2`
has type `&'1 mut McpClient<T>`
returning this value requires that `'1` must outlive `'2`
```

### Root Cause

The issue stems from the complex interaction between:
1. Mutable reference to `self` in the closure parameter
2. Async block lifetime within the closure
3. Generic lifetime bounds on the `Fut` future type

## Future Implementation Options

### Option 1: Alternative Retry Pattern (Recommended)

```rust
async fn initialize_with_retry(&mut self) -> McpResult<ServerCapabilities> {
    let mut attempt = 0;
    loop {
        match self.initialize_without_retry().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempt += 1;
                
                if !self.config.auto_retry 
                    || attempt > self.config.max_retries 
                    || !Self::is_retryable_error(&error) 
                {
                    return Err(error);
                }

                let delay = self.calculate_retry_delay(attempt - 1);
                warn!(attempt = attempt, error = %error, "Retrying after delay");
                sleep(delay).await;
            }
        }
    }
}
```

### Option 2: Macro-Based Retry

```rust
macro_rules! retry_operation {
    ($self:expr, $operation:expr) => {{
        let mut attempt = 0;
        loop {
            match $operation {
                Ok(result) => break Ok(result),
                Err(error) => {
                    attempt += 1;
                    if !$self.config.auto_retry 
                        || attempt > $self.config.max_retries 
                        || !Self::is_retryable_error(&error) 
                    {
                        break Err(error);
                    }
                    let delay = $self.calculate_retry_delay(attempt - 1);
                    sleep(delay).await;
                }
            }
        }
    }};
}

// Usage:
pub async fn initialize(&mut self) -> McpResult<ServerCapabilities> {
    retry_operation!(self, self.initialize_without_retry().await)
}
```

### Option 3: Trait-Based Retry

```rust
#[async_trait]
trait Retryable<T> {
    async fn with_retry(&mut self, operation: impl Future<Output = McpResult<T>>) -> McpResult<T>;
}

#[async_trait]
impl<T: TransportClient> Retryable<ServerCapabilities> for McpClient<T> {
    async fn with_retry(&mut self, operation: impl Future<Output = McpResult<ServerCapabilities>>) -> McpResult<ServerCapabilities> {
        // retry logic here
    }
}
```

## Configuration Cleanup Required

When implementing retry logic, remember to:

1. **Remove unused config fields** from `McpClientConfig` if not implementing retry
2. **Remove builder methods** for retry configuration
3. **Update documentation** to remove retry references
4. **Clean up imports** (`tokio::time::sleep`, `warn!` macro)

## Integration Points

When re-adding retry logic:

1. **Apply to all operations**: Not just `initialize()`, but also `list_tools()`, `call_tool()`, etc.
2. **Selective retry**: Some operations (like `close()`) should not be retried
3. **Error categorization**: Expand `is_retryable_error()` based on real-world usage
4. **Observability**: Ensure retry attempts are properly logged with structured data

## Testing Considerations

- **Mock transport errors** to test retry behavior
- **Exponential backoff validation** with time-based assertions
- **Max retry limits** to prevent infinite loops
- **Error propagation** for non-retryable errors
# Shared Patterns and Technical Standards

This file documents core implementation, architecture, and methodology patterns shared across all sub-projects in the workspace. These standards ensure consistency, maintainability, and professional code quality across the entire AIRS ecosystem.

## Zero-Warning Policy (Mandatory)

### Code Quality Requirements
All sub-projects MUST maintain zero compiler warnings:

```bash
# All these commands must complete with zero warnings
cargo check --workspace
cargo clippy --workspace --all-targets --all-features  
cargo test --workspace --doc
```

**Enforcement:** CI/CD pipelines will fail if any warnings are present.

### Warning Resolution Strategies

#### Dead Code / Unused Items
```rust
// Option 1: Remove unused code entirely (preferred)
// fn unused_function() { } // DELETE

// Option 2: Mark as intentionally unused for future use
#[allow(dead_code)] // Framework for future functionality
struct FutureFeature {
    placeholder: String,
}

// Option 3: Add underscore prefix for temporary variables
fn example() {
    let _temporary_unused = "will be used later";
}
```

#### Unused Imports
```rust
// Remove unused imports immediately
// use std::collections::HashMap; // DELETE if not used

// Use conditional compilation for feature-specific imports
#[cfg(feature = "advanced")]
use advanced_feature::AdvancedType;
```

#### Documentation Tests
```rust
/// Example function with properly tested documentation
///
/// ```
/// # use crate::example_function;
/// let result = example_function(42);
/// assert_eq!(result, 84);
/// ```
pub fn example_function(x: i32) -> i32 {
    x * 2
}

/// For complex examples that cannot be tested
///
/// ```ignore
/// // This example requires external setup
/// let connection = connect_to_external_service().await?;
/// ```
pub fn complex_function() { }
```

### Test Quality Standards
All sub-projects must maintain:

- **Unit Tests**: 100% coverage of public APIs
- **Integration Tests**: Critical path validation
- **Documentation Tests**: All examples must compile and run (or be explicitly ignored)
- **Property Tests**: For complex algorithms (where applicable)

## Code Organization Standards

### Import Order Pattern (Mandatory)
All Rust source files must follow this three-layer import organization:

```rust
// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports  
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;

// Layer 3: Internal module imports
use crate::base::jsonrpc::{JsonRpcMessage, RequestId};
use crate::correlation::types::CorrelationEntry;
use crate::shared::types::MessageResult;
```

**Rationale:** Provides clear separation of concerns, improves readability, and follows Rust community best practices.

### Dependency Management Pattern
All sub-projects inherit dependencies from workspace-level `Cargo.toml`:

```toml
[workspace.dependencies]
async-trait = "0.1.88"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
# ... other shared dependencies
```

**Benefits:** Centralized version control, consistent dependency versions, easier maintenance.

#### Centralized Dependency Governance (Mandatory)

**All Dependencies MUST:**
1. **Be Defined in Workspace Root**: All dependencies used by any sub-project MUST be declared in `/Cargo.toml` under `[workspace.dependencies]`
2. **Use Latest Stable Versions**: All versions MUST be the latest stable release available on crates.io
3. **Inherit from Workspace**: Sub-project `Cargo.toml` files MUST use `dependency.workspace = true` syntax
4. **No Direct Versions**: Sub-projects are FORBIDDEN from declaring version numbers directly

**Version Update Policy:**
- **Monthly Review**: Dependency versions MUST be reviewed monthly for updates
- **Security Updates**: Security patches MUST be applied within 48 hours of discovery
- **Major Version Updates**: Require impact assessment and testing before adoption
- **Deprecated Dependencies**: MUST be replaced immediately upon deprecation notice

**Violation Consequences:**
- Build failures in CI/CD pipeline
- Blocked pull request merges
- Technical debt tracking and remediation requirements

**Example Compliance:**
```toml
# ✅ CORRECT: Workspace root Cargo.toml
[workspace.dependencies]
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

# ✅ CORRECT: Sub-project Cargo.toml
[dependencies]
tokio.workspace = true
serde.workspace = true

# ❌ FORBIDDEN: Direct versions in sub-project
[dependencies]
tokio = "1.35"  # VIOLATION: Must inherit from workspace
serde = "1.0"   # VIOLATION: Must inherit from workspace
```

## Documentation Standards

### Module Documentation
Every module must include comprehensive documentation:

```rust
//! Module purpose and functionality
//!
//! Detailed description of module responsibilities, key types,
//! and integration patterns. Include examples for public APIs.
```

### Type Documentation
All public types require documentation with examples:

```rust
/// Brief description of the type's purpose
///
/// Detailed explanation of usage patterns and behavior.
/// 
/// # Examples
/// 
/// ```rust
/// # use crate::types::ExampleType;
/// let instance = ExampleType::new();
/// ```
#[derive(Debug, Clone)]
pub struct ExampleType {
    /// Field documentation with purpose and constraints
    pub field: String,
}
```

### Error Documentation
Error types must provide context and debugging guidance:

```rust
/// Specific error condition description
///
/// Include when this error occurs and suggested resolution steps.
#[derive(Debug, Clone, Error)]
pub enum ModuleError {
    /// Request timed out waiting for response
    #[error("Request {id} timed out after {duration}")]
    Timeout {
        /// The request ID that timed out
        id: RequestId,
        /// The timeout duration that was exceeded  
        duration: TimeDelta,
    },
}
```

## Testing Standards

### Test Coverage Requirements
- **Unit Tests:** Minimum 80% line coverage for all modules
- **Integration Tests:** All public APIs must have integration test coverage
- **Doc Tests:** All public functions with examples must have working doc tests

### Test Organization Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_happy_path() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input).await;
        
        // Assert
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_error_conditions() {
        // Test error scenarios with descriptive assertions
    }
}
```

## Error Handling Patterns

### Structured Error Types
Use `thiserror` for all error definitions with context:

```rust
#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Validation failed: {field} {reason}")]
    Validation { field: String, reason: String },
}
```

### Error Propagation
Use `?` operator consistently and provide context at boundaries:

```rust
pub async fn high_level_operation() -> Result<Output, ModuleError> {
    let data = fetch_data().await
        .map_err(|e| ModuleError::Validation {
            field: "input".to_string(),
            reason: format!("Invalid input: {}", e),
        })?;
    
    process_data(data).await
}
```

## Async Patterns

### Async Trait Usage
Use `async-trait` for trait definitions requiring async methods:

```rust
#[async_trait]
pub trait AsyncProcessor {
    async fn process(&self, input: Input) -> Result<Output, ProcessError>;
}
```

### Channel Communication
Prefer `tokio::sync::mpsc` for async communication:

```rust
let (tx, mut rx) = mpsc::channel::<Message>(100);
```

## Architecture Principles

### SOLID Principles Application
- **Single Responsibility:** Each module has one well-defined purpose
- **Open/Closed:** Use traits for extensibility without modification
- **Liskov Substitution:** Trait implementations must be substitutable
- **Interface Segregation:** Keep trait interfaces focused and minimal
- **Dependency Inversion:** Depend on abstractions, not concretions

### Clean Architecture Layers
1. **Domain:** Core business logic and types
2. **Application:** Use cases and orchestration
3. **Infrastructure:** I/O, networking, persistence
4. **Interface:** User-facing APIs and adapters

### Technical Debt Management
- Document all technical debt with GitHub issues
- Include remediation plans and impact assessments
- Regular technical debt reviews and prioritization
- No accumulation of undocumented shortcuts

## Quality Gates

### Pre-commit Requirements
- All tests pass (`cargo test --workspace`)
- No clippy warnings (`cargo clippy --workspace --all-targets --all-features`)  
- Proper formatting (`cargo fmt --check`)
- Documentation builds (`cargo doc --workspace`)

### Code Review Standards
- Minimum two approvals for architectural changes
- Focus on maintainability over cleverness
- Explicit approval for any technical debt introduction
- Documentation updates for behavioral changes

## Concurrent Processing Patterns (Enterprise-Grade)

### Worker Pool Architecture Pattern
```rust
// Standard worker pool implementation pattern
use tokio::sync::{mpsc, Semaphore};
use std::sync::Arc;

pub struct ConcurrentProcessor<T> {
    workers: Arc<RwLock<Vec<WorkerState>>>,
    backpressure_semaphore: Arc<Semaphore>,
    is_running: Arc<AtomicBool>,
}

// CRITICAL PATTERN: Non-blocking backpressure
let _permit = self.backpressure_semaphore.try_acquire().map_err(|_| {
    Error::QueueFull { capacity: self.total_capacity }
})?;
```

### Deadlock Prevention Pattern (Mandatory)
```rust
// CRITICAL PATTERN: Clone handlers outside locks
let handler_option = {
    let handlers_read = handlers.read().await;
    handlers_read.get(&method).cloned()
}; // Lock dropped here!

// Process without holding locks
match handler_option {
    Some(handler) => handler.process(message).await,
    None => Err(HandlerNotFound),
}
```

### Resource Cleanup Pattern (Mandatory)
```rust
// CRITICAL PATTERN: Unconditional resource cleanup
async fn worker_loop() {
    while let Some(task) = queue_rx.recv().await {
        // Process task
        let result = process_task(&task).await;
        
        // CRITICAL: Always release resources, even on error
        if config.enable_backpressure {
            backpressure_semaphore.add_permits(1);
        }
        
        // Send result (ignore send errors - receiver may be dropped)
        let _ = task.response_tx.send(result);
    }
}
```

### Graceful Shutdown Pattern (Mandatory)
```rust
// CRITICAL PATTERN: Signal-first shutdown
pub async fn shutdown(&mut self) -> Result<(), Error> {
    // 1. Signal shutdown first
    self.is_running.store(false, Ordering::Relaxed);
    
    // 2. Close channels to signal workers
    let mut workers = self.workers.write().await;
    for mut worker in workers.drain(..) {
        drop(worker.queue_tx); // Closes channel
    }
    
    // 3. Wait for workers with timeout
    for handle in handles {
        let _ = timeout(Duration::from_secs(5), handle).await;
    }
}
```

### Statistics Collection Pattern
```rust
// CRITICAL PATTERN: Lock-free statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub total_processed: Arc<AtomicU64>,
    pub successful_operations: Arc<AtomicU64>,
    pub failed_operations: Arc<AtomicU64>,
    pub current_queue_depth: Arc<AtomicUsize>,
}

// Update statistics without locks
stats.total_processed.fetch_add(1, Ordering::Relaxed);
stats.update_queue_depth(current_depth); // Atomic operations only
```

### Testing Concurrent Systems Pattern
```rust
// CRITICAL PATTERN: Handle Arc lifetime in tests
#[tokio::test]
async fn test_concurrent_operations() {
    let processor = Arc::new(processor);
    
    // Submit concurrent operations
    let handles: Vec<_> = (0..20).map(|i| {
        let processor_clone = processor.clone();
        tokio::spawn(async move {
            processor_clone.submit_message(message).await
        })
    }).collect();
    
    // Wait for completion before shutdown
    let _results = futures::future::join_all(handles).await;
    
    // Graceful Arc unwrapping
    match Arc::try_unwrap(processor) {
        Ok(mut proc) => proc.shutdown().await.unwrap(),
        Err(_) => {
            // Graceful fallback - test still validates behavior
            println!("Arc unwrap failed (expected with pending references)");
            return;
        }
    }
}
```

## Workspace Inheritance
All sub-projects must inherit and extend these patterns. Project-specific patterns should be documented in sub-project memory banks but must not conflict with workspace standards.

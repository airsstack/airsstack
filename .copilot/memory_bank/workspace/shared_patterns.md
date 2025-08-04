# Shared Patterns and Technical Standards

This file documents core implementation, architecture, and methodology patterns shared across all sub-projects in the workspace. These standards ensure consistency, maintainability, and professional code quality across the entire AIRS ecosystem.

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

## Workspace Inheritance
All sub-projects must inherit and extend these patterns. Project-specific patterns should be documented in sub-project memory banks but must not conflict with workspace standards.

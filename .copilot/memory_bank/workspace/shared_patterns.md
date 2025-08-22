# Shared Patterns and Technical Standards

This file documents core implementation, architecture, and methodology patterns shared across all sub-projects in the workspace. These standards ensure consistency, maintainability, and professional code quality across the entire AIRS ecosystem.

## Rust Language Standards (Mandatory)

### §1. Generic Type Usage - Zero-Cost Abstractions

**Principle**: Always prefer generics over trait objects for performance-critical paths.

#### Generic Trait Constraints (Preferred)
```rust
// ✅ Zero-cost abstractions with compile-time dispatch
pub struct Validator<J, S> 
where
    J: JwtValidator,
    S: ScopeValidator,
{
    jwt: J,
    scope: S,
}

impl<J, S> Validator<J, S> 
where
    J: JwtValidator,
    S: ScopeValidator,
{
    pub const fn new(jwt: J, scope: S) -> Self {
        Self { jwt, scope }
    }
}
```

#### Trait Objects (Only When Necessary)
```rust
// ❌ Avoid unless you need runtime polymorphism
struct Validator {
    jwt: Box<dyn JwtValidator>,    // Runtime dispatch + heap allocation
    scope: Box<dyn ScopeValidator>, // vtable lookup overhead
}

// ✅ Valid use cases for trait objects:
// - Heterogeneous collections: Vec<Box<dyn Trait>>
// - Plugin architectures with runtime loading
// - Configuration-driven type selection
```

### §2. Lifetime Management - Minimal Constraints

**Principle**: Avoid `'static` lifetimes unless truly required for global state.

#### Avoid Unnecessary `'static` Constraints
```rust
// ❌ Over-constrained - forces values to live entire program duration
pub fn process<T: Validator + 'static>(validator: T) -> Result<(), Error> {
    // This prevents using validators with shorter, adequate lifetimes
}

// ✅ Use appropriate lifetime bounds or none at all
pub fn process<T: Validator>(validator: T) -> Result<(), Error> {
    // Let compiler infer appropriate lifetimes
}

// ✅ `'static` only when actually storing in global state
static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config::load());
```

#### Lifetime Bound Guidelines
```rust
// ✅ Explicit lifetime when structure needs it
pub struct Processor<'a, T> {
    data: &'a T,
}

// ✅ No lifetime bounds when ownership is transferred
pub struct Processor<T> {
    data: T,  // Owned data, no lifetime needed
}
```

### §3. Memory Management - Stack vs Heap

**Principle**: Prefer stack allocation unless heap allocation provides clear benefits.

#### Avoid Unnecessary `Box<T>` Allocation
```rust
// ❌ Unnecessary heap allocation
struct Config {
    name: Box<String>,      // String is already heap-allocated
    settings: Box<HashMap<String, String>>,  // HashMap is already heap-allocated
}

// ✅ Direct ownership on stack
struct Config {
    name: String,          // Already heap-allocated internally
    settings: HashMap<String, String>,  // Already heap-allocated internally
}

// ✅ Valid Box<T> use cases:
// - Recursive data structures: Box<Node>
// - Large stack objects: Box<[u8; HUGE_SIZE]>
// - Trait objects: Box<dyn Trait>
// - Breaking ownership cycles
```

#### Smart Pointer Selection
```rust
// Choose appropriate smart pointer for the use case:
use std::rc::Rc;           // Single-threaded reference counting
use std::sync::Arc;        // Multi-threaded reference counting  
use std::cell::RefCell;    // Interior mutability (single-threaded)
use std::sync::Mutex;      // Interior mutability (multi-threaded)

// ✅ Arc for shared ownership across threads
pub struct SharedValidator {
    inner: Arc<ValidatorImpl>,
}

// ✅ Rc for shared ownership within single thread
pub struct LocalValidator {
    inner: Rc<ValidatorImpl>,
}
```

### §4. Trait Design Patterns

**Principle**: Design traits for flexibility and zero-cost abstractions.

#### Associated Types vs Generics
```rust
// ✅ Associated types for single implementation per type
pub trait Iterator {
    type Item;  // Each iterator has one specific item type
    fn next(&mut self) -> Option<Self::Item>;
}

// ✅ Generic parameters for multiple implementations
pub trait From<T> {
    fn from(value: T) -> Self;  // Can implement From<String>, From<&str>, etc.
}
```

#### Error Type Patterns
```rust
// ✅ Flexible error handling with associated types
pub trait Validator {
    type Error: Into<ValidationError> + Send + Sync;
    async fn validate(&self, input: &str) -> Result<Claims, Self::Error>;
}

// ✅ Allows different validators to have specific error types
impl Validator for JwtValidator {
    type Error = JwtError;  // Specific to JWT validation
    // ...
}
```

### §5. Advanced Builder Pattern - Progressive Type Refinement

**Principle**: Use progressive type refinement in builder patterns for maximum type safety with ergonomic APIs.

#### Builder Struct Design - No Premature Constraints
```rust
// ✅ CORRECT - No constraints on struct definition
pub struct ValidatorBuilder<J, S> {
    jwt: Option<J>,    // No trait bounds here
    scope: Option<S>,  // No trait bounds here
}

// ❌ INCORRECT - Overly restrictive constraints
pub struct ValidatorBuilder<J, S> 
where
    J: JwtValidator,    // Don't do this - prevents flexible construction
    S: ScopeValidator,  // Don't do this - prevents type evolution
{
    jwt: Option<J>,
    scope: Option<S>,
}
```

#### Progressive Constraint Application
```rust
impl<J, S> ValidatorBuilder<J, S> {
    // ✅ Apply constraints only where needed
    pub fn jwt<NewJ>(self, jwt_validator: NewJ) -> ValidatorBuilder<NewJ, S>
    where
        NewJ: JwtValidator,  // Constraint applied when setting component
    {
        ValidatorBuilder {
            jwt: Some(jwt_validator),
            scope: self.scope,
        }
    }

    // ✅ Constraints enforced at build time
    pub fn build(self) -> Result<Validator<J, S>, BuilderError>
    where
        J: JwtValidator,     // Required for final construction
        S: ScopeValidator,   // Required for final construction
    {
        let jwt = self.jwt.ok_or(BuilderError::MissingJwtValidator)?;
        let scope = self.scope.ok_or(BuilderError::MissingScopeValidator)?;
        Ok(Validator::new(jwt, scope))
    }
}
```

#### Type Evolution Flow
```rust
// Progressive type refinement enables this natural flow:
let builder = ValidatorBuilder::new();              // ValidatorBuilder<(), ()>
let with_jwt = builder.with_default_jwt(config)?;   // ValidatorBuilder<Jwt, ()>  
let with_scope = with_jwt.with_default_scope();     // ValidatorBuilder<Jwt, Scope>
let validator = with_scope.build()?;                // Validator<Jwt, Scope> - constraints checked here
```

#### Benefits of Progressive Type Refinement
- **Flexible Construction**: Can start with any types (including unit types)
- **Type Safety**: Constraints enforced exactly when needed, not before
- **Ergonomic API**: Natural building flow guided by type system
- **Compile-Time Guarantees**: Impossible states prevented without runtime overhead
- **Zero-Cost Abstractions**: No performance penalty for type safety

#### Anti-Patterns to Avoid
```rust
// ❌ Don't constrain struct when fields are optional
pub struct Builder<T: Trait> {
    field: Option<T>,  // Why require Trait when field might be None?
}

// ❌ Don't apply constraints globally when only some methods need them
impl<T: Trait> Builder<T> {  // This forces all impls to have Trait bound
    pub fn new() -> Self { ... }  // new() doesn't need T: Trait!
}

// ✅ Apply constraints per-method based on actual requirements
impl<T> Builder<T> {
    pub fn new() -> Self { ... }  // No unnecessary constraints
    
    pub fn use_trait(&self) -> Result<(), Error>
    where
        T: Trait,  // Constraint only where actually needed
    {
        self.field.as_ref().unwrap().trait_method()
    }
}
```

### §6. Performance Guidelines

**Summary**: These patterns prioritize performance through zero-cost abstractions while maintaining code clarity and safety.

**Enforcement**: Code reviews must verify adherence to these patterns. Use `cargo bench` to validate performance assumptions and `cargo expand` to verify monomorphization behavior.

**Migration Strategy**: When refactoring existing code, apply these patterns incrementally using the strangler fig pattern to minimize risk.

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

## Standards Compliance (Protocol & Security)

### Protocol Standards Documentation
All sub-projects implementing external protocols MUST maintain comprehensive standards compliance documentation:

**Documentation Requirements:**
- **Complete Protocol Specifications**: Official protocol documents with implementation guides
- **Security Standards**: RFC compliance documentation for security-critical protocols
- **Integration Patterns**: Convergence documentation for multiple standard implementations
- **Reference Architecture**: Implementation patterns that maintain standards compliance

**airs-mcp Standards References:**
- **OAuth 2.1 Compliance**: `sub_projects/airs-mcp/oauth2_rfc_specifications.md`
- **MCP Protocol Compliance**: `sub_projects/airs-mcp/mcp_official_specification.md`  
- **Security Integration**: Complete OAuth 2.1 + MCP 2025-06-18 convergence patterns

**Compliance Validation:**
- **100% Standards Adherence**: All protocol implementations must follow documented specifications
- **Security First**: Security-critical protocols require complete RFC compliance documentation
- **Interoperability**: Standards compliance ensures ecosystem compatibility and future-proofing

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

### §5. Dependency Management - Root Cargo.toml Organization

**Principle**: AIRS foundation crates MUST be prioritized and organized at the top of workspace dependencies for clear dependency hierarchy.

#### Root Cargo.toml Dependency Organization (MANDATORY)
```toml
[workspace.dependencies]
# Layer 1: AIRS Foundation Crates (MUST be at top)
airs-mcp = { path = "crates/airs-mcp" }
airs-mcp-fs = { path = "crates/airs-mcp-fs" }  
airs-memspec = { path = "crates/airs-memspec" }

# Layer 2: Core Runtime Dependencies  
tokio = { version = "1.47", features = ["full"] }
futures = { version = "0.3" }

# Layer 3: Serialization and Data Handling
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0" }

# Layer 4: Additional External Dependencies (alphabetical by category)
# ... rest of external dependencies
```

#### Rationale for Foundation Crate Priority
- **Dependency Hierarchy Clarity**: Internal AIRS crates represent the foundation layer that everything else builds upon
- **Maintenance Visibility**: Changes to foundation crates have the highest impact and should be immediately visible
- **Development Workflow**: Developers should see internal dependencies first when reviewing workspace configuration
- **Version Management**: Foundation crates use path dependencies and require different update strategies than external crates

#### Enforcement Pattern
```rust
// ✅ When adding new AIRS crates
[workspace.dependencies] 
# NEW AIRS crates must be added to Layer 1 (top section)
airs-new-crate = { path = "crates/airs-new-crate" }
airs-mcp = { path = "crates/airs-mcp" }
# ... existing AIRS crates

# External dependencies follow after all AIRS crates
tokio = { version = "1.47", features = ["full"] }
# ... external dependencies
```

#### Individual Crate Cargo.toml Pattern
```toml
# Individual crate dependencies should inherit from workspace
[dependencies]
# Foundation dependencies (inherit from workspace)
airs-mcp = { workspace = true }

# External dependencies (inherit from workspace)
tokio = { workspace = true }
serde = { workspace = true }
```

## Workspace Inheritance
All sub-projects must inherit and extend these patterns. Project-specific patterns should be documented in sub-project memory banks but must not conflict with workspace standards.

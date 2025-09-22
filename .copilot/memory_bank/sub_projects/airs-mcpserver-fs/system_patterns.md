# System Patterns: AIRS MCP Server - Filesystem

**Updated:** 2025-09-22  
**Sub-Project:** airs-mcpserver-fs  
**Context:** Architectural patterns for MCP server implementation

## Architecture Overview

### High-Level Architecture
**AIRS MCP Server - Filesystem** follows a layered architecture that prioritizes security, performance, and maintainability while providing seamless MCP protocol integration.

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Desktop                           │
│                   (MCP Client)                              │
└──────────────────────┬──────────────────────────────────────┘
                       │ JSON-RPC 2.0 / STDIO
┌──────────────────────▼──────────────────────────────────────┐
│                 Transport Layer                             │
│           (airs-mcp integration)                            │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│              Message Handler Layer                          │
│         (FilesystemMessageHandler)                          │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│               Security Framework                            │
│    (5-Layer: Path, Binary, Approval, Audit, Threat)        │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│             Filesystem Operations                           │
│    (Read, Write, Directory Management)                      │
└─────────────────────────────────────────────────────────────┘
```

### Component Relationships

#### 1. Transport Integration Layer
**Pattern**: Adapter pattern for MCP protocol compliance
**Responsibility**: Transforms MCP messages to internal operations
**Key Components**:
- `FilesystemMessageHandler`: Implements MessageHandler<()> trait
- `StdioTransportBuilder`: Configures STDIO transport for Claude Desktop
- Error transformation and response formatting

#### 2. Security Framework Layer
**Pattern**: Chain of Responsibility for security validations
**Responsibility**: Multi-layer security enforcement
**Key Components**:
- **Path Validation**: Prevents directory traversal attacks
- **Binary Restrictions**: Blocks potentially dangerous file types
- **Approval Workflows**: Human-in-the-loop authorization
- **Audit Logging**: Comprehensive operation tracking
- **Threat Detection**: Basic malware and anomaly detection

#### 3. Business Logic Layer
**Pattern**: Service layer with clear domain boundaries
**Responsibility**: Core filesystem operations
**Key Components**:
- File operations (read, write, delete)
- Directory management (create, list, navigate)
- Path resolution and normalization
- Content encoding detection and handling

## Key Technical Decisions

### Decision 1: Security-First Architecture
**Context**: Filesystem access presents significant security risks
**Decision**: Implement 5-layer security framework with human approval
**Rationale**: 
- Prevents unauthorized file system access
- Provides audit trail for compliance
- Balances security with usability
- Enables enterprise deployment

**Implementation Pattern**:
```rust
pub struct SecurityFramework {
    path_validator: PathValidator,
    binary_restrictor: BinaryRestrictor,
    approval_workflow: ApprovalWorkflow,
    audit_logger: AuditLogger,
    threat_detector: ThreatDetector,
}

impl SecurityFramework {
    pub async fn validate_operation(&self, operation: &FilesystemOperation) -> SecurityResult {
        self.path_validator.validate(&operation.path)?;
        self.binary_restrictor.check(&operation)?;
        self.approval_workflow.request_approval(&operation).await?;
        self.audit_logger.log_attempt(&operation);
        self.threat_detector.scan(&operation)?;
        Ok(SecurityApproval::Granted)
    }
}
```

### Decision 2: MessageHandler Integration Pattern
**Context**: New airs-mcp architecture requires MessageHandler implementation
**Decision**: Implement wrapper pattern preserving ToolProvider logic
**Rationale**:
- Maintains compatibility with latest airs-mcp architecture
- Preserves all existing business logic unchanged
- Enables seamless integration with transport layer
- Follows established AIRS patterns

**Implementation Pattern**:
```rust
pub struct FilesystemMessageHandler {
    provider: Arc<FilesystemToolProvider>,
}

impl MessageHandler<()> for FilesystemMessageHandler {
    async fn handle_message(&self, message: JsonRpcMessage) -> JsonRpcResponse {
        // Delegate to existing ToolProvider logic
        self.provider.handle_message(message).await
    }
}
```

### Decision 3: Configuration Management Architecture
**Context**: Complex security policies require flexible configuration
**Decision**: Hierarchical configuration with environment-specific overrides
**Rationale**:
- Supports development, staging, and production environments
- Enables fine-grained security policy control
- Provides clear configuration documentation
- Allows runtime configuration updates

**Implementation Pattern**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub logging: LoggingConfig,
}

impl ServerConfig {
    pub fn load() -> Result<Self> {
        ConfigBuilder::new()
            .add_source(File::with_name("default"))
            .add_source(File::with_name(&format!("config/{}", env)))
            .add_source(Environment::with_prefix("AIRS_MCP_FS"))
            .build()?
            .try_deserialize()
    }
}
```

## Design Patterns in Use

### 1. Factory Pattern - Tool Registration
**Usage**: Creating and registering MCP tools
**Benefit**: Consistent tool creation with proper configuration
```rust
pub struct ToolFactory;

impl ToolFactory {
    pub fn create_filesystem_tools() -> Vec<Tool> {
        vec![
            Self::create_read_file_tool(),
            Self::create_write_file_tool(),
            Self::create_list_directory_tool(),
            Self::create_delete_file_tool(),
        ]
    }
}
```

### 2. Strategy Pattern - Security Policies
**Usage**: Different security policies for different environments
**Benefit**: Flexible security configuration without code changes
```rust
pub trait SecurityPolicy {
    fn validate_path(&self, path: &Path) -> bool;
    fn requires_approval(&self, operation: &FilesystemOperation) -> bool;
}

pub struct DevelopmentPolicy;
pub struct ProductionPolicy;
```

### 3. Observer Pattern - Audit Logging
**Usage**: Logging filesystem operations across multiple observers
**Benefit**: Extensible logging without coupling to business logic
```rust
pub trait AuditObserver {
    fn on_operation(&self, operation: &FilesystemOperation, result: &OperationResult);
}

pub struct FileAuditLogger;
pub struct DatabaseAuditLogger;
pub struct SyslogAuditLogger;
```

### 4. Command Pattern - Operation Execution
**Usage**: Encapsulating filesystem operations for approval workflows
**Benefit**: Uniform operation handling with undo/redo support
```rust
pub trait FilesystemCommand {
    async fn execute(&self) -> OperationResult;
    async fn undo(&self) -> OperationResult;
    fn requires_approval(&self) -> bool;
}

pub struct ReadFileCommand;
pub struct WriteFileCommand;
pub struct DeleteFileCommand;
```

## Error Handling Patterns

### 1. Typed Error Hierarchy
**Pattern**: Structured error types with context preservation
**Implementation**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum FilesystemError {
    #[error("Security validation failed: {reason}")]
    SecurityViolation { reason: String },
    
    #[error("File operation failed: {operation} on {path}")]
    OperationFailed { operation: String, path: PathBuf },
    
    #[error("Configuration error: {context}")]
    ConfigurationError { context: String },
}
```

### 2. Result Chaining with Context
**Pattern**: Preserve error context through operation chains
**Implementation**:
```rust
pub type FilesystemResult<T> = Result<T, FilesystemError>;

impl FilesystemOperations {
    pub async fn read_file(&self, path: &Path) -> FilesystemResult<String> {
        self.security.validate_read(path)
            .map_err(|e| FilesystemError::SecurityViolation { reason: e.to_string() })?;
        
        fs::read_to_string(path).await
            .map_err(|e| FilesystemError::OperationFailed { 
                operation: "read".to_string(), 
                path: path.to_path_buf() 
            })
    }
}
```

## Performance Patterns

### 1. Async-First Architecture
**Pattern**: All I/O operations use async/await
**Benefit**: Non-blocking operations with excellent concurrency
```rust
impl FilesystemOperations {
    pub async fn batch_operations(&self, operations: Vec<FilesystemOperation>) -> Vec<OperationResult> {
        let tasks: Vec<_> = operations.into_iter()
            .map(|op| tokio::spawn(self.execute_operation(op)))
            .collect();
        
        futures::future::join_all(tasks).await
            .into_iter()
            .map(|result| result.unwrap_or_else(|_| OperationResult::Failed))
            .collect()
    }
}
```

### 2. Resource Management
**Pattern**: Bounded resource usage with automatic cleanup
**Implementation**:
```rust
pub struct ResourceManager {
    max_concurrent_operations: usize,
    operation_semaphore: Semaphore,
    temp_file_cleanup: Arc<TempFileCleanup>,
}

impl ResourceManager {
    pub async fn execute_with_limits<F, T>(&self, operation: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let _permit = self.operation_semaphore.acquire().await?;
        operation.await
    }
}
```

## Testing Patterns

### 1. Behavior-Driven Testing
**Pattern**: Test behaviors rather than implementation details
**Example**:
```rust
#[tokio::test]
async fn test_security_framework_blocks_unauthorized_access() {
    // Given: A filesystem server with security enabled
    let server = create_test_server_with_security().await;
    
    // When: Attempting to access a restricted path
    let result = server.read_file("/etc/passwd").await;
    
    // Then: Access should be denied with clear error message
    assert!(matches!(result, Err(FilesystemError::SecurityViolation { .. })));
}
```

### 2. Integration Testing Strategy
**Pattern**: Test complete workflows including security and approval
**Implementation**:
```rust
#[tokio::test]
async fn test_end_to_end_file_write_with_approval() {
    let (server, approval_receiver) = create_server_with_approval_workflow().await;
    
    // Start write operation (will be pending approval)
    let write_task = tokio::spawn(server.write_file("test.txt", "content"));
    
    // Simulate human approval
    approval_receiver.approve_next_operation().await;
    
    // Verify write completed successfully
    let result = write_task.await.unwrap();
    assert!(result.is_ok());
}
```

## Migration Patterns

### 1. Backward Compatibility Strategy
**Pattern**: Support both old and new interfaces during transition
**Implementation**:
```rust
// New canonical implementation
pub mod airs_mcpserver_fs {
    pub use crate::*;
}

// Legacy compatibility layer (temporary)
#[deprecated(note = "Use airs_mcpserver_fs instead")]
pub mod airs_mcp_fs {
    pub use crate::*;
}
```

### 2. Gradual Feature Migration
**Pattern**: Migrate features incrementally with validation
**Process**:
1. Implement feature in new structure
2. Validate against existing functionality
3. Update documentation and examples
4. Deprecate old implementation
5. Remove after transition period

This system patterns documentation establishes the architectural foundation for a robust, secure, and maintainable MCP filesystem server that preserves all existing functionality while enabling future ecosystem growth.
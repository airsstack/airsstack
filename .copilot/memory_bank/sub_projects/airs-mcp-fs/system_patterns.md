# System Patterns: AIRS MCP-FS

**Updated:** 2025-08-29  
**Architecture Status:** Implementation In Progress - Security Framework 67% Complete

## Core System Architecture

### Multi-Layer Architecture Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                    Claude Desktop                           │
│                   (MCP Client)                              │
└─────────────────────┬───────────────────────────────────────┘
                      │ STDIO Transport
                      │ JSON-RPC 2.0 Messages
┌─────────────────────▼───────────────────────────────────────┐
│                  AIRS MCP-FS                                │
│                  (MCP Server)                               │
├─────────────────────────────────────────────────────────────┤
│  Security Layer                                             │
│  ├─ Path Validation & Access Control                        │
│  ├─ Human-in-the-Loop Approval Workflows                    │
│  └─ Operation Audit Logging                                 │
├─────────────────────────────────────────────────────────────┤
│  Tool Layer                                                 │
│  ├─ read_file, write_file, list_directory                   │
│  ├─ create_directory, delete_file, move_file                │
│  └─ read_binary, write_binary, extract_content              │
├─────────────────────────────────────────────────────────────┤
│  Binary Processing Engine                                   │
│  ├─ Image Processing (resize, thumbnail, metadata)          │
│  ├─ PDF Processing (text extraction, image extraction)      │
│  ├─ Format Detection & Validation                           │
│  └─ Compression & Streaming for Large Files                 │
├─────────────────────────────────────────────────────────────┤
│  Filesystem Abstraction                                     │
│  ├─ Cross-Platform Path Handling                            │
│  ├─ Efficient I/O with Memory Management                    │
│  └─ File Watching & Change Detection                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                Local Filesystem                             │
│           (User's Development Environment)                  │
└─────────────────────────────────────────────────────────────┘
```

## 🏗️ Module Architecture Patterns

### Enhanced Sub-Module Organization Pattern
**Status**: Implemented in `security/permissions/` (2025-08-28)

**Pattern**: Large modules (>500 lines) should be refactored into focused sub-modules
**Implementation**: 
```
src/security/permissions/
├── mod.rs          - Module coordinator with architectural documentation
├── level.rs        - PermissionLevel enum hierarchy
├── rule.rs         - PathPermissionRule implementation  
├── evaluation.rs   - PermissionEvaluation framework
└── validator.rs    - PathPermissionValidator main engine
```

**Benefits Achieved**:
- **Single Responsibility**: Each module handles one focused concern
- **Enhanced Maintainability**: Clear separation improves debugging and development
- **Documentation Excellence**: Comprehensive docs with examples and security considerations
- **API Compatibility**: Zero breaking changes through proper re-exports
- **Developer Experience**: Easier onboarding and code navigation

**Documentation Standard**:
- Module-level docs with ASCII architecture diagrams
- Type-level docs with usage examples
- Method-level docs with security considerations
- Cross-references between related components

**Quality Gates**:
- All tests must pass during refactoring
- Zero compilation warnings maintained
- API compatibility verified through existing test suite
- Documentation coverage for all public APIs

### 1. Security-First Design Pattern ✅ **IMPLEMENTED**

**Pattern**: Every operation passes through security validation before execution
**Status**: Operational with 67% security framework complete (4/6 subtasks)
**Implementation**:
```rust
#[derive(Clone)]
pub struct SecurityManager {
    config: Arc<SecurityConfig>,
    policy_engine: Arc<PolicyEngine>,           // ✅ COMPLETE
    permission_validator: PathPermissionValidator, // ✅ COMPLETE  
    audit_logger: Arc<AuditLogger>,            // ✅ COMPLETE
}

impl SecurityManager {
    async fn validate_read_access(&self, path: &str) -> Result<(), SecurityError> {
        // ✅ IMPLEMENTED: Path-based permission validation with glob patterns
        // ✅ IMPLEMENTED: 5-level permission hierarchy (None → ReadOnly → ReadBasic → ReadWrite → Full)
        // ✅ IMPLEMENTED: Policy-based evaluation with deny-by-default security
        // ✅ IMPLEMENTED: Comprehensive audit logging with correlation IDs
    }
    
    async fn validate_write_access(&self, path: &str) -> Result<ApprovalToken, SecurityError> {
        // ✅ IMPLEMENTED: Advanced permission validation with rule priority
        // ✅ IMPLEMENTED: Policy integration with risk level assessment
        // 🔄 PENDING: Operation-type restrictions (Subtask 5.5)
        // 🔄 PENDING: Configuration validation (Subtask 5.7)
    }
}
```

**Security Framework Progress:**
- ✅ **PolicyEngine**: Real-time policy evaluation with TOML configuration
- ✅ **PathPermissionValidator**: Advanced glob pattern matching with inheritance  
- ✅ **AuditLogger**: Structured JSON logging with compliance records
- ✅ **Permission Hierarchy**: 5-level system with operation-specific validation
- 🔄 **Operation Restrictions**: Granular read/write/delete/create controls (next)
- 🔄 **Configuration Validation**: Startup validation with clear error messages

### 2. Modular Architecture Pattern ✅ **EVOLVED**

**Pattern**: Component-based design with clear separation of concerns
**Recent Evolution**: Permission system refactoring planned for improved maintainability
**Current Issue**: permissions.rs has grown to 541 lines, violating single responsibility

**Planned Refactoring** (security/permissions/ sub-module):
```rust
// Current: Single 541-line file
src/security/permissions.rs

// Target: Focused sub-modules with comprehensive documentation
src/security/permissions/
├── mod.rs           // Architectural overview + re-exports (§4.3 compliance)
├── level.rs         // PermissionLevel hierarchy (~120 lines)  
├── rule.rs          // PathPermissionRule matching (~180 lines)
├── evaluation.rs    // PermissionEvaluation results (~60 lines)
└── validator.rs     // PathPermissionValidator engine (~230 lines)
```

**Documentation Strategy**:
- **Module-level**: Architectural diagrams, quick start guides, security considerations
- **Type-level**: Purpose, examples, invariants, performance characteristics  
- **Method-level**: Parameters, return values, side effects, time complexity
- **Integration**: Cross-references, usage patterns, best practices

### 3. Configuration-Driven Security Policy Pattern ✅ **IMPLEMENTED**

**Pattern**: Critical operations require explicit human approval with context
**Implementation**:
```rust
pub struct ApprovalWorkflow {
    terminal_interface: TerminalInterface,
    approval_cache: Arc<Mutex<HashMap<OperationHash, ApprovalDecision>>>,
}

#[derive(Debug)]
pub struct ApprovalRequest {
    operation_type: OperationType,
    path: PathBuf,
    content_preview: String,
    estimated_impact: ImpactAssessment,
    security_context: SecurityContext,
}

impl ApprovalWorkflow {
    async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalDecision, ApprovalError> {
        // 1. Display operation context and preview
        // 2. Present clear approve/deny options
        // 3. Cache decision for similar operations
        // 4. Log approval decision with reasoning
    }
}
```

### 3. Binary Processing Strategy Pattern

**Pattern**: Intelligent format detection with specialized processing pipelines
**Implementation**:
```rust
pub trait BinaryProcessor {
    async fn process(&self, data: &[u8], options: ProcessingOptions) -> Result<ProcessedContent, ProcessingError>;
    fn supported_formats(&self) -> Vec<String>;
}

pub struct ImageProcessor;
impl BinaryProcessor for ImageProcessor {
    async fn process(&self, data: &[u8], options: ProcessingOptions) -> Result<ProcessedContent, ProcessingError> {
        // 1. Format detection via magic numbers
        // 2. EXIF metadata extraction
        // 3. Resize/thumbnail generation if requested
        // 4. Format conversion with quality control
    }
}

pub struct PdfProcessor;
impl BinaryProcessor for PdfProcessor {
    async fn process(&self, data: &[u8], options: ProcessingOptions) -> Result<ProcessedContent, ProcessingError> {
        // 1. PDF structure validation
        // 2. Text extraction with page granularity
        // 3. Embedded image extraction
        // 4. Metadata and security analysis
    }
}
```

### 4. Streaming Architecture Pattern

**Pattern**: Memory-efficient handling of large files through chunked processing
**Implementation**:
```rust
pub struct StreamingReader {
    file: tokio::fs::File,
    buffer_size: usize,
    compression: Option<CompressionType>,
}

impl StreamingReader {
    async fn read_chunks<F>(&mut self, mut processor: F) -> Result<(), StreamingError>
    where
        F: FnMut(Chunk) -> Result<ProcessingAction, ProcessingError>,
    {
        let mut buffer = vec![0; self.buffer_size];
        loop {
            let bytes_read = self.file.read(&mut buffer).await?;
            if bytes_read == 0 { break; }
            
            let chunk = Chunk::new(&buffer[..bytes_read]);
            match processor(chunk)? {
                ProcessingAction::Continue => continue,
                ProcessingAction::Stop => break,
                ProcessingAction::SkipToEnd => {
                    self.file.seek(SeekFrom::End(0)).await?;
                    break;
                }
            }
        }
        Ok(())
    }
}
```

## Design Patterns in Use

### 1. Strategy Pattern for File Processing
Different file types require different processing strategies, implemented through the `BinaryProcessor` trait hierarchy.

### 2. Chain of Responsibility for Security Validation
Security checks are applied in sequence: path validation → access control → approval workflow → audit logging.

### 3. Observer Pattern for Audit Logging
All operations emit events that are captured by the audit logging system for compliance and debugging.

### 4. Factory Pattern for Tool Creation
MCP tools are created through a factory pattern that handles registration, validation, and execution.

### 5. Command Pattern for Operation Execution
All filesystem operations are encapsulated as commands that can be queued, validated, and executed with rollback capabilities.

## Error Handling Patterns

### Comprehensive Error Taxonomy
```rust
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("Security violation: {0}")]
    SecurityViolation(SecurityError),
    
    #[error("Operation denied: {0}")]
    OperationDenied(String),
    
    #[error("File too large: {size} bytes exceeds limit")]
    FileTooLarge { size: u64 },
    
    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Binary processing error: {0}")]
    BinaryProcessing(#[from] BinaryProcessingError),
}
```

### Error Recovery Strategies
- **Graceful Degradation**: Fall back to basic operations when advanced features fail
- **User Guidance**: Provide clear, actionable error messages with suggested fixes
- **Automatic Retry**: Retry transient errors with exponential backoff
- **Safe Rollback**: Undo partial operations that fail mid-execution

## Configuration Management Patterns

### Hierarchical Configuration Loading
```rust
pub struct ConfigLoader {
    sources: Vec<ConfigSource>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            sources: vec![
                ConfigSource::EnvironmentVariables,
                ConfigSource::ProjectConfig("./.airs-mcp-fs.toml"),
                ConfigSource::UserConfig("~/.config/airs-mcp-fs/config.toml"),
                ConfigSource::SystemConfig("/etc/airs-mcp-fs/config.toml"),
                ConfigSource::Defaults,
            ],
        }
    }
    
    pub async fn load(&self) -> Result<FsConfig, ConfigError> {
        // Merge configuration from all sources with proper precedence
    }
}
```

### Configuration-Driven Security Policies
```toml
[security]
allowed_read_paths = ["~/Documents/**", "~/Projects/**", "./**"]
allowed_write_paths = ["~/Documents/**", "~/Projects/**"]
forbidden_patterns = ["\\.env$", "\\.ssh/.*", ".*\\.key$"]
max_file_size_mb = 100
require_approval_for_writes = true
enable_threat_detection = true

[performance]
max_concurrent_operations = 10
buffer_size_kb = 64
cache_size_mb = 50
streaming_threshold_mb = 10
```

## Performance Optimization Patterns

### 1. Async-First Design
All I/O operations are async to prevent blocking the MCP message processing loop.

### 2. Connection Pooling
Reuse resources and connections to minimize setup overhead.

### 3. Intelligent Caching
Cache frequently accessed files and metadata with configurable TTL.

### 4. Memory Management
Use streaming for large files and careful buffer management to prevent memory bloat.

### 5. Lazy Loading
Load binary processing engines only when needed for specific file types.

## Integration Patterns with AIRS Ecosystem

### Shared Foundation Pattern
```rust
// Reuse AIRS MCP infrastructure
use airs_mcp::{
    client::McpClient,
    transport::StdioTransport,
    tools::{Tool, ToolRegistry},
    error_handling::McpError,
};

pub struct AirsMcpFs {
    mcp_client: Arc<McpClient<StdioTransport>>,
    tool_registry: ToolRegistry,
    // ... airs-mcp-fs specific components
}
```

### Configuration Consistency Pattern
Follow AIRS workspace patterns for configuration file structure, naming conventions, and loading hierarchies.

### Logging and Monitoring Pattern
Use shared logging formats and monitoring patterns established in the AIRS ecosystem for consistency and integration.

## Future Architecture Considerations

### Plugin Architecture
Design hooks for extending functionality through plugins (custom file processors, security validators, approval workflows).

### Multi-Client Support
Architecture supports multiple concurrent MCP clients with session isolation and resource sharing.

### Cloud Integration
Foundation for future cloud storage integration while maintaining the same security and approval patterns.

### Performance Monitoring
Built-in metrics collection and performance monitoring for optimization and debugging.

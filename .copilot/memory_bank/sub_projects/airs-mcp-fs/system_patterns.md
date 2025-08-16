# System Patterns: AIRS MCP-FS

**Updated:** 2025-08-16  
**Architecture Status:** Design Phase - Implementation Pending

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

## Core Technical Patterns

### 1. Security-First Design Pattern

**Pattern**: Every operation passes through security validation before execution
**Implementation**:
```rust
#[derive(Clone)]
pub struct SecurityManager {
    config: Arc<SecurityConfig>,
    approval_workflow: Arc<ApprovalWorkflow>,
    audit_logger: Arc<AuditLogger>,
}

impl SecurityManager {
    async fn validate_read_access(&self, path: &str) -> Result<(), SecurityError> {
        // 1. Path canonicalization and traversal prevention
        // 2. Allowlist/denylist pattern matching
        // 3. File size and type validation
        // 4. Audit logging
    }
    
    async fn validate_write_access(&self, path: &str) -> Result<ApprovalToken, SecurityError> {
        // 1. Security validation (same as read)
        // 2. Human approval workflow trigger
        // 3. Approval token generation for authorized operations
    }
}
```

### 2. Human-in-the-Loop Approval Pattern

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

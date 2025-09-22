# Security & Compliance

## **Security Framework**

### **Defense in Depth**
1. **Binary File Restriction**: Complete blocking of binary file operations for maximum security
2. **Input Validation**: Path sanitization and canonicalization
3. **Access Control**: Configurable allowlists and denylists
4. **Human Approval**: Interactive approval for sensitive operations
5. **Threat Detection**: Enhanced security monitoring with binary file rejection
6. **Audit Logging**: Comprehensive operation tracking
7. **Resource Limits**: Prevention of DoS through resource exhaustion

### **Configuration-Driven Security**
```toml
# ~/.config/airs-mcpserver-fs/security.toml
[security]
# Binary processing is completely disabled for security
binary_processing_disabled = true
text_only_mode = true

# Paths where read operations are allowed
allowed_read_paths = [
    "~/Documents/**",
    "~/Desktop/**", 
    "~/Projects/**",
    "./**"
]

# Paths where write operations are allowed
allowed_write_paths = [
    "~/Documents/**",
    "~/Desktop/**",
    "~/Projects/**"
]

# Regex patterns for forbidden files
forbidden_patterns = [
    "\\.env$",
    "\\.ssh/.*",
    ".*\\.key$",
    ".*password.*",
    "/etc/.*"
]

# File size limits (for text files only)
max_file_size_mb = 100

# Approval requirements
require_approval_for_writes = true
require_approval_for_deletes = true

# Enhanced threat detection with binary restriction
enable_threat_detection = true
block_binary_files = true
```

## **Binary File Security**

### **Complete Binary Restriction**
AIRS MCP-FS employs a security-first approach by completely disabling binary file processing:

- **Attack Surface Reduction**: Eliminates entire classes of binary-based security vulnerabilities
- **Memory Safety**: Prevents buffer overflows and memory corruption from binary parsing
- **Malware Prevention**: Blocks execution of potentially malicious binary content
- **Resource Protection**: Eliminates resource exhaustion from complex binary processing
- **Compliance Enhancement**: Provides clear security boundaries for enterprise deployments

### **Text-Only Operations**
All file operations are restricted to text-based content:
- Source code files (`.rs`, `.ts`, `.js`, `.py`, etc.)
- Configuration files (`.toml`, `.json`, `.yaml`, etc.)
- Documentation files (`.md`, `.txt`, `.rst`, etc.)
- Data files with text content (`.csv`, `.log`, etc.)

### **Binary File Detection**
The system uses multiple detection methods to identify and block binary files:
- File extension validation
- Content-based binary detection
- Magic number analysis
- Comprehensive audit logging of rejection events

## **Audit & Compliance**
```rust
#[derive(Serialize)]
pub struct AuditRecord {
    timestamp: DateTime<Utc>,
    operation: String,
    path: String,
    user_id: Option<String>,
    client_info: ClientInfo,
    result: OperationResult,
    approval_status: Option<ApprovalStatus>,
    security_context: SecurityContext,
}

impl AuditLogger {
    pub async fn log_operation(&self, record: AuditRecord) -> Result<(), AuditError> {
        // Write to structured log file
        let log_entry = serde_json::to_string(&record)?;
        self.log_writer.write_line(&log_entry).await?;
        
        // Send to monitoring system if configured
        if let Some(monitor) = &self.monitoring_client {
            monitor.send_audit_event(&record).await?;
        }
        
        Ok(())
    }
}
```


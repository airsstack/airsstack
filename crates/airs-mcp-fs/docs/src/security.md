# Security & Compliance

## **Security Framework**

### **Defense in Depth**
1. **Input Validation**: Path sanitization and canonicalization
2. **Access Control**: Configurable allowlists and denylists
3. **Human Approval**: Interactive approval for sensitive operations
4. **Threat Detection**: Basic malware and anomaly detection
5. **Audit Logging**: Comprehensive operation tracking
6. **Resource Limits**: Prevention of DoS through resource exhaustion

### **Configuration-Driven Security**
```toml
# ~/.config/airs-mcp-fs/security.toml
[security]
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

# File size limits
max_file_size_mb = 100
max_binary_size_mb = 50

# Approval requirements
require_approval_for_writes = true
require_approval_for_deletes = true
require_approval_for_binary_writes = true

# Threat detection
enable_threat_detection = true
scan_binary_files = true
```

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


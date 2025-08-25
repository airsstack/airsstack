# Security Framework Architecture

**Category**: Security  
**Complexity**: High  
**Last Updated**: 2025-08-22  
**Maintainer**: Core Development Team

## Overview
**What is this knowledge about?**

This document outlines the security-first architecture for airs-mcp-fs, focusing on the human-in-the-loop approval workflows, threat detection, and secure filesystem access patterns that differentiate this tool from simple filesystem bridges.

**Why this knowledge is important**: Security is the primary differentiator for airs-mcp-fs. The security framework determines user trust, enterprise adoption potential, and overall system safety.

**Who should read this**: Anyone implementing security features, designing approval workflows, or integrating filesystem operations.

## Context & Background
**When and why was this approach chosen?**

The security-first architecture was established during the project brief and requirements phase based on several critical factors:

**Key Security Requirements**:
- **Enterprise Trust**: Filesystem access requires high security for enterprise adoption
- **AI Safety**: Autonomous file modification needs human oversight
- **Compliance**: Audit trails required for regulated environments
- **Threat Protection**: Local filesystem access creates attack surface

**Alternative approaches considered**:
- **Full Automation**: AI direct filesystem access (rejected - too risky)
- **Read-Only Access**: No modification capabilities (rejected - limited value)
- **Assumption-Based Security**: Design approval systems based on theoretical risk models (rejected - high failure rate)
- **Data-Driven Security with Behavioral Logging**: Evidence-based security design (selected)

## Technical Details
**How does this work?**

### Core Security Principles

#### 1. Human-in-the-Loop Approval Workflows
```rust
pub enum ApprovalRequired {
    WriteFile { path: PathBuf, size: u64, content_preview: String },
    DeleteFile { path: PathBuf, backup_available: bool },
    CreateDirectory { path: PathBuf, recursive: bool },
    MoveFile { from: PathBuf, to: PathBuf },
}

pub enum ApprovalResponse {
    Approved,
    Denied { reason: String },
    ModifyRequest { suggested_path: PathBuf },
}
```

#### 2. Configurable Security Policies
```rust
pub struct SecurityPolicy {
    pub allowlist_paths: Vec<PathBuf>,
    pub denylist_paths: Vec<PathBuf>,
    pub require_approval_for: Vec<OperationType>,
    pub max_file_size: u64,
    pub allowed_extensions: Option<Vec<String>>,
    pub auto_approve_patterns: Vec<PathPattern>,
}
```

#### 3. Threat Detection Framework
```rust
pub enum ThreatLevel {
    Low,    // Normal operation
    Medium, // Suspicious patterns detected
    High,   // Likely malicious activity
    Critical, // Active threat detected
}

pub struct ThreatAssessment {
    pub level: ThreatLevel,
    pub indicators: Vec<ThreatIndicator>,
    pub recommended_action: SecurityAction,
    pub confidence: f32,
}
```

### Security Layers

#### Layer 0: Behavioral Learning & Analytics
- **User Behavior Logging**: Comprehensive tracking of interaction patterns
- **Privacy-Preserving Data Collection**: Anonymized behavioral data for security design
- **Pattern Recognition**: ML-based detection of normal vs anomalous behavior
- **Evidence-Based Policy**: Security rules derived from actual user data rather than assumptions

#### Layer 1: Path Validation
- **Canonicalization**: Resolve symlinks and relative paths
- **Sandbox Enforcement**: Prevent directory traversal attacks
- **Allowlist/Denylist**: Configurable path restrictions based on behavioral insights
- **Permission Checks**: Verify filesystem permissions

#### Layer 2: Content Analysis
- **File Type Validation**: Magic number verification vs extension
- **Size Limits**: Dynamic limits based on user behavior patterns
- **Content Scanning**: Basic malware signature detection
- **Encoding Validation**: Prevent encoding-based attacks

#### Layer 3: Adaptive Operation Approval
- **Behavioral Risk Assessment**: Risk scoring based on user patterns and context
- **Data-Driven Approval Thresholds**: Approval requirements based on actual user behavior data
- **Auto-Approval Rules**: Evidence-based patterns for safe operations
- **Human Review**: Interactive approval for operations outside normal patterns
- **Approval History**: Track patterns and learn from decisions
- **Emergency Override**: Admin bypass for critical situations

#### Layer 4: Audit & Monitoring
- **Comprehensive Logging**: All operations logged with context
- **Threat Correlation**: Pattern detection across operations
- **Compliance Reports**: Structured audit trails
- **Alert System**: Real-time notifications for suspicious activity

## Code Examples
**Practical implementation examples**

### Security Policy Configuration
```rust
impl SecurityPolicy {
    pub fn development_default() -> Self {
        Self {
            allowlist_paths: vec![
                PathBuf::from("./src"),
                PathBuf::from("./docs"),
                PathBuf::from("./tests"),
            ],
            denylist_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/var"),
                PathBuf::from("/usr"),
            ],
            require_approval_for: vec![
                OperationType::WriteFile,
                OperationType::DeleteFile,
                OperationType::CreateDirectory,
            ],
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: Some(vec![
                "rs".to_string(),
                "toml".to_string(),
                "md".to_string(),
                "json".to_string(),
            ]),
            auto_approve_patterns: vec![
                PathPattern::new("./target/**").expect("valid pattern"),
                PathPattern::new("./**/*.tmp").expect("valid pattern"),
            ],
        }
    }
}
```

### Threat Detection Implementation
```rust
pub fn assess_operation_threat(
    operation: &FilesystemOperation,
    context: &SecurityContext,
    history: &OperationHistory,
) -> ThreatAssessment {
    let mut indicators = Vec::new();
    let mut threat_level = ThreatLevel::Low;

    // Rapid successive operations
    if history.operations_in_last_minute() > 50 {
        indicators.push(ThreatIndicator::RapidOperations);
        threat_level = ThreatLevel::Medium;
    }

    // Suspicious file patterns
    if operation.path().extension().map_or(false, |ext| {
        ["exe", "bat", "sh", "scr"].contains(&ext.to_str().unwrap_or(""))
    }) {
        indicators.push(ThreatIndicator::ExecutableFile);
        threat_level = ThreatLevel::High;
    }

    // Directory traversal attempts
    if operation.path().to_string_lossy().contains("../") {
        indicators.push(ThreatIndicator::DirectoryTraversal);
        threat_level = ThreatLevel::Critical;
    }

    ThreatAssessment {
        level: threat_level,
        indicators,
        recommended_action: match threat_level {
            ThreatLevel::Low => SecurityAction::Allow,
            ThreatLevel::Medium => SecurityAction::RequireApproval,
            ThreatLevel::High => SecurityAction::RequestJustification,
            ThreatLevel::Critical => SecurityAction::Block,
        },
        confidence: calculate_confidence(&indicators),
    }
}
```

## Performance Characteristics
**How does this perform?**

- **Path Validation**: O(1) for allowlist/denylist checks
- **Content Analysis**: O(n) where n = file size (limited by max_file_size)
- **Threat Assessment**: O(1) for most indicators, O(n) for history analysis
- **Approval Workflow**: Human latency (seconds to minutes)

**Performance Targets**:
- Security validation: <10ms for most operations
- Threat assessment: <50ms including history analysis
- Audit logging: <5ms additional overhead per operation

## Trade-offs & Limitations
**What are the constraints and compromises?**

### Security vs Usability
- **Approval Fatigue**: Too many approvals reduce security effectiveness
- **Developer Friction**: Security checks slow development workflow
- **False Positives**: Overly sensitive threat detection disrupts normal operations

### Performance vs Security
- **Validation Overhead**: Security checks add latency to all operations
- **Memory Usage**: Threat detection requires operation history storage
- **CPU Impact**: Content analysis for large files can be expensive

### Configuration Complexity
- **Policy Management**: Complex security policies are hard to configure correctly
- **Context Sensitivity**: One-size-fits-all policies don't work for all projects
- **Evolution Needs**: Security requirements change as projects mature

## Dependencies
**What does this rely on?**

### Internal Dependencies
- MCP message routing for approval workflow integration
- Filesystem abstraction layer for secure path handling
- Configuration management for security policy storage

### External Dependencies
- `walkdir` for safe directory traversal
- `magic` for file type detection
- `sha2` for content integrity verification
- Platform-specific permission APIs

### Security Dependencies
- Local privilege escalation prevention
- Secure inter-process communication for approval UI
- Cryptographic libraries for audit trail integrity

## Testing Strategy
**How is this tested?**

### Security Testing Approach
```rust
#[test]
fn test_directory_traversal_prevention() {
    let policy = SecurityPolicy::restrictive_default();
    let operation = FilesystemOperation::ReadFile {
        path: PathBuf::from("../../../etc/passwd"),
    };
    
    let result = validate_operation(&operation, &policy);
    assert!(matches!(result, Err(SecurityError::PathTraversalAttempt)));
}

#[test]
fn test_threat_detection_rapid_operations() {
    let mut history = OperationHistory::new();
    
    // Simulate rapid-fire operations
    for i in 0..100 {
        history.record_operation(FilesystemOperation::ReadFile {
            path: PathBuf::from(format!("file_{}.txt", i)),
        });
    }
    
    let assessment = assess_operation_threat(&next_operation, &context, &history);
    assert!(assessment.level >= ThreatLevel::Medium);
}
```

### Integration Testing
- End-to-end approval workflow testing
- Security policy enforcement validation
- Threat detection accuracy measurement
- Performance impact assessment

## Common Pitfalls
**What should developers watch out for?**

### Security Implementation Mistakes
- **TOCTOU Issues**: Time-of-check vs time-of-use race conditions
- **Path Canonicalization**: Failing to resolve symlinks before validation
- **Approval Bypass**: Implementing "convenience" bypasses that create vulnerabilities
- **Audit Gaps**: Missing security events in audit logs

### Usability Problems
- **Approval Spam**: Requesting approval for obviously safe operations
- **Poor Error Messages**: Security errors that don't help users understand what went wrong
- **Inflexible Policies**: Hard-coded security rules that don't adapt to project needs

### Performance Issues
- **Synchronous Validation**: Blocking operations during security checks
- **Excessive Logging**: Audit logs that impact performance
- **Memory Leaks**: Unbounded operation history storage

## Related Knowledge
**What else should I read?**

### Architecture Documents
- `docs/knowledges/architecture/filesystem-operations-design.md` (to be created)
- `docs/knowledges/architecture/mcp-integration-patterns.md` (to be created)

### Pattern Documents
- Human-computer interaction patterns for approval workflows
- Audit logging and compliance patterns
- Threat detection and response patterns

### Domain Knowledge
- Filesystem security best practices
- MCP security considerations
- Enterprise compliance requirements

## Evolution History

### Version 1.0 (Foundation - 2025-08-22)
- **Initial Design**: Security-first architecture established
- **Key Principles**: Human-in-the-loop, configurable policies, comprehensive audit
- **Status**: Design phase - implementation pending

### Future Enhancements Planned
- **Machine Learning**: AI-assisted threat detection based on usage patterns
- **Advanced Policies**: Context-aware security policies that adapt to project type
- **Integration**: Enterprise SSO and access control system integration
- **Automation**: Smart auto-approval based on historical decisions

### Security Evolution Priorities
1. **Implementation**: Basic security framework with core protections
2. **Refinement**: Policy tuning based on real-world usage
3. **Intelligence**: ML-enhanced threat detection and approval assistance
4. **Enterprise**: Advanced compliance and audit capabilities

## Implementation Notes

### Critical Security Requirements
- All filesystem operations MUST go through security validation
- Human approval workflows MUST be interruptible and resumable
- Audit logs MUST be tamper-evident and integrity-protected
- Threat detection MUST fail-safe (block on detection failure)

### Development Priorities
1. **Core Security**: Path validation and basic threat detection
2. **Approval Workflow**: Human-in-the-loop implementation
3. **Policy Engine**: Configurable security policy framework
4. **Audit System**: Comprehensive operation logging

This security framework represents the foundation of airs-mcp-fs trustworthiness and enterprise viability.

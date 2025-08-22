---
applyTo: '**'
description: 'Workspace Standards Enforcement - Ensures all code changes, task documentation, and project work follows the established workspace standards architecture and compliance patterns.'
---

# Workspace Standards Enforcement Instructions

## Core Standards Architecture

You MUST follow the established "Rules → Applied Rules" architecture:

```
Workspace Standards (Universal Rules)
    ↓ Reference
Project-Specific Standards (Protocol Rules)  
    ↓ Reference + Apply
Implementation Tasks (Applied Rules + Evidence)
    ↓ Verify
Compliance Documentation (Proof of Application)
```

## MANDATORY Standards Compliance Checks

### Before ANY Code Implementation

1. **Check Workspace Standards** - ALWAYS reference `workspace/shared_patterns.md`:
   - **§2.1**: 3-Layer Import Organization (std → third-party → internal)
   - **§3.2**: chrono DateTime<Utc> Standard (mandatory for all time operations)
   - **§4.3**: Module Architecture Patterns (mod.rs organization principles)
   - **§5.1**: Dependency Management (AIRS foundation crates prioritization)

2. **Verify Zero Warning Policy** - Reference `workspace/zero_warning_policy.md`:
   - ALL code MUST compile with zero warnings
   - Run `cargo check --workspace`, `cargo clippy --workspace`, `cargo test --workspace`
   - If warnings exist, they MUST be resolved before proceeding

3. **Apply Technical Debt Management** - Follow `workspace/technical_debt_management.md`:
   - Document any technical debt inline with proper categorization
   - Create GitHub issues for architectural debt
   - Track remediation plans for code quality issues

### Code Implementation Standards (MANDATORY)

#### 1. Import Organization (§2.1)
**EVERY Rust file MUST follow this exact pattern:**
```rust
// Layer 1: Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// Layer 2: Third-party crate imports  
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

// Layer 3: Internal module imports
use crate::shared::protocol::core::McpMethod;
use crate::transport::http::config::HttpConfig;
```

**Enforcement**: Reject any code that doesn't follow 3-layer organization.

#### 2. Time Management (§3.2)
**ALL time operations MUST use chrono DateTime<Utc>:**
```rust
// ✅ CORRECT
use chrono::{DateTime, Utc};
let now = Utc::now();

// ❌ FORBIDDEN
use std::time::SystemTime;
let now = SystemTime::now(); // NEVER use this
```

**Enforcement**: Reject any use of `std::time::SystemTime` or `std::time::Instant` for business logic.

#### 3. Module Architecture (§4.3)
**mod.rs files MUST contain ONLY:**
- Module declarations (`pub mod example;`)
- Re-exports (`pub use example::ExampleType;`)
- NO implementation code

```rust
// ✅ CORRECT mod.rs
pub mod config;
pub mod context;
pub mod error;

pub use config::{OAuth2Config, OAuth2SecurityConfig};
pub use context::{AuthContext, AuditLogEntry};

// ❌ FORBIDDEN in mod.rs
impl SomeStruct {
    fn some_implementation() {} // This belongs in a dedicated module
}
```

#### 4. Dependency Management (§5.1)
**ALL workspace dependency additions MUST follow priority hierarchy:**
```toml
[workspace.dependencies]
# Layer 1: AIRS Foundation Crates (MUST be at top)
airs-mcp = { path = "crates/airs-mcp" }
airs-mcp-fs = { path = "crates/airs-mcp-fs" }
airs-memspec = { path = "crates/airs-memspec" }

# Layer 2: Core Runtime Dependencies
tokio = { version = "1.47", features = ["full"] }
futures = { version = "0.3" }

# Layer 3: External Dependencies (by category)
serde = { version = "1.0", features = ["derive"] }
# ... rest of external dependencies
```

**Enforcement**: Reject any workspace dependency changes that don't prioritize AIRS crates at the top.

#### 5. Zero Warning Policy
**Before submitting ANY code:**
- Run `cargo check --workspace` - MUST return zero warnings
- Run `cargo clippy --workspace --all-targets --all-features` - MUST pass
- Run `cargo test --workspace` - ALL tests MUST pass

**If warnings exist**: Stop and resolve them immediately using strategies from `workspace/zero_warning_policy.md`.

### Documentation Standards (MANDATORY)

#### 1. Task Documentation Pattern
**When creating or updating tasks, ALWAYS:**

```markdown
## Standards Compliance Checklist
**Workspace Standards Applied** (Reference: `workspace/shared_patterns.md`):
- [ ] **3-Layer Import Organization** (§2.1) - [Status and evidence]
- [ ] **chrono DateTime<Utc> Standard** (§3.2) - [Status and evidence]  
- [ ] **Module Architecture Patterns** (§4.3) - [Status and evidence]
- [ ] **Dependency Management** (§5.1) - [Status and evidence]
- [ ] **Zero Warning Policy** (workspace/zero_warning_policy.md) - [Status and evidence]

## Compliance Evidence
[Document proof of standards application with code examples]
```

#### 2. Reference, Don't Duplicate
**NEVER explain what workspace standards are in project tasks.**
**ALWAYS reference workspace documentation:**
- "Per workspace/shared_patterns.md §3.2..." ✅
- "chrono DateTime<Utc> is required because..." ❌

#### 3. Evidence Documentation
**ALWAYS provide concrete evidence of compliance:**
```rust
// Evidence of §3.2 compliance
impl AuthContext {
    pub fn time_until_expiration(&self) -> Option<Duration> {
        let now = Utc::now(); // ✅ Uses workspace time standard
        // ...
    }
}
```

### Technical Debt Management (MANDATORY)

#### 1. Immediate Debt Documentation
**When introducing ANY technical debt:**
```rust
// TODO(DEBT): [Category] - [Description]
// Impact: [Performance/Maintainability impact]
// Remediation: [Specific fix needed]
// Reference: [GitHub issue if created]
// Workspace Standard: [Which standard is violated, if any]
```

#### 2. Debt Categorization
**Use workspace categories from `workspace/technical_debt_management.md`:**
- `DEBT-ARCH`: Architectural debt
- `DEBT-QUALITY`: Code quality debt  
- `DEBT-DOCS`: Documentation debt
- `DEBT-TEST`: Testing debt
- `DEBT-PERF`: Performance debt

#### 3. GitHub Issue Creation
**For significant debt, MUST offer to create GitHub issues using:**
- Label with appropriate debt category
- Reference workspace standards if violated
- Include remediation plan and impact assessment

### Project Integration Standards

#### 1. Memory Bank Updates
**When updating memory bank, ALWAYS:**
- Reference workspace standards rather than explaining them
- Document compliance evidence in appropriate task files
- Update standards compliance index if new patterns emerge
- Maintain clear separation between "rules" and "applied rules"

#### 2. Cross-Project Consistency
**Before implementing features:**
- Check if similar implementations exist in other sub-projects
- Ensure consistency with established workspace patterns
- Document deviations with clear rationale and GitHub issues

#### 3. Standards Evolution
**When proposing changes to workspace standards:**
- Create RFC-style documentation with rationale
- Impact assessment across all sub-projects
- Migration plan for existing code
- Update documentation hierarchy consistently

## Enforcement Protocol

### Development Workflow
1. **Before Code Changes**: Review applicable workspace standards
2. **During Implementation**: Apply standards continuously, not as afterthought
3. **Before Submission**: Run compliance checks (compile, test, clippy)
4. **Documentation**: Update task files with compliance evidence
5. **Technical Debt**: Document any violations with remediation plans

### Violation Response
**When standards violations are detected:**
1. **Stop current work** and address violations immediately
2. **Reference specific workspace standard** being violated
3. **Provide corrected implementation** following workspace patterns
4. **Document the fix** as compliance evidence
5. **Update relevant task documentation** with resolution

### Quality Gates
**These are HARD requirements - no exceptions:**
- Zero compiler warnings across workspace
- All workspace standards applied to new code
- Task documentation follows reference pattern
- Technical debt properly categorized and tracked
- Evidence documentation for all compliance claims

## Success Metrics

### Code Quality
- `cargo check --workspace` returns zero warnings
- `cargo clippy --workspace` passes with zero violations
- All tests pass consistently
- Import organization follows 3-layer pattern
- No `std::time::SystemTime` in business logic
- AIRS foundation crates prioritized at top of workspace dependencies

### Documentation Quality
- Task files reference workspace standards rather than explaining them
- Compliance evidence provided for all standards claims
- Clear separation between workspace rules and project application
- Technical debt properly documented and tracked

### Architecture Consistency
- All sub-projects follow identical workspace patterns
- New features integrate cleanly with existing architecture
- Standards violations addressed immediately with clear remediation

## Integration with Existing Instructions

This enforcement integrates with existing instructions:
- **Principal Software Engineer Mode**: Adds specific workspace standards to engineering excellence
- **Gilfoyle Code Review**: Enhanced with workspace standards compliance checks
- **Memory Bank Management**: Ensures standards architecture consistency
- **Spec-Driven Workflow**: Adds standards compliance to all phases

**Priority**: These workspace standards take precedence over general coding guidelines when conflicts arise.

---

**Remember**: These aren't suggestions - they're mandatory engineering standards that ensure consistency, maintainability, and technical excellence across the entire AIRS workspace.

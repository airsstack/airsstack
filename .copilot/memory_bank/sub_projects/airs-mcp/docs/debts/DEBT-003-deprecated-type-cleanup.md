# DEBT-003: Deprecated HttpStreamableTransport Cleanup

**Status**: Active  
**Priority**: Medium  
**Category**: Code Quality  
**Created**: 2025-08-21  
**Updated**: 2025-08-21  
**Estimated Effort**: 4 hours

## Problem Description
**What is the technical debt?**
- HttpStreamableTransport type alias remains in codebase as deprecated compatibility layer
- Examples and documentation still reference the deprecated type
- Migration path exists but hasn't been completed across all usage
- Impact on current development velocity: Low (deprecated warnings only)
- Impact on code maintainability: Medium (deprecated code paths require maintenance)

## Context & Reason
**Why was this debt incurred?**
- Backward compatibility requirement during HTTP transport architecture refactoring
- Gradual migration strategy to avoid breaking existing integrations
- Time constraints during Phase 2 delivery prevented immediate cleanup
- Conservative approach to ensure stability during architectural changes

## Current Impact
**How does this debt affect the project today?**
- Development velocity impact: Low (generates compiler warnings but doesn't block development)
- Code complexity increase: Low (simple type alias, minimal complexity)
- Testing difficulty: Low (deprecated paths still tested)
- Performance implications: None (type alias has no runtime overhead)
- Security concerns: None identified

## Future Risk
**What happens if this debt is not addressed?**
- Projected impact on future development: Low (deprecated code will eventually be removed)
- Risk of becoming impossible to fix: Very Low (straightforward find-and-replace operation)
- Potential for cascading problems: Low (well-contained change scope)

## Remediation Plan
**How should this debt be resolved?**
1. **Audit usage of deprecated type** (1 hour)
   - Search codebase for HttpStreamableTransport usage
   - Identify examples, tests, and documentation references
   - Create migration checklist for each usage

2. **Update examples and documentation** (2 hours)
   - Replace HttpStreamableTransport with HttpClientTransport in examples
   - Update documentation to use current type names
   - Update inline code comments and type annotations

3. **Remove deprecated type alias** (1 hour)
   - Remove HttpStreamableTransport type alias from http/mod.rs
   - Verify all tests pass with deprecated type removed
   - Update any remaining internal usage

## Code References
**Where is this debt located?**
- `crates/airs-mcp/src/transport/http/mod.rs` (line with type alias declaration)
- `crates/airs-mcp/examples/` (multiple example files potentially using deprecated type)
- `crates/airs-mcp/docs/` (documentation potentially referencing old type)
- `crates/airs-mcp/tests/` (test files potentially using deprecated type)

## Related Issues
**Links to related tracking**
- GitHub Issues: TBD (simple cleanup task)
- Related technical debt items: None (standalone cleanup)
- Task management references: Can be completed during any maintenance cycle

## Notes
**Additional context**
- Previous remediation attempts: None (deprecation warnings added recently)
- Alternative approaches considered:
  - Keep deprecated type indefinitely (rejected: accumulates technical debt)
  - Immediate breaking change (rejected: affects backward compatibility)
- Stakeholder discussions: Cleanup approved, timing flexible based on development cycles

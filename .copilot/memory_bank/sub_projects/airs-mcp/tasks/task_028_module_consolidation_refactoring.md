# [TASK-028] - Module Consolidation Refactoring

**Status:** pending  
**Added:** 2025-09-07  
**Updated:** 2025-09-07  
**Priority:** High  
**Category:** Architecture Refactoring  
**Estimated Effort:** 8-12 hours

## Original Request

During architecture review of the `airs-mcp` crate, significant functional overlap was discovered between three modules:
- `src/base/jsonrpc` (JSON-RPC 2.0 foundation)  
- `src/shared/protocol` (MCP protocol layer)
- `src/transport/mcp` (MCP-compliant transport)

This creates code duplication, API confusion, and maintenance burden that violates workspace standards for clean architecture and minimal dependencies.

## Thought Process

### **Architecture Analysis Findings**

**Code Duplication Evidence:**
- Identical serialization methods in `base/jsonrpc/message.rs` and `transport/mcp/message.rs`
- Multiple import paths for essentially the same functionality  
- Compatibility layer (`transport/mcp/compat.rs`) indicating design problems

**Workspace Standards Violations:**
- **Zero Warning Policy**: Code duplication creates maintenance warnings
- **Minimal Dependencies**: Three overlapping modules violate efficiency principles
- **Clear Architecture**: Overlapping responsibilities create confusion

**User Experience Impact:**
- Import path confusion ("which module should I use?")
- Multiple APIs for identical functionality
- "Legacy" vs "modern" patterns causing friction

### **Decision Analysis**

The overlap analysis led to **ADR-010: Module Consolidation - Protocol Architecture Unification**, which recommends consolidating all three modules into a single `src/protocol/` module that:

1. **Preserves the best aspects** of each module
2. **Eliminates duplication** while maintaining functionality
3. **Simplifies the API** with a single import path
4. **Follows workspace standards** and user preferences

## Implementation Plan

### **Phase 1: Foundation Setup** 
- [ ] Create new `src/protocol/` module structure
- [ ] Set up module organization following workspace standards
- [ ] Prepare migration staging area

### **Phase 2: Core Migration**
- [ ] **From `base/jsonrpc` → `protocol/message.rs`**
  - Preserve trait-based design (well-architected)
  - Preserve `JsonRpcMessage` trait, `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcNotification`
  - Preserve `RequestId` enum and all serialization methods  
  - Preserve zero-copy optimizations
- [ ] **From `shared/protocol` → `protocol/types.rs` + `protocol/message.rs`**
  - Migrate MCP-specific types (Uri, ProtocolVersion, ClientInfo, etc.) to `types.rs`
  - Migrate MCP message structures (InitializeRequest, etc.) to `message.rs`
  - Preserve type safety and validation patterns
- [ ] **From `transport/mcp` → `protocol/transport.rs`**
  - Migrate transport abstractions (`Transport` trait, `MessageHandler`, etc.)
  - Discard duplicate JsonRpcMessage struct (keep trait-based approach)
  - Remove compatibility layer (no longer needed)

### **Phase 3: Integration & Cleanup**
- [ ] Update all import statements across codebase
- [ ] Update public API in `lib.rs` with single import path
- [ ] Update examples to use new module structure
- [ ] Delete original three modules
- [ ] Update documentation to reflect new structure

### **Phase 4: Validation**
- [ ] Ensure all tests pass during and after migration
- [ ] Maintain zero compilation warnings
- [ ] Performance benchmarking to verify no degradation
- [ ] Update README and documentation

## Progress Tracking

**Overall Status:** pending - 0% (0/4 phases complete)

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 28.1 | Foundation Setup - Create new protocol module structure | not_started | 2025-09-07 | Ready for implementation |
| 28.2 | Core Migration - Migrate from three modules to unified structure | not_started | 2025-09-07 | Depends on 28.1 completion |
| 28.3 | Integration & Cleanup - Update imports and delete old modules | not_started | 2025-09-07 | Depends on 28.2 completion |
| 28.4 | Validation - Testing and performance verification | not_started | 2025-09-07 | Final validation step |

## Progress Log

### 2025-09-07
- Created TASK-028 based on comprehensive architecture analysis
- Links established to ADR-010 and DEBT-ARCH-004 documentation
- Implementation plan structured in 4 phases with clear dependencies
- Ready for development team to begin implementation

## Success Criteria

### **Technical Criteria**
1. ✅ Single `src/protocol/` module handles all JSON-RPC and MCP functionality
2. ✅ Zero code duplication in serialization methods  
3. ✅ Simplified public API with single import path
4. ✅ All existing tests continue to pass
5. ✅ Zero compilation warnings maintained
6. ✅ Performance characteristics preserved or improved

### **Quality Criteria**
1. ✅ Examples and documentation updated
2. ✅ Workspace standards compliance maintained
3. ✅ User preference compliance (generic types over `dyn`)
4. ✅ Clean migration with no breaking changes to public API

## Risk Assessment

### **Risk: Breaking Changes**
- **Impact**: High (affects all users)
- **Mitigation**: Maintain public API compatibility through careful re-exports in `lib.rs`

### **Risk: Large Refactoring Scope**  
- **Impact**: Medium (development time)
- **Mitigation**: Phase-by-phase migration with continuous testing

### **Risk: Import Path Changes**
- **Impact**: Low (internal reorganization)
- **Mitigation**: Update all examples and provide clear documentation

## Related Documentation

### **Architecture Decision Record**
- **ADR-010**: Module Consolidation - Protocol Architecture Unification
- **Location**: `docs/adr/ADR-010-module-consolidation-protocol-architecture.md`
- **Status**: Accepted (2025-09-07)
- **Decision**: Consolidate three overlapping modules into single `src/protocol/` module

### **Technical Debt Record**
- **DEBT-ARCH-004**: Module Consolidation Refactoring  
- **Location**: `docs/debts/DEBT-ARCH-004-module-consolidation-refactoring.md`
- **Priority**: High
- **Impact**: Maintenance Burden, Code Duplication, API Confusion

### **Evidence Documentation**
- **Code Analysis**: Line-by-line evidence in ADR-010 Context section
- **Usage Patterns**: Import confusion documented in examples analysis  
- **Workspace Compliance**: Standards violations detailed in DEBT-ARCH-004

## Dependencies

### **Prerequisites**
- ✅ Architecture analysis complete (ADR-010 approved)
- ✅ Technical debt documented (DEBT-ARCH-004 created)
- ✅ Implementation plan finalized

### **External Dependencies**
- No external dependencies (internal refactoring only)
- All changes are backward-compatible through public API re-exports

## Implementation Notes

### **Preservation Requirements**
- **Maintain all existing functionality** - no feature removal
- **Preserve performance characteristics** - maintain 8.5+ GiB/s throughput
- **Keep comprehensive test coverage** - all 345+ tests must continue passing
- **Maintain API compatibility** - users should not need code changes

### **Quality Gates**
- **Zero compilation warnings** throughout migration process
- **All tests pass** at each phase completion
- **Documentation updated** to reflect new structure  
- **Examples validated** with new import patterns

### **Migration Strategy**
The migration follows a **preserve-and-enhance** strategy:
1. **Keep the good parts** - trait-based design from `base/jsonrpc`
2. **Enhance with MCP extensions** - types and messages from `shared/protocol`
3. **Add transport abstractions** - clean interfaces from `transport/mcp`
4. **Eliminate duplication** - remove redundant implementations

## Acceptance Criteria

### **Functional Requirements**
- [ ] All existing functionality preserved
- [ ] Single import path for all protocol functionality
- [ ] Zero code duplication in core functionality
- [ ] Backward compatibility maintained

### **Quality Requirements**  
- [ ] All tests pass (current: 345+ tests)
- [ ] Zero compilation warnings
- [ ] Documentation updated and accurate
- [ ] Examples work with new structure

### **Performance Requirements**
- [ ] Maintain current performance characteristics (8.5+ GiB/s)
- [ ] No regression in memory usage
- [ ] No increase in binary size from consolidation

### **Compliance Requirements**
- [ ] Workspace standards adherence maintained
- [ ] User preferences respected (generic types over `dyn`)
- [ ] Clean architecture principles followed

---

**Next Action**: Begin Phase 1 (Foundation Setup) by creating the new `src/protocol/` module structure following the plan outlined in ADR-010.

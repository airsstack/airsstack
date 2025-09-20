# DEBT-005: JSON-RPC Protocol Testing and Configuration Enhancements

**Type**: Architecture Enhancement  
**Priority**: Medium  
**Impact**: Developer Experience, Testing Infrastructure  
**Effort**: T-shirt Size M  
**Created**: 2025-09-20  
**Status**: Identified  
**Category**: Configuration, Testing Infrastructure

## Issue Description

JSON-RPC edge case testing revealed several configuration and testing infrastructure gaps that limit developer experience and production configurability.

## Technical Context

**Root Cause**: Limited configuration options and test infrastructure challenges  
**Current State**: Core JSON-RPC validation is excellent, but surrounding infrastructure needs enhancement  
**Impact**: Makes protocol testing difficult and limits production deployment flexibility

## Specific Problems

### 1. OAuth2 Testing Integration Challenge
**Problem**: OAuth2 middleware blocks JSON-RPC protocol validation testing  
**Current Workaround**: Must use `/dev/tokens` endpoint for test authentication  
**Impact**: Complex test setup, difficult to isolate protocol validation from authentication

### 2. Request Size Limits Not Configurable
**Problem**: No configurable request size limits at HTTP transport layer  
**Current Behavior**: Large requests (1MB+) are processed without limits  
**Impact**: Potential DoS vulnerability, no control over resource usage

### 3. Parameter Validation Inconsistency
**Problem**: Different methods handle invalid parameters differently  
**Examples**:
- `tools/call` with empty name returns success with `isError: true`
- `resources/list` ignores invalid parameter types entirely
**Impact**: Inconsistent API behavior, unclear protocol compliance

## Proposed Solutions

### High Priority

#### 1. Test Mode Configuration
```rust
pub struct HttpAuthConfig {
    pub oauth2_strategy: Option<OAuth2StrategyAdapter>,
    pub test_mode: bool, // Bypass auth for protocol testing
}
```

#### 2. Request Size Limits
```rust
pub struct HttpTransportConfig {
    pub max_request_size: Option<usize>, // Default: 1MB
    pub max_json_depth: Option<usize>,   // Default: 100
}
```

### Medium Priority

#### 3. Strict Parameter Validation Mode
```rust
pub struct McpServerConfig {
    pub strict_parameter_validation: bool, // Default: false
}
```

## Implementation Plan

### Phase 1: Configuration Infrastructure
1. Add test mode to `HttpAuthConfig`
2. Implement request size limits in HTTP transport
3. Create configuration validation

### Phase 2: Parameter Validation Enhancement
1. Audit all MCP method parameter handling
2. Implement strict validation mode
3. Add comprehensive parameter validation tests

### Phase 3: Documentation and Tools
1. Document configuration options
2. Create JSON-RPC protocol testing guide
3. Add developer tools for protocol validation

## Benefits

**Developer Experience**:
- Easier protocol validation testing
- Clear configuration options
- Consistent API behavior

**Production Readiness**:
- DoS protection via request size limits
- Configurable security levels
- Better resource management

**Maintenance**:
- Isolated protocol testing
- Clear behavior documentation
- Reduced testing complexity

## Related Technical Debt

- None directly related (this is new debt identified from testing)

## Success Criteria

1. **Test Infrastructure**: JSON-RPC protocol tests can run without OAuth2 complexity
2. **Configuration**: Request size limits configurable and enforced
3. **Consistency**: Parameter validation behavior documented and consistent
4. **Documentation**: Clear guide for protocol testing and configuration options

## Effort Estimation

**T-shirt Size**: M (Medium)  
**Estimated Time**: 2-3 weeks  
**Dependencies**: None (isolated improvements)  
**Risk Level**: Low (additive changes, no breaking changes required)

## Migration Strategy

**Backward Compatibility**: All changes are additive with sensible defaults  
**Rollout Plan**: Gradual feature rollout with opt-in configuration  
**Testing Strategy**: Comprehensive integration tests for each configuration option

---

**Related Documents**:
- [KNOWLEDGE-018: JSON-RPC Edge Case Testing Findings](../knowledges/KNOWLEDGE-018-jsonrpc-edge-case-testing-findings.md)
- JSON-RPC 2.0 specification compliance documentation
- HTTP OAuth2 Server Integration implementation analysis

**Next Actions**:
1. Prioritize specific improvements based on business needs
2. Create GitHub issues for high-priority configuration options
3. Plan implementation timeline with development team
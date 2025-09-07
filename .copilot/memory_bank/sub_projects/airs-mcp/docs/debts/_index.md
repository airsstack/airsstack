# Technical Debt Registry - airs-mcp

**Last Updated**: 2025-09-07  
**Total Debt Items**: 6  
**High Priority Items**: 4  
**In Progress**: 0

## Summary by Category

### Architecture Debt
- **Critical**: 0 items
- **High**: 3 items  
- **Medium**: 0 items
- **Low**: 0 items

### Performance Debt
- **Critical**: 0 items
- **High**: 0 items
- **Medium**: 1 items
- **Low**: 0 items

### Security Debt
- **Critical**: 0 items
- **High**: 0 items
- **Medium**: 0 items
- **Low**: 0 items

### Code Quality Debt
- **Critical**: 0 items
- **High**: 0 items
- **Medium**: 1 items 
- **Low**: 0 items

## Active Debt (Requires Attention)

### High Priority
| ID | Description | Category | Created | Estimated Effort | Owner | GitHub Issue |
|----|-------------|----------|---------|------------------|-------|--------------|
| DEBT-001 | Correlation error handling inconsistency | Architecture | 2025-08-21 | 2-3 days | Core Team | TBD |
| DEBT-004 | HttpServerTransport adapter pattern incomplete | Architecture | 2025-09-01 | 3 weeks | Core Team | TBD |

### Medium Priority
| ID | Description | Category | Created | Estimated Effort | Owner | GitHub Issue |  
|----|-------------|----------|---------|------------------|-------|--------------|
| DEBT-001 | HTTP Transport Trait Impedance Mismatch | Architecture | 2025-09-01 | 2-3 weeks | Core Team | TBD |
| DEBT-002 | HTTP transport performance optimization | Performance | 2025-08-21 | 1-2 days | Core Team | TBD |
| DEBT-003 | Deprecated HttpStreamableTransport cleanup | Code Quality | 2025-08-21 | 4 hours | Core Team | TBD |

## Resolved Debt

| ID | Description | Category | Created | Resolved | Resolution |
|----|-------------|----------|---------|----------|------------|
| - | No resolved debt items yet | - | - | - | - |

## Technical Debt Details

### DEBT-001: HTTP Transport Trait Impedance Mismatch  
**Reference**: `docs/debts/DEBT-001-http-transport-trait-impedance-mismatch.md`  
**Impact**: Complex debugging, limited HTTP feature extensibility, architectural confusion  
**Root Cause**: Fundamental semantic mismatch between Transport trait (single connection) and HTTP (multi-session)  
**Remediation**: 2-3 week architectural redesign with HTTP-native interface or multi-session Transport trait

### DEBT-004: HttpServerTransport Adapter Pattern Incomplete  
**Reference**: `docs/debts/DEBT-004-http-server-transport-adapter-incomplete.md`  
**Impact**: Blocks HTTP transport usage with McpServerBuilder, forces StdioTransport workarounds  
**Root Cause**: Incomplete adapter implementation between AxumHttpServer and Transport trait  
**Remediation**: 3-week phased implementation to complete adapter pattern

## In Progress

| ID | Description | Category | Started | Progress | Expected Completion | Owner |
|----|-------------|----------|---------|----------|-------------------|-------|
| - | No items currently in progress | - | - | - | - | - |

## Recently Resolved (Last 30 Days)

| ID | Description | Category | Resolved | Resolution Summary | 
|----|-------------|----------|----------|-------------------|
| - | No recently resolved debt items | - | - | - |

## Low Priority / Backlog

| ID | Description | Category | Created | Estimated Effort | Notes |
|----|-------------|----------|---------|------------------|-------|
| - | No low priority items currently tracked | - | - | - | - |

## Abandoned/Won't Fix

| ID | Description | Category | Abandoned | Reason |
|----|-------------|----------|-----------|--------|
| - | No abandoned debt items | - | - | - |

## Debt Trends

### Monthly Debt Creation/Resolution
- **This Month**: 3 created / 0 resolved
- **Last Month**: N/A (first month tracking)  
- **Trend**: Baseline establishment

### Average Resolution Time
- **Critical**: N/A (no critical items)
- **High**: N/A (no resolved high items yet)
- **Medium**: N/A (no resolved medium items yet)

## Action Items
- [ ] Create GitHub Issues for architecture debt items requiring cross-team coordination
- [ ] Plan remediation for DEBT-001 correlation error handling
- [ ] Schedule performance optimization sprint for DEBT-002
- [ ] Complete deprecated code cleanup for DEBT-003

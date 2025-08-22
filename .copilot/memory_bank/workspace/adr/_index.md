# Architecture Decision Record Index

This registry tracks all architectural decisions made at the workspace level that affect multiple sub-projects.

## Active ADRs

### ADR-002: AIRS Foundation Crate Dependency Prioritization
**Date:** 2025-08-22  
**Status:** Accepted  
**Impact:** High - Affects all workspace dependency management  
**Summary:** AIRS foundation crates must be prioritized at the top of workspace dependencies for clear hierarchy and improved developer understanding.

### ADR-001: Workspace Standards Architecture  
**Date:** 2025-08-16  
**Status:** Accepted  
**Impact:** High - Establishes foundational compliance framework  
**Summary:** Establishes the "Rules → Applied Rules" architecture for workspace standards enforcement across all sub-projects.

## Review Schedule

- **Quarterly Review**: Every 3 months (next: 2025-11-22)
- **Trigger Reviews**: Major architectural changes, new foundation crates, standards violations
- **Annual Assessment**: December 2025

## Guidelines

- All workspace-level architectural decisions MUST be documented as ADRs
- ADRs affecting multiple sub-projects require this workspace-level registry
- Sub-project specific ADRs are tracked in individual sub-project ADR indices
- Status values: Proposed, Accepted, Deprecated, Superseded

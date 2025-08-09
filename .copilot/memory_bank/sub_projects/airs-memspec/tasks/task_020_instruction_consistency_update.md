# [task_020] - Instruction Consistency Update

**Status:** in_progress  
**Added:** 2025-08-09  
**Updated:** 2025-08-09

## Original Request
Update custom instructions to resolve format inconsistencies discovered during validation analysis, ensuring instructions accurately reflect the sophisticated validation capabilities already implemented in airs-memspec.

## Thought Process
Critical analysis revealed that while airs-memspec implements exceptional validation features (status format standardization, stale detection, cross-project consistency), the custom instructions contain format conflicts and don't document these capabilities. This creates confusion and the exact issues we recently resolved, despite the tool being robust enough to handle them.

**Key Discovery**: airs-memspec implementation quality EXCEEDS the originally recommended validation features - the tool already handles format variations gracefully via fuzzy parsing and implements comprehensive validation that surpasses expectations.

## Implementation Plan
1. **Standardize Status Formats**: Update both Memory Bank and Multi-Project instructions to use consistent lowercase format matching tool reality
2. **Document Validation Features**: Add comprehensive documentation of validation capabilities already implemented
3. **Remove Duplications**: Clean up duplicate content in multi-project instructions
4. **Add Validation Checklist**: Create mandatory validation checklist reflecting current tool capabilities
5. **Update Formatting Rules**: Ensure all format rules match the sophisticated fuzzy parsing already implemented

## Progress Tracking

**Overall Status:** in_progress - 20%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 20.1 | Analysis documentation completion | complete | 2025-08-09 | Memory bank updated with validation findings |
| 20.2 | Standardize status format specifications | not_started | 2025-08-09 | Update both instruction files to use lowercase consistent format |
| 20.3 | Document existing validation features | not_started | 2025-08-09 | Add sections describing validation already implemented |
| 20.4 | Remove duplicate content | not_started | 2025-08-09 | Clean up multi-project instruction duplications |
| 20.5 | Add validation checklist | not_started | 2025-08-09 | Create mandatory checklist reflecting tool capabilities |
| 20.6 | Validation and testing | not_started | 2025-08-09 | Verify instruction updates work with existing tool |

## Progress Log

### 2025-08-09
- **TASK CREATED**: Critical instruction consistency analysis complete
- **Subtask 20.1 COMPLETED**: Memory bank updated with comprehensive validation findings
- **Context Switch**: Switched active context from airs-mcp to airs-memspec
- **Discovery Documented**: airs-memspec validation implementation exceeds original recommendations
- **Critical Findings**:
  - Status format standardization already perfect via fuzzy parsing
  - Comprehensive validation system already operational  
  - Automated issue detection already implemented
  - Instructions contain format conflicts that tool handles gracefully
- **Next Steps**: Begin systematic instruction file updates to match implementation reality

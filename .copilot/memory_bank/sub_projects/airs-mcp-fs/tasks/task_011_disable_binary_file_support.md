# [task_011] - disable_binary_file_support

**Status:** pending  
**Added:** 2025-08-30  
**Updated:** 2025-08-30

## Original Request
For the project: `airs-mcp-fs`, I think we need to add one more task, which is to disable support to any binary files, such images. Current implemented solutions, this MCP tool also support to read and manage binary files, I think we need to disable it since it's too dangerous too.

It's mean, we also need to remove any configuration files related with the binary files and by default when this tool try to reading or writing to a file or from a file, we need to check first if those files are binary or not, and by default we should reject the tool request

## Thought Process
This is a critical security hardening task that requires a comprehensive approach to binary file restrictions. The current implementation supports various binary file formats (images, PDFs) which poses security risks in an MCP context where binary files could potentially:

1. **Contain malicious payloads** - Images and PDFs can contain embedded malware
2. **Bypass security scanning** - Binary content is harder to analyze for threats
3. **Enable data exfiltration** - Large binary files could be used to transfer sensitive data
4. **Create resource exhaustion** - Binary processing is resource-intensive

**Key insight**: This aligns with the "deny by default" security principle already established in the security framework. We should extend the existing security validation to categorically reject binary file operations.

**Implementation approach**: 
- Leverage existing `FormatDetector` to identify binary files
- Extend security validation pipeline to check file types
- Remove binary processing configuration options
- Update error handling to provide clear rejection messages
- Maintain text file processing for legitimate use cases

## Implementation Plan
### 1. **Binary Detection Integration** (Priority: High)
- Integrate `FormatDetector` into security validation pipeline
- Add binary file detection to `validate_read_access()` and `validate_write_access()`
- Implement file type checking before any file operations

### 2. **Configuration Cleanup** (Priority: High)  
- Remove `BinaryConfig.enable_image_processing` and `enable_pdf_processing` fields
- Remove all binary processing configuration options from TOML files
- Update configuration validation to reject binary-related settings

### 3. **Security Policy Updates** (Priority: High)
- Add "binary_files_denied" policy to explicitly reject binary operations
- Update existing security policies to clarify text-only support
- Enhance error messages to explain binary file restrictions

### 4. **Code Removal** (Priority: Medium)
- Remove or disable binary processing modules (`binary/processor.rs`, `binary/format.rs`)
- Remove binary-specific tools and handlers
- Clean up binary processing tests and examples

### 5. **Documentation Updates** (Priority: Medium)
- Update README to clarify text-only file support
- Add security documentation explaining binary file restrictions
- Update troubleshooting guides for binary file rejection scenarios

### 6. **Testing & Validation** (Priority: High)
- Add comprehensive tests for binary file rejection
- Test various binary file types (images, PDFs, executables, archives)
- Validate that text files continue to work normally
- Performance testing to ensure detection doesn't impact text file operations

## Progress Tracking

**Overall Status:** not_started - 0%

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 11.1 | Binary file detection integration into security pipeline | not_started | 2025-08-30 | Extend existing security validation with FormatDetector |
| 11.2 | Remove binary processing configuration options | not_started | 2025-08-30 | Clean up BinaryConfig and TOML configuration files |
| 11.3 | Update security policies for binary file denial | not_started | 2025-08-30 | Add explicit binary rejection policies and error messages |
| 11.4 | Remove or disable binary processing code modules | not_started | 2025-08-30 | Clean up binary/ modules while preserving format detection |
| 11.5 | Comprehensive testing for binary file rejection | not_started | 2025-08-30 | Test rejection of various binary formats with clear error messages |
| 11.6 | Update documentation and examples | not_started | 2025-08-30 | Clarify text-only support in README and documentation |

## Standards Compliance Checklist
**Workspace Standards Applied** (Reference: `workspace/shared_patterns.md`):
- [ ] **3-Layer Import Organization** (§2.1) - Will apply to all code changes
- [ ] **chrono DateTime<Utc> Standard** (§3.2) - Will apply if time operations needed  
- [ ] **Module Architecture Patterns** (§4.3) - Will maintain clean mod.rs organization
- [ ] **Dependency Management** (§5.1) - Will prioritize AIRS foundation crates
- [ ] **Zero Warning Policy** (workspace/zero_warning_policy.md) - Will ensure clean compilation

## Progress Log
### 2025-08-30
- Created task for binary file support removal
- Analyzed current binary processing implementation
- Developed comprehensive implementation plan with 6 subtasks
- Identified security benefits and implementation approach
- Ready for implementation planning and execution

## Technical Context
**Current binary support includes:**
- Image formats: JPEG, PNG, GIF, WebP, TIFF, BMP
- Document formats: PDF
- Binary processing pipeline with base64 encoding
- Configuration options for enabling/disabling binary processing
- Format detection using magic numbers and file extensions

**Files requiring modification:**
- `src/security/` - Add binary file rejection to validation
- `src/config/settings.rs` - Remove BinaryConfig options
- `src/binary/` - Remove or disable processing modules
- `src/mcp/handlers/file.rs` - Add binary rejection to file operations
- Configuration TOML files - Remove binary processing options
- Documentation files - Update to reflect text-only support

**Security Benefits:**
- Eliminates malware risk from binary file processing
- Prevents resource exhaustion from large binary files
- Reduces attack surface by removing complex binary parsing
- Aligns with "deny by default" security principle
- Simplifies security auditing and compliance

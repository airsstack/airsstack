# [TASK002] - Correlation Manager Implementation

**Status:** pending  
**Added:** 2025-08-01  
**Updated:** 2025-08-01

## Original Request
Implement the CorrelationManager for bidirectional request/response matching, supporting lock-free concurrency, timeout management, memory safety, and error propagation.

## Thought Process
- Essential for robust concurrent JSON-RPC communication.
- Enables reliable request tracking and response matching.
- Foundation for advanced integration and transport features.

## Implementation Plan
- Design CorrelationManager struct with DashMap and atomic request ID generation.
- Implement request lifecycle: register, send, correlate, resolve, cleanup.
- Integrate per-request and global timeout strategies.
- Propagate structured errors for diagnostics.
- Write unit tests for lifecycle, timeout, and error handling.

## Progress Tracking
**Overall Status:** not_started - 0%

### Subtasks
| ID   | Description                                 | Status      | Updated    | Notes                                 |
|------|---------------------------------------------|-------------|------------|---------------------------------------|
| 2.1  | Design CorrelationManager struct            | completed   | 2025-08-03 | File structure and types implemented |
| 2.2  | Implement request lifecycle methods         | not_started | 2025-08-01 | register, send, correlate, resolve    |
| 2.3  | Integrate timeout and cleanup logic         | not_started | 2025-08-01 | per-request/global timeout            |
| 2.4  | Implement error propagation                 | completed   | 2025-08-03 | Error types with structured info      |
| 2.5  | Write unit tests for all logic              | in_progress | 2025-08-03 | Basic tests done, full suite pending |

## Progress Log
### 2025-08-03
- **Phase 1 COMPLETED**: Architecture & file structure implemented
- Created correlation module with error.rs, types.rs, manager.rs (placeholder), tests.rs
- Implemented comprehensive error types with CorrelationResult<T>
- Implemented PendingRequest and RequestIdGenerator types
- Added 9 unit tests for error handling and type functionality  
- Updated lib.rs exports for correlation module
- All 28 unit tests + 30 doc tests passing
- Ready for Phase 4 (manager implementation)

### 2025-08-01
- Task created and ready for development.

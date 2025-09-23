# [task_003] - CLI Refactoring

**Status:** pending  
**Added:** 2025-09-23  
**Updated:** 2025-09-23

## Original Request
Refactor the main.rs file to create a standalone `cli` module with proper separation of concerns. The current main.rs is doing too much and violating the single responsibility principle.

## Thought Process
The current main.rs file contains:
- CLI argument definitions (clap structs)
- Command handlers for setup, config, and serve
- Logging initialization logic
- Main entry point coordination

This creates several issues:
- Difficult to test individual command handlers
- Poor separation of concerns
- Large, monolithic file that's hard to maintain
- CLI logic mixed with application logic

The proposed refactoring will create a clean, modular CLI architecture that improves maintainability, testability, and code organization.

## Implementation Plan

### Phase 1: CLI Module Structure Creation
1. Create `src/cli/` directory structure
2. Create module declarations and initial files
3. Define module interfaces and responsibilities

### Phase 2: Extract CLI Arguments
1. Move clap structs to `cli/args.rs`
2. Ensure proper imports and visibility
3. Validate argument parsing still works

### Phase 3: Extract Command Handlers
1. Create `cli/handlers/` module structure
2. Move `setup_directories()` to `cli/handlers/setup.rs`
3. Move `generate_config()` to `cli/handlers/config.rs`
4. Move `run_server()` to `cli/handlers/serve.rs`
5. Ensure proper error handling and async support

### Phase 4: Extract Logging Logic
1. Move logging initialization to `cli/logging.rs`
2. Support both file and console logging modes
3. Maintain environment-based configuration

### Phase 5: Simplify Main Entry Point
1. Reduce main.rs to minimal CLI integration
2. Create CLI coordinator in `cli/mod.rs`
3. Implement clean command routing

### Phase 6: Testing and Documentation
1. Add unit tests for each command handler
2. Add integration tests for CLI behavior
3. Update documentation with new architecture
4. Verify all existing functionality preserved

## Progress Tracking

**Overall Status:** pending - 0% completion

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 3.1 | Create CLI module directory structure | not_started | 2025-09-23 | Ready for implementation |
| 3.2 | Extract CLI arguments to args.rs | not_started | 2025-09-23 | Depends on 3.1 |
| 3.3 | Create command handlers module | not_started | 2025-09-23 | Depends on 3.2 |
| 3.4 | Extract setup command handler | not_started | 2025-09-23 | Depends on 3.3 |
| 3.5 | Extract config command handler | not_started | 2025-09-23 | Depends on 3.3 |
| 3.6 | Extract serve command handler | not_started | 2025-09-23 | Depends on 3.3 |
| 3.7 | Extract logging initialization | not_started | 2025-09-23 | Depends on 3.6 |
| 3.8 | Simplify main.rs entry point | not_started | 2025-09-23 | Depends on 3.7 |
| 3.9 | Add unit tests for handlers | not_started | 2025-09-23 | Depends on 3.8 |
| 3.10 | Add integration tests | not_started | 2025-09-23 | Depends on 3.9 |
| 3.11 | Update documentation | not_started | 2025-09-23 | Depends on 3.10 |
| 3.12 | Validate functionality preservation | not_started | 2025-09-23 | Final validation |

## Expected Outcomes

### Technical Benefits
- **Improved Maintainability**: Each module has a single, clear responsibility
- **Better Testability**: Individual handlers can be unit tested independently
- **Cleaner Architecture**: Proper separation between CLI concerns and application logic
- **Easier Extension**: New CLI commands can be added with minimal impact

### Code Quality Improvements
- **Reduced Complexity**: Transform 400+ line main.rs into focused, manageable modules
- **Better Error Handling**: Centralized error handling patterns across CLI commands
- **Enhanced Documentation**: Better organized documentation with examples per command
- **Improved Testing**: Comprehensive unit tests for each command handler

### File Structure After Refactoring
```
src/
├── main.rs                    # Minimal entry point (~20 lines)
├── cli/
│   ├── mod.rs                # CLI coordination and routing
│   ├── args.rs               # Command line argument definitions
│   ├── logging.rs            # Logging configuration
│   └── handlers/
│       ├── mod.rs            # Handler module declarations
│       ├── setup.rs          # setup_directories() implementation
│       ├── config.rs         # generate_config() implementation
│       └── serve.rs          # run_server() implementation
```

## Standards Compliance

**Workspace Standards Applied** (Reference: `workspace/shared_patterns.md`):
- [ ] **3-Layer Import Organization** (§2.1) - Applied to all new modules
- [ ] **chrono DateTime<Utc> Standard** (§3.2) - Maintained in extracted code
- [ ] **Module Architecture Patterns** (§4.3) - mod.rs files contain only declarations
- [ ] **Dependency Management** (§5.1) - AIRS foundation crates prioritized
- [ ] **Zero Warning Policy** - All refactored code must compile with zero warnings

## Progress Log

### 2025-09-23
- Created task for CLI refactoring
- Defined implementation plan with 6 phases
- Established subtask tracking with 12 detailed steps
- Ready for implementation pending user approval
# Dependency Injection Architecture - AIRS-MCP-FS

**Type**: Architecture Pattern  
**Category**: Design Pattern  
**Priority**: High  
**Status**: Planned  
**Date**: August 25, 2025  

## Problem Statement

The current `FilesystemMcpServer` constructor creates handler dependencies internally, leading to:
- **Tight Coupling**: Server constructor creates handler dependencies internally
- **Poor Testability**: Cannot inject mock handlers for unit testing
- **Inflexibility**: Cannot swap handler implementations
- **Violation of Single Responsibility**: Server manages both coordination and handler creation
- **Violation of Inversion of Control**: Dependencies flow inside-out

## Current Implementation

```rust
impl FilesystemMcpServer {
    pub async fn new(settings: Settings) -> Result<Self> {
        info!("Initializing AIRS MCP-FS filesystem server");

        // Initialize security manager with security config
        let security_manager = Arc::new(crate::security::SecurityManager::new(
            settings.security.clone(),
        ));

        // Create handlers with shared security manager
        let file_handler = FileHandler::new(Arc::clone(&security_manager));
        let directory_handler = DirectoryHandler::new(Arc::clone(&security_manager));

        Ok(Self {
            file_handler,
            directory_handler,
            settings: Arc::new(settings),
            _server_state: Arc::new(Mutex::new(ServerState::default())),
        })
    }
}
```

## Proposed Solution: Trait-Based Dependency Injection

### Phase 1: Extract Handler Traits

```rust
#[async_trait]
pub trait FileOperations: Send + Sync + std::fmt::Debug {
    async fn handle_read_file(&self, arguments: Value) -> McpResult<Vec<Content>>;
    async fn handle_write_file(&self, arguments: Value) -> McpResult<Vec<Content>>;
}

#[async_trait]
pub trait DirectoryOperations: Send + Sync + std::fmt::Debug {
    async fn handle_list_directory(&self, arguments: Value) -> McpResult<Vec<Content>>;
}
```

### Phase 2: Dependency Injection Constructor

```rust
impl FilesystemMcpServer {
    /// New constructor with dependency injection
    pub async fn new<F, D>(
        settings: Settings,
        file_handler: F,
        directory_handler: D,
    ) -> Result<Self>
    where
        F: FileOperations + 'static,
        D: DirectoryOperations + 'static,
    {
        Ok(Self {
            file_handler: Box::new(file_handler),
            directory_handler: Box::new(directory_handler),
            settings: Arc::new(settings),
            _server_state: Arc::new(Mutex::new(ServerState::default())),
        })
    }

    /// Convenience constructor for default handlers (backward compatibility)
    pub async fn with_default_handlers(settings: Settings) -> Result<Self> {
        let security_manager = Arc::new(SecurityManager::new(settings.security.clone()));
        let file_handler = FileHandler::new(Arc::clone(&security_manager));
        let directory_handler = DirectoryHandler::new(Arc::clone(&security_manager));
        
        Self::new(settings, file_handler, directory_handler).await
    }
}
```

## Benefits

### 1. Enhanced Testability
```rust
#[cfg(test)]
mod tests {
    struct MockFileHandler;
    struct MockDirectoryHandler;
    
    #[async_trait]
    impl FileOperations for MockFileHandler {
        async fn handle_read_file(&self, _args: Value) -> McpResult<Vec<Content>> {
            Ok(vec![Content::text("mock file content")])
        }
        async fn handle_write_file(&self, _args: Value) -> McpResult<Vec<Content>> {
            Ok(vec![Content::text("mock write success")])
        }
    }
    
    #[tokio::test]
    async fn test_server_with_mocks() {
        let settings = Settings::default();
        let mock_file = MockFileHandler;
        let mock_dir = MockDirectoryHandler;
        
        let server = FilesystemMcpServer::new(settings, mock_file, mock_dir).await.unwrap();
        // Test server behavior without real file I/O
    }
}
```

### 2. SOLID Principles Compliance
- **Single Responsibility**: Server coordinates, handlers handle operations
- **Open/Closed**: New handler implementations without server changes
- **Dependency Inversion**: Server depends on abstractions, not concretions

### 3. Architectural Flexibility
- Different handler implementations for testing vs production
- Handler composition and decoration patterns
- Easy environment-specific customization

## Implementation Strategy

### Phase 1: Trait Extraction
1. **Create traits** in `src/mcp/handlers/traits.rs`
2. **Implement traits** for existing `FileHandler` and `DirectoryHandler`
3. **Update server struct** to use trait objects

### Phase 2: Constructor Refactoring
1. **Create new dependency injection constructor**
2. **Maintain backward compatibility** with `with_default_handlers()`
3. **Update all existing instantiation code**

### Phase 3: Testing Enhancement
1. **Create mock handler implementations**
2. **Refactor tests** to use dependency injection
3. **Add integration tests** with real handlers

## Technical Considerations

### Memory Management
- Use `Box<dyn Trait>` for trait objects
- Ensure `Send + Sync + 'static` bounds for async context
- Consider `Arc<dyn Trait>` if handlers need to be shared

### Performance Impact
- Trait object dispatch has minimal overhead (< 1% performance impact)
- Benefits of testability outweigh minimal performance cost
- Consider generic parameters vs trait objects based on usage patterns

### Error Handling
- Maintain consistent error types across trait implementations
- Ensure error propagation works correctly with trait objects

## Migration Path

### Immediate Actions (Low Risk)
1. **Extract traits** - existing code continues to work
2. **Add new constructor** - additive change
3. **Implement traits** for existing handlers

### Future Actions (Medium Risk)
1. **Change struct fields** to trait objects (requires testing)
2. **Migrate existing instantiation** code
3. **Update all tests** to use new constructor

### Risk Mitigation
- **Maintain backward compatibility** during transition
- **Comprehensive test coverage** for both old and new patterns
- **Gradual migration** rather than big-bang approach

## Dependencies

- **External**: No new dependencies required
- **Internal**: Leverages existing `async-trait` crate
- **Compatibility**: Works with current MCP protocol integration

## Success Metrics

1. **Test Coverage**: Ability to mock handlers for unit testing
2. **Code Quality**: Reduced coupling between server and handlers
3. **Maintainability**: Easier to add new handler implementations
4. **Performance**: No significant performance degradation

---

**Timeline**: 2-3 development cycles  
**Effort**: Medium complexity  
**Impact**: High architectural improvement  
**Next Action**: Create implementation ticket for Phase 1

# Dependency Injection Architecture - AIRS MCP Server FS

**Type**: Architecture Pattern  
**Category**: Design Pattern  
**Priority**: High  
**Status**: Migrated from Legacy (airs-mcp-fs)  
**Date**: August 25, 2025  
**Migration Date**: September 23, 2025

## Problem Statement

The current `FilesystemMcpServer` constructor creates handler dependencies internally, leading to:
- **Tight Coupling**: Server constructor creates handler dependencies internally
- **Poor Testability**: Cannot inject mock handlers for unit testing
- **Inflexibility**: Cannot swap handler implementations
- **Violation of Single Responsibility**: Server manages both coordination and handler creation
- **Violation of Inversion of Control**: Dependencies flow inside-out

## Legacy Implementation (airs-mcp-fs)

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

## Current Implementation Status (airs-mcpserver-fs)

**Note**: The current airs-mcpserver-fs implementation uses `DefaultFilesystemMcpServer::with_default_handlers()` which follows similar patterns. This knowledge is preserved for future refactoring considerations.

## Proposed Solution: Trait-Based Dependency Injection

### Phase 1: Extract Handler Traits

```rust
#[async_trait]
pub trait FileOperations: Send + Sync + std::fmt::Debug {
    async fn read_file(&self, request: ReadFileRequest) -> Result<ReadFileResponse>;
    async fn write_file(&self, request: WriteFileRequest) -> Result<WriteFileResponse>;
}

#[async_trait]
pub trait DirectoryOperations: Send + Sync + std::fmt::Debug {
    async fn list_directory(&self, request: ListDirectoryRequest) -> Result<ListDirectoryResponse>;
}

#[async_trait]
pub trait SecurityValidation: Send + Sync + std::fmt::Debug {
    async fn validate_operation(&self, operation: &FileOperation) -> Result<()>;
    async fn requires_approval(&self, operation: &FileOperation) -> Result<bool>;
}
```

### Phase 2: Dependency Injection Constructor

```rust
impl FilesystemMcpServer {
    pub fn new(
        file_ops: Arc<dyn FileOperations>,
        directory_ops: Arc<dyn DirectoryOperations>,
        security: Arc<dyn SecurityValidation>,
        settings: Settings,
    ) -> Self {
        Self {
            file_handler: file_ops,
            directory_handler: directory_ops,
            security_manager: security,
            settings: Arc::new(settings),
            _server_state: Arc::new(Mutex::new(ServerState::default())),
        }
    }

    pub async fn with_default_handlers(settings: Settings) -> Result<Self> {
        let security_manager = Arc::new(crate::security::SecurityManager::new(
            settings.security.clone(),
        ));

        let file_handler = Arc::new(FileHandler::new(Arc::clone(&security_manager)));
        let directory_handler = Arc::new(DirectoryHandler::new(Arc::clone(&security_manager)));

        Ok(Self::new(
            file_handler,
            directory_handler,
            security_manager,
            settings,
        ))
    }
}
```

### Phase 3: Enable Testing with Mock Objects

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        FileHandler {}

        #[async_trait]
        impl FileOperations for FileHandler {
            async fn read_file(&self, request: ReadFileRequest) -> Result<ReadFileResponse>;
            async fn write_file(&self, request: WriteFileRequest) -> Result<WriteFileResponse>;
        }
    }

    #[tokio::test]
    async fn test_server_with_mock_handlers() {
        let mut mock_file_handler = MockFileHandler::new();
        mock_file_handler
            .expect_read_file()
            .returning(|_| Ok(ReadFileResponse::default()));

        let server = FilesystemMcpServer::new(
            Arc::new(mock_file_handler),
            Arc::new(MockDirectoryHandler::new()),
            Arc::new(MockSecurityManager::new()),
            Settings::default(),
        );

        // Test server behavior with mocked dependencies
    }
}
```

## Benefits of Dependency Injection

### Technical Benefits
1. **Improved Testability**: Mock dependencies can be injected for unit testing
2. **Loose Coupling**: Server depends on traits, not concrete implementations
3. **Flexibility**: Different handler implementations can be swapped at runtime
4. **Single Responsibility**: Server focuses on coordination, not dependency creation

### Design Benefits
1. **Inversion of Control**: Dependencies flow from outside-in
2. **Open/Closed Principle**: New handler implementations without modifying server
3. **Interface Segregation**: Clean trait boundaries for different concerns
4. **Dependency Inversion**: Depend on abstractions, not concretions

## Implementation Strategy

### Phase 1: Extract Traits (Low Risk)
- Define traits for existing functionality
- No behavioral changes to existing code
- Backward compatibility maintained

### Phase 2: Add Injection Constructor (Medium Risk)
- Add new constructor with trait dependencies
- Keep existing `with_default_handlers()` method
- Gradual migration path available

### Phase 3: Refactor Tests (High Value)
- Add comprehensive unit tests with mocks
- Improve test coverage and speed
- Enable TDD for future development

## Migration Considerations

### Compatibility
- **Backward Compatibility**: Maintain `with_default_handlers()` for existing users
- **API Stability**: No breaking changes to public interface
- **Performance**: No runtime overhead from trait dispatch

### Risk Mitigation
- **Incremental Implementation**: Each phase can be implemented separately
- **Rollback Strategy**: Original implementation preserved as fallback
- **Testing Strategy**: Comprehensive tests at each phase

## References

- **SOLID Principles**: Dependency Inversion, Single Responsibility, Open/Closed
- **Rust Design Patterns**: Trait objects, Arc for shared ownership
- **Testing Patterns**: Mock objects, dependency injection for testability

## Related Documentation

- **Workspace Standards**: Module architecture patterns (§4.3)
- **Security Framework**: Handler security validation patterns
- **MCP Integration**: Handler trait compatibility with MCP protocol

## Status

**Current**: Knowledge preserved from legacy implementation  
**Next Step**: Evaluate applicability to current airs-mcpserver-fs architecture  
**Timeline**: Future refactoring when testing improvements are prioritized
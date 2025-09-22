# Integration & Ecosystem

## **AIRS Ecosystem Integration**

### **Synergy with Existing Tools**
- **airs-mcp**: Leverages foundational MCP infrastructure and patterns
- **airs-memspec**: Could integrate for memory bank file management
- **airs-mcp-kb**: Natural pipeline for knowledge base document ingestion

### **Shared Architecture Patterns**
```rust
// Consistent with AIRS architectural patterns
pub struct AirsMcpFs {
    // Reuse MCP infrastructure
    mcp_foundation: Arc<AirsMcpFoundation>,
    
    // Consistent security patterns
    security_manager: Arc<SecurityManager>,
    
    // Standard error handling
    error_handler: Arc<ErrorHandler>,
    
    // Configuration management
    config: Arc<FsConfig>,
}
```

## **MCP Ecosystem Compatibility**
- **Claude Desktop**: Primary integration target
- **VS Code MCP Extensions**: Development environment integration
- **Custom MCP Clients**: API compatibility for third-party tools
- **Future MCP Tools**: Standard protocol ensures broad compatibility

## **Extension Points**
```rust
// Plugin architecture for custom file processors
#[async_trait]
pub trait FileProcessor: Send + Sync {
    fn supported_types(&self) -> Vec<FileType>;
    async fn process(&self, path: &Path, options: ProcessingOptions) -> Result<ProcessedContent, ProcessorError>;
}

// Registry for custom processors
pub struct ProcessorRegistry {
    processors: HashMap<FileType, Box<dyn FileProcessor>>,
}

impl ProcessorRegistry {
    pub fn register_processor(&mut self, processor: Box<dyn FileProcessor>) {
        for file_type in processor.supported_types() {
            self.processors.insert(file_type, processor);
        }
    }
    
    pub async fn process_file(&self, path: &Path, file_type: &FileType) -> Result<ProcessedContent, ProcessorError> {
        if let Some(processor) = self.processors.get(file_type) {
            processor.process(path, ProcessingOptions::default()).await
        } else {
            Err(ProcessorError::UnsupportedFileType(file_type.clone()))
        }
    }
}
```
# Performance & Scalability

## **Performance Targets**
- **Response Time**: <100ms for typical file operations
- **Large File Handling**: Support files up to 1GB with streaming
- **Concurrent Operations**: Handle 10+ concurrent file operations
- **Memory Usage**: <50MB baseline memory footprint

## **Optimization Strategies**

### **Memory Management**
```rust
// Streaming for large files
const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
const MAX_MEMORY_LOAD: u64 = 10 * 1024 * 1024; // 10MB

async fn read_large_file_streamed(&self, path: &Path) -> Result<FileStream, FsError> {
    let file = tokio::fs::File::open(path).await?;
    let file_size = file.metadata().await?.len();
    
    if file_size > MAX_MEMORY_LOAD {
        Ok(FileStream::chunked(file, CHUNK_SIZE))
    } else {
        Ok(FileStream::direct(file))
    }
}
```

### **Caching Strategy**
```rust
pub struct FileCache {
    metadata_cache: LruCache<PathBuf, FileMetadata>,
    content_cache: LruCache<PathBuf, CachedContent>,
    // Binary caching removed for security - text-only processing
}

impl FileCache {
    async fn get_or_compute_metadata(&mut self, path: &Path) -> Result<FileMetadata, FsError> {
        if let Some(cached) = self.metadata_cache.get(path) {
            if !cached.is_stale() {
                return Ok(cached.clone());
            }
        }
        
        let metadata = self.compute_metadata(path).await?;
        self.metadata_cache.put(path.to_path_buf(), metadata.clone());
        Ok(metadata)
    }
}
```

## **Scalability Considerations**
- **Async-First Design**: All I/O operations are non-blocking
- **Resource Limits**: Configurable limits prevent resource exhaustion
- **Connection Pooling**: Efficient handling of multiple MCP clients
- **Background Processing**: Heavy operations run in background tasks

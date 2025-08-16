# Testing & Quality Assurance

## **Testing Strategy**

### **Multi-Layer Testing Approach**
```rust
// Unit tests for core functionality
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_read_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Hello, World!").unwrap();
        
        let fs = AirsMcpFs::new(test_config()).await.unwrap();
        let result = fs.read_file(test_file.to_string_lossy().to_string(), None, None).await;
        
        assert!(result.is_ok());
        let content = result.unwrap();
        assert_eq!(content.content, "Hello, World!");
    }
    
    #[tokio::test]
    async fn test_binary_image_processing() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test image
        let img = image::RgbImage::new(100, 100);
        let test_image = temp_dir.path().join("test.png");
        img.save(&test_image).unwrap();
        
        let fs = AirsMcpFs::new(test_config()).await.unwrap();
        let options = BinaryProcessingOptions {
            image_options: ImageProcessingOptions {
                generate_thumbnail: true,
                max_dimension: Some(50),
                extract_metadata: true,
                ..Default::default()
            },
            ..Default::default()
        };
        
        let result = fs.read_binary_advanced(
            test_image.to_string_lossy().to_string(),
            Some(options)
        ).await;
        
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.thumbnail.is_some());
        assert!(content.metadata.is_some());
    }
}
```

### **Integration Testing**
```rust
// Integration tests with real MCP protocol
#[tokio::test]
async fn test_mcp_integration() {
    let (client, server) = create_test_mcp_connection().await;
    
    // Test file read through MCP protocol
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "read_file",
            "arguments": {
                "path": "./test-data/sample.txt"
            }
        }
    });
    
    let response = client.send_request(request).await.unwrap();
    assert_eq!(response["result"]["isError"], false);
    assert!(response["result"]["content"].is_string());
}
```

### **Security Testing**
```rust
// Security validation tests
#[tokio::test]
async fn test_path_traversal_prevention() {
    let fs = AirsMcpFs::new(test_config()).await.unwrap();
    
    // Attempt path traversal attack
    let result = fs.read_file("../../../etc/passwd".to_string(), None, None).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        FsError::Security(SecurityError::PathNotAllowed(_)) => {},
        _ => panic!("Expected path traversal to be blocked"),
    }
}

#[tokio::test]
async fn test_forbidden_file_patterns() {
    let fs = AirsMcpFs::new(test_config()).await.unwrap();
    
    // Attempt to read sensitive files
    let sensitive_files = vec![
        ".env",
        "config.key",
        ".ssh/id_rsa",
        "passwords.txt",
    ];
    
    for file in sensitive_files {
        let result = fs.read_file(file.to_string(), None, None).await;
        assert!(result.is_err(), "Should block access to {}", file);
    }
}
```

## **Performance Testing**
```rust
// Performance benchmarks
#[tokio::test]
async fn benchmark_file_operations() {
    let fs = AirsMcpFs::new(test_config()).await.unwrap();
    
    // Benchmark read operations
    let start = Instant::now();
    for i in 0..1000 {
        let _ = fs.read_file(format!("test-file-{}.txt", i), None, None).await;
    }
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 1000, "1000 reads should complete under 1 second");
    
    // Benchmark binary processing
    let start = Instant::now();
    let _ = fs.read_binary_advanced(
        "large-image.jpg".to_string(),
        Some(BinaryProcessingOptions::default())
    ).await;
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 500, "Binary processing should complete under 500ms");
}
```

## **Quality Metrics**
- **Test Coverage**: Target 90%+ line coverage
- **Performance**: <100ms response time for 95% of operations
- **Security**: Zero critical vulnerabilities in security audit
- **Reliability**: 99.9% uptime in continuous operation tests

---

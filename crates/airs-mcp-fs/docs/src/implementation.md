# Technical Implementation Details

## **Core Tool Implementations**

### **Primary File Operations**
```rust
#[derive(Clone)]
pub struct AirsMcpFs {
    config: Arc<FsConfig>,
    security: Arc<SecurityManager>,
    binary_processor: Arc<BinaryProcessor>,
    approval_workflow: Arc<ApprovalWorkflow>,
}

#[tool(description = "Read file contents with automatic encoding detection")]
async fn read_file(
    &self,
    path: String,
    encoding: Option<String>, // utf8, base64, auto
    max_size_mb: Option<u64>,
) -> Result<FileContent, FsError> {
    // Security validation
    self.security.validate_read_access(&path).await?;
    
    // File size check
    let metadata = tokio::fs::metadata(&path).await?;
    if metadata.len() > self.get_size_limit(max_size_mb) {
        return Err(FsError::FileTooLarge(metadata.len()));
    }
    
    // Determine encoding strategy
    let encoding = encoding.unwrap_or_else(|| self.detect_encoding(&path));
    
    match encoding.as_str() {
        "utf8" => self.read_text_file(&path).await,
        "base64" => self.read_binary_as_base64(&path).await,
        "auto" => self.read_with_auto_detection(&path).await,
        _ => Err(FsError::UnsupportedEncoding(encoding)),
    }
}

#[tool(description = "Write content to file with approval workflow")]
async fn write_file(
    &self,
    path: String,
    content: String,
    encoding: Option<String>,
    create_directories: Option<bool>,
    backup_existing: Option<bool>,
) -> Result<WriteResult, FsError> {
    // Security validation
    self.security.validate_write_access(&path).await?;
    
    // Human approval for write operations
    let approval_request = WriteApprovalRequest {
        path: path.clone(),
        content_preview: self.generate_content_preview(&content),
        operation_type: if Path::new(&path).exists() { 
            WriteType::Modify 
        } else { 
            WriteType::Create 
        },
        estimated_size: content.len(),
    };
    
    let approval = self.approval_workflow.request_approval(approval_request).await?;
    if !approval.approved {
        return Err(FsError::OperationDenied(approval.reason));
    }
    
    // Create directories if requested
    if create_directories.unwrap_or(false) {
        if let Some(parent) = Path::new(&path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    
    // Backup existing file if requested
    if backup_existing.unwrap_or(false) && Path::new(&path).exists() {
        self.create_backup(&path).await?;
    }
    
    // Write with appropriate encoding
    let encoding = encoding.unwrap_or_else(|| "utf8".to_string());
    match encoding.as_str() {
        "utf8" => self.write_text_file(&path, &content).await,
        "base64" => self.write_binary_from_base64(&path, &content).await,
        _ => Err(FsError::UnsupportedEncoding(encoding)),
    }
}
```

### **Advanced Binary Processing**
```rust
#[tool(description = "Read binary file with advanced processing options")]
async fn read_binary_advanced(
    &self,
    path: String,
    processing_options: Option<BinaryProcessingOptions>,
) -> Result<BinaryContent, FsError> {
    self.security.validate_read_access(&path).await?;
    
    let file_type = self.binary_processor.detect_file_type(&path).await?;
    let options = processing_options.unwrap_or_default();
    
    match file_type {
        FileType::Image(format) => {
            self.process_image(&path, &format, &options.image_options).await
        },
        FileType::Pdf => {
            self.process_pdf(&path, &options.pdf_options).await
        },
        FileType::Archive(format) => {
            self.process_archive(&path, &format, &options.archive_options).await
        },
        FileType::Video(format) => {
            self.process_video_metadata(&path, &format).await
        },
        FileType::Binary(mime_type) => {
            self.process_generic_binary(&path, &mime_type, &options).await
        },
    }
}

#[derive(Deserialize, Default)]
pub struct ImageProcessingOptions {
    generate_thumbnail: bool,
    max_dimension: Option<u32>,
    extract_metadata: bool,
    convert_format: Option<String>,
    compression_quality: Option<u8>,
}

impl BinaryProcessor {
    async fn process_image(
        &self,
        path: &str,
        format: &ImageFormat,
        options: &ImageProcessingOptions,
    ) -> Result<BinaryContent, FsError> {
        let img = image::open(path)?;
        
        let mut result = BinaryContent {
            path: path.to_string(),
            file_type: FileType::Image(*format),
            original_size: std::fs::metadata(path)?.len(),
            metadata: self.extract_image_metadata(path).await?,
            content: None,
            thumbnail: None,
            processed_variants: Vec::new(),
        };
        
        // Generate thumbnail
        if options.generate_thumbnail {
            let thumbnail = img.thumbnail(200, 200);
            let thumb_bytes = self.image_to_bytes(&thumbnail, &ImageFormat::Jpeg)?;
            result.thumbnail = Some(base64::encode(thumb_bytes));
        }
        
        // Resize if specified
        let processed_img = if let Some(max_dim) = options.max_dimension {
            if img.width() > max_dim || img.height() > max_dim {
                img.thumbnail(max_dim, max_dim)
            } else { img }
        } else { img };
        
        // Convert format if requested
        let output_format = if let Some(target_format) = &options.convert_format {
            ImageFormat::from_extension(target_format)
                .ok_or_else(|| FsError::UnsupportedFormat(target_format.clone()))?
        } else { *format };
        
        // Apply compression and encode
        let final_bytes = self.compress_image(
            &processed_img, 
            &output_format, 
            options.compression_quality.unwrap_or(85)
        )?;
        
        result.content = Some(base64::encode(final_bytes));
        result.processed_size = Some(result.content.as_ref().unwrap().len());
        
        Ok(result)
    }
}
```

## **Security Implementation**

### **Approval Workflow System**
```rust
#[async_trait]
pub trait ApprovalWorkflow: Send + Sync {
    async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalResponse, ApprovalError>;
    async fn get_approval_history(&self) -> Result<Vec<ApprovalRecord>, ApprovalError>;
}

pub struct InteractiveApprovalWorkflow {
    terminal: Terminal,
    config: ApprovalConfig,
}

impl ApprovalWorkflow for InteractiveApprovalWorkflow {
    async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalResponse, ApprovalError> {
        match request.operation_type {
            OperationType::Read => {
                // Automatic approval for read operations within allowed paths
                if self.is_auto_approved_read(&request.path) {
                    return Ok(ApprovalResponse::approved("Auto-approved read operation"));
                }
            },
            OperationType::Write | OperationType::Delete => {
                // Always require explicit approval for write/delete operations
                return self.request_interactive_approval(&request).await;
            },
        }
        
        self.request_interactive_approval(&request).await
    }
    
    async fn request_interactive_approval(&self, request: &ApprovalRequest) -> Result<ApprovalResponse, ApprovalError> {
        println!("\n🔐 AIRS MCP-FS: Operation Approval Required");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Operation: {}", request.operation_type);
        println!("Path: {}", request.path);
        
        if let Some(preview) = &request.content_preview {
            println!("Content Preview:");
            println!("{}", preview);
        }
        
        println!("\nDo you approve this operation? (y/N): ");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(ApprovalResponse::approved("User approved")),
            _ => Ok(ApprovalResponse::denied("User denied")),
        }
    }
}
```

### **Path Security Validation**
```rust
pub struct SecurityManager {
    config: SecurityConfig,
    path_validator: PathValidator,
    threat_detector: ThreatDetector,
}

impl SecurityManager {
    pub async fn validate_read_access(&self, path: &str) -> Result<(), SecurityError> {
        let canonical_path = self.path_validator.canonicalize(path)?;
        
        // Check against allowed read paths
        if !self.is_read_allowed(&canonical_path) {
            return Err(SecurityError::ReadNotAllowed(canonical_path.display().to_string()));
        }
        
        // Check forbidden patterns
        if self.matches_forbidden_pattern(&canonical_path) {
            return Err(SecurityError::ForbiddenPattern(path.to_string()));
        }
        
        // Check for potential threats
        self.threat_detector.scan_path(&canonical_path).await?;
        
        Ok(())
    }
    
    pub async fn validate_write_access(&self, path: &str) -> Result<(), SecurityError> {
        let canonical_path = self.path_validator.canonicalize_for_write(path)?;
        
        // Check against allowed write paths
        if !self.is_write_allowed(&canonical_path) {
            return Err(SecurityError::WriteNotAllowed(canonical_path.display().to_string()));
        }
        
        // Additional checks for write operations
        if self.is_system_critical_path(&canonical_path) {
            return Err(SecurityError::SystemCriticalPath(path.to_string()));
        }
        
        Ok(())
    }
    
    fn matches_forbidden_pattern(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in &self.config.forbidden_patterns {
            if pattern.is_match(&path_str) {
                return true;
            }
        }
        
        false
    }
}

#[derive(Deserialize)]
pub struct SecurityConfig {
    pub allowed_read_paths: Vec<String>,
    pub allowed_write_paths: Vec<String>,
    pub forbidden_patterns: Vec<String>, // Regex patterns
    pub max_file_size_mb: u64,
    pub require_approval_for_writes: bool,
    pub require_approval_for_deletes: bool,
    pub enable_threat_detection: bool,
}

// Example configuration
impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_read_paths: vec![
                "~/Documents/**".to_string(),
                "~/Desktop/**".to_string(),
                "~/Downloads/**".to_string(),
                "./".to_string(), // Current directory
            ],
            allowed_write_paths: vec![
                "~/Documents/**".to_string(),
                "~/Desktop/**".to_string(),
                "./".to_string(),
            ],
            forbidden_patterns: vec![
                r"\.env$".to_string(),
                r"\.ssh/.*".to_string(),
                r".*\.key$".to_string(),
                r".*\.pem$".to_string(),
                r"/etc/.*".to_string(),
                r".*password.*".to_string(),
            ],
            max_file_size_mb: 100,
            require_approval_for_writes: true,
            require_approval_for_deletes: true,
            enable_threat_detection: true,
        }
    }
}
```
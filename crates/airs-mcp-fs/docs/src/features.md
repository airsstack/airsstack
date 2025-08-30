# Core Features & Capabilities

## 1. **Fundamental Filesystem Operations**

### **File Operations**
```rust
// Read text and binary files with intelligent encoding
read_file(path: String, encoding: Option<String>) -> FileContent

// Write files with human approval workflow
write_file(path: String, content: String, create_directories: bool) -> WriteResult

// Atomic file operations for safety
move_file(source: String, destination: String) -> MoveResult
copy_file(source: String, destination: String) -> CopyResult
delete_file(path: String) -> DeleteResult
```

### **Directory Operations**
```rust
// Efficient directory listing with metadata
list_directory(path: String, recursive: bool, include_hidden: bool) -> DirectoryListing

// Directory management with permission checking
create_directory(path: String, recursive: bool) -> CreateResult
delete_directory(path: String, recursive: bool) -> DeleteResult
```

## 2. **Security-First File Processing**

### **Binary File Restriction**
- **Complete Binary Blocking**: All binary file operations are disabled for maximum security
- **Text-Only Processing**: Focus on development files like source code, configuration, and documentation
- **Attack Surface Reduction**: Eliminates entire classes of binary-based security vulnerabilities
- **Memory Safety**: Prevents buffer overflows and memory corruption from binary parsing
- **Malware Prevention**: Blocks execution of potentially malicious binary content

### **Supported Text File Types**
- **Source Code**: `.rs`, `.py`, `.js`, `.ts`, `.java`, `.cpp`, `.c`, `.go`, etc.
- **Configuration**: `.toml`, `.json`, `.yaml`, `.yml`, `.ini`, `.conf`, etc.
- **Documentation**: `.md`, `.txt`, `.rst`, `.adoc`, `.tex`, etc.
- **Data Files**: `.csv`, `.log`, `.sql`, `.xml`, `.html`, etc.
- **Web Files**: `.css`, `.scss`, `.less`, `.vue`, `.svelte`, etc.

### **Binary File Detection**
- **Extension-Based Validation**: Comprehensive list of known binary extensions
- **Content-Based Detection**: Analysis of file content to identify binary data
- **Magic Number Recognition**: Detection of binary file signatures
- **Audit Logging**: Comprehensive logging of all binary file rejection events

## 3. **Security & Access Control**

### **Multi-Layer Security Architecture**
```rust
// Configuration-driven security policies
SecurityPolicy {
    allowed_read_paths: Vec<PathBuf>,
    allowed_write_paths: Vec<PathBuf>,
    forbidden_patterns: Vec<Regex>,
    max_file_size_mb: u64,
    require_approval_for_writes: bool,
    require_approval_for_deletes: bool,
}
```

### **Human-in-the-Loop Workflows**
- **Write Operation Approval**: Interactive confirmation for file creation/modification
- **Dangerous Operation Protection**: Special approval for deletions, moves, executable files
- **Path Validation**: Prevent directory traversal and unauthorized access
- **Operation Auditing**: Comprehensive logging of all filesystem interactions

### **Risk-Based Controls**
- **File Type Restrictions**: Configurable policies for different file types
- **Size Limitations**: Prevent resource exhaustion from large file operations
- **Rate Limiting**: Prevent abuse through excessive operation requests
- **Quarantine Mode**: Safe handling of potentially dangerous files


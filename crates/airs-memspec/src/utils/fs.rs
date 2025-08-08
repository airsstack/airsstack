// File system operations
// Utilities for file and directory operations

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Error types for file system operations
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Memory bank not found at: {path}\n\nSuggestion: Run 'airs-memspec install' to set up the memory bank structure.")]
    PathNotFound { path: PathBuf },

    #[error("Path is not a directory: {path}\n\nSuggestion: Please remove the file and try again, or choose a different location.")]
    NotADirectory { path: PathBuf },

    #[error("Permission denied: {path}\n\nSuggestion: Check file permissions or try running with appropriate privileges.")]
    PermissionDenied { path: PathBuf },

    #[error(
        "File already exists: {path}\n\nSuggestion: Use --force flag to overwrite existing files."
    )]
    FileExists { path: PathBuf },

    #[error("Parse error: {message}\n\nSuggestion: {suggestion}")]
    ParseError { message: String, suggestion: String },

    #[error("Memory bank structure is incomplete or corrupted at: {path}\n\nSuggestion: Run 'airs-memspec install --force' to restore the memory bank structure.")]
    IncompleteStructure { path: PathBuf },

    #[error("Invalid memory bank format in: {file}\n\nDetails: {details}\n\nSuggestion: Check the file format or restore from backup. Run 'airs-memspec install --force' to reset.")]
    InvalidFormat { file: PathBuf, details: String },
}

/// Result type for file system operations
pub type FsResult<T> = Result<T, FsError>;

/// Create a directory and all parent directories if they don't exist
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> FsResult<()> {
    let path = path.as_ref();

    if path.exists() {
        if !path.is_dir() {
            return Err(FsError::NotADirectory {
                path: path.to_path_buf(),
            });
        }
        return Ok(());
    }

    fs::create_dir_all(path).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => FsError::PermissionDenied {
            path: path.to_path_buf(),
        },
        _ => FsError::Io(e),
    })
}

/// Write content to a file, creating parent directories if needed
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> FsResult<()> {
    let path = path.as_ref();

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    fs::write(path, content).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => FsError::PermissionDenied {
            path: path.to_path_buf(),
        },
        _ => FsError::Io(e),
    })
}

/// Write content to a file, optionally overwriting existing files
pub fn write_file_with_overwrite<P: AsRef<Path>>(
    path: P,
    content: &str,
    overwrite: bool,
) -> FsResult<()> {
    let path = path.as_ref();

    if path.exists() && !overwrite {
        return Err(FsError::FileExists {
            path: path.to_path_buf(),
        });
    }

    write_file(path, content)
}

/// Check if a path exists and is accessible
pub fn path_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Check if a path is a directory
pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Check if a path is a file
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Resolve a path to an absolute path
pub fn resolve_path<P: AsRef<Path>>(path: P) -> FsResult<PathBuf> {
    let path = path.as_ref();

    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(FsError::Io)
    }
}

/// Validate that a directory is writable
pub fn validate_writable_directory<P: AsRef<Path>>(path: P) -> FsResult<()> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(FsError::PathNotFound {
            path: path.to_path_buf(),
        });
    }

    if !path.is_dir() {
        return Err(FsError::NotADirectory {
            path: path.to_path_buf(),
        });
    }

    // Try to create a temporary file to test writability
    let test_file = path.join(".airs_memspec_write_test");
    match fs::write(&test_file, "") {
        Ok(_) => {
            // Clean up test file
            let _ = fs::remove_file(&test_file);
            Ok(())
        }
        Err(e) => match e.kind() {
            io::ErrorKind::PermissionDenied => Err(FsError::PermissionDenied {
                path: path.to_path_buf(),
            }),
            _ => Err(FsError::Io(e)),
        },
    }
}

/// Validate memory bank structure integrity
pub fn validate_memory_bank_structure<P: AsRef<Path>>(workspace_path: P) -> FsResult<()> {
    let workspace_path = workspace_path.as_ref();
    let memory_bank_path = workspace_path.join(".copilot").join("memory_bank");

    // Check if memory bank directory exists
    if !memory_bank_path.exists() {
        return Err(FsError::PathNotFound {
            path: memory_bank_path,
        });
    }

    // Check required structure
    let required_files = ["current_context.md"];

    let required_dirs = ["workspace", "sub_projects"];

    // Validate required files
    for file in &required_files {
        let file_path = memory_bank_path.join(file);
        if !file_path.exists() {
            return Err(FsError::IncompleteStructure { path: file_path });
        }

        // Basic content validation for current_context.md
        if file == &"current_context.md" {
            let content = fs::read_to_string(&file_path).map_err(FsError::Io)?;

            if content.trim().is_empty() {
                return Err(FsError::InvalidFormat {
                    file: file_path,
                    details: "File is empty".to_string(),
                });
            }
        }
    }

    // Validate required directories
    for dir in &required_dirs {
        let dir_path = memory_bank_path.join(dir);
        if !dir_path.exists() {
            return Err(FsError::IncompleteStructure { path: dir_path });
        }

        if !dir_path.is_dir() {
            return Err(FsError::NotADirectory { path: dir_path });
        }
    }

    Ok(())
}

/// Generate comprehensive error recovery suggestions based on error type and context
pub fn generate_recovery_suggestions(error: &FsError, workspace_path: Option<&Path>) -> String {
    match error {
        FsError::PathNotFound { path } => {
            if path.to_string_lossy().contains("memory_bank") {
                format!(
                    "Memory Bank Setup Required:\n\
                    1. Run 'airs-memspec install' to create the memory bank structure\n\
                    2. If this is a new project, run 'airs-memspec install --template multi-project'\n\
                    3. If you have an existing memory bank, check the path: {}\n\
                    4. Verify you're in the correct workspace directory",
                    path.display()
                )
            } else {
                format!(
                    "File Not Found:\n\
                    1. Check if the path exists: {}\n\
                    2. Verify file permissions\n\
                    3. If this is a project file, it may need to be created manually",
                    path.display()
                )
            }
        }

        FsError::IncompleteStructure { path } => {
            format!(
                "Memory Bank Structure Issues:\n\
                1. Run 'airs-memspec install --force' to restore missing components\n\
                2. Check for accidentally deleted files in: {}\n\
                3. Restore from backup if available\n\
                4. For new projects, use 'airs-memspec install --template multi-project'",
                path.display()
            )
        }

        FsError::InvalidFormat { file, details } => {
            format!(
                "File Format Issues:\n\
                1. Check the file syntax: {}\n\
                2. Problem: {}\n\
                3. Restore from backup or git history\n\
                4. Run 'airs-memspec install --force' to reset to defaults\n\
                5. For YAML issues, validate syntax with an online YAML validator",
                file.display(),
                details
            )
        }

        FsError::PermissionDenied { path } => {
            format!(
                "Permission Issues:\n\
                1. Check file ownership: ls -la {}\n\
                2. Fix permissions: chmod 644 {} (for files) or chmod 755 {} (for directories)\n\
                3. Check if the file is being used by another process\n\
                4. On Windows, run terminal as administrator if needed",
                path.display(),
                path.display(),
                path.display()
            )
        }

        FsError::FileExists { path } => {
            format!(
                "File Already Exists:\n\
                1. Use --force flag to overwrite: 'airs-memspec install --force'\n\
                2. Backup existing file: cp {} {}.backup\n\
                3. Remove existing file: rm {}\n\
                4. Choose a different target location",
                path.display(),
                path.display(),
                path.display()
            )
        }

        FsError::ParseError { message, .. } => {
            if message.contains("Sub-project") && message.contains("not found") {
                let workspace_info = if let Some(ws_path) = workspace_path {
                    format!("Workspace: {}", ws_path.display())
                } else {
                    "Current workspace".to_string()
                };

                format!(
                    "Project Not Found:\n\
                    1. List available projects: 'airs-memspec status'\n\
                    2. Check project name spelling\n\
                    3. {}\n\
                    4. If project should exist, check .copilot/memory_bank/sub_projects/ directory\n\
                    5. Create new project structure if needed",
                    workspace_info
                )
            } else {
                format!(
                    "Parsing Error:\n\
                    1. Check file syntax and format\n\
                    2. Problem: {}\n\
                    3. Validate against expected format\n\
                    4. Restore from backup if needed",
                    message
                )
            }
        }

        FsError::NotADirectory { path } => {
            format!(
                "Path Conflict:\n\
                1. Remove the conflicting file: rm {}\n\
                2. Create the directory: mkdir -p {}\n\
                3. Check if path is correct\n\
                4. Ensure you have write permissions",
                path.display(),
                path.display()
            )
        }

        FsError::Io(io_error) => {
            format!(
                "System Error:\n\
                1. Check available disk space: df -h\n\
                2. Verify file system is writable\n\
                3. System error: {}\n\
                4. Try again in a few moments\n\
                5. Check system logs for more details",
                io_error
            )
        }
    }
}

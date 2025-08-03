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
    #[error("Path does not exist: {path}")]
    PathNotFound { path: PathBuf },
    #[error("Path is not a directory: {path}")]
    NotADirectory { path: PathBuf },
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    #[error("File already exists: {path}")]
    FileExists { path: PathBuf },
    #[error("Parse error: {0}")]
    ParseError(String),
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

//! Error handling edge case tests
//!
//! Tests various error scenarios and validates that the improved error handling
//! provides helpful messages and recovery suggestions.

use airs_memspec::utils::fs::{validate_memory_bank_structure, FsError};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_memory_bank_validation_missing_directory() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // Test with no memory bank directory
    let result = validate_memory_bank_structure(workspace_path);

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Check that error contains helpful suggestion
    assert!(error_msg.contains("Suggestion: Run 'airs-memspec install'"));
    assert!(error_msg.contains("Memory bank not found"));
}

#[test]
fn test_memory_bank_validation_incomplete_structure() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // Create partial memory bank structure (missing required files)
    let memory_bank_path = workspace_path.join(".copilot").join("memory_bank");
    fs::create_dir_all(&memory_bank_path).unwrap();

    // Missing current_context.md file
    let result = validate_memory_bank_structure(workspace_path);

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Check for specific error type and helpful message
    assert!(error_msg.contains("incomplete or corrupted"));
    assert!(error_msg.contains("install --force"));
}

#[test]
fn test_memory_bank_validation_empty_required_file() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // Create complete structure but with empty required file
    let memory_bank_path = workspace_path.join(".copilot").join("memory_bank");
    fs::create_dir_all(&memory_bank_path).unwrap();
    fs::create_dir_all(memory_bank_path.join("workspace")).unwrap();
    fs::create_dir_all(memory_bank_path.join("sub_projects")).unwrap();

    // Create empty current_context.md
    fs::write(memory_bank_path.join("current_context.md"), "").unwrap();

    let result = validate_memory_bank_structure(workspace_path);

    assert!(result.is_err());
    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Check for invalid format error
    assert!(error_msg.contains("Invalid memory bank format"));
    assert!(error_msg.contains("File is empty"));
    assert!(error_msg.contains("install --force"));
}

#[test]
fn test_memory_bank_validation_valid_structure() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_path = temp_dir.path();

    // Create complete valid structure
    let memory_bank_path = workspace_path.join(".copilot").join("memory_bank");
    fs::create_dir_all(&memory_bank_path).unwrap();
    fs::create_dir_all(memory_bank_path.join("workspace")).unwrap();
    fs::create_dir_all(memory_bank_path.join("sub_projects")).unwrap();

    // Create valid current_context.md with content
    fs::write(
        memory_bank_path.join("current_context.md"),
        "# Current Context\n\nSome content here",
    )
    .unwrap();

    let result = validate_memory_bank_structure(workspace_path);

    assert!(
        result.is_ok(),
        "Valid memory bank structure should pass validation"
    );
}

#[test]
fn test_parse_error_with_suggestions() {
    let error = FsError::ParseError {
        message: "YAML parsing failed".to_string(),
        suggestion: "Check your YAML syntax".to_string(),
    };

    let error_msg = error.to_string();
    assert!(error_msg.contains("YAML parsing failed"));
    assert!(error_msg.contains("Suggestion: Check your YAML syntax"));
}

#[test]
fn test_path_not_found_error_message() {
    let error = FsError::PathNotFound {
        path: PathBuf::from("/some/missing/path"),
    };

    let error_msg = error.to_string();
    assert!(error_msg.contains("Memory bank not found"));
    assert!(error_msg.contains("airs-memspec install"));
}

#[test]
fn test_permission_denied_error_message() {
    let error = FsError::PermissionDenied {
        path: PathBuf::from("/restricted/path"),
    };

    let error_msg = error.to_string();
    assert!(error_msg.contains("Permission denied"));
    assert!(error_msg.contains("Check file permissions"));
}

#[test]
fn test_file_exists_error_message() {
    let error = FsError::FileExists {
        path: PathBuf::from("/existing/file"),
    };

    let error_msg = error.to_string();
    assert!(error_msg.contains("File already exists"));
    assert!(error_msg.contains("--force flag"));
}

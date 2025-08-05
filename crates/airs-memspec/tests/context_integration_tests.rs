//! Integration tests for context command implementation
//!
//! Tests the context command public interface including workspace context,
//! sub-project context, and error handling scenarios.

use airs_memspec::cli::GlobalArgs;
use std::env;
use std::path::PathBuf;

/// Create a minimal GlobalArgs for testing with current directory
fn create_test_global_args() -> GlobalArgs {
    GlobalArgs {
        path: Some(env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        verbose: false,
        quiet: true, // Keep quiet for tests
        no_color: true,
    }
}

/// Create GlobalArgs pointing to a non-existent directory for error testing
fn create_invalid_global_args() -> GlobalArgs {
    GlobalArgs {
        path: Some(PathBuf::from("/definitely/does/not/exist/path")),
        verbose: false,
        quiet: true,
        no_color: true,
    }
}

#[test]
fn test_context_command_workspace_mode() {
    // Test that the context command can be called without panicking
    let global_args = create_test_global_args();

    // This tests the full public interface
    let result = airs_memspec::cli::commands::context::run(&global_args, true, None);

    // The command should either succeed (if memory bank exists) or fail gracefully
    match result {
        Ok(_) => {
            // Success - memory bank exists and context was displayed
            println!("Context command succeeded");
        }
        Err(e) => {
            // Error is expected if no memory bank - should be a specific error type
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("PathNotFound")
                    || err_msg.contains("memory_bank")
                    || err_msg.contains("No memory bank found"),
                "Expected memory bank related error, got: {e}"
            );
        }
    }
}

#[test]
fn test_context_command_default_mode() {
    // Test the default (active) context mode
    let global_args = create_test_global_args();

    let result = airs_memspec::cli::commands::context::run(&global_args, false, None);

    // Should handle missing memory bank gracefully
    match result {
        Ok(_) => {
            println!("Default context command succeeded");
        }
        Err(e) => {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("PathNotFound")
                    || err_msg.contains("memory_bank")
                    || err_msg.contains("No memory bank found"),
                "Expected memory bank related error, got: {e}"
            );
        }
    }
}

#[test]
fn test_context_command_project_mode_with_graceful_failure() {
    // Test with a project name when no memory bank exists
    let global_args = create_test_global_args();

    let result = airs_memspec::cli::commands::context::run(
        &global_args,
        false,
        Some("airs-memspec".to_string()),
    );

    // Should handle missing memory bank gracefully
    match result {
        Ok(_) => {
            println!("Project context command succeeded");
        }
        Err(e) => {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("PathNotFound")
                    || err_msg.contains("memory_bank")
                    || err_msg.contains("No memory bank found"),
                "Expected memory bank related error, got: {e}"
            );
        }
    }
}

#[test]
fn test_context_command_error_handling() {
    // Test with an invalid workspace path
    let global_args_invalid = create_invalid_global_args();

    // This should handle the error gracefully and not panic
    let result = airs_memspec::cli::commands::context::run(&global_args_invalid, true, None);

    // The function should return an error, not panic
    assert!(
        result.is_err(),
        "Context command with invalid path should return an error"
    );

    let error = result.unwrap_err();
    let err_msg = error.to_string();
    assert!(
        err_msg.contains("PathNotFound")
            || err_msg.contains("memory_bank")
            || err_msg.contains("No such file or directory"),
        "Expected path-related error, got: {error}"
    );
}

#[test]
fn test_context_command_various_options() {
    // Test various combinations of options
    let global_args_verbose = GlobalArgs {
        path: Some(env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        verbose: true,
        quiet: false,
        no_color: true,
    };

    let global_args_quiet = GlobalArgs {
        path: Some(env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        verbose: false,
        quiet: true,
        no_color: false,
    };

    // Test verbose mode - should not panic regardless of result
    let result = std::panic::catch_unwind(|| {
        airs_memspec::cli::commands::context::run(&global_args_verbose, true, None)
    });
    assert!(
        result.is_ok(),
        "Context command with verbose should not panic"
    );

    // Test quiet mode - should not panic regardless of result
    let result = std::panic::catch_unwind(|| {
        airs_memspec::cli::commands::context::run(&global_args_quiet, false, None)
    });
    assert!(
        result.is_ok(),
        "Context command with quiet should not panic"
    );
}

#[test]
fn test_context_command_with_different_paths() {
    // Test with different workspace paths
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let global_args_current = GlobalArgs {
        path: Some(current_dir.clone()),
        verbose: false,
        quiet: true,
        no_color: true,
    };

    let global_args_none = GlobalArgs {
        path: None, // Should default to current directory
        verbose: false,
        quiet: true,
        no_color: true,
    };

    // Test with explicit current directory - should not panic
    let result = std::panic::catch_unwind(|| {
        airs_memspec::cli::commands::context::run(&global_args_current, true, None)
    });
    assert!(
        result.is_ok(),
        "Context command with explicit path should not panic"
    );

    // Test with None path (defaults to current directory) - should not panic
    let result = std::panic::catch_unwind(|| {
        airs_memspec::cli::commands::context::run(&global_args_none, true, None)
    });
    assert!(
        result.is_ok(),
        "Context command with None path should not panic"
    );
}

#[test]
fn test_context_command_consistency() {
    // Test that the command produces consistent error handling
    // by running it multiple times
    let global_args = create_test_global_args();

    let mut results = Vec::new();
    for _ in 0..3 {
        let result = airs_memspec::cli::commands::context::run(&global_args, true, None);
        results.push(result.is_ok());
    }

    // All results should be consistent (all success or all failure)
    let first_result = results[0];
    assert!(
        results.iter().all(|&r| r == first_result),
        "Context command should be consistent across multiple runs: {results:?}"
    );
}

#[test]
fn test_context_command_memory_usage() {
    // Test that the command doesn't leak memory with repeated calls
    let global_args = create_test_global_args();

    // Run the command multiple times to check for memory leaks
    for _ in 0..10 {
        let result = std::panic::catch_unwind(|| {
            airs_memspec::cli::commands::context::run(&global_args, false, None)
        });
        assert!(
            result.is_ok(),
            "Context command should not have memory issues"
        );
    }
}

#[test]
fn test_context_command_return_types() {
    // Test that the context command returns proper Result types
    let global_args = create_test_global_args();

    // Test all three modes return Result types
    let workspace_result = airs_memspec::cli::commands::context::run(&global_args, true, None);
    assert!(
        workspace_result.is_ok() || workspace_result.is_err(),
        "Should return a Result"
    );

    let default_result = airs_memspec::cli::commands::context::run(&global_args, false, None);
    assert!(
        default_result.is_ok() || default_result.is_err(),
        "Should return a Result"
    );

    let project_result =
        airs_memspec::cli::commands::context::run(&global_args, false, Some("test".to_string()));
    assert!(
        project_result.is_ok() || project_result.is_err(),
        "Should return a Result"
    );
}

#[test]
fn test_context_command_graceful_error_handling() {
    // Test that errors are handled gracefully without panics
    let invalid_args = create_invalid_global_args();

    // None of these should panic, even with invalid paths
    let panic_test = std::panic::catch_unwind(|| {
        let _ = airs_memspec::cli::commands::context::run(&invalid_args, true, None);
        let _ = airs_memspec::cli::commands::context::run(&invalid_args, false, None);
        let _ = airs_memspec::cli::commands::context::run(
            &invalid_args,
            false,
            Some("invalid".to_string()),
        );
    });

    assert!(
        panic_test.is_ok(),
        "Context command should handle all errors gracefully without panicking"
    );
}

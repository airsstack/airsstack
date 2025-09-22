//! Debug tests for path validation and permission system
//!
//! These tests verify that the dual-layer security system works correctly
//! for real-world scenarios like journal file access using dynamic paths.

use airs_mcpserver_fs::{
    config::{
        settings::{RiskLevel, SecurityPolicy},
        FilesystemConfig, OperationConfig, SecurityConfig,
    },
    filesystem::validation::PathValidator,
    mcp::types::OperationType,
    security::manager::SecurityManager,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Get the user's home directory dynamically for cross-platform testing
fn get_home_dir() -> PathBuf {
    dirs::home_dir().expect("Could not determine home directory")
}

/// Create test directory paths using the home directory
fn create_test_paths() -> (String, String, PathBuf, PathBuf) {
    let home = get_home_dir();
    let home_str = home.to_string_lossy().to_string();

    // Create cross-platform patterns
    let documents_pattern = format!("{}/Documents/**/*", home_str);
    let projects_pattern = format!("{}/Projects/**/*", home_str);

    // Create test paths that simulate the journal structure
    let test_journal_dir = home.join("Documents").join("test_journal").join("entries");
    let test_journal_file = test_journal_dir.join("2025_08_29.md");

    (
        documents_pattern,
        projects_pattern,
        test_journal_dir,
        test_journal_file,
    )
}

#[test]
fn test_path_validator_globset_patterns() {
    println!("🧪 Testing PathValidator with dynamic globset patterns");

    let (documents_pattern, projects_pattern, test_journal_dir, test_journal_file) =
        create_test_paths();

    println!("🏠 Using home directory: {}", get_home_dir().display());
    println!("📋 Documents pattern: {}", documents_pattern);
    println!("📋 Projects pattern: {}", projects_pattern);

    // Test patterns with dynamic home directory
    let allowed_patterns = vec![documents_pattern.clone(), projects_pattern];
    let denied_patterns = vec!["**/.git/**".to_string(), "**/.env*".to_string()];

    let validator = PathValidator::new(allowed_patterns, denied_patterns);

    // Test directory path validation
    println!("📁 Testing directory: {:?}", test_journal_dir);

    match validator.validate_path(&test_journal_dir) {
        Ok(_) => println!("✅ Directory validation passed"),
        Err(e) => {
            println!("❌ Directory validation failed: {:?}", e);
            panic!(
                "Directory should be allowed by pattern {}",
                documents_pattern
            );
        }
    }

    // Test file path validation
    println!("📄 Testing file: {:?}", test_journal_file);

    match validator.validate_path(&test_journal_file) {
        Ok(_) => println!("✅ File validation passed"),
        Err(e) => {
            println!("❌ File validation failed: {:?}", e);
            panic!("File should be allowed by pattern {}", documents_pattern);
        }
    }
}

#[test]
fn test_globset_pattern_matching() {
    println!("🧪 Testing raw globset pattern matching with dynamic paths");

    use globset::{Glob, GlobSetBuilder};

    let (documents_pattern, _, test_journal_dir, test_journal_file) = create_test_paths();
    let home = get_home_dir();

    println!("🏠 Testing with home directory: {}", home.display());
    println!("📋 Using pattern: {}", documents_pattern);

    // Create globset with our dynamic pattern
    let mut builder = GlobSetBuilder::new();

    match Glob::new(&documents_pattern) {
        Ok(glob) => {
            builder.add(glob);
            match builder.build() {
                Ok(globset) => {
                    let test_paths = vec![
                        home.join("Documents")
                            .join("test_journal")
                            .to_string_lossy()
                            .to_string(),
                        test_journal_dir.to_string_lossy().to_string(),
                        test_journal_file.to_string_lossy().to_string(),
                        home.join("Documents")
                            .join("test.txt")
                            .to_string_lossy()
                            .to_string(),
                    ];

                    for path in test_paths {
                        let matches = globset.is_match(&path);
                        println!(
                            "🔍 Pattern '{}' matches '{}': {}",
                            documents_pattern, path, matches
                        );

                        // All these should match our pattern
                        assert!(matches, "Pattern should match path: {}", path);
                    }
                }
                Err(e) => panic!("Failed to build globset: {:?}", e),
            }
        }
        Err(e) => panic!("Failed to create glob pattern: {:?}", e),
    }
}

#[test]
fn test_security_manager_creation() {
    println!("🧪 Testing SecurityManager creation with dynamic SecurityConfig");

    let (documents_pattern, projects_pattern, _, _) = create_test_paths();

    println!("🏠 Using home directory: {}", get_home_dir().display());
    println!("📋 Documents pattern: {}", documents_pattern);
    println!("📋 Projects pattern: {}", projects_pattern);

    // Create minimal SecurityConfig for testing with dynamic paths
    let filesystem_config = FilesystemConfig {
        allowed_paths: vec![documents_pattern.clone(), projects_pattern],
        denied_paths: vec!["**/.git/**".to_string(), "**/.env*".to_string()],
    };

    let operations_config = OperationConfig {
        read_allowed: true,
        write_requires_policy: false,
        delete_requires_explicit_allow: true,
        create_dir_allowed: true,
    };

    // Create security policies with dynamic paths
    let mut policies = HashMap::new();
    let journal_policy = SecurityPolicy {
        patterns: vec![documents_pattern],
        operations: vec!["read".to_string(), "write".to_string(), "list".to_string()],
        risk_level: RiskLevel::Low,
        description: Some("Personal documents and journal files".to_string()),
    };
    policies.insert("journal_files".to_string(), journal_policy);

    let security_config = SecurityConfig {
        filesystem: filesystem_config,
        operations: operations_config,
        policies,
    };

    match SecurityManager::new(security_config) {
        Ok(_) => {
            println!("✅ SecurityManager created successfully");
            println!("📊 Manager configuration loaded with dynamic paths");
        }
        Err(e) => {
            println!("❌ SecurityManager creation failed: {:?}", e);
            panic!("SecurityManager should be created successfully");
        }
    }
}

#[test]
fn test_operation_type_usage() {
    println!("🧪 Testing OperationType enum usage");

    // Test all operation types
    let operations = vec![
        OperationType::Read,
        OperationType::Write,
        OperationType::List,
        OperationType::Delete,
    ];

    for op in operations {
        println!("🔧 Operation type: {:?}", op);
        // Just verify they can be created and formatted
        let _debug_str = format!("{:?}", op);
    }

    println!("✅ All operation types work correctly");
}

#[test]
fn test_permission_system_integration() {
    println!("🧪 Testing full permission system integration - simulating real-world scenarios");

    let (documents_pattern, _projects_pattern, test_journal_dir, test_journal_file) =
        create_test_paths();

    println!("🏠 Using home directory: {}", get_home_dir().display());
    println!("📂 Test journal directory: {:?}", test_journal_dir);
    println!("📄 Test journal file: {:?}", test_journal_file);

    // Create filesystem config
    let filesystem_config = FilesystemConfig {
        allowed_paths: vec![documents_pattern.clone()],
        denied_paths: vec!["**/.git/**".to_string(), "**/.env*".to_string()],
    };

    let operations_config = OperationConfig {
        read_allowed: true,
        write_requires_policy: false,
        delete_requires_explicit_allow: true,
        create_dir_allowed: true,
    };

    // Create security policies that match your Claude Desktop configuration
    let mut policies = HashMap::new();
    let journal_policy = SecurityPolicy {
        patterns: vec![documents_pattern.clone()],
        operations: vec!["read".to_string(), "write".to_string(), "list".to_string()],
        risk_level: RiskLevel::Low,
        description: Some("Journal files access policy".to_string()),
    };
    policies.insert("journal_files".to_string(), journal_policy);

    let security_config = SecurityConfig {
        filesystem: filesystem_config,
        operations: operations_config,
        policies,
    };

    // Test SecurityManager creation
    let _manager = match SecurityManager::new(security_config) {
        Ok(mgr) => {
            println!("✅ SecurityManager created successfully");
            mgr
        }
        Err(e) => {
            println!("❌ SecurityManager creation failed: {:?}", e);
            panic!("SecurityManager should be created successfully");
        }
    };

    // Test PathValidator directly
    let validator = PathValidator::new(
        vec![documents_pattern.clone()],
        vec!["**/.git/**".to_string()],
    );

    // Test different scenarios that caused issues
    let test_scenarios = vec![
        ("Directory listing", &test_journal_dir, OperationType::List),
        ("File reading", &test_journal_file, OperationType::Read),
        ("File writing", &test_journal_file, OperationType::Write),
    ];

    for (scenario, path, _operation) in test_scenarios {
        println!("\n🔍 Testing scenario: {} with path: {:?}", scenario, path);

        // Test PathValidator first
        match validator.validate_path(path) {
            Ok(_) => println!("  ✅ PathValidator: {} validation passed", scenario),
            Err(e) => {
                println!(
                    "  ❌ PathValidator: {} validation failed: {:?}",
                    scenario, e
                );
                // Don't panic here, let's see what SecurityManager does
            }
        }

        // Test with SecurityManager would require creating file operations
        // This tests the pattern matching which is the core issue
        println!(
            "  📋 Pattern '{}' should match '{}'",
            documents_pattern,
            path.display()
        );

        use globset::{Glob, GlobSetBuilder};
        let mut builder = GlobSetBuilder::new();
        if let Ok(glob) = Glob::new(&documents_pattern) {
            builder.add(glob);
            if let Ok(globset) = builder.build() {
                let path_str = path.to_string_lossy();
                let matches = globset.is_match(path_str.as_ref());
                println!("  🎯 Direct globset match result: {}", matches);

                if !matches {
                    println!("  ⚠️  WARNING: Pattern does not match! This indicates the permission issue!");
                    println!("  🔧 Pattern: {}", documents_pattern);
                    println!("  🔧 Path: {}", path_str);
                }
            }
        }
    }

    println!("\n🏁 Permission system integration test completed");
}

#[test]
fn test_claude_desktop_simulation() {
    println!("🧪 Testing Claude Desktop MCP integration simulation");

    let (documents_pattern, _, test_journal_dir, test_journal_file) = create_test_paths();

    println!("🤖 Simulating Claude Desktop MCP server scenario");
    println!("📋 Configuration pattern: {}", documents_pattern);

    // This test simulates the exact scenario that was failing
    // where file reading worked but directory listing failed

    let validator = PathValidator::new(
        vec![documents_pattern.clone()],
        vec!["**/.git/**".to_string(), "**/.env*".to_string()],
    );

    // Test the specific operations that were failing
    println!("\n🔍 Testing directory listing (was failing in Claude Desktop):");
    match validator.validate_path(&test_journal_dir) {
        Ok(_) => println!("  ✅ Directory listing validation passed"),
        Err(e) => {
            println!("  ❌ Directory listing validation failed: {:?}", e);
            println!("  🚨 This reproduces the Claude Desktop issue!");
        }
    }

    println!("\n🔍 Testing file reading (was working in Claude Desktop):");
    match validator.validate_path(&test_journal_file) {
        Ok(_) => println!("  ✅ File reading validation passed"),
        Err(e) => {
            println!("  ❌ File reading validation failed: {:?}", e);
            println!("  🚨 This is unexpected - file reading should work!");
        }
    }

    // Test pattern matching directly to isolate the issue
    println!("\n🔍 Testing raw pattern matching:");
    use globset::{Glob, GlobSetBuilder};
    let mut builder = GlobSetBuilder::new();

    if let Ok(glob) = Glob::new(&documents_pattern) {
        builder.add(glob);
        if let Ok(globset) = builder.build() {
            let dir_matches = globset.is_match(test_journal_dir.to_string_lossy().as_ref());
            let file_matches = globset.is_match(test_journal_file.to_string_lossy().as_ref());

            println!("  📁 Directory matches pattern: {}", dir_matches);
            println!("  📄 File matches pattern: {}", file_matches);

            if dir_matches && file_matches {
                println!("  ✅ Both paths match - pattern system is working correctly");
            } else {
                println!("  ❌ Pattern matching failed - this is the root cause!");
            }
        }
    }

    println!("\n🤖 Claude Desktop simulation completed");
}

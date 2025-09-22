//! Integration test for configuration validation functionality
//!
//! Tests that configuration validation works correctly at the application level.

#![allow(clippy::expect_used, clippy::bool_assert_comparison)]

// Layer 1: Standard library imports
// (None needed)

// Layer 2: Third-party crate imports
// (None needed)

// Layer 3: Internal module imports
use airs_mcp_fs::config::{ConfigurationValidator, Settings};

#[test]
fn test_configuration_validation_integration() {
    // Test that default settings pass validation
    let settings = Settings::default();
    let validation_result = settings.validate().expect("Validation should not error");

    assert!(
        validation_result.is_valid,
        "Default settings should be valid. Errors: {:?}",
        validation_result.errors
    );

    // Verify that the settings can be loaded successfully (includes validation)
    let loaded_settings =
        Settings::load().expect("Settings should load successfully with validation");
    assert_eq!(loaded_settings.server.name, "airs-mcp-fs");
}

#[test]
fn test_configuration_validation_comprehensive() {
    // Test validation with permissive settings for testing
    let settings = Settings::builder().permissive().build();

    // Test security configuration validation specifically
    let security_validation = ConfigurationValidator::validate_security_config(&settings.security)
        .expect("Security validation should not error");

    assert!(
        security_validation.is_valid,
        "Security configuration should be valid. Errors: {:?}",
        security_validation.errors
    );

    // Verify that policies exist and are valid
    assert!(
        !settings.security.policies.is_empty(),
        "Should have security policies"
    );
    assert!(
        settings.security.policies.contains_key("source_code"),
        "Should have source_code policy"
    );
    assert!(
        settings.security.policies.contains_key("documentation"),
        "Should have documentation policy"
    );

    // Test permissive mode settings
    assert!(
        !settings.security.operations.write_requires_policy,
        "Permissive mode should not require policies for writes"
    );
    assert!(
        !settings.security.operations.delete_requires_explicit_allow,
        "Permissive mode should not require explicit delete permissions"
    );
    assert!(
        settings
            .security
            .policies
            .contains_key("permissive_universal"),
        "Permissive mode should have universal policy"
    );
}

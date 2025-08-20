//! MCP Method to OAuth Scope Validation
//!
//! This module provides mapping and validation of MCP protocol methods
//! to required OAuth 2.1 scopes for fine-grained access control.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use tracing::{debug, warn};

// Layer 3: Internal module imports
use crate::oauth2::{config::ScopeMapping, error::OAuth2Error, error::OAuth2Result};

/// Validator for MCP method to OAuth scope mappings
pub struct ScopeValidator {
    /// Map of MCP method -> required OAuth scope
    scope_mappings: HashMap<String, ScopeMapping>,
}

impl ScopeValidator {
    /// Create a new scope validator with the given mappings
    pub fn new(mappings: Vec<ScopeMapping>) -> Self {
        let scope_mappings: HashMap<String, ScopeMapping> = mappings
            .into_iter()
            .map(|mapping| (mapping.method.clone(), mapping))
            .collect();

        debug!(
            "Created scope validator with {} mappings",
            scope_mappings.len()
        );
        Self { scope_mappings }
    }

    /// Create a scope validator with default MCP mappings
    pub fn with_default_mappings() -> Self {
        use crate::oauth2::config::OAuth2Config;
        Self::new(OAuth2Config::default_scope_mappings())
    }

    /// Validate that the provided scopes allow access to the specified MCP method
    pub fn validate_method_access(&self, method: &str, user_scopes: &[String]) -> OAuth2Result<()> {
        // Find the scope mapping for this method
        let mapping = self.scope_mappings.get(method);

        match mapping {
            Some(mapping) => {
                // Check if the required scope is present in user scopes
                let has_required_scope = user_scopes.contains(&mapping.scope);

                if has_required_scope {
                    debug!(
                        "Access granted to method '{}' with scope '{}'",
                        method, mapping.scope
                    );
                    Ok(())
                } else if mapping.optional {
                    warn!(
                        "Optional scope '{}' missing for method '{}', allowing access",
                        mapping.scope, method
                    );
                    Ok(())
                } else {
                    debug!(
                        "Access denied to method '{}': required scope '{}' not found in user scopes: {:?}",
                        method, mapping.scope, user_scopes
                    );
                    Err(OAuth2Error::InsufficientScope {
                        required: mapping.scope.clone(),
                        provided: user_scopes.join(" "),
                    })
                }
            }
            None => {
                // No scope mapping found - this could be:
                // 1. A new MCP method not yet configured
                // 2. A custom extension method
                // 3. An invalid method name

                warn!(
                    "No scope mapping found for method '{}', denying access by default",
                    method
                );
                Err(OAuth2Error::InsufficientScope {
                    required: format!("mcp:{}:*", method.split('/').next().unwrap_or(method)),
                    provided: user_scopes.join(" "),
                })
            }
        }
    }

    /// Get the required scope for a given MCP method
    pub fn get_required_scope(&self, method: &str) -> Option<&str> {
        self.scope_mappings.get(method).map(|m| m.scope.as_str())
    }

    /// Get all configured method-scope mappings
    pub fn get_all_mappings(&self) -> Vec<&ScopeMapping> {
        self.scope_mappings.values().collect()
    }

    /// Check if a method is configured (has a scope mapping)
    pub fn is_method_configured(&self, method: &str) -> bool {
        self.scope_mappings.contains_key(method)
    }

    /// Add or update a scope mapping
    pub fn add_mapping(&mut self, mapping: ScopeMapping) {
        debug!(
            "Adding scope mapping: {} -> {}",
            mapping.method, mapping.scope
        );
        self.scope_mappings.insert(mapping.method.clone(), mapping);
    }

    /// Remove a scope mapping
    pub fn remove_mapping(&mut self, method: &str) -> Option<ScopeMapping> {
        debug!("Removing scope mapping for method: {}", method);
        self.scope_mappings.remove(method)
    }

    /// Get all configured MCP methods
    pub fn get_configured_methods(&self) -> Vec<&str> {
        self.scope_mappings.keys().map(|s| s.as_str()).collect()
    }

    /// Get all required scopes across all methods
    pub fn get_all_required_scopes(&self) -> Vec<&str> {
        let mut scopes: Vec<&str> = self
            .scope_mappings
            .values()
            .map(|m| m.scope.as_str())
            .collect();
        scopes.sort();
        scopes.dedup();
        scopes
    }

    /// Validate multiple methods at once (batch validation)
    pub fn validate_batch_access(
        &self,
        methods: &[String],
        user_scopes: &[String],
    ) -> OAuth2Result<()> {
        for method in methods {
            self.validate_method_access(method, user_scopes)?;
        }
        Ok(())
    }

    /// Get the minimum scopes required for the given methods
    pub fn get_minimum_scopes_for_methods(&self, methods: &[String]) -> Vec<String> {
        let mut required_scopes = Vec::new();

        for method in methods {
            if let Some(mapping) = self.scope_mappings.get(method) {
                if !mapping.optional && !required_scopes.contains(&mapping.scope) {
                    required_scopes.push(mapping.scope.clone());
                }
            }
        }

        required_scopes.sort();
        required_scopes
    }

    /// Check if user has sufficient scopes for all given methods
    pub fn check_sufficient_scopes(
        &self,
        methods: &[String],
        user_scopes: &[String],
    ) -> (Vec<String>, Vec<String>) {
        let mut granted_methods = Vec::new();
        let mut denied_methods = Vec::new();

        for method in methods {
            match self.validate_method_access(method, user_scopes) {
                Ok(()) => granted_methods.push(method.clone()),
                Err(_) => denied_methods.push(method.clone()),
            }
        }

        (granted_methods, denied_methods)
    }
}

/// Helper functions for common scope patterns
impl ScopeValidator {
    /// Check if user has read access to a resource type
    pub fn has_read_access(&self, resource_type: &str, user_scopes: &[String]) -> bool {
        let read_scope = format!("mcp:{}:read", resource_type);
        user_scopes.contains(&read_scope)
    }

    /// Check if user has write/execute access to a resource type
    pub fn has_write_access(&self, resource_type: &str, user_scopes: &[String]) -> bool {
        let write_scope = format!("mcp:{}:execute", resource_type);
        let admin_scope = format!("mcp:{}:admin", resource_type);
        user_scopes.contains(&write_scope) || user_scopes.contains(&admin_scope)
    }

    /// Check if user has admin access to a resource type
    pub fn has_admin_access(&self, resource_type: &str, user_scopes: &[String]) -> bool {
        let admin_scope = format!("mcp:{}:admin", resource_type);
        user_scopes.contains(&admin_scope)
    }

    /// Extract resource type from MCP method (e.g., "tools/call" -> "tools")
    pub fn extract_resource_type(method: &str) -> &str {
        method.split('/').next().unwrap_or(method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::config::ScopeMapping;

    fn create_test_validator() -> ScopeValidator {
        let mappings = vec![
            ScopeMapping {
                method: "tools/call".to_string(),
                scope: "mcp:tools:execute".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "tools/list".to_string(),
                scope: "mcp:tools:read".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "resources/read".to_string(),
                scope: "mcp:resources:read".to_string(),
                optional: false,
            },
            ScopeMapping {
                method: "debug/info".to_string(),
                scope: "mcp:debug:read".to_string(),
                optional: true,
            },
        ];
        ScopeValidator::new(mappings)
    }

    #[test]
    fn test_validate_method_access_success() {
        let validator = create_test_validator();
        let user_scopes = vec![
            "mcp:tools:execute".to_string(),
            "mcp:resources:read".to_string(),
        ];

        // Should succeed for method with required scope
        assert!(validator
            .validate_method_access("tools/call", &user_scopes)
            .is_ok());

        // Should succeed for method with different required scope
        assert!(validator
            .validate_method_access("resources/read", &user_scopes)
            .is_ok());
    }

    #[test]
    fn test_validate_method_access_insufficient_scope() {
        let validator = create_test_validator();
        let user_scopes = vec!["mcp:resources:read".to_string()];

        // Should fail for method requiring different scope
        let result = validator.validate_method_access("tools/call", &user_scopes);
        assert!(result.is_err());

        if let Err(OAuth2Error::InsufficientScope { required, provided }) = result {
            assert_eq!(required, "mcp:tools:execute");
            assert_eq!(provided, "mcp:resources:read");
        } else {
            panic!("Expected InsufficientScope error");
        }
    }

    #[test]
    fn test_validate_method_access_optional_scope() {
        let validator = create_test_validator();
        let user_scopes = vec!["mcp:tools:execute".to_string()];

        // Should succeed for optional scope even if not present
        assert!(validator
            .validate_method_access("debug/info", &user_scopes)
            .is_ok());
    }

    #[test]
    fn test_validate_method_access_unknown_method() {
        let validator = create_test_validator();
        let user_scopes = vec!["mcp:tools:execute".to_string()];

        // Should fail for unknown method
        let result = validator.validate_method_access("unknown/method", &user_scopes);
        assert!(result.is_err());

        if let Err(OAuth2Error::InsufficientScope { required, .. }) = result {
            assert_eq!(required, "mcp:unknown:*");
        } else {
            panic!("Expected InsufficientScope error");
        }
    }

    #[test]
    fn test_get_required_scope() {
        let validator = create_test_validator();

        assert_eq!(
            validator.get_required_scope("tools/call"),
            Some("mcp:tools:execute")
        );
        assert_eq!(
            validator.get_required_scope("resources/read"),
            Some("mcp:resources:read")
        );
        assert_eq!(validator.get_required_scope("unknown/method"), None);
    }

    #[test]
    fn test_batch_validation() {
        let validator = create_test_validator();
        let methods = vec!["tools/call".to_string(), "resources/read".to_string()];
        let user_scopes = vec![
            "mcp:tools:execute".to_string(),
            "mcp:resources:read".to_string(),
        ];

        // Should succeed when all required scopes are present
        assert!(validator
            .validate_batch_access(&methods, &user_scopes)
            .is_ok());

        // Should fail when any required scope is missing
        let insufficient_scopes = vec!["mcp:tools:execute".to_string()];
        assert!(validator
            .validate_batch_access(&methods, &insufficient_scopes)
            .is_err());
    }

    #[test]
    fn test_get_minimum_scopes_for_methods() {
        let validator = create_test_validator();
        let methods = vec![
            "tools/call".to_string(),
            "tools/list".to_string(),
            "debug/info".to_string(), // Optional - should not be included
        ];

        let min_scopes = validator.get_minimum_scopes_for_methods(&methods);
        assert_eq!(min_scopes.len(), 2);
        assert!(min_scopes.contains(&"mcp:tools:execute".to_string()));
        assert!(min_scopes.contains(&"mcp:tools:read".to_string()));
        assert!(!min_scopes.contains(&"mcp:debug:read".to_string()));
    }

    #[test]
    fn test_check_sufficient_scopes() {
        let validator = create_test_validator();
        let methods = vec![
            "tools/call".to_string(),
            "tools/list".to_string(),
            "resources/read".to_string(),
        ];
        let user_scopes = vec![
            "mcp:tools:execute".to_string(),
            "mcp:resources:read".to_string(),
        ];

        let (granted, denied) = validator.check_sufficient_scopes(&methods, &user_scopes);

        assert_eq!(granted.len(), 2);
        assert!(granted.contains(&"tools/call".to_string()));
        assert!(granted.contains(&"resources/read".to_string()));

        assert_eq!(denied.len(), 1);
        assert!(denied.contains(&"tools/list".to_string()));
    }

    #[test]
    fn test_helper_methods() {
        let validator = create_test_validator();
        let scopes = vec![
            "mcp:tools:read".to_string(),
            "mcp:resources:execute".to_string(),
            "mcp:admin:admin".to_string(),
        ];

        assert!(validator.has_read_access("tools", &scopes));
        assert!(!validator.has_read_access("unknown", &scopes));

        assert!(validator.has_write_access("resources", &scopes));
        assert!(!validator.has_write_access("tools", &scopes));

        assert!(validator.has_admin_access("admin", &scopes));
        assert!(!validator.has_admin_access("tools", &scopes));
    }

    #[test]
    fn test_extract_resource_type() {
        assert_eq!(ScopeValidator::extract_resource_type("tools/call"), "tools");
        assert_eq!(
            ScopeValidator::extract_resource_type("resources/read"),
            "resources"
        );
        assert_eq!(
            ScopeValidator::extract_resource_type("simple_method"),
            "simple_method"
        );
    }

    #[test]
    fn test_add_remove_mappings() {
        let mut validator = create_test_validator();

        // Test adding new mapping
        let new_mapping = ScopeMapping {
            method: "custom/method".to_string(),
            scope: "mcp:custom:execute".to_string(),
            optional: false,
        };

        validator.add_mapping(new_mapping);
        assert!(validator.is_method_configured("custom/method"));
        assert_eq!(
            validator.get_required_scope("custom/method"),
            Some("mcp:custom:execute")
        );

        // Test removing mapping
        let removed = validator.remove_mapping("custom/method");
        assert!(removed.is_some());
        assert!(!validator.is_method_configured("custom/method"));
    }

    #[test]
    fn test_get_all_scopes() {
        let validator = create_test_validator();
        let all_scopes = validator.get_all_required_scopes();

        assert!(all_scopes.contains(&"mcp:tools:execute"));
        assert!(all_scopes.contains(&"mcp:tools:read"));
        assert!(all_scopes.contains(&"mcp:resources:read"));
        assert!(all_scopes.contains(&"mcp:debug:read"));

        // Should be sorted and deduplicated
        let mut sorted_scopes = all_scopes.clone();
        sorted_scopes.sort();
        assert_eq!(all_scopes, sorted_scopes);
    }
}

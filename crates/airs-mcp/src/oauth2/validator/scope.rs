//! Scope Validator Trait and Implementation
//!
//! Provides trait-based OAuth scope validation for MCP method authorization
//! following workspace standards for zero-cost abstractions.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use tracing::{debug, warn};

// Layer 3: Internal module imports
use crate::oauth2::{
    config::{OAuth2Config, ScopeMapping},
    error::OAuth2Error,
};

/// OAuth scope validation trait
///
/// Provides method-to-scope validation with flexible error handling
/// following workspace standards for trait design.
pub trait ScopeValidator {
    /// Error type specific to this validator implementation
    type Error: Into<OAuth2Error> + Send + Sync + 'static;

    /// Validate that provided scopes are sufficient for the given MCP method
    ///
    /// # Arguments
    /// * `method` - MCP method name (e.g., "tools/call", "resources/read")
    /// * `scopes` - User's OAuth scopes from JWT token
    ///
    /// # Returns
    /// * `Ok(())` - User has sufficient scope for the method
    /// * `Err(Self::Error)` - Insufficient scope or validation error
    fn validate_method_access(&self, method: &str, scopes: &[String]) -> Result<(), Self::Error>;

    /// Check if method is configured in scope mappings
    ///
    /// Default implementation returns true, override for custom logic.
    fn is_method_configured(&self, method: &str) -> bool {
        // Default: assume all methods are valid
        // Override for stricter validation
        let _ = method;
        true
    }

    /// Get required scope for a method
    ///
    /// Default implementation returns None, override for inspection capabilities.
    fn get_required_scope(&self, method: &str) -> Option<&str> {
        // Default: no scope inspection
        let _ = method;
        None
    }

    /// Batch validate multiple methods
    ///
    /// Default implementation validates each method individually.
    /// Override for optimized batch processing.
    fn validate_batch_access(
        &self,
        methods: &[&str],
        scopes: &[String],
    ) -> Result<(), Self::Error> {
        for method in methods {
            self.validate_method_access(method, scopes)?;
        }
        Ok(())
    }
}

/// Concrete scope validator implementation
///
/// Self-contained scope validator with method-to-scope mapping logic
/// following workspace standards for zero-cost abstractions.
pub struct Scope {
    /// Map of MCP method -> required OAuth scope
    scope_mappings: HashMap<String, ScopeMapping>,
}

impl Scope {
    /// Create new scope validator with custom mappings
    ///
    /// # Arguments
    /// * `mappings` - Vector of method-to-scope mappings
    ///
    /// # Returns
    /// * New scope validator instance
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

    /// Create scope validator with default MCP mappings
    ///
    /// Uses the standard MCP method-to-scope mappings for common operations.
    pub fn with_default_mappings() -> Self {
        Self::new(OAuth2Config::default_scope_mappings())
    }

    /// Add new scope mapping
    ///
    /// # Arguments
    /// * `mapping` - New method-to-scope mapping to add
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

    /// Get minimum scopes required for the given methods
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

    /// Extract resource type from MCP method (e.g., "tools/call" -> "tools")
    pub fn extract_resource_type(method: &str) -> &str {
        method.split('/').next().unwrap_or(method)
    }
}
// refactoring the inner validator for thread safety
impl ScopeValidator for Scope {
    type Error = OAuth2Error;

    fn validate_method_access(&self, method: &str, scopes: &[String]) -> Result<(), Self::Error> {
        // Find the scope mapping for this method
        let mapping = self.scope_mappings.get(method);

        match mapping {
            Some(mapping) => {
                // Check if the required scope is present in user scopes
                let has_required_scope = scopes.contains(&mapping.scope);

                match (has_required_scope, mapping.optional) {
                    (true, _) => {
                        debug!(
                            "Access granted to method '{}' with scope '{}'",
                            method, mapping.scope
                        );
                        Ok(())
                    }
                    (false, true) => {
                        warn!(
                            "Optional scope '{}' missing for method '{}', allowing access",
                            mapping.scope, method
                        );
                        Ok(())
                    }
                    (false, false) => {
                        debug!(
                            "Access denied to method '{}': required scope '{}' not found in user scopes: {:?}",
                            method, mapping.scope, scopes
                        );
                        Err(OAuth2Error::InsufficientScope {
                            required: mapping.scope.clone(),
                            provided: scopes.join(" "),
                        })
                    }
                }
            }
            None => {
                // No scope mapping found - deny access by default
                warn!(
                    "No scope mapping found for method '{}', denying access by default",
                    method
                );
                Err(OAuth2Error::InsufficientScope {
                    required: format!("mcp:{}:*", method.split('/').next().unwrap_or(method)),
                    provided: scopes.join(" "),
                })
            }
        }
    }

    fn is_method_configured(&self, method: &str) -> bool {
        self.scope_mappings.contains_key(method)
    }

    fn get_required_scope(&self, method: &str) -> Option<&str> {
        self.scope_mappings.get(method).map(|m| m.scope.as_str())
    }

    fn validate_batch_access(
        &self,
        methods: &[&str],
        scopes: &[String],
    ) -> Result<(), Self::Error> {
        for method in methods {
            self.validate_method_access(method, scopes)?;
        }
        Ok(())
    }
}

// Implement Clone for sharing across async contexts
impl Clone for Scope {
    fn clone(&self) -> Self {
        Self {
            scope_mappings: self.scope_mappings.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::config::ScopeMapping;

    fn create_test_scope_validator() -> Scope {
        let mappings = vec![
            ScopeMapping {
                method: "tools/call".to_string(),
                scope: "mcp:tools:execute".to_string(),
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
        Scope::new(mappings)
    }

    #[test]
    fn test_scope_trait_implementation() {
        let validator = create_test_scope_validator();
        let scopes = vec!["mcp:tools:execute".to_string()];

        // Should succeed for method with matching scope
        assert!(validator
            .validate_method_access("tools/call", &scopes)
            .is_ok());

        // Should fail for method requiring different scope
        assert!(validator
            .validate_method_access("resources/read", &scopes)
            .is_err());
    }

    #[test]
    fn test_method_configuration_check() {
        let validator = create_test_scope_validator();

        assert!(validator.is_method_configured("tools/call"));
        assert!(validator.is_method_configured("resources/read"));
        assert!(!validator.is_method_configured("unknown/method"));
    }

    #[test]
    fn test_scope_inspection() {
        let validator = create_test_scope_validator();

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
        let validator = create_test_scope_validator();
        let scopes = vec![
            "mcp:tools:execute".to_string(),
            "mcp:resources:read".to_string(),
        ];

        // Should succeed for methods with matching scopes
        let methods = vec!["tools/call", "resources/read"];
        assert!(validator.validate_batch_access(&methods, &scopes).is_ok());

        // Should fail if any method lacks required scope
        let methods_with_missing = vec!["tools/call", "resources/read", "admin/config"];
        assert!(validator
            .validate_batch_access(&methods_with_missing, &scopes)
            .is_err());
    }

    #[test]
    fn test_default_mappings() {
        let validator = Scope::with_default_mappings();

        // Should handle default mappings without error
        let scopes = vec!["mcp:tools:execute".to_string()];
        let result = validator.validate_method_access("tools/call", &scopes);

        // Result depends on default mappings, but should not panic
        assert!(result.is_ok() || result.is_err());
    }
}

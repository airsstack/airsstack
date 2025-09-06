//! Generic Authorization Context Types
//!
//! Framework-agnostic authorization contexts that work with any authentication
//! mechanism. All contexts are stack-allocated for zero-cost abstractions.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};

// Layer 3: Internal module imports

/// Base trait for authorization contexts
///
/// Marker trait that ensures all authorization contexts can be used
/// in generic authorization policies. Implemented by all concrete context types.
pub trait AuthzContext: Send + Sync + 'static {}

/// No-authorization context for development and internal services
///
/// This context contains no authentication information and is designed
/// to compile to zero overhead when used with NoAuthorizationPolicy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoAuthContext;

impl AuthzContext for NoAuthContext {}

impl NoAuthContext {
    /// Create a new no-auth context (const function for compile-time optimization)
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NoAuthContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Generic scope-based authorization context
///
/// Framework-agnostic context that works with any authentication mechanism
/// that provides scopes or permissions. Can be used with OAuth2, JWT, API keys,
/// or any other authentication system.
#[derive(Debug, Clone)]
pub struct ScopeAuthContext {
    /// Subject identifier (user, service, key ID, etc.)
    subject: String,
    /// Available scopes/permissions for authorization decisions
    scopes: Vec<String>,
    /// Additional metadata for authorization (issuer, audience, etc.)
    metadata: HashMap<String, String>,
    /// Context expiration time (if applicable)
    expires_at: Option<DateTime<Utc>>,
}

impl AuthzContext for ScopeAuthContext {}

impl ScopeAuthContext {
    /// Create a new scope-based authorization context
    ///
    /// # Arguments
    /// * `subject` - Subject identifier (user ID, API key ID, service name, etc.)
    /// * `scopes` - Available scopes/permissions
    /// * `metadata` - Additional authorization metadata
    /// * `expires_at` - Optional expiration time
    pub fn new(
        subject: String,
        scopes: Vec<String>,
        metadata: HashMap<String, String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            subject,
            scopes,
            metadata,
            expires_at,
        }
    }

    /// Create a simple scope context with minimal information
    ///
    /// Useful for testing or simple authorization scenarios.
    pub fn simple(subject: String, scopes: Vec<String>) -> Self {
        Self::new(subject, scopes, HashMap::new(), None)
    }

    /// Get the subject identifier
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Get available scopes
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Get authorization metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Get expiration time
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }

    /// Check if context has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string())
    }

    /// Check if context has wildcard scope for a prefix
    pub fn has_wildcard_scope(&self, prefix: &str) -> bool {
        let wildcard = format!("{prefix}:*");
        self.scopes.contains(&wildcard)
    }

    /// Check if the context is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Add a scope to this context
    pub fn add_scope(&mut self, scope: String) {
        if !self.scopes.contains(&scope) {
            self.scopes.push(scope);
        }
    }

    /// Add metadata to this context
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// Binary authorization context for simple allow/deny scenarios
///
/// Minimal context for systems that only need to know if access
/// is allowed or denied without complex permission checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryAuthContext {
    /// Subject identifier
    subject_id: u64,
    /// Whether access is allowed
    allowed: bool,
}

impl AuthzContext for BinaryAuthContext {}

impl BinaryAuthContext {
    /// Create a new binary authorization context
    ///
    /// # Arguments
    /// * `subject_id` - Numeric subject identifier
    /// * `allowed` - Whether access is allowed
    pub const fn new(subject_id: u64, allowed: bool) -> Self {
        Self {
            subject_id,
            allowed,
        }
    }

    /// Create an allowed context
    pub const fn allowed(subject_id: u64) -> Self {
        Self::new(subject_id, true)
    }

    /// Create a denied context
    pub const fn denied(subject_id: u64) -> Self {
        Self::new(subject_id, false)
    }

    /// Get subject ID
    pub fn subject_id(&self) -> u64 {
        self.subject_id
    }

    /// Check if access is allowed
    pub fn is_allowed(&self) -> bool {
        self.allowed
    }
}

// Implement ScopeContext trait for ScopeAuthContext
use super::policy::ScopeContext;

impl ScopeContext for ScopeAuthContext {
    fn scopes(&self) -> &[String] {
        &self.scopes
    }
}

// Create convenient type aliases that can be used with any authentication system
/// OAuth2 authorization context - alias for ScopeAuthContext
///
/// This allows OAuth2 systems to use the generic scope-based context
/// without creating framework-specific types.
pub type OAuth2AuthContext = ScopeAuthContext;

/// API Key authorization context - alias for ScopeAuthContext
///
/// This allows API key systems to use the generic scope-based context
/// with permissions treated as scopes.
pub type ApiKeyAuthContext = ScopeAuthContext;

/// JWT authorization context - alias for ScopeAuthContext
///
/// This allows JWT systems to use the generic scope-based context
/// with JWT claims treated as scopes.
pub type JwtAuthContext = ScopeAuthContext;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_auth_context() {
        let context = NoAuthContext::new();
        assert_eq!(context, NoAuthContext);

        let default_context = NoAuthContext;
        assert_eq!(context, default_context);
    }

    #[test]
    fn test_scope_auth_context() {
        let scopes = vec!["mcp:read".to_string(), "mcp:write".to_string()];
        let context = ScopeAuthContext::simple("user123".to_string(), scopes);

        assert_eq!(context.subject(), "user123");
        assert_eq!(context.scopes().len(), 2);
        assert!(context.has_scope("mcp:read"));
        assert!(context.has_scope("mcp:write"));
        assert!(!context.has_scope("admin:delete"));
        assert!(!context.is_expired());
    }

    #[test]
    fn test_scope_auth_context_wildcard() {
        let scopes = vec!["mcp:*".to_string()];
        let context = ScopeAuthContext::simple("service123".to_string(), scopes);

        assert!(context.has_wildcard_scope("mcp"));
        assert!(!context.has_wildcard_scope("admin"));
    }

    #[test]
    fn test_scope_auth_context_expiration() {
        let scopes = vec!["test:read".to_string()];
        let expires_at = Some(Utc::now() - chrono::Duration::days(1));
        let context = ScopeAuthContext::new(
            "expired_user".to_string(),
            scopes,
            HashMap::new(),
            expires_at,
        );

        assert!(context.is_expired());
    }

    #[test]
    fn test_scope_auth_context_mutation() {
        let mut context = ScopeAuthContext::simple("user123".to_string(), vec![]);

        context.add_scope("mcp:read".to_string());
        assert!(context.has_scope("mcp:read"));

        context.add_metadata("issuer".to_string(), "auth-service".to_string());
        assert_eq!(
            context.metadata().get("issuer"),
            Some(&"auth-service".to_string())
        );
    }

    #[test]
    fn test_binary_auth_context() {
        let allowed_context = BinaryAuthContext::allowed(123);
        assert_eq!(allowed_context.subject_id(), 123);
        assert!(allowed_context.is_allowed());

        let denied_context = BinaryAuthContext::denied(456);
        assert_eq!(denied_context.subject_id(), 456);
        assert!(!denied_context.is_allowed());
    }

    #[test]
    fn test_type_aliases() {
        // Test that type aliases work correctly
        let oauth2_context = OAuth2AuthContext::simple(
            "oauth2_user".to_string(),
            vec!["mcp:*".to_string()],
        );
        assert_eq!(oauth2_context.subject(), "oauth2_user");

        let apikey_context = ApiKeyAuthContext::simple(
            "key123".to_string(),
            vec!["mcp:read".to_string()],
        );
        assert_eq!(apikey_context.subject(), "key123");

        let jwt_context = JwtAuthContext::simple(
            "jwt_user".to_string(),
            vec!["mcp:admin".to_string()],
        );
        assert_eq!(jwt_context.subject(), "jwt_user");
    }
}

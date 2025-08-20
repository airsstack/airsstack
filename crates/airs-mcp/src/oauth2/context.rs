//! OAuth Authentication Context
//!
//! This module provides authentication context structures that carry
//! validated OAuth 2.1 token information through the request pipeline.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::oauth2::jwt_validator::JwtClaims;

/// Authentication context for OAuth 2.1 authenticated requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// JWT claims from the validated token
    pub claims: JwtClaims,

    /// User's granted scopes
    pub scopes: Vec<String>,

    /// Timestamp when this context was created
    pub created_at: DateTime<Utc>,

    /// Token expiration time (if available)
    pub expires_at: Option<DateTime<Utc>>,

    /// Request ID for audit logging
    pub request_id: Option<String>,

    /// Additional context metadata
    pub metadata: AuthMetadata,
}

/// Additional authentication metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthMetadata {
    /// Client IP address
    pub client_ip: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Custom attributes for extensibility
    pub custom_attributes: HashMap<String, String>,
}

impl AuthContext {
    /// Create a new authentication context from validated JWT claims
    pub fn new(claims: JwtClaims, scopes: Vec<String>) -> Self {
        let expires_at = claims
            .exp
            .map(|exp| DateTime::from_timestamp(exp, 0).unwrap_or_else(|| Utc::now()));

        Self {
            claims,
            scopes,
            created_at: Utc::now(),
            expires_at,
            request_id: None,
            metadata: AuthMetadata::default(),
        }
    }

    /// Create a new authentication context with request ID for audit logging
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add client IP to the context
    pub fn with_client_ip(mut self, client_ip: String) -> Self {
        self.metadata.client_ip = Some(client_ip);
        self
    }

    /// Add User-Agent to the context
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.metadata.user_agent = Some(user_agent);
        self
    }

    /// Add custom attribute to the context
    pub fn with_custom_attribute(mut self, key: String, value: String) -> Self {
        self.metadata.custom_attributes.insert(key, value);
        self
    }

    /// Get the user ID from the token subject
    pub fn user_id(&self) -> &str {
        &self.claims.sub
    }

    /// Get the token audience
    pub fn audience(&self) -> Option<&str> {
        self.claims.aud.as_deref()
    }

    /// Get the token issuer
    pub fn issuer(&self) -> Option<&str> {
        self.claims.iss.as_deref()
    }

    /// Get the JWT ID (jti)
    pub fn jwt_id(&self) -> Option<&str> {
        self.claims.jti.as_deref()
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires_at) => Utc::now() > expires_at,
            None => false, // No expiration time means never expires
        }
    }

    /// Check if the context is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Get time until expiration
    pub fn time_until_expiration(&self) -> Option<Duration> {
        self.expires_at.and_then(|expires_at| {
            let duration = expires_at - Utc::now();
            if duration.num_seconds() > 0 {
                Some(duration)
            } else {
                None // Token is already expired
            }
        })
    }

    /// Check if user has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string())
    }

    /// Check if user has any of the specified scopes
    pub fn has_any_scope(&self, scopes: &[String]) -> bool {
        scopes.iter().any(|scope| self.has_scope(scope))
    }

    /// Check if user has all of the specified scopes
    pub fn has_all_scopes(&self, scopes: &[String]) -> bool {
        scopes.iter().all(|scope| self.has_scope(scope))
    }

    /// Get scopes that match a pattern (e.g., "mcp:tools:*")
    pub fn get_scopes_matching(&self, pattern: &str) -> Vec<&String> {
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            self.scopes
                .iter()
                .filter(|scope| scope.starts_with(prefix))
                .collect()
        } else {
            self.scopes
                .iter()
                .filter(|scope| *scope == pattern)
                .collect()
        }
    }

    /// Create an audit log entry for this authentication context
    pub fn create_audit_entry(&self, action: &str, resource: &str) -> AuditLogEntry {
        AuditLogEntry {
            timestamp: Utc::now(),
            user_id: self.user_id().to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            scopes: self.scopes.clone(),
            client_ip: self.metadata.client_ip.clone(),
            user_agent: self.metadata.user_agent.clone(),
            request_id: self.request_id.clone(),
            jwt_id: self.jwt_id().map(|s| s.to_string()),
            success: true, // Will be updated based on operation result
        }
    }

    /// Convert to a summary for logging (without sensitive data)
    pub fn to_log_summary(&self) -> AuthContextSummary {
        AuthContextSummary {
            user_id: self.user_id().to_string(),
            scopes: self.scopes.clone(),
            expires_at: self.expires_at,
            client_ip: self.metadata.client_ip.clone(),
            request_id: self.request_id.clone(),
        }
    }
}

/// Audit log entry for authentication events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub scopes: Vec<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub jwt_id: Option<String>,
    pub success: bool,
}

/// Authentication context summary for logging (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContextSummary {
    pub user_id: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub client_ip: Option<String>,
    pub request_id: Option<String>,
}

/// Trait for extracting authentication context from request extensions
pub trait AuthContextExt {
    /// Extract the authentication context from the request
    fn auth_context(&self) -> Option<&AuthContext>;

    /// Extract the authentication context mutably
    fn auth_context_mut(&mut self) -> Option<&mut AuthContext>;
}

// Implementation for Axum's request extensions
impl AuthContextExt for axum::http::Extensions {
    fn auth_context(&self) -> Option<&AuthContext> {
        self.get::<AuthContext>()
    }

    fn auth_context_mut(&mut self) -> Option<&mut AuthContext> {
        self.get_mut::<AuthContext>()
    }
}

/// Helper macros for working with authentication context
#[macro_export]
macro_rules! require_auth {
    ($extensions:expr) => {
        $extensions
            .auth_context()
            .ok_or_else(|| $crate::oauth2::error::OAuth2Error::MissingAuthorization)?
    };
}

#[macro_export]
macro_rules! require_scope {
    ($context:expr, $scope:expr) => {
        if !$context.has_scope($scope) {
            return Err($crate::oauth2::error::OAuth2Error::InsufficientScope {
                required: $scope.to_string(),
                provided: $context.scopes.join(" "),
            });
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::jwt_validator::JwtClaims;

    fn create_test_claims() -> JwtClaims {
        JwtClaims {
            sub: "user123".to_string(),
            aud: Some("mcp-server".to_string()),
            iss: Some("https://auth.example.com".to_string()),
            exp: Some(
                Utc::now().timestamp() + 3600, // Expires in 1 hour
            ),
            nbf: None,
            iat: None,
            jti: Some("jwt-123".to_string()),
            scope: Some("mcp:tools:execute mcp:resources:read".to_string()),
            scopes: None,
        }
    }

    #[test]
    fn test_auth_context_creation() {
        let claims = create_test_claims();
        let scopes = vec![
            "mcp:tools:execute".to_string(),
            "mcp:resources:read".to_string(),
        ];

        let context = AuthContext::new(claims.clone(), scopes.clone());

        assert_eq!(context.user_id(), "user123");
        assert_eq!(context.audience(), Some("mcp-server"));
        assert_eq!(context.issuer(), Some("https://auth.example.com"));
        assert_eq!(context.jwt_id(), Some("jwt-123"));
        assert_eq!(context.scopes, scopes);
        assert!(!context.is_expired());
    }

    #[test]
    fn test_auth_context_builders() {
        let claims = create_test_claims();
        let scopes = vec!["mcp:tools:execute".to_string()];

        let context = AuthContext::new(claims, scopes)
            .with_request_id("req-123".to_string())
            .with_client_ip("192.168.1.1".to_string())
            .with_user_agent("TestAgent/1.0".to_string())
            .with_custom_attribute("tenant".to_string(), "example-org".to_string());

        assert_eq!(context.request_id, Some("req-123".to_string()));
        assert_eq!(context.metadata.client_ip, Some("192.168.1.1".to_string()));
        assert_eq!(
            context.metadata.user_agent,
            Some("TestAgent/1.0".to_string())
        );
        assert_eq!(
            context.metadata.custom_attributes.get("tenant"),
            Some(&"example-org".to_string())
        );
    }

    #[test]
    fn test_scope_checking() {
        let claims = create_test_claims();
        let scopes = vec![
            "mcp:tools:execute".to_string(),
            "mcp:resources:read".to_string(),
            "mcp:admin:all".to_string(),
        ];

        let context = AuthContext::new(claims, scopes);

        // Test individual scope checking
        assert!(context.has_scope("mcp:tools:execute"));
        assert!(context.has_scope("mcp:resources:read"));
        assert!(!context.has_scope("mcp:tools:admin"));

        // Test any scope checking
        assert!(context.has_any_scope(&vec![
            "mcp:tools:execute".to_string(),
            "mcp:unknown:scope".to_string(),
        ]));
        assert!(!context.has_any_scope(&vec!["mcp:unknown:scope".to_string()]));

        // Test all scopes checking
        assert!(context.has_all_scopes(&vec![
            "mcp:tools:execute".to_string(),
            "mcp:resources:read".to_string(),
        ]));
        assert!(!context.has_all_scopes(&vec![
            "mcp:tools:execute".to_string(),
            "mcp:unknown:scope".to_string(),
        ]));
    }

    #[test]
    fn test_scope_pattern_matching() {
        let claims = create_test_claims();
        let scopes = vec![
            "mcp:tools:execute".to_string(),
            "mcp:tools:read".to_string(),
            "mcp:resources:read".to_string(),
        ];

        let context = AuthContext::new(claims, scopes);

        // Test pattern matching
        let tools_scopes = context.get_scopes_matching("mcp:tools:*");
        assert_eq!(tools_scopes.len(), 2);
        assert!(tools_scopes.contains(&&"mcp:tools:execute".to_string()));
        assert!(tools_scopes.contains(&&"mcp:tools:read".to_string()));

        let exact_scope = context.get_scopes_matching("mcp:resources:read");
        assert_eq!(exact_scope.len(), 1);
        assert!(exact_scope.contains(&&"mcp:resources:read".to_string()));
    }

    #[test]
    fn test_expiration_checking() {
        let mut claims = create_test_claims();

        // Test with expired token
        claims.exp = Some(
            Utc::now().timestamp() - 3600, // Expired 1 hour ago
        );

        let context = AuthContext::new(claims, vec![]);
        assert!(context.is_expired());
        assert!(!context.is_valid());
        assert!(context.time_until_expiration().is_none());
    }

    #[test]
    fn test_audit_log_entry() {
        let claims = create_test_claims();
        let scopes = vec!["mcp:tools:execute".to_string()];

        let context = AuthContext::new(claims, scopes)
            .with_request_id("req-123".to_string())
            .with_client_ip("192.168.1.1".to_string());

        let audit_entry = context.create_audit_entry("tools/call", "calculator");

        assert_eq!(audit_entry.user_id, "user123");
        assert_eq!(audit_entry.action, "tools/call");
        assert_eq!(audit_entry.resource, "calculator");
        assert_eq!(audit_entry.request_id, Some("req-123".to_string()));
        assert_eq!(audit_entry.client_ip, Some("192.168.1.1".to_string()));
        assert_eq!(audit_entry.jwt_id, Some("jwt-123".to_string()));
        assert!(audit_entry.success);
    }

    #[test]
    fn test_log_summary() {
        let claims = create_test_claims();
        let scopes = vec!["mcp:tools:execute".to_string()];

        let context = AuthContext::new(claims, scopes.clone())
            .with_request_id("req-123".to_string())
            .with_client_ip("192.168.1.1".to_string());

        let summary = context.to_log_summary();

        assert_eq!(summary.user_id, "user123");
        assert_eq!(summary.scopes, scopes);
        assert_eq!(summary.request_id, Some("req-123".to_string()));
        assert_eq!(summary.client_ip, Some("192.168.1.1".to_string()));
        // JWT claims and other sensitive data should not be in summary
    }
}

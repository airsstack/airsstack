//! Main OAuth2 Validator Implementation
//!
//! Provides unified OAuth2 validation through composition of JWT and scope
//! validators using zero-cost abstractions following workspace standards.

// Layer 1: Standard library imports
// (none for this module)

// Layer 2: Third-party crate imports
// (none for this module)

// Layer 3: Internal module imports
use super::{JwtValidator, ScopeValidator};
use crate::oauth2::{context::AuthContext, error::OAuth2Result, types::JwtClaims};

/// Main OAuth2 validator using zero-cost generic composition
///
/// Combines JWT and scope validation through trait composition following
/// workspace standards §1 (Generic type usage) and §3 (Stack allocation).
///
/// # Type Parameters
/// * `J` - JWT validator implementation (must implement `JwtValidator`)
/// * `S` - Scope validator implementation (must implement `ScopeValidator`)
///
/// # Examples
///
/// ```rust
/// use airs_mcp::oauth2::validator::{Validator, Jwt, Scope};
/// use airs_mcp::oauth2::config::OAuth2Config;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OAuth2Config::default();
///
/// // Create with concrete types - zero runtime cost
/// let jwt = Jwt::new(config)?;
/// let scope = Scope::with_default_mappings();
/// let validator = Validator::new(jwt, scope);
///
/// // Validator is ready for use
/// // let auth_context = validator.validate_request("jwt_token", "tools/call").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Validator<J, S>
where
    J: JwtValidator,
    S: ScopeValidator,
{
    /// JWT validator for token validation and claims extraction
    jwt: J,
    /// Scope validator for method authorization
    scope: S,
}

impl<J, S> Validator<J, S>
where
    J: JwtValidator,
    S: ScopeValidator,
{
    /// Create new validator with JWT and scope validators
    ///
    /// Following workspace standards §1, this uses const fn for compile-time
    /// optimization when possible.
    ///
    /// # Arguments
    /// * `jwt` - JWT validator instance
    /// * `scope` - Scope validator instance
    ///
    /// # Returns
    /// * New validator instance with zero-cost composition
    pub const fn new(jwt: J, scope: S) -> Self {
        Self { jwt, scope }
    }

    /// Validate OAuth2 request with JWT token and MCP method authorization
    ///
    /// This is the main entry point for OAuth2 validation, combining JWT
    /// token validation with scope-based method authorization.
    ///
    /// # Arguments
    /// * `token` - JWT Bearer token from Authorization header
    /// * `method` - MCP method being accessed (e.g., "tools/call")
    ///
    /// # Returns
    /// * `Ok(AuthContext)` - Successfully authenticated and authorized request
    /// * `Err(OAuth2Error)` - Authentication or authorization failed
    ///
    /// # Process
    /// 1. Validate JWT token and extract claims
    /// 2. Extract OAuth scopes from token claims
    /// 3. Validate scopes against required method permissions
    /// 4. Create AuthContext with user information
    pub async fn validate_request(&self, token: &str, method: &str) -> OAuth2Result<AuthContext> {
        // Step 1: Validate JWT token
        let claims = self.jwt.validate(token).await.map_err(Into::into)?;

        // Step 2: Extract scopes from token
        let scopes = self.jwt.extract_scopes(&claims);

        // Step 3: Validate method access
        self.scope
            .validate_method_access(method, &scopes)
            .map_err(Into::into)?;

        // Step 4: Create authenticated context
        let scopes_for_context = scopes.clone();
        Ok(AuthContext::new(claims, scopes_for_context))
    }

    /// Validate JWT token only (without scope checking)
    ///
    /// Useful for scenarios where you need token validation but method-specific
    /// authorization is handled separately.
    ///
    /// # Arguments
    /// * `token` - JWT Bearer token to validate
    ///
    /// # Returns
    /// * `Ok((JwtClaims, Vec<String>))` - Validated claims and extracted scopes
    /// * `Err(OAuth2Error)` - Token validation failed
    pub async fn validate_token_only(&self, token: &str) -> OAuth2Result<(JwtClaims, Vec<String>)> {
        let claims = self.jwt.validate(token).await.map_err(Into::into)?;
        let scopes = self.jwt.extract_scopes(&claims);
        Ok((claims, scopes))
    }

    /// Validate method access only (assuming token already validated)
    ///
    /// Useful for scenarios where JWT validation is cached or handled upstream
    /// but method authorization needs to be checked.
    ///
    /// # Arguments
    /// * `method` - MCP method being accessed
    /// * `scopes` - User's OAuth scopes (from JWT token)
    ///
    /// # Returns
    /// * `Ok(())` - User has sufficient permissions
    /// * `Err(OAuth2Error)` - Insufficient permissions
    pub fn validate_method_only(&self, method: &str, scopes: &[String]) -> OAuth2Result<()> {
        self.scope
            .validate_method_access(method, scopes)
            .map_err(Into::into)
    }

    /// Batch validate multiple methods for efficiency
    ///
    /// When checking access to multiple methods simultaneously, this can be
    /// more efficient than individual calls.
    ///
    /// # Arguments
    /// * `token` - JWT Bearer token to validate
    /// * `methods` - Multiple MCP methods to check
    ///
    /// # Returns
    /// * `Ok(AuthContext)` - User has access to all methods
    /// * `Err(OAuth2Error)` - Authentication failed or missing permissions
    pub async fn validate_batch_request(
        &self,
        token: &str,
        methods: &[&str],
    ) -> OAuth2Result<AuthContext> {
        // Validate token once
        let claims = self.jwt.validate(token).await.map_err(Into::into)?;
        let scopes = self.jwt.extract_scopes(&claims);

        // Batch validate all methods
        self.scope
            .validate_batch_access(methods, &scopes)
            .map_err(Into::into)?;

        let scopes_for_context = scopes.clone();
        Ok(AuthContext::new(claims, scopes_for_context))
    }

    /// Get reference to JWT validator
    ///
    /// Provides access to underlying JWT validator for advanced scenarios.
    /// Use sparingly to maintain abstraction.
    pub fn jwt_validator(&self) -> &J {
        &self.jwt
    }

    /// Get reference to scope validator
    ///
    /// Provides access to underlying scope validator for advanced scenarios.
    /// Use sparingly to maintain abstraction.
    pub fn scope_validator(&self) -> &S {
        &self.scope
    }

    /// Check if method is configured in scope mappings
    ///
    /// Useful for validating requests before processing or providing
    /// better error messages.
    pub fn is_method_configured(&self, method: &str) -> bool {
        self.scope.is_method_configured(method)
    }

    /// Get required scope for a method
    ///
    /// Useful for error messages or API documentation generation.
    pub fn get_required_scope(&self, method: &str) -> Option<&str> {
        self.scope.get_required_scope(method)
    }
}

// Implement Clone when both validators are cloneable
impl<J, S> Clone for Validator<J, S>
where
    J: JwtValidator + Clone,
    S: ScopeValidator + Clone,
{
    fn clone(&self) -> Self {
        Self {
            jwt: self.jwt.clone(),
            scope: self.scope.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::error::OAuth2Error;

    // Mock implementations for testing
    struct MockJwtValidator {
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl JwtValidator for MockJwtValidator {
        type Error = OAuth2Error;

        async fn validate(&self, _token: &str) -> Result<JwtClaims, Self::Error> {
            if self.should_fail {
                Err(OAuth2Error::InvalidToken("Mock failure".to_string()))
            } else {
                Ok(JwtClaims {
                    sub: "test_user".to_string(),
                    scope: Some("mcp:tools:execute mcp:resources:read".to_string()),
                    scopes: None,
                    aud: None,
                    iss: None,
                    exp: None,
                    nbf: None,
                    iat: None,
                    jti: None,
                })
            }
        }

        fn extract_scopes(&self, claims: &JwtClaims) -> Vec<String> {
            claims
                .scope
                .as_deref()
                .unwrap_or("")
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        }
    }

    struct MockScopeValidator {
        should_fail: bool,
    }

    impl ScopeValidator for MockScopeValidator {
        type Error = OAuth2Error;

        fn validate_method_access(
            &self,
            method: &str,
            _scopes: &[String],
        ) -> Result<(), Self::Error> {
            if self.should_fail || method == "forbidden/method" {
                Err(OAuth2Error::InsufficientScope {
                    required: "test:scope".to_string(),
                    provided: "none".to_string(),
                })
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_successful_validation() {
        let jwt = MockJwtValidator { should_fail: false };
        let scope = MockScopeValidator { should_fail: false };
        let validator = Validator::new(jwt, scope);

        let result = validator
            .validate_request("valid_token", "tools/call")
            .await;
        assert!(result.is_ok());

        let auth_context = result.unwrap();
        assert_eq!(auth_context.user_id(), "test_user");
    }

    #[tokio::test]
    async fn test_jwt_validation_failure() {
        let jwt = MockJwtValidator { should_fail: true };
        let scope = MockScopeValidator { should_fail: false };
        let validator = Validator::new(jwt, scope);

        let result = validator
            .validate_request("invalid_token", "tools/call")
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            OAuth2Error::InvalidToken(_) => {} // Expected
            _ => panic!("Expected InvalidToken error"),
        }
    }

    #[tokio::test]
    async fn test_scope_validation_failure() {
        let jwt = MockJwtValidator { should_fail: false };
        let scope = MockScopeValidator { should_fail: false };
        let validator = Validator::new(jwt, scope);

        let result = validator
            .validate_request("valid_token", "forbidden/method")
            .await;
        assert!(result.is_err());

        match result.unwrap_err() {
            OAuth2Error::InsufficientScope { .. } => {} // Expected
            _ => panic!("Expected InsufficientScope error"),
        }
    }

    #[tokio::test]
    async fn test_token_only_validation() {
        let jwt = MockJwtValidator { should_fail: false };
        let scope = MockScopeValidator { should_fail: false };
        let validator = Validator::new(jwt, scope);

        let result = validator.validate_token_only("valid_token").await;
        assert!(result.is_ok());

        let (claims, scopes) = result.unwrap();
        assert_eq!(claims.sub, "test_user");
        assert_eq!(scopes, vec!["mcp:tools:execute", "mcp:resources:read"]);
    }

    #[test]
    fn test_method_only_validation() {
        let jwt = MockJwtValidator { should_fail: false };
        let scope = MockScopeValidator { should_fail: false };
        let validator = Validator::new(jwt, scope);

        let scopes = vec!["mcp:tools:execute".to_string()];

        // Should succeed for allowed method
        assert!(validator
            .validate_method_only("tools/call", &scopes)
            .is_ok());

        // Should fail for forbidden method
        assert!(validator
            .validate_method_only("forbidden/method", &scopes)
            .is_err());
    }

    #[tokio::test]
    async fn test_batch_validation() {
        let jwt = MockJwtValidator { should_fail: false };
        let scope = MockScopeValidator { should_fail: false };
        let validator = Validator::new(jwt, scope);

        let methods = vec!["tools/call", "resources/read"];
        let result = validator
            .validate_batch_request("valid_token", &methods)
            .await;
        assert!(result.is_ok());

        // Should fail if any method is forbidden
        let methods_with_forbidden = vec!["tools/call", "forbidden/method"];
        let result = validator
            .validate_batch_request("valid_token", &methods_with_forbidden)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_cost_composition() {
        // This test validates that our generic composition compiles correctly
        // and provides the expected interface

        let jwt = MockJwtValidator { should_fail: false };
        let scope = MockScopeValidator { should_fail: false };
        let validator = Validator::new(jwt, scope);

        // Should provide access to underlying validators
        let _jwt_ref = validator.jwt_validator();
        let _scope_ref = validator.scope_validator();

        // Should provide utility methods
        assert!(validator.is_method_configured("any_method"));
        assert!(validator.get_required_scope("any_method").is_none());
    }
}

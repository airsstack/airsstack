//! OAuth2 Authentication Strategy Implementation
//!
//! OAuth2 strategy that directly integrates with the existing OAuth2 validation
//! infrastructure using oauth2::validator::Validator<J, S> without wrapper abstractions.

// Layer 1: Standard library imports

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::request::OAuth2Request;
use crate::authentication::context::AuthContext;
use crate::authentication::error::{AuthError, AuthResult};
use crate::authentication::method::AuthMethod;
use crate::authentication::request::AuthRequest;
use crate::authentication::strategy::AuthenticationStrategy;
use crate::oauth2::validator::{JwtValidator, ScopeValidator, Validator};

/// OAuth2 authentication strategy
///
/// Directly uses oauth2::validator::Validator<J, S> for token validation
/// and scope checking, providing clean integration with existing OAuth2 infrastructure.
///
/// # Type Parameters
/// * `J` - JWT validator implementation (must implement JwtValidator)
/// * `S` - Scope validator implementation (must implement ScopeValidator)
///
/// # Examples
///
/// ```rust
/// use airs_mcp::authentication::strategies::oauth2::{OAuth2Strategy, OAuth2Request, OAuth2AuthRequest};
/// use airs_mcp::oauth2::validator::{Validator, Jwt, Scope};
/// use airs_mcp::oauth2::config::OAuth2Config;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OAuth2Config::default();
///
/// // Create OAuth2 validator with concrete types
/// let jwt = Jwt::new(config)?;
/// let scope = Scope::with_default_mappings();
/// let validator = Validator::new(jwt, scope);
///
/// // Create OAuth2 strategy
/// let strategy = OAuth2Strategy::new(validator);
///
/// // Use strategy for authentication
/// let oauth2_request = OAuth2Request::new("bearer_token".to_string())
///     .with_method("tools/call".to_string());
/// let auth_request = OAuth2AuthRequest::new(oauth2_request);
///
/// // let auth_context = strategy.authenticate(&auth_request).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct OAuth2Strategy<J, S>
where
    J: JwtValidator,
    S: ScopeValidator,
{
    /// OAuth2 validator for JWT and scope validation
    validator: Validator<J, S>,
}

impl<J, S> OAuth2Strategy<J, S>
where
    J: JwtValidator,
    S: ScopeValidator,
{
    /// Create new OAuth2 strategy with validator
    ///
    /// # Arguments
    /// * `validator` - OAuth2 validator instance for JWT and scope validation
    ///
    /// # Returns
    /// * New OAuth2Strategy instance ready for authentication
    pub fn new(validator: Validator<J, S>) -> Self {
        Self { validator }
    }

    /// Get reference to underlying OAuth2 validator
    ///
    /// Provides access to the OAuth2 validator for advanced scenarios.
    /// Use sparingly to maintain abstraction boundaries.
    pub fn validator(&self) -> &Validator<J, S> {
        &self.validator
    }
}

#[async_trait]
impl<J, S> AuthenticationStrategy<OAuth2Request, crate::oauth2::context::AuthContext>
    for OAuth2Strategy<J, S>
where
    J: JwtValidator + Send + Sync + 'static,
    S: ScopeValidator + Send + Sync + 'static,
{
    fn method(&self) -> AuthMethod {
        AuthMethod::new("oauth2")
    }

    async fn authenticate(
        &self,
        request: &impl AuthRequest<OAuth2Request>,
    ) -> AuthResult<AuthContext<crate::oauth2::context::AuthContext>> {
        let oauth2_req = request.inner();

        // Perform OAuth2 validation using existing infrastructure
        let oauth2_auth_context = if let Some(method) = &oauth2_req.method {
            // Validate both token and method access together
            self.validator
                .validate_request(&oauth2_req.bearer_token, method)
                .await
                .map_err(|e| {
                    AuthError::InvalidCredentials(format!("OAuth2 validation failed: {e}"))
                })?
        } else {
            // Token-only validation, convert to AuthContext
            let (claims, scopes) = self
                .validator
                .validate_token_only(&oauth2_req.bearer_token)
                .await
                .map_err(|e| {
                    AuthError::InvalidCredentials(format!("OAuth2 token validation failed: {e}"))
                })?;

            crate::oauth2::context::AuthContext::new(claims, scopes)
        };

        // Create authentication context with OAuth2 data
        let mut auth_context = AuthContext::new(AuthMethod::new("oauth2"), oauth2_auth_context);

        // Add metadata from request to auth context
        for (key, value) in &oauth2_req.metadata {
            auth_context = auth_context.add_metadata(key, value);
        }

        Ok(auth_context)
    }

    async fn validate(
        &self,
        context: &AuthContext<crate::oauth2::context::AuthContext>,
    ) -> AuthResult<bool> {
        // Extract token from OAuth2 auth context and validate
        // Note: oauth2::context::AuthContext doesn't store the original token,
        // so we need to check expiration and validity through claims
        let oauth2_context = &context.auth_data;

        // Check if context is still valid (not expired)
        Ok(oauth2_context.is_valid())
    }
}

// Implement Clone when both validators are cloneable
impl<J, S> Clone for OAuth2Strategy<J, S>
where
    J: JwtValidator + Clone,
    S: ScopeValidator + Clone,
{
    fn clone(&self) -> Self {
        Self {
            validator: self.validator.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::{error::OAuth2Error, types::JwtClaims, validator::Validator};

    // Mock implementations for testing
    #[derive(Clone)]
    struct MockJwtValidator {
        should_fail: bool,
    }

    #[async_trait]
    impl JwtValidator for MockJwtValidator {
        type Error = OAuth2Error;

        async fn validate(&self, _token: &str) -> Result<JwtClaims, Self::Error> {
            if self.should_fail {
                Err(OAuth2Error::InvalidToken(
                    "Mock JWT validation failure".to_string(),
                ))
            } else {
                Ok(JwtClaims {
                    sub: "test_user_123".to_string(),
                    scope: Some("mcp:tools:execute mcp:resources:read".to_string()),
                    scopes: None,
                    aud: Some("test_audience".to_string()),
                    iss: Some("https://auth.example.com".to_string()),
                    exp: Some(chrono::Utc::now().timestamp() + 3600), // 1 hour from now
                    nbf: None,
                    iat: Some(chrono::Utc::now().timestamp()),
                    jti: Some("test_jwt_id".to_string()),
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
    #[derive(Clone)]
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
                    required: "admin:access".to_string(),
                    provided: "user:basic".to_string(),
                })
            } else {
                Ok(())
            }
        }

        fn is_method_configured(&self, _method: &str) -> bool {
            true
        }

        fn get_required_scope(&self, _method: &str) -> Option<&str> {
            Some("mcp:tools:execute")
        }
    }

    fn create_test_strategy(
        jwt_fail: bool,
        scope_fail: bool,
    ) -> OAuth2Strategy<MockJwtValidator, MockScopeValidator> {
        let jwt = MockJwtValidator {
            should_fail: jwt_fail,
        };
        let scope = MockScopeValidator {
            should_fail: scope_fail,
        };
        let validator = Validator::new(jwt, scope);
        OAuth2Strategy::new(validator)
    }

    #[tokio::test]
    async fn test_oauth2_strategy_successful_authentication() {
        let strategy = create_test_strategy(false, false);

        let oauth2_request = OAuth2Request::new("valid_bearer_token".to_string())
            .with_method("tools/call".to_string())
            .with_metadata("client_ip", "192.168.1.1");

        let auth_request = super::super::request::OAuth2AuthRequest::new(oauth2_request);

        let result = strategy.authenticate(&auth_request).await;
        assert!(result.is_ok());

        let auth_context = result.unwrap();
        assert_eq!(auth_context.method.as_str(), "oauth2");
        assert_eq!(auth_context.auth_data.user_id(), "test_user_123");

        // Verify metadata was preserved
        assert_eq!(
            auth_context.metadata.get("client_ip"),
            Some(&"192.168.1.1".to_string())
        );
    }

    #[tokio::test]
    async fn test_oauth2_strategy_token_only_authentication() {
        let strategy = create_test_strategy(false, false);

        // Request without method - should do token-only validation
        let oauth2_request = OAuth2Request::new("valid_bearer_token".to_string())
            .with_metadata("user_agent", "test-client/1.0");

        let auth_request = super::super::request::OAuth2AuthRequest::new(oauth2_request);

        let result = strategy.authenticate(&auth_request).await;
        assert!(result.is_ok());

        let auth_context = result.unwrap();
        assert_eq!(auth_context.method.as_str(), "oauth2");
        assert_eq!(auth_context.auth_data.user_id(), "test_user_123");
        assert_eq!(
            auth_context.metadata.get("user_agent"),
            Some(&"test-client/1.0".to_string())
        );
    }

    #[tokio::test]
    async fn test_oauth2_strategy_jwt_validation_failure() {
        let strategy = create_test_strategy(true, false); // JWT validation fails

        let oauth2_request = OAuth2Request::new("invalid_bearer_token".to_string());
        let auth_request = super::super::request::OAuth2AuthRequest::new(oauth2_request);

        let result = strategy.authenticate(&auth_request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AuthError::InvalidCredentials(msg) => {
                assert!(msg.contains("OAuth2"));
                assert!(msg.contains("validation failed"));
            }
            _ => panic!("Expected InvalidCredentials error"),
        }
    }

    #[tokio::test]
    async fn test_oauth2_strategy_scope_validation_failure() {
        let strategy = create_test_strategy(false, true); // Scope validation fails

        let oauth2_request = OAuth2Request::new("valid_bearer_token".to_string())
            .with_method("forbidden/method".to_string());

        let auth_request = super::super::request::OAuth2AuthRequest::new(oauth2_request);

        let result = strategy.authenticate(&auth_request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AuthError::InvalidCredentials(msg) => {
                assert!(msg.contains("OAuth2"));
                assert!(msg.contains("validation failed"));
            }
            _ => panic!("Expected InvalidCredentials error"),
        }
    }

    #[tokio::test]
    async fn test_oauth2_strategy_validate() {
        let strategy = create_test_strategy(false, false);

        // Create a valid auth context first
        let oauth2_request = OAuth2Request::new("valid_bearer_token".to_string());
        let auth_request = super::super::request::OAuth2AuthRequest::new(oauth2_request);
        let auth_context = strategy.authenticate(&auth_request).await.unwrap();

        // Validate the context
        let is_valid = strategy.validate(&auth_context).await.unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_oauth2_strategy_method() {
        let strategy = create_test_strategy(false, false);
        assert_eq!(strategy.method().as_str(), "oauth2");
    }

    #[test]
    fn test_oauth2_strategy_clone() {
        let strategy = create_test_strategy(false, false);
        let cloned_strategy = strategy.clone();
        assert_eq!(
            strategy.method().as_str(),
            cloned_strategy.method().as_str()
        );
    }
}

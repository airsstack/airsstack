//! Authentication Manager
//!
//! Simple generic authentication manager with zero-cost abstractions.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use crate::authentication::context::AuthContext;
use crate::authentication::error::{AuthError, AuthResult};
use crate::authentication::method::AuthMethod;
use crate::authentication::request::AuthRequest;
use crate::authentication::strategy::AuthenticationStrategy;

/// Default authentication timeout
pub const DEFAULT_AUTH_TIMEOUT: Duration = Duration::from_secs(30);

/// Default audit logging state
pub const DEFAULT_ENABLE_AUDIT_LOGGING: bool = true;

/// Generic authentication manager with single strategy
///
/// Pure generic implementation with compile-time dispatch.
/// No dynamic allocation or trait objects.
pub struct AuthenticationManager<S, T, D>
where
    S: AuthenticationStrategy<T, D>,
    T: Send + Sync,
    D: Send + Sync + 'static,
{
    strategy: S,
    config: ManagerConfig,
    _phantom: std::marker::PhantomData<(T, D)>,
}

impl<S, T, D> AuthenticationManager<S, T, D>
where
    S: AuthenticationStrategy<T, D>,
    T: Send + Sync,
    D: Send + Sync + 'static,
{
    /// Create authentication manager with strategy
    pub fn new(strategy: S) -> Self {
        Self {
            strategy,
            config: ManagerConfig::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set manager configuration
    pub fn with_config(mut self, config: ManagerConfig) -> Self {
        self.config = config;
        self
    }

    /// Get authentication method handled by this manager
    pub fn method(&self) -> AuthMethod {
        self.strategy.method()
    }

    /// Authenticate a request using the configured strategy
    pub async fn authenticate(&self, request: &impl AuthRequest<T>) -> AuthResult<AuthContext<D>> {
        // Apply timeout if configured
        let auth_future = self.strategy.authenticate(request);

        if let Some(timeout) = self.config.timeout {
            tokio::time::timeout(timeout, auth_future)
                .await
                .map_err(|_| AuthError::Timeout)?
        } else {
            auth_future.await
        }
    }

    /// Validate an existing authentication context
    pub async fn validate(&self, context: &AuthContext<D>) -> AuthResult<bool> {
        // Check if this manager can handle the auth method
        if context.method != self.strategy.method() {
            return Ok(false);
        }

        // Check expiration
        if context.is_expired() {
            return Ok(false);
        }

        // Delegate to strategy for detailed validation
        self.strategy.validate(context).await
    }

    /// Get strategy reference (for advanced usage)
    pub fn strategy(&self) -> &S {
        &self.strategy
    }
}

/// Authentication manager configuration
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    /// Authentication timeout
    pub timeout: Option<Duration>,

    /// Enable detailed audit logging
    pub enable_audit_logging: bool,

    /// Custom manager attributes
    pub custom_attributes: HashMap<String, String>,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagerConfig {
    /// Create new configuration with defaults
    pub fn new() -> Self {
        Self {
            timeout: Some(DEFAULT_AUTH_TIMEOUT),
            enable_audit_logging: DEFAULT_ENABLE_AUDIT_LOGGING,
            custom_attributes: HashMap::new(),
        }
    }

    /// Create new configuration with defaults (alias for new)
    pub fn default() -> Self {
        Self::new()
    }

    /// Set authentication timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Disable authentication timeout
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Enable/disable audit logging
    pub fn with_audit_logging(mut self, enable: bool) -> Self {
        self.enable_audit_logging = enable;
        self
    }

    /// Add custom attribute
    pub fn add_attribute<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.custom_attributes.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authentication::context::AuthContext;
    use crate::authentication::method::AuthMethod;
    use async_trait::async_trait;

    #[derive(Debug, Clone)]
    struct TestStrategy;

    #[derive(Debug, Clone)]
    struct TestData {
        user_id: String,
    }

    #[derive(Debug)]
    struct TestRequest;

    impl crate::authentication::request::AuthRequest<TestRequest> for TestRequest {
        fn custom_attribute(&self, _key: &str) -> Option<String> {
            None
        }

        fn custom_attributes(&self) -> HashMap<String, String> {
            HashMap::new()
        }

        fn inner(&self) -> &TestRequest {
            self
        }
    }

    #[async_trait]
    impl AuthenticationStrategy<TestRequest, TestData> for TestStrategy {
        fn method(&self) -> AuthMethod {
            AuthMethod::new("test")
        }

        async fn authenticate(
            &self,
            _request: &impl crate::authentication::request::AuthRequest<TestRequest>,
        ) -> AuthResult<AuthContext<TestData>> {
            Ok(AuthContext::new(
                self.method(),
                TestData {
                    user_id: "test_user".to_string(),
                },
            ))
        }

        async fn validate(&self, _context: &AuthContext<TestData>) -> AuthResult<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_authentication_manager() {
        let strategy = TestStrategy;
        let manager = AuthenticationManager::new(strategy);
        let request = TestRequest;

        let context = manager.authenticate(&request).await.unwrap();
        assert_eq!(context.method.as_str(), "test");
        assert_eq!(context.auth_data.user_id, "test_user");

        let is_valid = manager.validate(&context).await.unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_manager_config() {
        let config = ManagerConfig::new()
            .with_timeout(Duration::from_secs(60))
            .with_audit_logging(false)
            .add_attribute("test", "value");

        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
        assert!(!config.enable_audit_logging);
        assert_eq!(
            config.custom_attributes.get("test"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_AUTH_TIMEOUT, Duration::from_secs(30));
        assert!(DEFAULT_ENABLE_AUDIT_LOGGING);
    }
}

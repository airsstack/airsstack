//! Authorization Policy Traits
//!
//! Zero-cost generic authorization policies that use compile-time specialization
//! instead of runtime dispatch. Each policy is a concrete type that gets inlined
//! by the compiler for maximum performance.

// Layer 1: Standard library imports
use std::marker::PhantomData;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use super::context::AuthzContext;
use super::error::{AuthzError, AuthzResult};

/// Zero-cost generic authorization policy trait
///
/// This trait uses pure generics without any `dyn` patterns to ensure
/// zero runtime dispatch. Each implementation is specialized at compile time.
///
/// # Type Parameters
/// * `C` - Authorization context type (stack-allocated)
/// * `R` - Request type containing method and metadata
pub trait AuthorizationPolicy<C, R>: Send + Sync + 'static
where
    C: AuthzContext,
    R: Send + Sync,
{
    /// Check if a request is authorized given an authentication context
    ///
    /// This method is inlined at compile time for zero-cost abstractions.
    ///
    /// # Arguments
    /// * `context` - Authentication context (stack-allocated)
    /// * `request` - Request containing method and metadata
    ///
    /// # Returns
    /// * `Ok(())` if authorized
    /// * `Err(AuthzError)` if denied or error occurred
    fn authorize(&self, context: &C, request: &R) -> AuthzResult<()>;

    /// Get policy name for debugging and logging
    fn policy_name(&self) -> &'static str;
}

/// No-op authorization policy that always allows access
///
/// This policy compiles to zero code in release builds when used with NoAuthContext.
/// Perfect for development environments or internal services that don't need authorization.
#[derive(Debug, Clone, Copy)]
pub struct NoAuthorizationPolicy<C> {
    _phantom: PhantomData<C>,
}

impl<C> NoAuthorizationPolicy<C> {
    /// Create a new no-authorization policy
    ///
    /// This is a const function to enable compile-time initialization.
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<C> Default for NoAuthorizationPolicy<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, R> AuthorizationPolicy<C, R> for NoAuthorizationPolicy<C>
where
    C: AuthzContext,
    R: Send + Sync,
{
    #[inline(always)]
    fn authorize(&self, _context: &C, _request: &R) -> AuthzResult<()> {
        // Always allow - this gets completely optimized away in release builds
        Ok(())
    }

    fn policy_name(&self) -> &'static str {
        "NoAuthorization"
    }
}

/// Scope-based authorization policy for OAuth2 and similar token-based systems
///
/// This policy checks that the authentication context contains the required
/// scopes for the requested method.
#[derive(Debug, Clone)]
pub struct ScopeBasedPolicy {
    /// Default scope prefix (e.g., "mcp" for MCP protocol)
    scope_prefix: String,
    /// Whether to allow wildcard scopes (e.g., "mcp:*")
    allow_wildcard: bool,
}

impl ScopeBasedPolicy {
    /// Create a new scope-based policy
    ///
    /// # Arguments
    /// * `scope_prefix` - Prefix for all scopes (e.g., "mcp")
    /// * `allow_wildcard` - Whether to allow wildcard scopes like "mcp:*"
    pub fn new(scope_prefix: String, allow_wildcard: bool) -> Self {
        Self {
            scope_prefix,
            allow_wildcard,
        }
    }

    /// Create a new MCP-specific scope policy
    ///
    /// Uses "mcp" as the scope prefix and allows wildcard scopes.
    pub fn mcp() -> Self {
        Self::new("mcp".to_string(), true)
    }

    /// Check if required scope is present in the context
    fn check_scope(&self, required: &str, available: &[String]) -> bool {
        // Check for exact scope match
        if available.contains(&required.to_string()) {
            return true;
        }

        // Check for wildcard scope if enabled
        if self.allow_wildcard {
            let wildcard = format!("{}:*", &self.scope_prefix);
            if available.contains(&wildcard) {
                return true;
            }
        }

        false
    }
}

/// Request interface for scope-based authorization
pub trait ScopeRequest {
    /// Get the method name for scope checking
    fn method(&self) -> &str;
}

/// Context interface for scope-based authorization  
pub trait ScopeContext: AuthzContext {
    /// Get available scopes from the authentication context
    fn scopes(&self) -> &[String];
}

impl<C, R> AuthorizationPolicy<C, R> for ScopeBasedPolicy
where
    C: ScopeContext,
    R: ScopeRequest + Send + Sync,
{
    fn authorize(&self, context: &C, request: &R) -> AuthzResult<()> {
        let method = request.method();
        let required_scope = format!("{}:{method}", &self.scope_prefix);
        let available_scopes = context.scopes();

        if self.check_scope(&required_scope, available_scopes) {
            Ok(())
        } else {
            Err(AuthzError::InsufficientScope {
                required: required_scope,
                available: available_scopes.to_vec(),
            })
        }
    }

    fn policy_name(&self) -> &'static str {
        "ScopeBased"
    }
}

/// Binary authorization policy that allows or denies all requests
///
/// Useful for emergency lockdown or testing scenarios where you need
/// a simple allow/deny policy.
#[derive(Debug, Clone, Copy)]
pub struct BinaryAuthorizationPolicy {
    /// Whether to allow or deny all requests
    allow_all: bool,
}

impl BinaryAuthorizationPolicy {
    /// Create a policy that allows all requests
    pub const fn allow_all() -> Self {
        Self { allow_all: true }
    }

    /// Create a policy that denies all requests
    pub const fn deny_all() -> Self {
        Self { allow_all: false }
    }
}

impl<C, R> AuthorizationPolicy<C, R> for BinaryAuthorizationPolicy
where
    C: AuthzContext,
    R: Send + Sync,
{
    #[inline(always)]
    fn authorize(&self, _context: &C, _request: &R) -> AuthzResult<()> {
        if self.allow_all {
            Ok(())
        } else {
            Err(AuthzError::AccessDenied {
                reason: "Binary policy denies all access".to_string(),
            })
        }
    }

    fn policy_name(&self) -> &'static str {
        if self.allow_all {
            "BinaryAllowAll"
        } else {
            "BinaryDenyAll"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestContext {
        scopes: Vec<String>,
    }

    impl AuthzContext for TestContext {}

    impl ScopeContext for TestContext {
        fn scopes(&self) -> &[String] {
            &self.scopes
        }
    }

    #[derive(Debug)]
    struct TestRequest {
        method: String,
    }

    impl ScopeRequest for TestRequest {
        fn method(&self) -> &str {
            &self.method
        }
    }

    #[test]
    fn test_no_authorization_policy() {
        let policy = NoAuthorizationPolicy::<TestContext>::new();
        let context = TestContext { scopes: vec![] };
        let request = TestRequest {
            method: "test".to_string(),
        };

        let result = policy.authorize(&context, &request);
        assert!(result.is_ok());
        assert_eq!(<NoAuthorizationPolicy<TestContext> as AuthorizationPolicy<TestContext, TestRequest>>::policy_name(&policy), "NoAuthorization");
    }

    #[test]
    fn test_scope_based_policy_exact_match() {
        let policy = ScopeBasedPolicy::mcp();
        let context = TestContext {
            scopes: vec!["mcp:initialize".to_string()],
        };
        let request = TestRequest {
            method: "initialize".to_string(),
        };

        let result = policy.authorize(&context, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scope_based_policy_wildcard_match() {
        let policy = ScopeBasedPolicy::mcp();
        let context = TestContext {
            scopes: vec!["mcp:*".to_string()],
        };
        let request = TestRequest {
            method: "initialize".to_string(),
        };

        let result = policy.authorize(&context, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scope_based_policy_insufficient_scope() {
        let policy = ScopeBasedPolicy::mcp();
        let context = TestContext {
            scopes: vec!["api:read".to_string()],
        };
        let request = TestRequest {
            method: "initialize".to_string(),
        };

        let result = policy.authorize(&context, &request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthzError::InsufficientScope { required, available } => {
                assert_eq!(required, "mcp:initialize");
                assert_eq!(available, vec!["api:read"]);
            }
            _ => panic!("Expected InsufficientScope error"),
        }
    }

    #[test]
    fn test_binary_authorization_policy_allow() {
        let policy = BinaryAuthorizationPolicy::allow_all();
        let context = TestContext { scopes: vec![] };
        let request = TestRequest {
            method: "test".to_string(),
        };

        let result = policy.authorize(&context, &request);
        assert!(result.is_ok());
        assert_eq!(<BinaryAuthorizationPolicy as AuthorizationPolicy<TestContext, TestRequest>>::policy_name(&policy), "BinaryAllowAll");
    }

    #[test]
    fn test_binary_authorization_policy_deny() {
        let policy = BinaryAuthorizationPolicy::deny_all();
        let context = TestContext { scopes: vec![] };
        let request = TestRequest {
            method: "test".to_string(),
        };

        let result = policy.authorize(&context, &request);
        assert!(result.is_err());
        assert_eq!(<BinaryAuthorizationPolicy as AuthorizationPolicy<TestContext, TestRequest>>::policy_name(&policy), "BinaryDenyAll");
    }
}

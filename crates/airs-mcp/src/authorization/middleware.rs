//! Authorization Middleware
//!
//! Zero-cost generic authorization middleware that combines authentication contexts,
//! method extraction, and authorization policies into a unified authorization pipeline.

// Layer 1: Standard library imports
use std::marker::PhantomData;

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
use super::context::AuthzContext;
use super::error::AuthzResult;
use super::extractor::MethodExtractor;
use super::policy::{AuthorizationPolicy, ScopeRequest};

/// Zero-cost authorization middleware
///
/// This middleware combines authentication context, method extraction, and authorization
/// policies into a single, compile-time specialized authorization pipeline.
///
/// # Type Parameters
/// * `C` - Authorization context type (stack-allocated)
/// * `R` - Request type
/// * `P` - Authorization policy type
/// * `E` - Method extractor type
///
/// Each combination creates a unique middleware type at compile time,
/// allowing the compiler to inline all authorization logic for maximum performance.
#[derive(Debug)]
pub struct AuthorizationMiddleware<C, R, P, E>
where
    C: AuthzContext,
    R: Send + Sync + Clone + 'static,
    P: AuthorizationPolicy<C, AuthorizationRequest<R>>,
    E: MethodExtractor<R>,
{
    /// Authorization policy (compile-time specialized)
    policy: P,
    /// Method extractor (compile-time specialized)
    extractor: E,
    /// Phantom data for type parameters
    _phantom: PhantomData<(C, R)>,
}

impl<C, R, P, E> AuthorizationMiddleware<C, R, P, E>
where
    C: AuthzContext,
    R: Send + Sync + Clone + 'static,
    P: AuthorizationPolicy<C, AuthorizationRequest<R>>,
    E: MethodExtractor<R>,
{
    /// Create a new authorization middleware
    ///
    /// # Arguments
    /// * `policy` - Authorization policy to apply
    /// * `extractor` - Method extractor to use
    pub fn new(policy: P, extractor: E) -> Self {
        Self {
            policy,
            extractor,
            _phantom: PhantomData,
        }
    }

    /// Authorize a request using the configured policy and extractor
    ///
    /// This method is inlined at compile time for zero-cost abstractions.
    ///
    /// # Arguments
    /// * `context` - Authorization context from authentication layer
    /// * `request` - Request to authorize
    ///
    /// # Returns
    /// * `Ok(())` if request is authorized
    /// * `Err(AuthzError)` if request is denied or error occurred
    #[inline]
    pub fn authorize(&self, context: &C, request: &R) -> AuthzResult<()> {
        // Extract method from request (inlined at compile time)
        let method = self.extractor.extract_method(request)?;

        // Create authorization request
        let authz_request = AuthorizationRequest::new(method, request);

        // Apply authorization policy (inlined at compile time)
        self.policy.authorize(context, &authz_request)
    }

    /// Get policy name for debugging
    pub fn policy_name(&self) -> &'static str {
        self.policy.policy_name()
    }

    /// Get extractor name for debugging
    pub fn extractor_name(&self) -> &'static str {
        self.extractor.extractor_name()
    }
}

/// Authorization request wrapper
///
/// Wraps the original request with extracted method information for authorization decisions.
#[derive(Debug)]
pub struct AuthorizationRequest<R> {
    /// Extracted method name
    method: String,
    /// Original request
    request: R,
}

impl<R> AuthorizationRequest<R> {
    /// Create a new authorization request
    ///
    /// # Arguments
    /// * `method` - Extracted method name
    /// * `request` - Original request
    pub fn new(method: String, request: &R) -> Self
    where
        R: Clone,
    {
        Self {
            method,
            request: request.clone(),
        }
    }

    /// Get the extracted method name
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get the original request
    pub fn request(&self) -> &R {
        &self.request
    }
}

impl<R> ScopeRequest for AuthorizationRequest<R> {
    fn method(&self) -> &str {
        &self.method
    }
}

/// Builder for creating authorization middleware with type-safe configuration
#[derive(Debug)]
pub struct AuthorizationMiddlewareBuilder<C, R>
where
    C: AuthzContext,
{
    _phantom: PhantomData<(C, R)>,
}

impl<C, R> AuthorizationMiddlewareBuilder<C, R>
where
    C: AuthzContext,
    R: Send + Sync + Clone + 'static,
{
    /// Create a new middleware builder
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Build middleware with policy and extractor
    ///
    /// This method creates a compile-time specialized middleware instance.
    ///
    /// # Arguments
    /// * `policy` - Authorization policy to apply
    /// * `extractor` - Method extractor to use
    pub fn build<P, E>(
        self,
        policy: P,
        extractor: E,
    ) -> AuthorizationMiddleware<C, R, P, E>
    where
        P: AuthorizationPolicy<C, AuthorizationRequest<R>>,
        E: MethodExtractor<R>,
    {
        AuthorizationMiddleware::new(policy, extractor)
    }
}

impl<C, R> Default for AuthorizationMiddlewareBuilder<C, R>
where
    C: AuthzContext,
    R: Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

// Convenience type aliases for common configurations

/// No-authorization middleware that always allows requests (zero overhead)
pub type NoAuthMiddleware<R, E> = AuthorizationMiddleware<
    crate::authorization::context::NoAuthContext,
    R,
    crate::authorization::policy::NoAuthorizationPolicy<crate::authorization::context::NoAuthContext>,
    E,
>;

/// Scope-based authorization middleware for token-based systems
pub type ScopeMiddleware<R, E> = AuthorizationMiddleware<
    crate::authorization::context::ScopeAuthContext,
    R,
    crate::authorization::policy::ScopeBasedPolicy,
    E,
>;

// Helper functions for creating common middleware configurations

/// Create a no-authorization middleware (development mode)
///
/// This middleware compiles to zero overhead when used with NoAuthContext.
pub fn no_auth_middleware<R, E>(extractor: E) -> NoAuthMiddleware<R, E>
where
    R: Send + Sync + Clone + 'static,
    E: MethodExtractor<R>,
{
    AuthorizationMiddleware::new(
        crate::authorization::policy::NoAuthorizationPolicy::new(),
        extractor,
    )
}

/// Create a scope-based authorization middleware
///
/// This middleware uses scope-based authorization for token-based systems.
pub fn scope_middleware<R, E>(
    scope_policy: crate::authorization::policy::ScopeBasedPolicy,
    extractor: E,
) -> ScopeMiddleware<R, E>
where
    R: Send + Sync + Clone + 'static,
    E: MethodExtractor<R>,
{
    AuthorizationMiddleware::new(scope_policy, extractor)
}

/// Create an MCP scope-based authorization middleware
///
/// Convenience function for creating MCP-specific scope authorization.
pub fn mcp_scope_middleware<R, E>(extractor: E) -> ScopeMiddleware<R, E>
where
    R: Send + Sync + Clone + 'static,
    E: MethodExtractor<R>,
{
    let policy = crate::authorization::policy::ScopeBasedPolicy::mcp();
    scope_middleware(policy, extractor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::context::{NoAuthContext, ScopeAuthContext};
    use crate::authorization::extractor::{JsonRpcMethodExtractor, SimpleJsonRpcRequest};
    use crate::authorization::policy::{NoAuthorizationPolicy, ScopeBasedPolicy};
    use serde_json::json;

    #[test]
    fn test_no_auth_middleware() {
        let extractor = JsonRpcMethodExtractor::new();
        let middleware = no_auth_middleware(extractor);

        let context = NoAuthContext::new();
        let payload = json!({
            "method": "initialize",
            "id": 1
        });
        let request = SimpleJsonRpcRequest::new(payload);

        let result = middleware.authorize(&context, &request);
        assert!(result.is_ok());

        assert_eq!(middleware.policy_name(), "NoAuthorization");
        assert_eq!(middleware.extractor_name(), "JsonRpcMethodExtractor");
    }

    #[test]
    fn test_scope_middleware_authorized() {
        let policy = ScopeBasedPolicy::mcp();
        let extractor = JsonRpcMethodExtractor::new();
        let middleware = scope_middleware(policy, extractor);

        // Create context with required scope
        let context = ScopeAuthContext::simple(
            "user123".to_string(),
            vec!["mcp:initialize".to_string()],
        );

        let payload = json!({
            "method": "initialize",
            "id": 1
        });
        let request = SimpleJsonRpcRequest::new(payload);

        let result = middleware.authorize(&context, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scope_middleware_unauthorized() {
        let policy = ScopeBasedPolicy::mcp();
        let extractor = JsonRpcMethodExtractor::new();
        let middleware = scope_middleware(policy, extractor);

        // Create context without required scope
        let context = ScopeAuthContext::simple(
            "user123".to_string(),
            vec!["api:read".to_string()],
        );

        let payload = json!({
            "method": "initialize",
            "id": 1
        });
        let request = SimpleJsonRpcRequest::new(payload);

        let result = middleware.authorize(&context, &request);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.is_permission_error());
    }

    #[test]
    fn test_scope_middleware_wildcard() {
        let middleware = mcp_scope_middleware(JsonRpcMethodExtractor::new());

        // Create context with wildcard scope
        let context = ScopeAuthContext::simple(
            "admin".to_string(),
            vec!["mcp:*".to_string()],
        );

        let payload = json!({
            "method": "initialize",
            "id": 1
        });
        let request = SimpleJsonRpcRequest::new(payload);

        let result = middleware.authorize(&context, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_middleware_builder() {
        let builder = AuthorizationMiddlewareBuilder::<NoAuthContext, SimpleJsonRpcRequest>::new();
        
        let policy = NoAuthorizationPolicy::new();
        let extractor = JsonRpcMethodExtractor::new();
        
        let middleware = builder.build(policy, extractor);

        assert_eq!(middleware.policy_name(), "NoAuthorization");
        assert_eq!(middleware.extractor_name(), "JsonRpcMethodExtractor");
    }

    #[test]
    fn test_authorization_request() {
        let payload = json!({
            "method": "test",
            "id": 1
        });
        let original_request = SimpleJsonRpcRequest::new(payload);

        let authz_request = AuthorizationRequest::new("test_method".to_string(), &original_request);

        assert_eq!(authz_request.method(), "test_method");
        // Original request is cloned, so we can access it
    }
}

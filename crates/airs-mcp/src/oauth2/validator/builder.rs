//! Validator Builder Pattern Implementation
//!
//! Provides type-safe builder pattern for OAuth2 validator construction
//! following workspace standards for ergonomic API design.

// Layer 1: Standard library imports
// (none for this module)

// Layer 2: Third-party crate imports
// (none for this module)

// Layer 3: Internal module imports
use super::{JwtValidator, ScopeValidator, Validator};
use crate::oauth2::{
    config::{OAuth2Config, ScopeMapping},
    error::{OAuth2Error, OAuth2Result},
    validator::{Jwt, Scope},
};

/// Builder error types
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("JWT validator is required but not provided")]
    MissingJwtValidator,

    #[error("Scope validator is required but not provided")]
    MissingScopeValidator,

    #[error("OAuth2 configuration error: {0}")]
    ConfigurationError(#[from] OAuth2Error),
}

impl From<BuilderError> for OAuth2Error {
    fn from(err: BuilderError) -> Self {
        match err {
            BuilderError::MissingJwtValidator => {
                OAuth2Error::Configuration("JWT validator not configured".to_string())
            }
            BuilderError::MissingScopeValidator => {
                OAuth2Error::Configuration("Scope validator not configured".to_string())
            }
            BuilderError::ConfigurationError(oauth_err) => oauth_err,
        }
    }
}

/// Type-safe builder for OAuth2 validator construction
///
/// Following workspace standards §1 (Generic type usage), this builder
/// uses generics to maintain zero-cost abstractions while providing
/// a convenient construction API.
///
/// # Type Parameters
/// * `J` - JWT validator type (optional until build)
/// * `S` - Scope validator type (optional until build)
///
/// # Examples
///
/// ```rust
/// use airs_mcp::oauth2::validator::ValidatorBuilder;
/// use airs_mcp::oauth2::config::OAuth2Config;
/// use airs_mcp::oauth2::validator::{Jwt, Scope};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = OAuth2Config::default();
///
/// // Build with default components
/// let validator = ValidatorBuilder::new()
///     .with_default_jwt(config.clone())?
///     .with_default_scope()
///     .build()?;
///
/// // Build with custom components
/// let jwt = Jwt::new(config.clone())?;
/// let scope = Scope::with_default_mappings();
/// let validator = ValidatorBuilder::new()
///     .jwt(jwt)
///     .scope(scope)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct ValidatorBuilder<J, S> {
    /// Optional JWT validator (None until set)
    jwt: Option<J>,
    /// Optional scope validator (None until set)
    scope: Option<S>,
}

impl ValidatorBuilder<(), ()> {
    /// Create new validator builder
    ///
    /// Following workspace standards §2 (No unnecessary 'static),
    /// this constructor has no lifetime constraints.
    pub fn new() -> Self {
        Self {
            jwt: None,
            scope: None,
        }
    }
}

impl<J, S> ValidatorBuilder<J, S> {
    /// Set JWT validator component
    ///
    /// # Arguments
    /// * `jwt_validator` - JWT validator implementation
    ///
    /// # Returns
    /// * Builder with JWT validator configured
    pub fn jwt<NewJ>(self, jwt_validator: NewJ) -> ValidatorBuilder<NewJ, S>
    where
        NewJ: JwtValidator,
    {
        ValidatorBuilder {
            jwt: Some(jwt_validator),
            scope: self.scope,
        }
    }

    /// Set scope validator component
    ///
    /// # Arguments
    /// * `scope_validator` - Scope validator implementation
    ///
    /// # Returns
    /// * Builder with scope validator configured
    pub fn scope<NewS>(self, scope_validator: NewS) -> ValidatorBuilder<J, NewS>
    where
        NewS: ScopeValidator,
    {
        ValidatorBuilder {
            jwt: self.jwt,
            scope: Some(scope_validator),
        }
    }

    /// Create JWT validator from OAuth2 configuration
    ///
    /// Convenience method for common case of creating JWT validator
    /// from configuration.
    ///
    /// # Arguments
    /// * `config` - OAuth2 configuration
    ///
    /// # Returns
    /// * Builder with JWT validator configured
    /// * Error if JWT validator creation fails
    pub fn with_default_jwt(
        self,
        config: OAuth2Config,
    ) -> Result<ValidatorBuilder<Jwt, S>, BuilderError> {
        let jwt_validator = Jwt::new(config)?;
        Ok(ValidatorBuilder {
            jwt: Some(jwt_validator),
            scope: self.scope,
        })
    }

    /// Create scope validator with default MCP mappings
    ///
    /// Convenience method for common case of using default scope mappings.
    ///
    /// # Returns
    /// * Builder with scope validator configured
    pub fn with_default_scope(self) -> ValidatorBuilder<J, Scope> {
        let scope_validator = Scope::with_default_mappings();
        ValidatorBuilder {
            jwt: self.jwt,
            scope: Some(scope_validator),
        }
    }

    /// Create scope validator with custom mappings
    ///
    /// Convenience method for creating scope validator with specific mappings.
    ///
    /// # Arguments
    /// * `mappings` - Custom scope mappings
    ///
    /// # Returns
    /// * Builder with scope validator configured
    pub fn with_scope_mappings(self, mappings: Vec<ScopeMapping>) -> ValidatorBuilder<J, Scope> {
        let scope_validator = Scope::new(mappings);
        ValidatorBuilder {
            jwt: self.jwt,
            scope: Some(scope_validator),
        }
    }

    /// Build the final validator
    ///
    /// Following workspace standards §3 (Stack allocation), this
    /// constructs the validator without heap allocation.
    ///
    /// # Returns
    /// * `Ok(Validator<J, S>)` - Successfully built validator
    /// * `Err(BuilderError)` - Missing required components
    pub fn build(self) -> Result<Validator<J, S>, BuilderError>
    where
        J: JwtValidator,
        S: ScopeValidator,
    {
        let jwt = self.jwt.ok_or(BuilderError::MissingJwtValidator)?;
        let scope = self.scope.ok_or(BuilderError::MissingScopeValidator)?;
        Ok(Validator::new(jwt, scope))
    }
}

// Implement Default for convenience
impl Default for ValidatorBuilder<(), ()> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common validator configurations
///
/// Create validator with default configuration
///
/// # Arguments
/// * `config` - OAuth2 configuration
///
/// # Returns
/// * `Ok(Validator)` - Validator with default JWT and scope validators
/// * `Err(OAuth2Error)` - Configuration error
pub fn create_default_validator(config: OAuth2Config) -> OAuth2Result<Validator<Jwt, Scope>> {
    ValidatorBuilder::new()
        .with_default_jwt(config)?
        .with_default_scope()
        .build()
        .map_err(Into::into)
}

/// Create validator with custom scope mappings
///
/// # Arguments
/// * `config` - OAuth2 configuration
/// * `mappings` - Custom scope mappings
///
/// # Returns
/// * `Ok(Validator)` - Validator with default JWT and custom scope validator
/// * `Err(OAuth2Error)` - Configuration error
pub fn create_validator_with_mappings(
    config: OAuth2Config,
    mappings: Vec<ScopeMapping>,
) -> OAuth2Result<Validator<Jwt, Scope>> {
    ValidatorBuilder::new()
        .with_default_jwt(config)?
        .with_scope_mappings(mappings)
        .build()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth2::config::ScopeMapping;

    #[test]
    fn test_builder_creation() {
        let builder = ValidatorBuilder::new();

        // Should be able to create builder
        assert!(builder.jwt.is_none());
        assert!(builder.scope.is_none());
    }

    #[test]
    fn test_builder_missing_components() {
        // This test is for API design validation only
        // We cannot actually call build() without proper trait implementations
        let builder = ValidatorBuilder::new();

        // Test that builder creates with empty components
        assert!(builder.jwt.is_none());
        assert!(builder.scope.is_none());

        // Note: Cannot test build() here because () doesn't implement required traits
        // This is actually good - the type system prevents invalid builds at compile time
    }

    #[test]
    fn test_default_validator_creation() {
        let config = OAuth2Config::default();
        let result = create_default_validator(config);

        // Should handle creation (success or configuration error)
        match result {
            Ok(_validator) => {
                // Successfully created
            }
            Err(OAuth2Error::Configuration(_)) => {
                // Expected for default config without proper JWKS URL
            }
            Err(other) => panic!("Unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_custom_mappings_validator() {
        let config = OAuth2Config::default();
        let mappings = vec![ScopeMapping {
            method: "custom/method".to_string(),
            scope: "custom:scope".to_string(),
            optional: false,
        }];

        let result = create_validator_with_mappings(config, mappings);

        // Should handle creation (success or configuration error)
        match result {
            Ok(_validator) => {
                // Successfully created
            }
            Err(OAuth2Error::Configuration(_)) => {
                // Expected for default config without proper JWKS URL
            }
            Err(other) => panic!("Unexpected error: {other:?}"),
        }
    }

    #[test]
    fn test_builder_type_safety() {
        // This test validates the type system prevents invalid builds

        let builder = ValidatorBuilder::new();

        // Can add components in any order
        let builder_with_scope = builder.with_default_scope();

        // Type system should enforce that we can't build without JWT
        // (This would fail to compile if uncommented)
        // let result = builder_with_scope.build();

        let _ = builder_with_scope; // Avoid unused variable warning
    }

    #[test]
    fn test_convenience_functions_interface() {
        // Test that convenience functions have the expected interface

        let config = OAuth2Config::default();
        let _result1 = create_default_validator(config.clone());

        let mappings = vec![];
        let _result2 = create_validator_with_mappings(config, mappings);

        // Should compile without issues (actual functionality tested above)
    }
}

//! Authentication Method Abstraction
//!
//! Simple string-based authentication method identifier for maximum extensibility.

// Layer 1: Standard library imports
use std::fmt::{Display, Formatter, Result as FmtResult};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (none for this module)

/// Simple authentication method identifier
///
/// Just a string wrapper that allows any authentication method to be defined.
/// No predefined strategies - completely extensible.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::authentication::AuthMethod;
///
/// let oauth2 = AuthMethod::new("oauth2");
/// let api_key = AuthMethod::new("apikey");
/// let ldap = AuthMethod::new("ldap");
/// let custom = AuthMethod::new("my-custom-auth");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthMethod(String);

impl AuthMethod {
    /// Create new authentication method
    pub fn new<S: Into<String>>(identifier: S) -> Self {
        Self(identifier.into())
    }

    /// Get method identifier
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this matches a specific identifier
    pub fn is(&self, identifier: &str) -> bool {
        self.0 == identifier
    }
}

impl Display for AuthMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for AuthMethod {
    fn from(identifier: &str) -> Self {
        Self::new(identifier)
    }
}

impl From<String> for AuthMethod {
    fn from(identifier: String) -> Self {
        Self::new(identifier)
    }
}

impl AsRef<str> for AuthMethod {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_method_creation() {
        let method = AuthMethod::new("oauth2");
        assert_eq!(method.as_str(), "oauth2");
        assert!(method.is("oauth2"));
        assert!(!method.is("apikey"));
    }

    #[test]
    fn test_auth_method_from_str() {
        let method: AuthMethod = "apikey".into();
        assert_eq!(method.as_str(), "apikey");
    }

    #[test]
    fn test_auth_method_display() {
        let method = AuthMethod::new("custom-auth");
        assert_eq!(format!("{method}"), "custom-auth");
    }
}

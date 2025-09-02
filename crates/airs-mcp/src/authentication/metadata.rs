//! Authentication Metadata
//!
//! Simple HashMap wrapper for authentication metadata.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

// Layer 2: Third-party crate imports

// Layer 3: Internal module imports
// (none for this module)

/// Simple authentication metadata as HashMap wrapper
///
/// Provides a thin wrapper around HashMap for authentication metadata
/// with convenience methods for common operations.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::authentication::AuthMetadata;
///
/// let metadata = AuthMetadata::new()
///     .add("client_ip", "192.168.1.1")
///     .add("user_agent", "test-agent");
///
/// assert_eq!(metadata.get("client_ip"), Some(&"192.168.1.1".to_string()));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AuthMetadata(HashMap<String, String>);

impl AuthMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    
    /// Add attribute with builder pattern
    pub fn add<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.0.insert(key.into(), value.into());
        self
    }
    
    /// Get attribute value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
    
    /// Insert attribute
    pub fn insert<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.0.insert(key.into(), value.into());
    }
    
    /// Extend with another metadata or HashMap
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (String, String)>,
    {
        self.0.extend(iter);
    }
    
    /// Create from HashMap
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self(map)
    }
    
    /// Convert to HashMap
    pub fn into_map(self) -> HashMap<String, String> {
        self.0
    }
}

impl Deref for AuthMetadata {
    type Target = HashMap<String, String>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AuthMetadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<HashMap<String, String>> for AuthMetadata {
    fn from(map: HashMap<String, String>) -> Self {
        Self(map)
    }
}

impl From<AuthMetadata> for HashMap<String, String> {
    fn from(metadata: AuthMetadata) -> Self {
        metadata.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_metadata_creation() {
        let metadata = AuthMetadata::new()
            .add("key1", "value1")
            .add("key2", "value2");
        
        assert_eq!(metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get("key2"), Some(&"value2".to_string()));
        assert_eq!(metadata.get("key3"), None);
    }

    #[test]
    fn test_auth_metadata_deref() {
        let mut metadata = AuthMetadata::new();
        metadata.insert("test", "value");
        
        // Test Deref - can use HashMap methods directly
        assert_eq!(metadata.len(), 1);
        assert!(metadata.contains_key("test"));
    }

    #[test]
    fn test_auth_metadata_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), "value".to_string());
        
        let metadata = AuthMetadata::from(map.clone());
        assert_eq!(metadata.into_map(), map);
    }
}

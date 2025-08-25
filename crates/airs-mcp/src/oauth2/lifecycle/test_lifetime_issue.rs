//! Test to demonstrate why 'static is needed

use super::config::TokenLifecycleConfig;
use super::manager::TokenLifecycleManager;
use super::traits::*;
use super::types::*;
use crate::oauth2::{AuthContext, OAuth2Result};
use async_trait::async_trait;
use std::sync::Arc;

/// This is a problematic cache provider that holds a non-static reference
#[derive(Debug)]
pub struct ProblematicCacheProvider<'a> {
    pub data: &'a str, // Non-static reference!
}

#[async_trait]
impl<'a> TokenCacheProvider for ProblematicCacheProvider<'a> {
    async fn store(
        &self,
        _key: TokenCacheKey,
        _entry: TokenCacheEntry,
        _ttl: Option<std::time::Duration>,
    ) -> OAuth2Result<()> {
        println!("Storing with data: {}", self.data);
        Ok(())
    }

    async fn retrieve(&self, _key: &TokenCacheKey) -> OAuth2Result<Option<TokenCacheEntry>> {
        Ok(None)
    }

    async fn remove(&self, _key: &TokenCacheKey) -> OAuth2Result<bool> {
        Ok(false)
    }

    async fn exists(&self, _key: &TokenCacheKey) -> OAuth2Result<bool> {
        Ok(false)
    }

    async fn get_expiration(
        &self,
        _key: &TokenCacheKey,
    ) -> OAuth2Result<Option<chrono::DateTime<chrono::Utc>>> {
        Ok(None)
    }

    async fn update_expiration(
        &self,
        _key: &TokenCacheKey,
        _new_expiration: chrono::DateTime<chrono::Utc>,
    ) -> OAuth2Result<()> {
        Ok(())
    }

    async fn clear_expired(&self) -> OAuth2Result<u64> {
        Ok(0)
    }

    async fn get_metrics(&self) -> OAuth2Result<TokenCacheMetrics> {
        Ok(TokenCacheMetrics::default())
    }

    async fn list_keys(&self) -> OAuth2Result<Vec<TokenCacheKey>> {
        Ok(vec![])
    }
}

// Let's try to create a manager with this problematic type
pub fn test_problematic_case() {
    // This would be the dangerous scenario
    let temp_data = "temporary data".to_string();

    let problematic_cache = ProblematicCacheProvider {
        data: &temp_data, // Borrowing local data!
    };

    // If we try to put this in Arc and move it around...
    // let arc_cache = Arc::new(problematic_cache);  // This would be dangerous!

    // The 'static bound prevents this dangerous usage
}

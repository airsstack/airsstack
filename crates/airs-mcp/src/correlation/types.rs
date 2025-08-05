//! Type definitions for the correlation system
//!
//! This module contains core type definitions used throughout the correlation
//! system for request tracking, ID generation, and result handling.

use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, TimeDelta, Utc};
use serde_json::Value;
use tokio::sync::oneshot;

// Import RequestId directly from error module
use crate::correlation::error::RequestId;
// Re-export CorrelationResult for convenience
pub use crate::correlation::error::CorrelationResult;

/// A pending request awaiting correlation with its response
///
/// This structure maintains all necessary information for a request that has been
/// registered but not yet correlated with a response, including timeout information
/// and the communication channel for delivering the result.
#[derive(Debug)]
pub struct PendingRequest {
    /// Channel sender for delivering the correlated response or error
    pub sender: oneshot::Sender<CorrelationResult<Value>>,

    /// Timestamp when the request was created (UTC)
    pub created_at: DateTime<Utc>,

    /// Maximum time to wait for a response
    pub timeout: TimeDelta,

    /// Original request data for debugging and context
    pub request_data: Value,
}

impl PendingRequest {
    /// Create a new pending request
    ///
    /// # Parameters
    ///
    /// - `sender`: Channel for sending the correlated response
    /// - `timeout`: Maximum time to wait for response
    /// - `request_data`: Original request for debugging context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::types::PendingRequest;
    /// use tokio::sync::oneshot;
    /// use chrono::TimeDelta;
    /// use serde_json::json;
    ///
    /// let (sender, _receiver) = oneshot::channel();
    /// let pending = PendingRequest::new(
    ///     sender,
    ///     TimeDelta::seconds(30),
    ///     json!({"method": "test"})
    /// );
    /// ```
    pub fn new(
        sender: oneshot::Sender<CorrelationResult<Value>>,
        timeout: TimeDelta,
        request_data: Value,
    ) -> Self {
        Self {
            sender,
            created_at: Utc::now(),
            timeout,
            request_data,
        }
    }

    /// Check if this request has expired based on current time
    ///
    /// # Returns
    ///
    /// `true` if the request has exceeded its timeout period
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp::correlation::types::PendingRequest;
    /// # use tokio::sync::oneshot;
    /// # use chrono::TimeDelta;
    /// # use serde_json::json;
    /// #
    /// # let (sender, _receiver) = oneshot::channel();
    /// # let pending = PendingRequest::new(
    /// #     sender,
    /// #     TimeDelta::seconds(30),
    /// #     json!({"method": "test"})
    /// # );
    /// #
    /// if pending.is_expired() {
    ///     println!("Request has timed out");
    /// }
    /// ```
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        now.signed_duration_since(self.created_at) > self.timeout
    }

    /// Check if this request has expired at a specific timestamp
    ///
    /// This is more efficient than `is_expired()` when checking multiple requests
    /// as it avoids repeated system calls to get the current time.
    ///
    /// # Arguments
    ///
    /// * `now` - The current timestamp to compare against
    ///
    /// # Returns
    ///
    /// `true` if the request has exceeded its timeout period at the given time
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airs_mcp::correlation::types::PendingRequest;
    /// # use tokio::sync::oneshot;
    /// # use chrono::{TimeDelta, Utc};
    /// # use serde_json::json;
    /// #
    /// # let (sender, _receiver) = oneshot::channel();
    /// # let pending = PendingRequest::new(
    /// #     sender,
    /// #     TimeDelta::seconds(30),
    /// #     json!({"method": "test"})
    /// # );
    /// #
    /// let now = Utc::now();
    /// if pending.is_expired_at(&now) {
    ///     println!("Request has timed out");
    /// }
    /// ```
    pub fn is_expired_at(&self, now: &chrono::DateTime<chrono::Utc>) -> bool {
        now.signed_duration_since(self.created_at) > self.timeout
    }

    /// Get the remaining time before this request expires
    ///
    /// # Returns
    ///
    /// `Some(TimeDelta)` with remaining time, or `None` if already expired
    pub fn time_remaining(&self) -> Option<TimeDelta> {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.created_at);

        if elapsed >= self.timeout {
            None
        } else {
            Some(self.timeout - elapsed)
        }
    }
}

/// Thread-safe request ID generator
///
/// Generates unique request IDs using an atomic counter, ensuring no collisions
/// in concurrent environments. IDs are generated as numeric RequestId values.
#[derive(Debug)]
pub struct RequestIdGenerator {
    /// Atomic counter for generating unique IDs
    counter: AtomicU64,
}

impl RequestIdGenerator {
    /// Create a new request ID generator
    ///
    /// The generator starts with ID 1 and increments atomically for each request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::types::RequestIdGenerator;
    ///
    /// let generator = RequestIdGenerator::new();
    /// let id1 = generator.next_id();
    /// let id2 = generator.next_id();
    /// // id1 and id2 are guaranteed to be different
    /// ```
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }

    /// Generate the next unique request ID
    ///
    /// This method is thread-safe and can be called concurrently from multiple
    /// tasks without risk of ID collision.
    ///
    /// # Returns
    ///
    /// A unique `RequestId` with numeric value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airs_mcp::correlation::types::RequestIdGenerator;
    ///
    /// let generator = RequestIdGenerator::new();
    /// let id = generator.next_id();
    /// println!("Generated ID: {:?}", id);
    /// ```
    pub fn next_id(&self) -> RequestId {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        RequestId::new_number(id as i64)
    }

    /// Get the current counter value (next ID that would be generated)
    ///
    /// This is primarily useful for testing and debugging purposes.
    ///
    /// # Returns
    ///
    /// The current counter value
    pub fn current_count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

impl Default for RequestIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_pending_request_creation() {
        let (sender, _receiver) = oneshot::channel();
        let timeout = TimeDelta::seconds(30);
        let request_data = serde_json::json!({"method": "test"});

        let pending = PendingRequest::new(sender, timeout, request_data.clone());

        assert_eq!(pending.timeout, timeout);
        assert_eq!(pending.request_data, request_data);
        assert!(!pending.is_expired()); // Should not be expired immediately
    }

    #[test]
    fn test_pending_request_expiration() {
        let (sender, _receiver) = oneshot::channel();
        let timeout = TimeDelta::milliseconds(1); // Very short timeout
        let request_data = serde_json::json!({"method": "test"});

        let pending = PendingRequest::new(sender, timeout, request_data);

        // Wait a bit and check expiration
        thread::sleep(std::time::Duration::from_millis(10));
        assert!(pending.is_expired());
        assert!(pending.time_remaining().is_none());
    }

    #[test]
    fn test_request_id_generator() {
        let generator = RequestIdGenerator::new();

        let id1 = generator.next_id();
        let id2 = generator.next_id();
        let id3 = generator.next_id();

        // IDs should be unique and sequential
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_request_id_generator_concurrent() {
        let generator = Arc::new(RequestIdGenerator::new());
        let mut handles = vec![];

        // Spawn multiple threads generating IDs
        for _ in 0..5 {
            // Reduced from 10 for faster testing
            let gen = Arc::clone(&generator);
            let handle = thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..10 {
                    // Reduced from 100 for faster testing
                    ids.push(gen.next_id());
                }
                ids
            });
            handles.push(handle);
        }

        // Collect all generated IDs
        let mut all_ids = vec![];
        for handle in handles {
            let mut ids = handle.join().unwrap();
            all_ids.append(&mut ids);
        }

        // Check that all IDs are unique by converting to HashSet
        let unique_ids: HashSet<_> = all_ids.into_iter().collect();
        assert_eq!(unique_ids.len(), 50); // 5 threads * 10 IDs each
    }

    #[test]
    fn test_request_id_generator_counter() {
        let generator = RequestIdGenerator::new();

        assert_eq!(generator.current_count(), 1);

        let _id1 = generator.next_id();
        assert_eq!(generator.current_count(), 2);

        let _id2 = generator.next_id();
        assert_eq!(generator.current_count(), 3);
    }
}

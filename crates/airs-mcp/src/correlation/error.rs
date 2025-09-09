//! Error types for the correlation system
//!
//! This module defines all error types related to request/response correlation,
//! providing structured error information for debugging and monitoring.

use chrono::TimeDelta;
use thiserror::Error;

/// Request ID type alias for consistency with JSON-RPC base types
pub type RequestId = crate::protocol::RequestId;

/// Correlation error types
///
/// These errors represent various failure modes in the correlation system,
/// each providing specific context for debugging and operational monitoring.
#[derive(Debug, Clone, Error)]
pub enum CorrelationError {
    /// Request timed out waiting for response
    #[error("Request {id} timed out after {duration}")]
    Timeout {
        /// The request ID that timed out
        id: RequestId,
        /// The timeout duration that was exceeded
        duration: TimeDelta,
    },

    /// Request was not found in the correlation table
    #[error("Request {id} not found (may have completed or been cancelled)")]
    RequestNotFound {
        /// The request ID that was not found
        id: RequestId,
    },

    /// Attempt to correlate response for already completed request
    #[error("Request {id} has already been completed")]
    AlreadyCompleted {
        /// The request ID that was already completed
        id: RequestId,
    },

    /// Communication channel was closed unexpectedly
    #[error("Channel error for request {id}: {details}")]
    ChannelClosed {
        /// The request ID associated with the channel
        id: RequestId,
        /// Additional error details
        details: String,
    },

    /// Internal correlation system error
    #[error("Internal correlation error: {message}")]
    Internal {
        /// Error message describing the internal issue
        message: String,
    },

    /// Request was explicitly cancelled
    #[error("Request {id} was cancelled")]
    Cancelled {
        /// The request ID that was cancelled
        id: RequestId,
    },
}

/// Single result type for all correlation operations
///
/// This type provides a consistent error handling pattern throughout the correlation
/// system, supporting various return types while maintaining error context.
///
/// # Examples
///
/// ```rust
/// use airs_mcp::correlation::{CorrelationResult, CorrelationError};
/// use serde_json::Value;
///
/// // For response correlation
/// let response_result: CorrelationResult<Value> = Ok(serde_json::json!({"result": "success"}));
///
/// // For operation success/failure
/// let operation_result: CorrelationResult<()> = Ok(());
///
/// // For error cases
/// let error_result: CorrelationResult<Value> = Err(CorrelationError::Internal {
///     message: "Test error".to_string()
/// });
/// ```
pub type CorrelationResult<T> = std::result::Result<T, CorrelationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let timeout_error = CorrelationError::Timeout {
            id: RequestId::new_string("test-123"),
            duration: TimeDelta::seconds(30),
        };

        let display = format!("{timeout_error}");
        assert!(display.contains("test-123"));
        assert!(display.contains("timed out"));
    }

    #[test]
    fn test_error_debug() {
        let not_found_error = CorrelationError::RequestNotFound {
            id: RequestId::new_number(42),
        };

        let debug = format!("{not_found_error:?}");
        assert!(debug.contains("RequestNotFound"));
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_result_type_usage() {
        let success: CorrelationResult<String> = Ok("test".to_string());
        assert!(success.is_ok());

        let failure: CorrelationResult<String> = Err(CorrelationError::Internal {
            message: "test failure".to_string(),
        });
        assert!(failure.is_err());
    }
}

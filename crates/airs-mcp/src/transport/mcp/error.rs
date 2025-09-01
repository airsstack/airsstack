//! Transport Error Types
//!
//! Error types for transport-level operations, separate from JSON-RPC protocol errors.

/// Transport-level error types
///
/// These errors represent transport layer failures, separate from
/// JSON-RPC protocol errors that are part of the message format.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// Connection failed or was lost
    #[error("Connection error: {message}")]
    Connection { message: String },

    /// Message serialization/deserialization failed
    #[error("Serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    /// I/O error during transport operations
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Transport was closed
    #[error("Transport is closed")]
    Closed,

    /// Transport-specific error
    #[error("Transport error: {message}")]
    Transport { message: String },

    /// Timeout during operation
    #[error("Operation timed out after {duration_ms}ms")]
    Timeout { duration_ms: u64 },
}

impl TransportError {
    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection {
            message: message.into(),
        }
    }

    /// Create a transport-specific error
    pub fn transport(message: impl Into<String>) -> Self {
        Self::Transport {
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(duration_ms: u64) -> Self {
        Self::Timeout { duration_ms }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_error_creation() {
        let conn_err = TransportError::connection("Connection refused");
        assert!(matches!(conn_err, TransportError::Connection { .. }));

        let timeout_err = TransportError::timeout(5000);
        assert!(matches!(
            timeout_err,
            TransportError::Timeout { duration_ms: 5000 }
        ));

        let transport_err = TransportError::transport("Custom transport error");
        assert!(matches!(transport_err, TransportError::Transport { .. }));
    }

    #[test]
    fn test_error_display() {
        let error = TransportError::connection("Connection refused");
        let error_string = format!("{}", error);
        assert!(error_string.contains("Connection error"));
        assert!(error_string.contains("Connection refused"));
    }
}

//! Compatibility Bridge for Legacy Systems
//!
//! Compatibility layer for gradual migration from existing trait-based message system
//! to the new flat JsonRpcMessage structure.

use crate::base::jsonrpc::message::JsonRpcMessage as LegacyJsonRpcMessage;

use super::JsonRpcMessage;

/// Compatibility bridge for existing JsonRpcMessage implementations
///
/// This allows gradual migration from the existing trait-based message system
/// to the new flat JsonRpcMessage structure without breaking existing code.
impl From<JsonRpcMessage> for Vec<u8> {
    fn from(message: JsonRpcMessage) -> Self {
        message.to_bytes().unwrap_or_default()
    }
}

impl TryFrom<Vec<u8>> for JsonRpcMessage {
    type Error = serde_json::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(&bytes)
    }
}

impl TryFrom<&[u8]> for JsonRpcMessage {
    type Error = serde_json::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

/// Bridge to convert from legacy JsonRpcMessage trait implementations
///
/// This enables gradual migration by allowing conversion from existing
/// message types to the new MCP-compliant format.
impl JsonRpcMessage {
    /// Convert from any legacy JsonRpcMessage trait implementation
    pub fn from_legacy<T>(legacy_message: &T) -> Result<Self, serde_json::Error>
    where
        T: LegacyJsonRpcMessage,
    {
        let json = legacy_message.to_json()?;
        Self::from_json(&json)
    }
}

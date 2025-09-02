//! HTTP Authentication Adapters
//!
//! This module provides HTTP transport-specific authentication adapters that bridge
//! the generic authentication strategies with HTTP transport requirements.

pub mod oauth2_adapter;

pub use oauth2_adapter::OAuth2StrategyAdapter;

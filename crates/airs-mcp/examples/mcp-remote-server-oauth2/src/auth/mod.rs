//! OAuth2 Authentication Setup for AirsStack MCP
//!
//! This module sets up OAuth2 authentication components that integrate
//! with AirsStack's MCP Transport infrastructure:
//!
//! - OAuth2Strategy with JWT validation
//! - OAuth2StrategyAdapter for HTTP transport
//! - RSA key management for JWT signing/validation
//! - Test token generation for development

pub mod keys;
pub mod tokens;
pub mod setup;

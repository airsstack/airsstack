//! Testing Infrastructure for OAuth2 MCP Server
//!
//! This module provides test infrastructure to support OAuth2 authentication testing:
//!
//! - Mock JWKS server for JWT validation testing
//! - Token generation endpoints for different test scenarios
//! - Server information endpoints for debugging

pub mod jwks;
pub mod endpoints;

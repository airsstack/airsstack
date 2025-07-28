# AIRS Project Brief

## Project Vision
Production-grade Rust implementation ecosystem for AI model communication protocols, starting with Model Context Protocol (MCP).

## Core Objective
Create a 100% MCP specification-compliant implementation with sub-millisecond message processing, leveraging Rust's type system for compile-time protocol compliance.

## Workspace Architecture
- **airs-mcp**: Core MCP implementation with JSON-RPC 2.0 foundation
- **airs-cli**: (Future) Command-line tools for MCP interaction  
- **airs-server**: (Future) Standalone MCP server implementation
- **airs-common**: (Future) Shared utilities and types

## Development Methodology Integration
- **Memory Bank**: Persistent project intelligence across memory resets
- **Spec-Driven Workflow**: ANALYZE → DESIGN → IMPLEMENT → VALIDATE → REFLECT → HANDOFF
- **EARS Notation**: Structured, testable requirements
- **Gilfoyle Code Review**: Technical excellence with sardonic precision

## Quality Standards
- Sub-millisecond message processing (99th percentile)
- 100% MCP specification compliance
- Type-safe protocol implementation
- Production-ready observability
- Zero-compromise architectural decisions

## Current Priority
JSON-RPC 2.0 foundation in airs-mcp crate - the foundational layer for all MCP functionality.
This foundation must be bulletproof before any MCP-specific features are implemented.
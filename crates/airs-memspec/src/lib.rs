//! # airs-memspec - Multi-Project Memory Bank Management
//!
//! A sophisticated CLI tool and library for managing Multi-Project Memory Bank structures
//! that streamline AI-assisted development with GitHub Copilot integration. Provides
//! workspace-aware context management, project navigation, and comprehensive task tracking.
//!
//! ## Core Functionality
//!
//! - **Memory Bank Installation**: Deploy standardized memory bank structures
//! - **Context Analysis**: Analyze and display workspace context across multiple projects
//! - **Project Status**: Monitor progress and health across sub-projects
//! - **Task Tracking**: View and track development tasks with detailed progress logging (read-only)
//! - **AI Integration**: Embed GitHub Copilot instruction templates for consistent AI guidance
//!
//! ## Architecture Overview
//!
//! The library is organized into five main modules:
//!
//! - [`cli`] - Command-line interface and argument parsing
//! - [`embedded`] - Static instruction templates and content
//! - [`models`] - Domain models for workspace, projects, and tasks
//! - [`parser`] - Content parsing and context correlation
//! - [`utils`] - Shared utilities and helper functions
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use airs_memspec::{
//!     cli::args::{Cli, Commands},
//!     embedded::instructions::available_templates,
//!     parser::navigation::MemoryBankNavigator,
//! };
//! use clap::Parser;
//! use std::path::Path;
//!
//! // Parse CLI arguments
//! let args = Cli::parse();
//!
//! // Discover memory bank structure
//! let structure = MemoryBankNavigator::discover_structure(Path::new("."))?;
//!
//! // Access available instruction templates
//! let templates = available_templates();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Memory Bank Structure
//!
//! The Multi-Project Memory Bank organizes development context using a standardized
//! directory structure:
//!
//! ```text
//! .copilot/memory_bank/
//! ├── current_context.md           # Active project context
//! ├── workspace/                   # Workspace-level files
//! │   ├── project_brief.md
//! │   ├── shared_patterns.md
//! │   └── workspace_architecture.md
//! ├── context_snapshots/           # Historical context states
//! └── sub_projects/                # Individual project contexts
//!     └── {project_name}/
//!         ├── project_brief.md
//!         ├── active_context.md
//!         ├── tech_context.md
//!         └── tasks/
//!             ├── _index.md
//!             └── task_*.md
//! ```
//!
//! ## Professional Development Standards
//!
//! This library enforces professional development practices including:
//!
//! - **Zero-Warning Policy**: All code compiles without warnings
//! - **Comprehensive Testing**: Unit, integration, and documentation tests
//! - **Rich Documentation**: All public APIs include examples and usage patterns
//! - **Error Handling**: Robust error types with detailed context
//! - **Performance**: Efficient parsing and correlation algorithms

/// Command-line interface and argument parsing
pub mod cli;
/// Embedded instruction templates and static content
pub mod embedded;
/// Domain models for workspace, projects, and task tracking
pub mod models;
/// Content parsing and context correlation engines
pub mod parser;
/// Shared utilities and helper functions
pub mod utils;

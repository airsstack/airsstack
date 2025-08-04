//! Workspace domain models and functionality
//!
//! This module contains data structures and operations related to workspace-level
//! configuration, shared patterns, context management, and historical snapshots.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::sub_project::SubProject;

/// Root workspace configuration and metadata
///
/// Represents the top-level workspace containing multiple sub-projects,
/// shared patterns, and workspace-wide configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    /// Workspace metadata and configuration
    pub metadata: WorkspaceMetadata,

    /// Shared patterns and architectural decisions
    pub shared_patterns: SharedPatterns,

    /// Current active context tracking
    pub current_context: CurrentContext,

    /// Map of sub-project name to sub-project data
    pub sub_projects: HashMap<String, SubProject>,

    /// Historical context snapshots
    pub snapshots: Vec<ContextSnapshot>,
}

/// Workspace-level metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceMetadata {
    /// Workspace name/identifier
    pub name: String,

    /// Brief description of the workspace purpose
    pub description: Option<String>,

    /// Workspace creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,

    /// Workspace version for compatibility tracking
    pub version: String,

    /// Root directory path
    pub root_path: PathBuf,

    /// Additional metadata fields
    pub metadata: HashMap<String, String>,
}

/// Shared patterns and architectural decisions across the workspace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedPatterns {
    /// Core implementation patterns
    pub implementation_patterns: Vec<Pattern>,

    /// Architecture and design patterns  
    pub architecture_patterns: Vec<Pattern>,

    /// Methodology and workflow patterns
    pub methodology_patterns: Vec<Pattern>,

    /// Cross-cutting concerns and shared utilities
    pub shared_utilities: Vec<SharedUtility>,
}

/// A documented pattern or practice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pattern {
    /// Pattern name/identifier
    pub name: String,

    /// Detailed pattern description
    pub description: String,

    /// When to apply this pattern
    pub usage_context: String,

    /// Code examples or templates
    pub examples: Vec<String>,

    /// Related patterns or references
    pub references: Vec<String>,

    /// Pattern category/tags
    pub tags: Vec<String>,
}

/// Shared utility or cross-cutting concern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedUtility {
    /// Utility name
    pub name: String,

    /// Purpose and functionality
    pub description: String,

    /// Location/path to the utility
    pub location: String,

    /// API or usage documentation
    pub usage: String,

    /// Dependencies and requirements
    pub dependencies: Vec<String>,
}

/// Current context tracking for active sub-project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrentContext {
    /// Currently active sub-project
    pub active_sub_project: String,

    /// When context was last switched
    pub switched_on: DateTime<Utc>,

    /// Who/what triggered the context switch
    pub switched_by: String,

    /// Current status/phase description
    pub status: String,

    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Historical context snapshot for restoration and analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextSnapshot {
    /// Snapshot creation timestamp
    pub timestamp: DateTime<Utc>,

    /// Human-readable description
    pub description: String,

    /// Active sub-project at snapshot time
    pub active_sub_project: String,

    /// Workspace state at snapshot time
    pub workspace_state: WorkspaceMetadata,

    /// Sub-project states at snapshot time
    pub sub_project_states: HashMap<String, SubProject>,
}

impl Workspace {
    /// Create a new workspace with default configuration
    pub fn new(name: String, root_path: PathBuf) -> Self {
        let now = Utc::now();

        Self {
            metadata: WorkspaceMetadata {
                name: name.clone(),
                description: None,
                created_at: now,
                updated_at: now,
                version: "1.0.0".to_string(),
                root_path,
                metadata: HashMap::new(),
            },
            shared_patterns: SharedPatterns {
                implementation_patterns: Vec::new(),
                architecture_patterns: Vec::new(),
                methodology_patterns: Vec::new(),
                shared_utilities: Vec::new(),
            },
            current_context: CurrentContext {
                active_sub_project: String::new(),
                switched_on: now,
                switched_by: "system".to_string(),
                status: "initialized".to_string(),
                metadata: HashMap::new(),
            },
            sub_projects: HashMap::new(),
            snapshots: Vec::new(),
        }
    }

    /// Add a new sub-project to the workspace
    pub fn add_sub_project(&mut self, name: String, sub_project: SubProject) {
        self.sub_projects.insert(name, sub_project);
        self.metadata.updated_at = Utc::now();
    }

    /// Switch active context to a different sub-project
    pub fn switch_context(
        &mut self,
        sub_project: String,
        switched_by: String,
    ) -> Result<(), String> {
        if !self.sub_projects.contains_key(&sub_project) {
            return Err(format!("Sub-project '{}' not found", sub_project));
        }

        self.current_context = CurrentContext {
            active_sub_project: sub_project,
            switched_on: Utc::now(),
            switched_by,
            status: "active".to_string(),
            metadata: HashMap::new(),
        };

        self.metadata.updated_at = Utc::now();
        Ok(())
    }

    /// Get the currently active sub-project
    pub fn get_active_sub_project(&self) -> Option<&SubProject> {
        if self.current_context.active_sub_project.is_empty() {
            return None;
        }
        self.sub_projects
            .get(&self.current_context.active_sub_project)
    }

    /// Create a context snapshot for historical tracking
    pub fn create_snapshot(&mut self, description: String) {
        let snapshot = ContextSnapshot {
            timestamp: Utc::now(),
            description,
            active_sub_project: self.current_context.active_sub_project.clone(),
            workspace_state: self.metadata.clone(),
            sub_project_states: self.sub_projects.clone(),
        };

        self.snapshots.push(snapshot);
        self.metadata.updated_at = Utc::now();
    }
}

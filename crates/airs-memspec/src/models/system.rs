//! System architecture and technical design domain
//!
//! This module contains data structures for documenting system architecture,
//! technical decisions, component relationships, and design patterns.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::workspace::Pattern;

/// System architecture and design patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemPatterns {
    /// System architecture description
    pub architecture: ArchitectureDescription,

    /// Key technical decisions
    pub technical_decisions: Vec<TechnicalDecision>,

    /// Design patterns in use
    pub design_patterns: Vec<Pattern>,

    /// Component relationships
    pub component_relationships: Vec<ComponentRelationship>,
}

/// Architecture description and documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitectureDescription {
    /// High-level architecture overview
    pub overview: String,

    /// System components
    pub components: Vec<Component>,

    /// Data flow description
    pub data_flow: String,

    /// Integration points
    pub integrations: Vec<Integration>,

    /// Architecture diagrams or references
    pub diagrams: Vec<String>,
}

/// System component definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    /// Component name
    pub name: String,

    /// Component purpose and responsibility
    pub purpose: String,

    /// Interface definition
    pub interface: String,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Location or path
    pub location: String,
}

/// Integration point with external systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Integration {
    /// Integration name
    pub name: String,

    /// External system or service
    pub external_system: String,

    /// Integration type (API, database, file, etc.)
    pub integration_type: String,

    /// Protocol or method
    pub protocol: String,

    /// Configuration requirements
    pub configuration: HashMap<String, String>,
}

/// Technical decision documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalDecision {
    /// Decision title
    pub title: String,

    /// What was decided
    pub decision: String,

    /// Context that led to the decision
    pub context: String,

    /// Alternatives considered
    pub alternatives: Vec<String>,

    /// Rationale for the chosen approach
    pub rationale: String,

    /// Expected impact
    pub impact: String,

    /// Decision timestamp
    pub decided_at: DateTime<Utc>,

    /// Who made the decision
    pub decided_by: String,

    /// Review conditions or schedule
    pub review_criteria: Option<String>,
}

/// Component relationship definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentRelationship {
    /// Source component
    pub from_component: String,

    /// Target component
    pub to_component: String,

    /// Relationship type (depends_on, calls, inherits_from, etc.)
    pub relationship_type: String,

    /// Relationship description
    pub description: String,

    /// Any constraints or conditions
    pub constraints: Vec<String>,
}

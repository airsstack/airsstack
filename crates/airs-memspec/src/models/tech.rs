//! Technology context and infrastructure domain
//!
//! This module contains data structures for documenting technology stacks,
//! development environments, deployment contexts, and technical constraints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::monitoring::MonitoringSetup;

/// Technology context and constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechContext {
    /// Technologies used in the project
    pub technologies: Vec<Technology>,

    /// Development setup requirements
    pub development_setup: DevelopmentSetup,

    /// Technical constraints
    pub constraints: Vec<TechnicalConstraint>,

    /// Dependencies and their management
    pub dependencies: DependencyManagement,

    /// Deployment and infrastructure
    pub deployment: DeploymentContext,
}

/// Technology definition and usage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Technology {
    /// Technology name
    pub name: String,

    /// Version or version range
    pub version: String,

    /// Purpose in the project
    pub purpose: String,

    /// Configuration or setup notes
    pub configuration: String,

    /// Documentation links
    pub documentation: Vec<String>,
}

/// Development environment setup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DevelopmentSetup {
    /// Required tools and their versions
    pub required_tools: Vec<Technology>,

    /// Environment variables needed
    pub environment_variables: HashMap<String, String>,

    /// Setup instructions
    pub setup_instructions: Vec<String>,

    /// IDE or editor configuration
    pub ide_configuration: Vec<String>,

    /// Local development scripts
    pub development_scripts: HashMap<String, String>,
}

/// Technical constraint documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalConstraint {
    /// Constraint name
    pub name: String,

    /// Detailed description
    pub description: String,

    /// Why this constraint exists
    pub rationale: String,

    /// Impact on implementation
    pub impact: String,

    /// Workarounds or mitigations
    pub mitigations: Vec<String>,
}

/// Dependency management configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyManagement {
    /// Package manager used
    pub package_manager: String,

    /// Lock file location
    pub lock_file: String,

    /// Update policy
    pub update_policy: String,

    /// Security scanning configuration
    pub security_scanning: Option<String>,

    /// Known dependency issues
    pub known_issues: Vec<String>,
}

/// Deployment and infrastructure context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeploymentContext {
    /// Target environments
    pub environments: Vec<Environment>,

    /// Deployment strategy
    pub deployment_strategy: String,

    /// Infrastructure requirements
    pub infrastructure: Vec<InfrastructureRequirement>,

    /// Monitoring and observability
    pub monitoring: MonitoringSetup,
}

/// Environment definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    /// Environment name (dev, staging, prod, etc.)
    pub name: String,

    /// Environment purpose
    pub purpose: String,

    /// Configuration differences
    pub configuration: HashMap<String, String>,

    /// Access requirements
    pub access: Vec<String>,

    /// Health check endpoints
    pub health_checks: Vec<String>,
}

/// Infrastructure requirement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InfrastructureRequirement {
    /// Resource type (compute, storage, network, etc.)
    pub resource_type: String,

    /// Specification requirements
    pub specifications: HashMap<String, String>,

    /// Scaling requirements
    pub scaling: Option<String>,

    /// High availability requirements
    pub high_availability: bool,

    /// Security requirements
    pub security_requirements: Vec<String>,
}

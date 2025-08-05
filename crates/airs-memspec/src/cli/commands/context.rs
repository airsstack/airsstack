//! Context command implementation
//!
//! Displays current project context, active development focus, and architectural decisions.
//! Provides both workspace-level overview and sub-project specific context views.

use crate::cli::GlobalArgs;
use crate::parser::context::{ContextCorrelator, WorkspaceContext};
use crate::utils::fs::FsResult;
use crate::utils::output::{OutputConfig, OutputFormatter};
use crate::utils::templates::{ContextTemplate, WorkspaceContextTemplate};

/// Run the context command to display current development context
///
/// This function provides read-only access to current project context, showing:
/// - Active development focus and recent changes
/// - Architectural decisions and technical patterns
/// - Integration points and constraints
/// - Current work phase and next steps
///
/// # Arguments
///
/// * `global` - Global CLI arguments including path and output preferences
/// * `workspace` - Whether to show workspace-level context and integration points
/// * `project` - Optional sub-project name for focused context display
///
/// # Output Modes
///
/// - **Default Mode**: Shows current active sub-project context
/// - **Workspace Mode** (--workspace): Shows workspace-level integration and architecture
/// - **Sub-Project Mode** (--project <name>): Shows specific sub-project context
pub fn run(
    global: &GlobalArgs,
    workspace: bool,
    project: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize output formatter with user preferences
    let output_config = OutputConfig::new(global.no_color, global.verbose, global.quiet);
    let formatter = OutputFormatter::new(output_config);

    // Initialize context correlator and discover workspace structure
    let mut correlator = ContextCorrelator::new();

    // Determine workspace path from global args or current directory
    let current_dir = std::env::current_dir()?;
    let workspace_path = global.path.as_deref().unwrap_or(&current_dir);

    // Discover and correlate workspace context
    formatter.verbose(&format!(
        "Analyzing workspace context: {}",
        workspace_path.display()
    ));
    let workspace_context = correlator.discover_and_correlate(workspace_path)?;

    // Generate context display based on requested mode
    if workspace {
        show_workspace_context(&formatter, workspace_context)?;
    } else if let Some(project_name) = project {
        show_sub_project_context(&formatter, workspace_context, &project_name)?;
    } else {
        show_active_context(&formatter, workspace_context)?;
    }

    Ok(())
}

/// Display workspace-level context and architectural overview
///
/// Shows workspace architecture, integration points, shared patterns,
/// and cross-project relationships.
fn show_workspace_context(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
) -> FsResult<()> {
    // Use professional template system for workspace context display
    let elements = WorkspaceContextTemplate::render(workspace_context);
    formatter.render_layout(&elements);

    Ok(())
}

/// Display context for a specific sub-project
///
/// Shows detailed context for the requested sub-project including
/// active development focus, technical patterns, and current status.
fn show_sub_project_context(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
    project_name: &str,
) -> FsResult<()> {
    // Find the requested sub-project
    if let Some(sub_project_context) = workspace_context.sub_project_contexts.get(project_name) {
        // Use professional template system for project context display
        let elements = ContextTemplate::render(sub_project_context);
        formatter.render_layout(&elements);
    } else {
        formatter.error(&format!(
            "Sub-project '{}' not found. Available projects: {}",
            project_name,
            workspace_context
                .sub_project_contexts
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    Ok(())
}

/// Display context for the currently active sub-project
///
/// Shows context for whichever sub-project is currently active
/// according to current_context.md.
fn show_active_context(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
) -> FsResult<()> {
    let active_project = &workspace_context.current_context.active_sub_project;
    formatter.header(&format!("Active Context: {active_project}"));

    // Delegate to sub-project context display
    show_sub_project_context(formatter, workspace_context, active_project)
}


//! Context command implementation
//!
//! Displays current project context, active development focus, and architectural decisions.
//! Provides both workspace-level overview and sub-project specific context views.

use crate::cli::GlobalArgs;
use crate::parser::context::{ContextCorrelator, WorkspaceContext};
use crate::utils::fs::FsResult;
use crate::utils::output::{OutputConfig, OutputFormatter};

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
        show_workspace_context(&formatter, &workspace_context)?;
    } else if let Some(project_name) = project {
        show_sub_project_context(&formatter, &workspace_context, &project_name)?;
    } else {
        show_active_context(&formatter, &workspace_context)?;
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
    formatter.header("Workspace Context");

    // Active context information
    let active_project = &workspace_context.current_context.active_sub_project;
    formatter.info(&format!("🎯 Active Sub-Project: {}", active_project));

    // Sub-projects overview
    let sub_project_count = workspace_context.sub_project_contexts.len();
    formatter.info(&format!(
        "📁 Sub-Projects: {} discovered",
        sub_project_count
    ));

    // List all sub-projects with health status
    if !workspace_context.sub_project_contexts.is_empty() {
        formatter.info("\n📋 Sub-Project Overview:");
        for (name, context) in &workspace_context.sub_project_contexts {
            let health_icon = match context.derived_status.health {
                crate::parser::context::ProjectHealth::Healthy => "✅",
                crate::parser::context::ProjectHealth::Warning => "⚠️",
                crate::parser::context::ProjectHealth::Critical => "❌",
                crate::parser::context::ProjectHealth::Unknown => "❓",
            };
            let completion = context.task_summary.completion_percentage;
            formatter.info(&format!(
                "  {} {} - {:.0}% complete - {}",
                health_icon, name, completion, context.derived_status.current_phase
            ));
        }
    }

    // Workspace-level content
    display_workspace_patterns(formatter, workspace_context)?;
    display_integration_points(formatter, workspace_context)?;

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
    formatter.header(&format!("Sub-Project Context: {}", project_name));

    // Find the requested sub-project
    if let Some(sub_project_context) = workspace_context.sub_project_contexts.get(project_name) {
        // Project status
        let health_icon = match sub_project_context.derived_status.health {
            crate::parser::context::ProjectHealth::Healthy => "✅",
            crate::parser::context::ProjectHealth::Warning => "⚠️",
            crate::parser::context::ProjectHealth::Critical => "❌",
            crate::parser::context::ProjectHealth::Unknown => "❓",
        };

        formatter.info(&format!(
            "{} Health: {:?}",
            health_icon, sub_project_context.derived_status.health
        ));
        formatter.info(&format!(
            "🔄 Phase: {}",
            sub_project_context.derived_status.current_phase
        ));
        formatter.info(&format!(
            "📈 Progress: {:.1}%",
            sub_project_context.task_summary.completion_percentage
        ));

        // Active context from active_context.md
        display_active_development_focus(formatter, sub_project_context)?;

        // Technical patterns and architecture
        display_sub_project_patterns(formatter, sub_project_context)?;

        // Task summary
        display_task_context(formatter, sub_project_context)?;
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
    formatter.header(&format!("Active Context: {}", active_project));

    // Delegate to sub-project context display
    show_sub_project_context(formatter, workspace_context, active_project)
}

/// Display workspace-level architectural patterns and decisions
fn display_workspace_patterns(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
) -> FsResult<()> {
    formatter.info("\n🏗️ Workspace Architecture:");

    // Extract patterns from workspace content if available
    if let Some(shared_patterns) = &workspace_context.workspace_content.shared_patterns {
        if !shared_patterns.sections.is_empty() {
            for (title, _content) in &shared_patterns.sections {
                if title.to_lowercase().contains("pattern")
                    || title.to_lowercase().contains("standard")
                    || title.to_lowercase().contains("principle")
                {
                    formatter.info(&format!("  📐 {}", title));
                }
            }
        }
    } else {
        formatter.info("  No shared patterns documentation found");
    }

    Ok(())
}

/// Display integration points between sub-projects
fn display_integration_points(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
) -> FsResult<()> {
    formatter.info("\n🔗 Integration Points:");

    // For now, show basic integration information
    // This could be enhanced to parse actual integration documentation
    let project_count = workspace_context.sub_project_contexts.len();
    if project_count > 1 {
        formatter.info(&format!(
            "  Multi-project workspace with {} sub-projects",
            project_count
        ));
        formatter.info("  Integration analysis available in workspace documentation");
    } else {
        formatter.info("  Single sub-project workspace");
    }

    Ok(())
}

/// Display active development focus from active_context.md
fn display_active_development_focus(
    formatter: &OutputFormatter,
    sub_project_context: &crate::parser::context::SubProjectContext,
) -> FsResult<()> {
    formatter.info("\n🎯 Active Development Focus:");

    if let Some(active_context) = &sub_project_context.content.active_context {
        // Extract key sections from active context
        let focus_sections: Vec<_> = active_context
            .sections
            .iter()
            .filter(|(title, _content)| {
                title.to_lowercase().contains("focus")
                    || title.to_lowercase().contains("current")
                    || title.to_lowercase().contains("next")
                    || title.to_lowercase().contains("recent")
            })
            .take(3) // Limit to 3 most relevant sections
            .collect();

        for (title, content) in focus_sections {
            formatter.info(&format!("  🔸 {}", title));
            if !content.trim().is_empty() {
                // Show first line of content as summary
                if let Some(first_line) = content.lines().next() {
                    if !first_line.trim().is_empty() {
                        formatter.info(&format!("    {}", first_line.trim()));
                    }
                }
            }
        }
    } else {
        formatter.info("  No active context documentation found");
    }

    Ok(())
}

/// Display sub-project technical patterns and architecture
fn display_sub_project_patterns(
    formatter: &OutputFormatter,
    sub_project_context: &crate::parser::context::SubProjectContext,
) -> FsResult<()> {
    formatter.info("\n🏗️ Technical Patterns:");

    if let Some(system_patterns) = &sub_project_context.content.system_patterns {
        let pattern_sections: Vec<_> = system_patterns
            .sections
            .iter()
            .filter(|(title, _content)| {
                title.to_lowercase().contains("pattern")
                    || title.to_lowercase().contains("architecture")
                    || title.to_lowercase().contains("design")
            })
            .take(3)
            .collect();

        for (title, _content) in pattern_sections {
            formatter.info(&format!("  📐 {}", title));
        }
    } else {
        formatter.info("  No system patterns documentation found");
    }

    Ok(())
}

/// Display task-related context and current priorities
fn display_task_context(
    formatter: &OutputFormatter,
    sub_project_context: &crate::parser::context::SubProjectContext,
) -> FsResult<()> {
    let task_summary = &sub_project_context.task_summary;

    formatter.info("\n📋 Task Context:");
    formatter.info(&format!("  Total Tasks: {}", task_summary.total_tasks));

    // Show task distribution
    if !task_summary.tasks_by_status.is_empty() {
        for (status, tasks) in &task_summary.tasks_by_status {
            if !tasks.is_empty() {
                let status_icon = match status {
                    crate::parser::markdown::TaskStatus::Completed => "✅",
                    crate::parser::markdown::TaskStatus::InProgress => "🔄",
                    crate::parser::markdown::TaskStatus::NotStarted => "⏳",
                    crate::parser::markdown::TaskStatus::Blocked => "❌",
                    crate::parser::markdown::TaskStatus::Abandoned => "🚫",
                    crate::parser::markdown::TaskStatus::Unknown(_) => "❓",
                };
                formatter.info(&format!("  {} {:?}: {}", status_icon, status, tasks.len()));
            }
        }
    } // Show next priority tasks
    if !task_summary.next_tasks.is_empty() {
        formatter.info("\n🎯 Next Priority Tasks:");
        for task in task_summary.next_tasks.iter().take(3) {
            formatter.info(&format!("  • {}", task.title));
        }
    }

    Ok(())
}

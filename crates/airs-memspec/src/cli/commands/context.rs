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
            let mut pattern_count = 0;
            for (title, content) in &shared_patterns.sections {
                if (title.to_lowercase().contains("pattern")
                    || title.to_lowercase().contains("standard")
                    || title.to_lowercase().contains("principle")
                    || title.to_lowercase().contains("policy"))
                    && pattern_count < 5
                {
                    formatter.info(&format!("  📐 {}", title));

                    // Show first meaningful line of content as summary
                    let lines: Vec<&str> = content.lines().collect();
                    for line in lines.iter().take(5) {
                        let trimmed = line.trim();
                        if !trimmed.is_empty()
                            && !trimmed.starts_with('#')
                            && !trimmed.starts_with("```")
                            && trimmed.len() > 20
                        // Only show substantial content
                        {
                            formatter.info(&format!("    • {}", trimmed));
                            break;
                        }
                    }
                    pattern_count += 1;
                }
            }
        }
    }

    // Extract architecture information from workspace architecture
    if let Some(workspace_arch) = &workspace_context.workspace_content.workspace_architecture {
        formatter.info("\n  📋 Architectural Principles:");
        for (title, content) in &workspace_arch.sections {
            if title.to_lowercase().contains("pattern")
                || title.to_lowercase().contains("architecture")
                || title.to_lowercase().contains("layer")
            {
                formatter.info(&format!("    🔧 {}", title));

                // Extract key bullet points or principles
                for line in content.lines().take(3) {
                    let trimmed = line.trim();
                    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                        formatter.info(&format!("      {}", trimmed));
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Display integration points between sub-projects
fn display_integration_points(
    formatter: &OutputFormatter,
    workspace_context: &WorkspaceContext,
) -> FsResult<()> {
    formatter.info("\n🔗 Integration Points:");

    let project_count = workspace_context.sub_project_contexts.len();
    if project_count > 1 {
        formatter.info(&format!(
            "  Multi-project workspace with {} sub-projects",
            project_count
        ));

        // Analyze integration patterns from workspace architecture
        if let Some(workspace_arch) = &workspace_context.workspace_content.workspace_architecture {
            for (title, content) in &workspace_arch.sections {
                if title.to_lowercase().contains("integration")
                    || title.to_lowercase().contains("communication")
                    || title.to_lowercase().contains("dependency")
                {
                    formatter.info(&format!("  🔄 {}", title));

                    // Extract integration patterns
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if (trimmed.starts_with("- ") || trimmed.starts_with("* "))
                            && (trimmed.contains("crate")
                                || trimmed.contains("transport")
                                || trimmed.contains("API"))
                        {
                            formatter.info(&format!("    {}", trimmed));
                        }
                    }
                }
            }
        }

        // Show sub-project specializations and relationships
        formatter.info("\n  🎯 Sub-Project Specializations:");
        for (name, context) in &workspace_context.sub_project_contexts {
            if let Some(project_brief) = &context.content.project_brief {
                // Extract purpose or role from project brief
                for (section_title, section_content) in &project_brief.sections {
                    if section_title.to_lowercase().contains("purpose")
                        || section_title.to_lowercase().contains("responsibility")
                        || section_title.to_lowercase().contains("role")
                    {
                        let lines: Vec<&str> = section_content.lines().collect();
                        for line in lines.iter().take(3) {
                            let trimmed = line.trim();
                            if !trimmed.is_empty()
                                && !trimmed.starts_with('#')
                                && trimmed.len() > 15
                            {
                                formatter.info(&format!("    🔸 {}: {}", name, trimmed));
                                break;
                            }
                        }
                        break;
                    }
                }
            }
        }

        // Show workspace-level dependencies and shared resources
        if let Some(shared_patterns) = &workspace_context.workspace_content.shared_patterns {
            for (title, content) in &shared_patterns.sections {
                if title.to_lowercase().contains("dependency")
                    || title.to_lowercase().contains("shared")
                    || title.to_lowercase().contains("common")
                {
                    formatter.info(&format!("  📦 {}", title));

                    // Extract key dependency or sharing patterns
                    for line in content.lines().take(2) {
                        let trimmed = line.trim();
                        if (trimmed.starts_with("- ") || trimmed.starts_with("* "))
                            && trimmed.len() > 10
                        {
                            formatter.info(&format!("    {}", trimmed));
                        }
                    }
                    break;
                }
            }
        }
    } else {
        formatter.info("  Single sub-project workspace");

        // For single project, show internal architecture
        if let Some((project_name, context)) = workspace_context.sub_project_contexts.iter().next()
        {
            formatter.info(&format!("  🏗️ {} Internal Architecture:", project_name));

            if let Some(system_patterns) = &context.content.system_patterns {
                for (title, content) in &system_patterns.sections {
                    if title.to_lowercase().contains("component")
                        || title.to_lowercase().contains("layer")
                        || title.to_lowercase().contains("flow")
                    {
                        formatter.info(&format!("    🔧 {}", title));

                        // Show architectural components
                        for line in content.lines().take(3) {
                            let trimmed = line.trim();
                            if (trimmed.starts_with("- ") || trimmed.starts_with("* "))
                                && trimmed.len() > 15
                            {
                                formatter.info(&format!("      {}", trimmed));
                            }
                        }
                        break;
                    }
                }
            }
        }
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
                    || title.to_lowercase().contains("change")
                    || title.to_lowercase().contains("decision")
            })
            .take(4) // Expanded to show more relevant sections
            .collect();

        if !focus_sections.is_empty() {
            for (title, content) in focus_sections {
                formatter.info(&format!("  🔸 {}", title));

                // Extract bullet points or key information
                let mut shown_lines = 0;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if shown_lines < 3
                        && (trimmed.starts_with("- ")
                            || trimmed.starts_with("* ")
                            || trimmed.contains("✅")
                            || trimmed.contains("🔄")
                            || (trimmed.starts_with("**") && trimmed.contains("**")))
                    {
                        let clean_line = if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                            trimmed.to_string()
                        } else {
                            format!("  {}", trimmed)
                        };

                        if clean_line.len() > 10 {
                            formatter.info(&format!("    {}", clean_line));
                            shown_lines += 1;
                        }
                    } else if shown_lines == 0
                        && !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && trimmed.len() > 20
                    {
                        // Show first meaningful sentence if no bullet points
                        formatter.info(&format!("    {}", trimmed));
                        shown_lines += 1;
                    }
                }
            }
        } else {
            // Show any available sections if no specific focus sections found
            let any_sections: Vec<_> = active_context.sections.iter().take(2).collect();
            for (title, content) in any_sections {
                formatter.info(&format!("  📌 {}", title));
                if let Some(first_line) = content
                    .lines()
                    .find(|line| !line.trim().is_empty() && line.trim().len() > 15)
                {
                    formatter.info(&format!("    {}", first_line.trim()));
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
                    || title.to_lowercase().contains("component")
                    || title.to_lowercase().contains("flow")
            })
            .take(5)
            .collect();

        for (title, content) in pattern_sections {
            formatter.info(&format!("  📐 {}", title));

            // Extract key architectural details
            let mut detail_count = 0;
            for line in content.lines() {
                let trimmed = line.trim();
                if detail_count < 2
                    && ((trimmed.starts_with("- ") && trimmed.contains(":"))
                        || (trimmed.starts_with("* ") && trimmed.contains(":"))
                        || (trimmed.starts_with("**") && trimmed.contains("**"))
                        || (trimmed.contains("✅") && trimmed.len() > 20))
                {
                    let clean_line = trimmed
                        .replace("**", "")
                        .replace("✅", "")
                        .trim()
                        .to_string();
                    if clean_line.len() > 15 {
                        formatter.info(&format!("    • {}", clean_line));
                        detail_count += 1;
                    }
                }
            }
        }

        // Show implementation status if available
        let implementation_sections: Vec<_> = system_patterns
            .sections
            .iter()
            .filter(|(title, _content)| {
                title.to_lowercase().contains("implementation")
                    || title.to_lowercase().contains("status")
                    || title.to_lowercase().contains("objective")
            })
            .take(2)
            .collect();

        if !implementation_sections.is_empty() {
            formatter.info("\n  🎯 Implementation Status:");
            for (_title, content) in implementation_sections {
                for line in content.lines().take(4) {
                    let trimmed = line.trim();
                    if trimmed.contains("✅") || trimmed.contains("❌") || trimmed.contains("🔄")
                    {
                        formatter.info(&format!("    {}", trimmed));
                    }
                }
            }
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

// Tasks command implementation
// Handles task management and tracking operations

use crate::cli::{GlobalArgs, TaskAction};
use crate::models::types::{TaskStatus, TaskSummary};
use crate::parser::context::ContextCorrelator;
use crate::parser::navigation::MemoryBankNavigator;
use crate::utils::fs::FsError;
use crate::utils::output::OutputManager;
use chrono::{DateTime, Local};
use std::collections::HashMap;

/// Run the tasks command
pub fn run(global: &GlobalArgs, action: TaskAction) -> Result<(), Box<dyn std::error::Error>> {
    let output = OutputManager::new(global);

    match action {
        TaskAction::List { status, project } => list_tasks(global, &output, status, project),
        TaskAction::Add {
            title,
            project,
            description,
        } => add_task(global, &output, title, project, description),
        TaskAction::Update {
            task_id,
            status,
            note,
        } => update_task(global, &output, task_id, status, note),
        TaskAction::Show { task_id } => show_task(global, &output, task_id),
    }
}

/// List tasks with optional filtering
fn list_tasks(
    global: &GlobalArgs,
    output: &OutputManager,
    status_filter: Option<String>,
    project_filter: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Discover memory bank structure
    let navigator = MemoryBankNavigator::discover(global)?;
    let correlator = ContextCorrelator::new(navigator);
    let workspace_context = correlator.get_workspace_context()?;

    output.header("Task Management");

    // Apply project filter
    let projects_to_show = if let Some(project_name) = project_filter {
        // Filter to specific project
        let filtered: Vec<_> = workspace_context
            .sub_projects
            .iter()
            .filter(|p| p.name == project_name)
            .collect();

        if filtered.is_empty() {
            output.error(&format!("Project '{}' not found", project_name));
            return Err(Box::new(FsError::PathNotFound {
                path: format!("sub_projects/{}", project_name).into(),
            }));
        }
        filtered
    } else {
        // Show all projects
        workspace_context.sub_projects.iter().collect()
    };

    // Group tasks by status for better organization
    let mut task_groups: HashMap<TaskStatus, Vec<(&str, &TaskSummary)>> = HashMap::new();

    for project in projects_to_show {
        for task in &project.tasks {
            // Apply status filter
            if let Some(ref filter) = status_filter {
                let matches = match filter.as_str() {
                    "all" => true,
                    "active" => matches!(task.status, TaskStatus::InProgress),
                    "pending" => matches!(task.status, TaskStatus::Pending),
                    "completed" => matches!(task.status, TaskStatus::Completed),
                    "blocked" => matches!(task.status, TaskStatus::Blocked),
                    _ => true,
                };

                if !matches {
                    continue;
                }
            }

            task_groups
                .entry(task.status.clone())
                .or_insert_with(Vec::new)
                .push((&project.name, task));
        }
    }

    // Display tasks organized by status with priority
    let status_order = [
        TaskStatus::InProgress,
        TaskStatus::Blocked,
        TaskStatus::Pending,
        TaskStatus::Completed,
    ];

    let mut total_tasks = 0;

    for status in status_order {
        if let Some(tasks) = task_groups.get(&status) {
            if tasks.is_empty() {
                continue;
            }

            // Sort tasks by priority (implementation needed) and then by name
            let mut sorted_tasks = tasks.clone();
            sorted_tasks.sort_by(|a, b| {
                // First sort by project name, then by task ID
                a.0.cmp(b.0).then_with(|| a.1.id.cmp(&b.1.id))
            });

            let status_header = match status {
                TaskStatus::InProgress => "🚀 In Progress",
                TaskStatus::Blocked => "🚫 Blocked",
                TaskStatus::Pending => "📋 Pending",
                TaskStatus::Completed => "✅ Completed",
            };

            output.success(&format!(
                "\n{} ({} tasks)",
                status_header,
                sorted_tasks.len()
            ));
            output.separator();

            for (project_name, task) in sorted_tasks {
                let progress_info = if task.completion_percentage > 0 {
                    format!(" - {}%", task.completion_percentage)
                } else {
                    String::new()
                };

                let project_prefix = if project_filter.is_none() {
                    format!("[{}] ", project_name)
                } else {
                    String::new()
                };

                output.info(&format!(
                    "  {}{} - {}{}",
                    project_prefix, task.id, task.title, progress_info
                ));

                if let Some(ref description) = task.description {
                    output.verbose(&format!("    📝 {}", description));
                }

                // Show latest update if available
                if let Some(ref updated) = task.last_updated {
                    output.verbose(&format!("    🕒 Updated: {}", updated));
                }
            }

            total_tasks += sorted_tasks.len();
        }
    }

    if total_tasks == 0 {
        let filter_desc = match (&status_filter, &project_filter) {
            (Some(s), Some(p)) => format!(" matching status '{}' in project '{}'", s, p),
            (Some(s), None) => format!(" with status '{}'", s),
            (None, Some(p)) => format!(" in project '{}'", p),
            (None, None) => String::new(),
        };

        output.warning(&format!("No tasks found{}", filter_desc));
    } else {
        output.success(&format!("\nTotal: {} tasks displayed", total_tasks));
    }

    Ok(())
}

/// Add a new task (placeholder implementation)
fn add_task(
    _global: &GlobalArgs,
    output: &OutputManager,
    title: String,
    project: Option<String>,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    output.info("Task creation functionality");
    output.info(&format!("Title: {}", title));

    if let Some(proj) = project {
        output.info(&format!("Project: {}", proj));
    }

    if let Some(desc) = description {
        output.info(&format!("Description: {}", desc));
    }

    output.warning("Task creation not yet implemented - requires file writing capabilities");
    Ok(())
}

/// Update an existing task (placeholder implementation)
fn update_task(
    _global: &GlobalArgs,
    output: &OutputManager,
    task_id: String,
    status: Option<String>,
    note: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    output.info("Task update functionality");
    output.info(&format!("Task ID: {}", task_id));

    if let Some(stat) = status {
        output.info(&format!("New Status: {}", stat));
    }

    if let Some(note_text) = note {
        output.info(&format!("Note: {}", note_text));
    }

    output.warning("Task updates not yet implemented - requires file writing capabilities");
    Ok(())
}

/// Show detailed task information
fn show_task(
    global: &GlobalArgs,
    output: &OutputManager,
    task_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Discover memory bank structure
    let navigator = MemoryBankNavigator::discover(global)?;
    let correlator = ContextCorrelator::new(navigator);
    let workspace_context = correlator.get_workspace_context()?;

    // Find the task across all projects
    let mut found_task = None;
    let mut found_project = None;

    for project in &workspace_context.sub_projects {
        for task in &project.tasks {
            if task.id == task_id {
                found_task = Some(task);
                found_project = Some(&project.name);
                break;
            }
        }
        if found_task.is_some() {
            break;
        }
    }

    match found_task {
        Some(task) => {
            output.header(&format!("Task Details: {}", task.id));
            output.info(&format!("Title: {}", task.title));
            output.info(&format!("Project: {}", found_project.unwrap()));
            output.info(&format!("Status: {:?}", task.status));

            if task.completion_percentage > 0 {
                output.info(&format!("Progress: {}%", task.completion_percentage));
            }

            if let Some(ref description) = task.description {
                output.info(&format!("Description: {}", description));
            }

            if let Some(ref updated) = task.last_updated {
                output.info(&format!("Last Updated: {}", updated));
            }

            // TODO: Show subtasks and progress log if available
            output.verbose("💡 Use the tasks update command to modify this task");
        }
        None => {
            output.error(&format!("Task '{}' not found", task_id));
            output.info("💡 Use 'tasks list' to see all available tasks");
            return Err(Box::new(FsError::PathNotFound {
                path: format!("task {}", task_id).into(),
            }));
        }
    }

    Ok(())
}

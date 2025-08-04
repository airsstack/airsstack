// Tasks command implementation
// Handles task management and tracking operations

use crate::cli::{GlobalArgs, TaskAction};
use crate::models::types::TaskStatus;
use crate::parser::context::ContextCorrelator;
use crate::parser::markdown::TaskItem;
use crate::parser::navigation::MemoryBankNavigator;
use crate::utils::fs::FsError;
use crate::utils::output::{OutputConfig, OutputFormatter};
use std::collections::HashMap;
use std::path::PathBuf;

/// Run the tasks command
pub fn run(global: &GlobalArgs, action: TaskAction) -> Result<(), Box<dyn std::error::Error>> {
    let output_config = OutputConfig::new(global.no_color, global.verbose, global.quiet);
    let formatter = OutputFormatter::new(output_config);

    match action {
        TaskAction::List { status, project } => list_tasks(global, &formatter, status, project),
        TaskAction::Add {
            title,
            project,
            description,
        } => add_task(global, &formatter, title, project, description),
        TaskAction::Update {
            task_id,
            status,
            note,
        } => update_task(global, &formatter, task_id, status, note),
        TaskAction::Show { task_id } => show_task(global, &formatter, task_id),
    }
}

/// List tasks with optional filtering
fn list_tasks(
    global: &GlobalArgs,
    formatter: &OutputFormatter,
    status_filter: Option<String>,
    project_filter: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory for memory bank discovery
    let start_path = if let Some(ref path) = global.path {
        path.clone()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    // Discover memory bank structure
    let structure = MemoryBankNavigator::discover_structure(&start_path)?;
    let mut correlator = ContextCorrelator::new();
    let workspace_context = correlator.discover_and_correlate(&structure.root_path)?;

    formatter.header("Task Management");

    // Apply project filter
    let projects_to_show = if let Some(project_name) = project_filter {
        // Filter to specific project
        let filtered: Vec<_> = workspace_context
            .sub_project_contexts
            .iter()
            .filter(|(name, _)| **name == project_name)
            .collect();

        if filtered.is_empty() {
            formatter.error(&format!("Project '{}' not found", project_name));
            return Err(Box::new(FsError::PathNotFound {
                path: format!("sub_projects/{}", project_name).into(),
            }));
        }
        filtered
    } else {
        // Show all projects
        workspace_context.sub_project_contexts.iter().collect()
    };

    // Group tasks by status for better organization
    let mut task_groups: HashMap<TaskStatus, Vec<(&str, &TaskItem)>> = HashMap::new();

    for (project_name, project_context) in projects_to_show {
        // Get all tasks from the task summary
        for (status, tasks) in &project_context.task_summary.tasks_by_status {
            for task in tasks {
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
                    .entry(status.clone())
                    .or_insert_with(Vec::new)
                    .push((project_name, task));
            }
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

            // Sort tasks by project name, then by task ID
            let mut sorted_tasks = tasks.clone();
            sorted_tasks.sort_by(|a, b| {
                a.0.cmp(b.0).then_with(|| match (&a.1.id, &b.1.id) {
                    (Some(a_id), Some(b_id)) => a_id.cmp(b_id),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.1.title.cmp(&b.1.title),
                })
            });

            let status_header = match status {
                TaskStatus::InProgress => "🚀 In Progress",
                TaskStatus::Blocked => "🚫 Blocked",
                TaskStatus::Pending => "📋 Pending",
                TaskStatus::Completed => "✅ Completed",
            };

            formatter.success(&format!(
                "\n{} ({} tasks)",
                status_header,
                sorted_tasks.len()
            ));
            formatter.separator();

            for (project_name, task) in sorted_tasks {
                let project_prefix = if project_filter.is_none() {
                    format!("[{}] ", project_name)
                } else {
                    String::new()
                };

                let task_id = task.id.as_deref().unwrap_or("no-id");

                formatter.info(&format!("  {}{} - {}", project_prefix, task_id, task.title));

                if let Some(ref details) = task.details {
                    formatter.verbose(&format!("    📝 {}", details));
                }

                // Show latest update if available
                if let Some(ref updated) = task.updated {
                    formatter.verbose(&format!("    🕒 Updated: {}", updated));
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

        formatter.warning(&format!("No tasks found{}", filter_desc));
    } else {
        formatter.success(&format!("\nTotal: {} tasks displayed", total_tasks));

        // Show task summary statistics
        for (project_name, project_context) in workspace_context.sub_project_contexts.iter() {
            if project_filter.is_some() && Some(project_name.as_str()) != project_filter.as_deref()
            {
                continue;
            }

            let summary = &project_context.task_summary;
            if summary.total_tasks > 0 {
                formatter.verbose(&format!(
                    "📊 [{}] {} total tasks - {:.1}% complete",
                    project_name, summary.total_tasks, summary.completion_percentage
                ));
            }
        }
    }

    Ok(())
}

/// Add a new task (placeholder implementation)
fn add_task(
    _global: &GlobalArgs,
    formatter: &OutputFormatter,
    title: String,
    project: Option<String>,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    formatter.info("Task creation functionality");
    formatter.info(&format!("Title: {}", title));

    if let Some(proj) = project {
        formatter.info(&format!("Project: {}", proj));
    }

    if let Some(desc) = description {
        formatter.info(&format!("Description: {}", desc));
    }

    formatter.warning("Task creation not yet implemented - requires file writing capabilities");
    Ok(())
}

/// Update an existing task (placeholder implementation)
fn update_task(
    _global: &GlobalArgs,
    formatter: &OutputFormatter,
    task_id: String,
    status: Option<String>,
    note: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    formatter.info("Task update functionality");
    formatter.info(&format!("Task ID: {}", task_id));

    if let Some(stat) = status {
        formatter.info(&format!("New Status: {}", stat));
    }

    if let Some(note_text) = note {
        formatter.info(&format!("Note: {}", note_text));
    }

    formatter.warning("Task updates not yet implemented - requires file writing capabilities");
    Ok(())
}

/// Show detailed task information
fn show_task(
    global: &GlobalArgs,
    formatter: &OutputFormatter,
    task_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory for memory bank discovery
    let start_path = if let Some(ref path) = global.path {
        path.clone()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    // Discover memory bank structure
    let structure = MemoryBankNavigator::discover_structure(&start_path)?;
    let mut correlator = ContextCorrelator::new();
    let workspace_context = correlator.discover_and_correlate(&structure.root_path)?;

    // Find the task across all projects
    let mut found_task = None;
    let mut found_project = None;

    for (project_name, project_context) in &workspace_context.sub_project_contexts {
        for (_status, tasks) in &project_context.task_summary.tasks_by_status {
            for task in tasks {
                if let Some(ref id) = task.id {
                    if *id == task_id {
                        found_task = Some(task);
                        found_project = Some(project_name.as_str());
                        break;
                    }
                }
            }
            if found_task.is_some() {
                break;
            }
        }
        if found_task.is_some() {
            break;
        }
    }

    match found_task {
        Some(task) => {
            let task_id_display = task.id.as_deref().unwrap_or("no-id");
            formatter.header(&format!("Task Details: {}", task_id_display));
            formatter.info(&format!("Title: {}", task.title));
            formatter.info(&format!("Project: {}", found_project.unwrap()));
            formatter.info(&format!("Status: {:?}", task.status));

            if let Some(ref details) = task.details {
                formatter.info(&format!("Details: {}", details));
            }

            if let Some(ref updated) = task.updated {
                formatter.info(&format!("Last Updated: {}", updated));
            }

            // Show project-level task summary
            if let Some(project_context) = workspace_context
                .sub_project_contexts
                .get(found_project.unwrap())
            {
                let summary = &project_context.task_summary;
                formatter.verbose(&format!(
                    "📊 Project has {} total tasks ({:.1}% complete)",
                    summary.total_tasks, summary.completion_percentage
                ));
            }

            formatter.verbose("💡 Use the tasks update command to modify this task");
        }
        None => {
            formatter.error(&format!("Task '{}' not found", task_id));
            formatter.info("💡 Use 'tasks list' to see all available tasks");
            return Err(Box::new(FsError::PathNotFound {
                path: format!("task {}", task_id).into(),
            }));
        }
    }

    Ok(())
}

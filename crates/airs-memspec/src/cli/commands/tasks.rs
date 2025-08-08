// Tasks command implementation
// Handles task viewing and tracking operations (read-only)

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{NaiveDate, Utc};

use crate::cli::{GlobalArgs, TaskAction};
use crate::parser::context::ContextCorrelator;
use crate::parser::markdown::{TaskItem, TaskStatus};
use crate::parser::navigation::MemoryBankNavigator;
use crate::utils::fs::FsError;
use crate::utils::output::{OutputConfig, OutputFormatter};

/// Run the tasks command
pub fn run(global: &GlobalArgs, action: TaskAction) -> Result<(), Box<dyn std::error::Error>> {
    let output_config = OutputConfig::new(global.no_color, global.verbose, global.quiet);
    let formatter = OutputFormatter::new(output_config);

    match action {
        TaskAction::List {
            status,
            project,
            show_all,
            include_completed,
        } => list_tasks(
            global,
            &formatter,
            status,
            project,
            show_all,
            include_completed,
        ),
        TaskAction::Show { task_id } => show_task(global, &formatter, task_id),
    }
}

/// List tasks with optional filtering
fn list_tasks(
    global: &GlobalArgs,
    formatter: &OutputFormatter,
    status_filter: Option<String>,
    project_filter: Option<String>,
    show_all: bool,
    include_completed: bool,
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

    formatter.header("Task Tracking (Read-Only)");

    // Apply project filter
    let projects_to_show = if let Some(ref project_name) = project_filter {
        // Filter to specific project
        let filtered: Vec<_> = workspace_context
            .sub_project_contexts
            .iter()
            .filter(|(name, _)| *name == project_name)
            .collect();

        if filtered.is_empty() {
            formatter.error(&format!("Project '{project_name}' not found"));
            return Err(Box::new(FsError::PathNotFound {
                path: format!("sub_projects/{project_name}").into(),
            }));
        }
        filtered
    } else {
        // Show all projects
        workspace_context.sub_project_contexts.iter().collect()
    };

    // Group tasks by status for better organization
    let mut task_groups: HashMap<TaskStatus, Vec<(&str, &TaskItem)>> = HashMap::new();

    // Determine if we should apply smart filtering
    let use_smart_filtering = !show_all && status_filter.is_none() && project_filter.is_none();

    // Get active project from current_context.md for smart filtering
    let active_project = if use_smart_filtering {
        get_active_project_from_context(&structure.root_path)
    } else {
        None
    };

    for (project_name, project_context) in projects_to_show {
        // Get all tasks from the task summary
        for (status, tasks) in &project_context.task_summary.tasks_by_status {
            for task in tasks {
                // Skip tasks without proper IDs (e.g., checkbox items from planning docs)
                if task.id.is_none() {
                    continue;
                }

                // Apply filtering logic
                let should_include = if use_smart_filtering {
                    apply_smart_filter(
                        task,
                        project_name,
                        active_project.as_deref(),
                        include_completed,
                    )
                } else {
                    apply_standard_filter(task, &status_filter, include_completed)
                };

                if should_include {
                    task_groups
                        .entry(status.clone())
                        .or_default()
                        .push((project_name, task));
                }
            }
        }
    }

    // Apply smart limit if using smart filtering
    let task_groups = if use_smart_filtering {
        apply_smart_limit(task_groups, 15)
    } else {
        task_groups
    };

    // Display tasks organized by status with priority
    let status_order = [
        TaskStatus::InProgress,
        TaskStatus::Blocked,
        TaskStatus::NotStarted,
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
                TaskStatus::NotStarted => "📋 Pending",
                TaskStatus::Completed => "✅ Completed",
                TaskStatus::Abandoned => "❌ Abandoned",
                TaskStatus::Unknown(_) => "❓ Unknown",
            };

            formatter.section_divider(Some(&format!(
                "{} ({} tasks)",
                status_header,
                sorted_tasks.len()
            )));
            eprintln!(); // Add spacing

            for (project_name, task) in &sorted_tasks {
                let project_prefix = if project_filter.is_none() {
                    format!("[{project_name}] ")
                } else {
                    String::new()
                };

                let task_id = task.id.as_deref().unwrap_or("no-id");

                // Add stale indicator for tasks that haven't been updated recently
                let stale_indicator = if is_task_stale(task, 7) {
                    match task.status {
                        TaskStatus::InProgress => " 🕒", // Clock for stale in-progress
                        TaskStatus::NotStarted => " ⏰", // Alarm for stale pending
                        _ => "",
                    }
                } else {
                    ""
                };

                formatter.bullet_point(
                    "▶",
                    &format!(
                        "{}{} - {}{}",
                        project_prefix, task_id, task.title, stale_indicator
                    ),
                    0,
                );

                if let Some(ref details) = task.details {
                    formatter.bullet_point("📝", details, 1);
                }

                // Show latest update if available, with stale warning
                if let Some(ref updated) = task.updated {
                    let update_info = if is_task_stale(task, 7) {
                        format!("Updated: {updated} (STALE - over 7 days ago)")
                    } else {
                        format!("Updated: {updated}")
                    };
                    formatter.bullet_point("🕒", &update_info, 1);
                }

                eprintln!(); // Add spacing between tasks
            }

            total_tasks += sorted_tasks.len();
        }
    }

    if total_tasks == 0 {
        let filter_desc = match (&status_filter, &project_filter) {
            (Some(s), Some(p)) => format!(" matching status '{s}' in project '{p}'"),
            (Some(s), None) => format!(" with status '{s}'"),
            (None, Some(p)) => format!(" in project '{p}'"),
            (None, None) => String::new(),
        };

        formatter.warning(&format!("No tasks found{filter_desc}"));
    } else {
        formatter.separator();

        // Enhanced project summary with visual progress indicators
        for (project_name, project_context) in workspace_context.sub_project_contexts.iter() {
            if project_filter.is_some() && Some(project_name.as_str()) != project_filter.as_deref()
            {
                continue;
            }

            let summary = &project_context.task_summary;
            if summary.total_tasks > 0 {
                let completed = summary
                    .tasks_by_status
                    .get(&TaskStatus::Completed)
                    .map(|tasks| tasks.len())
                    .unwrap_or(0);
                let in_progress = summary
                    .tasks_by_status
                    .get(&TaskStatus::InProgress)
                    .map(|tasks| tasks.len())
                    .unwrap_or(0);
                let pending = summary
                    .tasks_by_status
                    .get(&TaskStatus::NotStarted)
                    .map(|tasks| tasks.len())
                    .unwrap_or(0);

                formatter.task_summary(
                    summary.total_tasks,
                    completed,
                    in_progress,
                    pending,
                    Some(project_name),
                );
            }
        }

        // Provide contextual help messages based on filtering mode
        if use_smart_filtering {
            formatter.verbose("🧠 Smart filtering active: showing 15 most relevant tasks");
            if let Some(ref active) = active_project {
                formatter.verbose(&format!("📋 Focusing on active project: {active}"));
            }
            formatter
                .verbose("🕒 Stale detection: tasks unchanged for 7+ days marked with 🕒 or ⏰");
            formatter.verbose(
                "💡 Use --all to see all tasks or --status/--project for custom filtering",
            );
            formatter.verbose("💡 Use --completed to include completed tasks in smart view");
        } else {
            formatter
                .verbose("🕒 Stale detection: tasks unchanged for 7+ days marked with clock icons");
            formatter.verbose(
                "💡 Use --status <filter> to narrow results (active, pending, completed, blocked)",
            );
            if project_filter.is_none() && workspace_context.sub_project_contexts.len() > 1 {
                formatter.verbose("💡 Use --project <name> to focus on a specific project");
            }
        }
    }

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
        for tasks in project_context.task_summary.tasks_by_status.values() {
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
            formatter.header(&format!("Task Details: {task_id_display}"));
            formatter.info(&format!("Title: {}", task.title));
            formatter.info(&format!("Project: {}", found_project.unwrap()));
            formatter.info(&format!("Status: {:?}", task.status));

            if let Some(ref details) = task.details {
                formatter.info(&format!("Details: {details}"));
            }

            if let Some(ref updated) = task.updated {
                formatter.info(&format!("Last Updated: {updated}"));
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

            formatter.verbose("💡 Use 'tasks list' to filter and view tasks")
        }
        None => {
            formatter.error(&format!("Task '{task_id}' not found"));
            formatter.info("💡 Use 'tasks list' to see all available tasks");
            return Err(Box::new(FsError::PathNotFound {
                path: format!("task {task_id}").into(),
            }));
        }
    }

    Ok(())
}

/// Get the active project from current_context.md
fn get_active_project_from_context(root_path: &PathBuf) -> Option<String> {
    use std::fs;

    let context_file = root_path.join("current_context.md");
    if let Ok(content) = fs::read_to_string(&context_file) {
        // Look for "Active Sub-Project:" pattern in the content
        for line in content.lines() {
            if line.starts_with("**Active Sub-Project:**") {
                // Extract project name from "**Active Sub-Project:** project-name"
                if let Some(project) = line.split(':').nth(1) {
                    return Some(project.trim().to_string());
                }
            }
        }
    }

    None
}

/// Apply smart filtering rules
fn apply_smart_filter(
    task: &TaskItem,
    project_name: &str,
    active_project: Option<&str>,
    include_completed: bool,
) -> bool {
    // Check if task is stale (in progress for more than 7 days)
    let is_stale = is_task_stale(task, 7);

    match task.status {
        // Always show in-progress tasks (regardless of project)
        // But prioritize stale tasks for attention
        TaskStatus::InProgress => true,

        // Always show blocked tasks (high priority)
        TaskStatus::Blocked => true,

        // Show pending tasks only from active project (or all if no active project)
        // Also show stale pending tasks as they may need attention
        TaskStatus::NotStarted => {
            is_stale || active_project.map_or(true, |active| project_name == active)
        }

        // Show completed tasks only if explicitly requested
        TaskStatus::Completed => include_completed,

        // Show abandoned tasks only if explicitly requested
        TaskStatus::Abandoned => include_completed,

        // Unknown status - be conservative and include
        TaskStatus::Unknown(_) => true,
    }
}

/// Apply standard filtering (non-smart mode)
fn apply_standard_filter(
    task: &TaskItem,
    status_filter: &Option<String>,
    include_completed: bool,
) -> bool {
    // Apply status filter
    if let Some(ref filter) = status_filter {
        let matches = match filter.as_str() {
            "all" => true,
            "active" => matches!(task.status, TaskStatus::InProgress),
            "pending" => matches!(task.status, TaskStatus::NotStarted),
            "completed" => matches!(task.status, TaskStatus::Completed),
            "blocked" => matches!(task.status, TaskStatus::Blocked),
            _ => true,
        };

        if !matches {
            return false;
        }
    }

    // Apply completed filter unless explicitly included
    if matches!(task.status, TaskStatus::Completed | TaskStatus::Abandoned) && !include_completed {
        return false;
    }

    true
}

/// Apply smart limit of 15 tasks, prioritizing by status
fn apply_smart_limit<'a>(
    mut task_groups: HashMap<TaskStatus, Vec<(&'a str, &'a TaskItem)>>,
    limit: usize,
) -> HashMap<TaskStatus, Vec<(&'a str, &'a TaskItem)>> {
    let mut total_tasks = 0;

    // Count total tasks
    for tasks in task_groups.values() {
        total_tasks += tasks.len();
    }

    // If under limit, return as-is
    if total_tasks <= limit {
        return task_groups;
    }

    // Priority order for smart limiting
    let priority_order = [
        TaskStatus::InProgress,
        TaskStatus::Blocked,
        TaskStatus::NotStarted,
        TaskStatus::Completed,
        TaskStatus::Abandoned,
    ];

    let mut remaining_limit = limit;
    let mut filtered_groups = HashMap::new();

    for status in priority_order {
        if remaining_limit == 0 {
            break;
        }

        if let Some(mut tasks) = task_groups.remove(&status) {
            if tasks.len() <= remaining_limit {
                // Take all tasks of this status
                remaining_limit -= tasks.len();
                filtered_groups.insert(status, tasks);
            } else {
                // Take only the most recent tasks (assuming they are sorted)
                tasks.truncate(remaining_limit);
                filtered_groups.insert(status, tasks);
                remaining_limit = 0;
            }
        }
    }

    filtered_groups
}

/// Check if a task is stale (unchanged for more than specified days)
fn is_task_stale(task: &TaskItem, days_threshold: i64) -> bool {
    if let Some(ref updated_str) = task.updated {
        // Try to parse the date in YYYY-MM-DD format
        if let Ok(updated_date) = NaiveDate::parse_from_str(updated_str, "%Y-%m-%d") {
            let updated_utc = updated_date
                .and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc())
                .unwrap_or_else(|| Utc::now());

            let days_since_update = (Utc::now() - updated_utc).num_days();
            return days_since_update >= days_threshold;
        }
    }

    // If no valid date or parsing fails, consider it stale to be safe
    true
}

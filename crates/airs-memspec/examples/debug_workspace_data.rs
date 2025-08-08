// Quick test to examine actual data in WorkspaceContext
use airs_memspec::parser::context::ContextCorrelator;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let workspace_path = current_dir.parent().unwrap().parent().unwrap(); // Go up to workspace root

    println!("🔍 Analyzing workspace at: {}", workspace_path.display());

    let mut correlator = ContextCorrelator::new();
    let workspace_context = correlator.discover_and_correlate(workspace_path)?;

    println!("\n📊 WORKSPACE DATA ANALYSIS:");
    println!("Current Context: {:?}", workspace_context.current_context);

    println!(
        "\n📂 Sub-projects found: {}",
        workspace_context.sub_project_contexts.len()
    );
    for (name, context) in &workspace_context.sub_project_contexts {
        println!("\n🔸 Project: {}", name);
        println!("  • Health: {:?}", context.derived_status.health);
        println!("  • Total Tasks: {}", context.task_summary.total_tasks);
        println!(
            "  • Completion: {:.1}%",
            context.task_summary.completion_percentage
        );
        println!(
            "  • Current Phase: {}",
            context.derived_status.current_phase
        );

        if let Some(active_context) = &context.content.active_context {
            if let Some(title) = &active_context.metadata.title {
                println!("  • Active Context Title: {}", title);
            }
            println!(
                "  • Active Context Content: {} chars",
                active_context.content.len()
            );
        }

        if let Some(progress) = &context.content.progress {
            if let Some(title) = &progress.metadata.title {
                println!("  • Progress Title: {}", title);
            }
            println!("  • Progress Content: {} chars", progress.content.len());
        }
    }

    if let Some(workspace_progress) = &workspace_context.workspace_content.workspace_progress {
        println!("\n🏢 Workspace Progress Data:");
        if let Some(title) = &workspace_progress.metadata.title {
            println!("  • Title: {}", title);
        }
        println!(
            "  • Content length: {} chars",
            workspace_progress.content.len()
        );
        // Print first few lines
        let lines: Vec<&str> = workspace_progress.content.lines().take(3).collect();
        for line in lines {
            if !line.trim().is_empty() {
                println!("  • Content: {}", line.trim());
                break; // Just one line for brevity
            }
        }
    }

    Ok(())
}

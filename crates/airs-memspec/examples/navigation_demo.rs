//! Example demonstrating memory bank navigation functionality

use airs_memspec::parser::navigation::MemoryBankNavigator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to discover the memory bank structure from current directory
    let current_dir = std::env::current_dir()?;
    println!(
        "Searching for memory bank structure starting from: {}",
        current_dir.display()
    );

    match MemoryBankNavigator::discover_structure(&current_dir) {
        Ok(structure) => {
            println!("✅ Memory bank structure discovered!");
            println!("Root path: {}", structure.root_path.display());

            // Show workspace files
            println!("\n📁 Workspace files:");
            if let Some(path) = &structure.workspace.project_brief {
                println!("  • project_brief.md: {}", path.display());
            }
            if let Some(path) = &structure.workspace.shared_patterns {
                println!("  • shared_patterns.md: {}", path.display());
            }
            if let Some(path) = &structure.workspace.workspace_architecture {
                println!("  • workspace_architecture.md: {}", path.display());
            }
            if let Some(path) = &structure.workspace.workspace_progress {
                println!("  • workspace_progress.md: {}", path.display());
            }

            // Show current context
            if let Some(path) = &structure.current_context {
                println!("\n📋 Current context: {}", path.display());

                // Try to get active sub-project
                match MemoryBankNavigator::get_active_sub_project(&structure) {
                    Ok(Some(active_project)) => {
                        println!("  🎯 Active sub-project: {active_project}");
                    }
                    Ok(None) => {
                        println!("  ⚠️  No active sub-project specified");
                    }
                    Err(e) => {
                        println!("  ❌ Error reading active sub-project: {e}");
                    }
                }
            }

            // Show sub-projects
            println!("\n🏗️  Sub-projects ({}):", structure.sub_projects.len());
            for (name, sub_project) in &structure.sub_projects {
                println!("  📦 {name}");
                println!("    Path: {}", sub_project.root_path.display());

                let file_count = [
                    &sub_project.project_brief,
                    &sub_project.product_context,
                    &sub_project.active_context,
                    &sub_project.system_patterns,
                    &sub_project.tech_context,
                    &sub_project.progress,
                ]
                .iter()
                .filter(|f| f.is_some())
                .count();

                println!(
                    "    Files: {} core files, {} task files",
                    file_count,
                    sub_project.task_files.len()
                );
            }

            // Show validation results
            let warnings = MemoryBankNavigator::validate_structure(&structure);
            if !warnings.is_empty() {
                println!("\n⚠️  Structure validation warnings:");
                for warning in warnings {
                    println!("  • {warning}");
                }
            } else {
                println!("\n✅ Memory bank structure is complete!");
            }
        }
        Err(e) => {
            println!("❌ Failed to discover memory bank structure: {e}");
            println!("\n💡 Make sure you're running this from within a workspace that has a .copilot/memory_bank/ directory");
        }
    }

    Ok(())
}

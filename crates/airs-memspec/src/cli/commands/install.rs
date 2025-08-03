// Install command implementation
// Handles memory bank setup and initial configuration

use crate::cli::GlobalArgs;
use crate::embedded::instructions::{available_templates, InstructionTemplate};
use crate::utils::fs::{self, FsResult};
use std::path::PathBuf;

/// Default installation directory relative to current directory
const DEFAULT_INSTALL_DIR: &str = ".copilot/instructions";

/// Run the install command
pub fn run(
    global: &GlobalArgs,
    target: Option<PathBuf>,
    force: bool,
    template: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine the target directory
    let target_dir = determine_target_directory(&global.path, target)
        .map_err(|e| format!("Failed to determine target directory: {}", e))?;

    // Select the instruction template
    let instruction_template = select_template(template.as_deref())?;

    // Check if files exist and handle accordingly
    let file_path = target_dir.join(instruction_template.filename());
    if fs::path_exists(&file_path) && !force {
        return Err(format!(
            "File already exists: {}\nUse --force to overwrite existing files",
            file_path.display()
        )
        .into());
    }

    // Perform the installation
    install_instructions(&target_dir, &instruction_template, force)
        .map_err(|e| format!("Installation failed: {}", e))?;

    // Report success
    println!(
        "✅ Successfully installed {} to {}",
        instruction_template.description(),
        target_dir.display()
    );

    if !global.quiet {
        println!("📁 File: {}", file_path.display());
        println!("🎯 Use these instructions with GitHub Copilot for enhanced AI assistance");

        if global.verbose {
            println!("📊 Installation details:");
            println!("   Template: {}", instruction_template.description());
            println!("   Target directory: {}", target_dir.display());
            println!(
                "   File size: {} bytes",
                instruction_template.content().len()
            );
        }
    }

    Ok(())
}

/// Determine the target directory for installation
fn determine_target_directory(
    global_path: &Option<PathBuf>,
    target: Option<PathBuf>,
) -> FsResult<PathBuf> {
    let base_dir = match global_path {
        Some(path) => fs::resolve_path(path)?,
        None => fs::resolve_path(".")?,
    };

    let target_dir = match target {
        Some(path) => {
            if path.is_absolute() {
                path
            } else {
                base_dir.join(path)
            }
        }
        None => base_dir.join(DEFAULT_INSTALL_DIR),
    };

    fs::resolve_path(target_dir)
}

/// Select the appropriate instruction template
fn select_template(
    template_name: Option<&str>,
) -> Result<InstructionTemplate, Box<dyn std::error::Error>> {
    let templates = available_templates();

    match template_name {
        Some(name) => {
            // Try to find template by name (case-insensitive)
            let name_lower = name.to_lowercase();
            for template in templates {
                if template.filename().to_lowercase().contains(&name_lower)
                    || template.description().to_lowercase().contains(&name_lower)
                    || name_lower.contains("multi")
                    || name_lower.contains("memory")
                {
                    return Ok(template);
                }
            }
            Err(format!(
                "Template '{}' not found. Available templates: {}",
                name,
                available_templates()
                    .iter()
                    .map(|t| t.description())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .into())
        }
        None => {
            // Default to the first (and currently only) template
            Ok(templates.into_iter().next().unwrap())
        }
    }
}

/// Install the instruction files to the target directory
fn install_instructions(
    target_dir: &PathBuf,
    template: &InstructionTemplate,
    force: bool,
) -> FsResult<()> {
    // Create the target directory
    fs::create_dir_all(target_dir)?;

    // Validate that we can write to the directory
    fs::validate_writable_directory(target_dir)?;

    // Write the instruction file
    let file_path = target_dir.join(template.filename());
    fs::write_file_with_overwrite(&file_path, template.content(), force)?;

    // Validate the installation
    validate_installation(target_dir, template)?;

    Ok(())
}

/// Validate that the installation was successful
fn validate_installation(target_dir: &PathBuf, template: &InstructionTemplate) -> FsResult<()> {
    let file_path = target_dir.join(template.filename());

    // Check that the file exists and is readable
    if !fs::is_file(&file_path) {
        return Err(crate::utils::fs::FsError::PathNotFound { path: file_path });
    }

    // Verify the file content matches what we wrote
    let content = std::fs::read_to_string(&file_path).map_err(crate::utils::fs::FsError::Io)?;

    if content != template.content() {
        return Err(crate::utils::fs::FsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File content validation failed",
        )));
    }

    Ok(())
}

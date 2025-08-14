//! Install command implementation for airs-memspec CLI
//!
//! This module handles the deployment of Multi-Project Memory Bank instructions
//! to the user's workspace, providing the foundation for AI-enhanced development
//! workflows with GitHub Copilot.
//!
//! # Installation Process
//!
//! The install command follows a comprehensive workflow:
//! 1. **Path Resolution**: Determine target directory with fallback to defaults
//! 2. **Template Selection**: Choose appropriate instruction template
//! 3. **Conflict Detection**: Check for existing files and handle overwrite logic
//! 4. **Validation**: Verify write permissions and directory structure
//! 5. **Deployment**: Write instruction files with integrity verification
//! 6. **Confirmation**: Provide user feedback and next steps
//!
//! # Safety & Reliability
//!
//! - **Non-destructive by default**: Requires explicit `--force` to overwrite
//! - **Path validation**: Comprehensive checks for permissions and accessibility
//! - **Atomic operations**: File operations complete fully or fail cleanly
//! - **Content verification**: Post-installation integrity checking
//! - **User guidance**: Clear error messages with actionable suggestions
//!
//! # Examples
//!
//! ```bash
//! # Default installation to .copilot/instructions/
//! airs-memspec install
//!
//! # Custom target directory
//! airs-memspec install --path /custom/location
//!
//! # Force overwrite existing files
//! airs-memspec install --force
//!
//! # Verbose output for debugging
//! airs-memspec install --verbose
//! ```

use crate::cli::GlobalArgs;
use crate::embedded::instructions::{available_templates, InstructionTemplate};
use crate::utils::fs::{self, FsResult};
use crate::utils::output::{OutputConfig, OutputFormatter};
use std::path::{Path, PathBuf};

/// Default installation directory relative to the current working directory
///
/// This follows GitHub Copilot's standard convention for workspace-level
/// configuration files. The `.copilot/instructions/` directory structure
/// ensures compatibility with existing Copilot tooling and IDE integrations.
const DEFAULT_INSTALL_DIR: &str = ".copilot/instructions";

/// Execute the install command with comprehensive error handling and user feedback
///
/// This is the main entry point for the install command, orchestrating the entire
/// installation workflow from argument processing to final user confirmation.
///
/// # Installation Workflow
///
/// 1. **Configuration**: Set up output formatting based on user preferences
/// 2. **Path Resolution**: Determine target directory with validation
/// 3. **Template Selection**: Choose instruction template (currently Multi-Project Memory Bank)
/// 4. **Conflict Handling**: Check for existing files and respect overwrite preferences
/// 5. **Installation**: Deploy files with atomic operations and integrity verification
/// 6. **User Feedback**: Provide clear success/failure messages and next steps
///
/// # Error Handling Strategy
///
/// All errors are captured and presented to users with:
/// - Clear descriptions of what went wrong
/// - Actionable suggestions for resolution
/// - Appropriate error codes for automation
/// - Respect for quiet mode preferences (errors always shown)
///
/// # Arguments
///
/// * `global` - Global CLI arguments including verbosity and path overrides
/// * `target` - Optional target directory override (defaults to DEFAULT_INSTALL_DIR)
/// * `force` - Whether to overwrite existing files without prompting
/// * `template` - Optional template selection (currently only Multi-Project Memory Bank available)
///
/// # Returns
///
/// * `Ok(())` - Installation completed successfully
/// * `Err(Box<dyn std::error::Error>)` - Installation failed with user-friendly error message
///
/// # Examples
///
/// ```rust,ignore
/// // Standard installation
/// run(&global_args, None, false, None)?;
///
/// // Force overwrite with custom path
/// run(&global_args, Some("/custom/path".into()), true, None)?;
/// ```
pub fn run(
    global: &GlobalArgs,
    target: Option<PathBuf>,
    force: bool,
    template: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize output formatter with user preferences
    let output_config = OutputConfig::new(global.no_color, global.verbose, global.quiet);
    let formatter = OutputFormatter::new(output_config);

    // Resolve target directory with comprehensive path validation
    let target_dir = determine_target_directory(&global.path, target).map_err(|e| {
        formatter.error(&format!("Failed to determine target directory: {e}"));
        format!("Failed to determine target directory: {e}")
    })?;

    // Select instruction template with fallback to default
    let instruction_template = select_template(template.as_deref()).inspect_err(|e| {
        formatter.error(&e.to_string());
    })?;

    // Perform conflict detection and handle existing files
    let file_path = target_dir.join(instruction_template.filename());
    if fs::path_exists(&file_path) && !force {
        let error_msg = format!(
            "File already exists: {}\nUse --force to overwrite existing files",
            file_path.display()
        );
        formatter.error(&error_msg);
        return Err(error_msg.into());
    }

    // Provide detailed progress information in verbose mode
    formatter.verbose(&format!(
        "Installing template: {}",
        instruction_template.description()
    ));
    formatter.verbose(&format!("Target directory: {}", target_dir.display()));

    // Execute installation with comprehensive error handling
    install_instructions(&target_dir, &instruction_template, force).map_err(|e| {
        let error_msg = format!("Installation failed: {e}");
        formatter.error(&error_msg);
        error_msg
    })?;

    // Confirm successful installation with user-friendly feedback
    formatter.success(&format!(
        "Successfully installed {} to {}",
        instruction_template.description(),
        target_dir.display()
    ));

    // Provide additional context and next steps (respects quiet mode)
    if !global.quiet {
        formatter.info(&format!("📁 File: {}", file_path.display()));
        formatter
            .message("🎯 Use these instructions with GitHub Copilot for enhanced AI assistance");

        // Show detailed installation summary in verbose mode
        if global.verbose {
            formatter.header("Installation Details");
            formatter.message(&format!(
                "   Template: {}",
                instruction_template.description()
            ));
            formatter.message(&format!("   Target directory: {}", target_dir.display()));
            formatter.message(&format!(
                "   File size: {} bytes",
                instruction_template.content().len()
            ));
        }
    }

    Ok(())
}

/// Resolve and validate the target directory for instruction installation
///
/// This function handles the complex logic of determining where instructions
/// should be installed, taking into account global path overrides, command-line
/// arguments, and default fallback locations.
///
/// # Path Resolution Logic
///
/// 1. **Base Directory**: Use global path override or current directory
/// 2. **Target Resolution**: Apply target override or use default location
/// 3. **Absolute Paths**: Handle absolute paths without modification
/// 4. **Relative Paths**: Resolve relative to the base directory
/// 5. **Path Canonicalization**: Convert to absolute, canonical form
///
/// # Security Considerations
///
/// - Validates that resolved paths are accessible
/// - Prevents directory traversal attacks through path canonicalization
/// - Ensures write permissions before attempting operations
///
/// # Arguments
///
/// * `global_path` - Global path override from --path flag
/// * `target` - Specific target directory from command arguments
///
/// # Returns
///
/// * `Ok(PathBuf)` - Validated, canonical target directory path
/// * `Err(FsError)` - Path resolution or validation failed
///
/// # Examples
///
/// ```rust,ignore
/// // Use default location (.copilot/instructions)
/// let default_target = determine_target_directory(&None, None)?;
///
/// // Global path override with default subdirectory
/// let workspace_target = determine_target_directory(&Some("/workspace".into()), None)?;
///
/// // Specific target directory
/// let custom_target = determine_target_directory(&None, Some("custom/path".into()))?;
/// ```
fn determine_target_directory(
    global_path: &Option<PathBuf>,
    target: Option<PathBuf>,
) -> FsResult<PathBuf> {
    // Establish base directory (global override or current directory)
    let base_dir = match global_path {
        Some(path) => fs::resolve_path(path)?,
        None => fs::resolve_path(".")?,
    };

    // Apply target directory logic with absolute/relative path handling
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

    // Canonicalize and validate the final path
    fs::resolve_path(target_dir)
}

/// Select the appropriate instruction template based on user preference
///
/// This function provides a flexible template selection mechanism that can
/// accommodate future expansion to multiple instruction templates while
/// maintaining backward compatibility and user-friendly defaults.
///
/// # Template Matching Logic
///
/// For named template selection, the function performs fuzzy matching:
/// 1. **Exact filename match** (case-insensitive)
/// 2. **Description substring match** (case-insensitive)
/// 3. **Keyword matching** for common terms like "multi", "memory"
/// 4. **Fallback error** with available template listing
///
/// # Current Templates
///
/// - **Multi-Project Memory Bank**: Comprehensive workspace and sub-project organization
///
/// # Future Extensibility
///
/// The template system is designed to support:
/// - Multiple instruction variants for different workflows
/// - Template versioning and compatibility checking
/// - Custom template loading from external sources
/// - Template parameterization and customization
///
/// # Arguments
///
/// * `template_name` - Optional template identifier (filename, description, or keyword)
///
/// # Returns
///
/// * `Ok(InstructionTemplate)` - Selected template ready for installation
/// * `Err(Box<dyn std::error::Error>)` - Template not found with available options
///
/// # Examples
///
/// ```rust,ignore
/// // Use default template
/// let default_template = select_template(None)?;
///
/// // Select by keyword
/// let memory_template = select_template(Some("memory"))?;
///
/// // Select by description fragment
/// let multi_template = select_template(Some("multi-project"))?;
/// ```
fn select_template(
    template_name: Option<&str>,
) -> Result<InstructionTemplate, Box<dyn std::error::Error>> {
    let templates = available_templates();

    match template_name {
        Some(name) => {
            // Perform fuzzy matching against available templates
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
            // Template not found - provide helpful error with available options
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
            // Use default template (currently Multi-Project Memory Bank)
            Ok(templates.into_iter().next().unwrap())
        }
    }
}

/// Execute the core installation process with atomic operations and validation
///
/// This function performs the actual file system operations required to deploy
/// instruction templates, with comprehensive error handling and integrity checking
/// to ensure reliable installation outcomes.
///
/// # Installation Steps
///
/// 1. **Directory Creation**: Ensure target directory exists with proper permissions
/// 2. **Permission Validation**: Verify write access before attempting operations
/// 3. **File Deployment**: Write instruction content with overwrite handling
/// 4. **Integrity Verification**: Validate installation success and content accuracy
///
/// # Atomic Operation Guarantee
///
/// Installation either completes successfully or fails cleanly:
/// - Directory creation is idempotent
/// - File writes use temporary files and atomic renames where possible
/// - Partial failures leave the filesystem in a consistent state
/// - Validation catches corruption before reporting success
///
/// # Arguments
///
/// * `target_dir` - Validated target directory for installation
/// * `template` - Instruction template to deploy
/// * `force` - Whether to overwrite existing files
///
/// # Returns
///
/// * `Ok(())` - Installation completed and validated successfully
/// * `Err(FsError)` - Installation failed with specific error context
///
/// # Error Recovery
///
/// Failed installations provide enough context for users to:
/// - Understand what went wrong (permissions, disk space, etc.)
/// - Take corrective action (fix permissions, free space, etc.)
/// - Retry the operation safely
fn install_instructions(
    target_dir: &PathBuf,
    template: &InstructionTemplate,
    force: bool,
) -> FsResult<()> {
    // Ensure target directory exists with appropriate permissions
    fs::create_dir_all(target_dir)?;

    // Validate write permissions before attempting file operations
    fs::validate_writable_directory(target_dir)?;

    // Deploy the instruction file with overwrite handling
    let file_path = target_dir.join(template.filename());
    fs::write_file_with_overwrite(&file_path, template.content(), force)?;

    // Verify installation integrity and content accuracy
    validate_installation(target_dir, template)?;

    Ok(())
}

/// Verify installation integrity and content accuracy post-deployment
///
/// This function provides comprehensive validation to ensure that the installation
/// process completed successfully and the deployed files match expectations.
///
/// # Validation Checks
///
/// 1. **File Existence**: Verify the instruction file was created successfully
/// 2. **Readability**: Ensure the file is accessible and readable
/// 3. **Content Integrity**: Validate that file contents match the source template
/// 4. **Size Verification**: Confirm file size matches expected template size
///
/// # Security Implications
///
/// Content validation protects against:
/// - Partial writes due to disk space issues
/// - Filesystem corruption during write operations
/// - Race conditions in multi-process environments
/// - Tampering or modification during installation
///
/// # Arguments
///
/// * `target_dir` - Directory where installation occurred
/// * `template` - Template that was supposed to be installed
///
/// # Returns
///
/// * `Ok(())` - Installation validated successfully
/// * `Err(FsError)` - Validation failed with specific error details
///
/// # Failure Recovery
///
/// Validation failures indicate serious issues that typically require:
/// - Retrying the installation process
/// - Checking filesystem integrity
/// - Verifying available disk space
/// - Examining file system permissions
fn validate_installation(target_dir: &Path, template: &InstructionTemplate) -> FsResult<()> {
    let file_path = target_dir.join(template.filename());

    // Verify file existence and accessibility
    if !fs::is_file(&file_path) {
        return Err(crate::utils::fs::FsError::PathNotFound { path: file_path });
    }

    // Read and validate file content integrity
    let content = std::fs::read_to_string(&file_path).map_err(crate::utils::fs::FsError::Io)?;

    // Ensure deployed content exactly matches the source template
    if content != template.content() {
        return Err(crate::utils::fs::FsError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File content validation failed - deployed content does not match template",
        )));
    }

    Ok(())
}

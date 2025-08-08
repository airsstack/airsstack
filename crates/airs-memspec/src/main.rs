// airs-memspec CLI entry point
// Memory bank management tool for AI development

use airs_memspec::cli;
use airs_memspec::utils::fs::{generate_recovery_suggestions, FsError};
use airs_memspec::utils::output::{OutputConfig, OutputFormatter};

fn main() {
    if let Err(e) = cli::run() {
        // Create basic output config for error display
        let config = OutputConfig::new(false, false, false);
        let formatter = OutputFormatter::new(config);

        // Format error with professional styling
        formatter.error(&format!("Operation failed: {e}"));

        // Check if this is an FsError that we can provide detailed recovery for
        if let Some(fs_error) = e.downcast_ref::<FsError>() {
            let recovery = generate_recovery_suggestions(fs_error, None);
            formatter.info(&recovery);
        } else {
            // Generic recovery suggestion for other error types
            let error_msg = e.to_string();
            if !error_msg.contains("Suggestion:") {
                formatter.info("\nFor help:\n• Run 'airs-memspec --help' for usage information\n• Run 'airs-memspec install' if you need to set up a memory bank");
            }
        }

        std::process::exit(1);
    }
}

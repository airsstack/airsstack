// airs-memspec CLI entry point
// Memory bank management tool for AI development

use airs_memspec::cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

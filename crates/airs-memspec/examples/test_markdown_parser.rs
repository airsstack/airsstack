use airs_memspec::parser::markdown::MarkdownParser;

fn main() {
    // Test simple markdown parsing
    let content = r#"# Test Document

This is a test document.

## Section 1

Some content here.

## Section 2

More content here.
"#;

    let parsed = MarkdownParser::parse_content(content).unwrap();

    println!("Title: {:?}", parsed.metadata.title);
    println!("Sections found: {}", parsed.sections.len());
    for (key, value) in &parsed.sections {
        println!("  '{key}': '{value}'");
    }
    println!();

    // Test task index parsing
    let task_content = r#"# Task Index

- [task_001] First task - completed
- [task_002] Second task - in progress
- [task_003] Third task - blocked
"#;

    let task_parsed = MarkdownParser::parse_content(task_content).unwrap();
    println!("Tasks found: {}", task_parsed.tasks.len());
    for task in &task_parsed.tasks {
        println!(
            "  ID: {:?}, Title: '{}', Status: {:?}",
            task.id, task.title, task.status
        );
    }
}

// Example demonstrating the layout engine capabilities
// This shows how the layout engine can produce README-style output

use airs_memspec::utils::layout::{
    Alignment, HeaderStyle, IndentedItem, LayoutElement, LayoutEngine,
};
use airs_memspec::utils::output::OutputConfig;

fn main() {
    // Create output config (no color for clean demo output)
    let config = OutputConfig::new(true, false, false); // no_color=true
    let engine = LayoutEngine::new(config);

    // Create workspace status layout matching README example
    let workspace_elements = vec![
        LayoutElement::Header {
            icon: "🏢".to_string(),
            title: "AIRS Workspace".to_string(),
            style: HeaderStyle::Heavy,
        },
        LayoutElement::FieldRow {
            label: "Status".to_string(),
            value: "Active Development - Foundation Phase".to_string(),
            alignment: Alignment::LeftAligned(15),
        },
        LayoutElement::FieldRow {
            label: "Focus".to_string(),
            value: "MCP Protocol Implementation & Tooling".to_string(),
            alignment: Alignment::LeftAligned(15),
        },
        LayoutElement::FieldRow {
            label: "Updated".to_string(),
            value: "2 hours ago".to_string(),
            alignment: Alignment::LeftAligned(15),
        },
        LayoutElement::EmptyLine,
        LayoutElement::FieldRow {
            label: "Projects".to_string(),
            value: "2 active, 0 paused".to_string(),
            alignment: Alignment::LeftAligned(15),
        },
        LayoutElement::TreeItem {
            level: 0,
            is_last: false,
            icon: "🟢".to_string(),
            text: "airs-mcp      Week 1/14 - JSON-RPC Foundation".to_string(),
        },
        LayoutElement::TreeItem {
            level: 0,
            is_last: true,
            icon: "🟡".to_string(),
            text: "airs-memspec  Planning - CLI Development".to_string(),
        },
        LayoutElement::EmptyLine,
        LayoutElement::FieldRow {
            label: "Next Milestone".to_string(),
            value: "JSON-RPC 2.0 Core Complete (3 days)".to_string(),
            alignment: Alignment::LeftAligned(15),
        },
        LayoutElement::FieldRow {
            label: "Blockers".to_string(),
            value: "None".to_string(),
            alignment: Alignment::LeftAligned(15),
        },
    ];

    println!("=== Workspace Status Demo ===");
    let workspace_output = engine.render(&workspace_elements);
    println!("{}", workspace_output);

    // Create context layout matching README example
    let context_elements = vec![
        LayoutElement::Header {
            icon: "🎯".to_string(),
            title: "airs-mcp Active Context".to_string(),
            style: HeaderStyle::Heavy,
        },
        LayoutElement::Section {
            title: "Current Focus".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: vec![IndentedItem {
                    bullet: "".to_string(),
                    text: "JSON-RPC 2.0 Foundation & Transport Layer Implementation".to_string(),
                    indent_level: 0,
                }],
            }],
        },
        LayoutElement::Section {
            title: "Active Work".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: vec![
                    IndentedItem {
                        bullet: "🔧".to_string(),
                        text: "Implementing MCP error code extensions".to_string(),
                        indent_level: 0,
                    },
                    IndentedItem {
                        bullet: "📝".to_string(),
                        text: "Serde integration and serialization testing".to_string(),
                        indent_level: 0,
                    },
                    IndentedItem {
                        bullet: "⏱️".to_string(),
                        text: "Started 4 hours ago".to_string(),
                        indent_level: 0,
                    },
                ],
            }],
        },
        LayoutElement::Section {
            title: "Integration Points".to_string(),
            children: vec![LayoutElement::IndentedList {
                items: vec![
                    IndentedItem {
                        bullet: "•".to_string(),
                        text: "Transport abstraction for STDIO and HTTP".to_string(),
                        indent_level: 0,
                    },
                    IndentedItem {
                        bullet: "•".to_string(),
                        text: "State machine for protocol lifecycle management".to_string(),
                        indent_level: 0,
                    },
                    IndentedItem {
                        bullet: "•".to_string(),
                        text: "Security layer for OAuth 2.1 + PKCE".to_string(),
                        indent_level: 0,
                    },
                ],
            }],
        },
    ];

    println!("=== Context Demo ===");
    let context_output = engine.render(&context_elements);
    println!("{}", context_output);
}

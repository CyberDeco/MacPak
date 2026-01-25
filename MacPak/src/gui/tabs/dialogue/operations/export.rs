//! Export operations - HTML and DE2 export

use std::collections::HashSet;
use std::fmt::Write;

use floem::reactive::{SignalGet, SignalUpdate};
use crate::dialog::{Dialog, NodeConstructor};

use crate::gui::state::DialogueState;

/// Export dialog to HTML format
pub fn export_html(state: DialogueState) {
    let Some(dialog) = state.current_dialog.get() else {
        return;
    };

    std::thread::spawn(move || {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save HTML Export")
            .add_filter("HTML", &["html"])
            .set_file_name("dialog.html")
            .save_file()
        {
            state.status_message.set("Exporting to HTML...".to_string());

            match generate_html(&dialog) {
                Ok(html) => {
                    match std::fs::write(&path, html) {
                        Ok(()) => {
                            state.status_message.set(format!("Exported to {}", path.display()));
                        }
                        Err(e) => {
                            state.status_message.set(format!("Write error: {e}"));
                        }
                    }
                }
                Err(e) => {
                    state.status_message.set(format!("Export error: {e}"));
                }
            }
        }
    });
}

/// Export dialog to DE2 format
pub fn export_de2(state: DialogueState) {
    let Some(_dialog) = state.current_dialog.get() else {
        return;
    };

    std::thread::spawn(move || {
        if let Some(_path) = rfd::FileDialog::new()
            .set_title("Save DE2 Export")
            .add_filter("LSJ", &["lsj"])
            .set_file_name("dialog_de2.lsj")
            .save_file()
        {
            state.status_message.set("Exporting to DE2...".to_string());

            // TODO: Implement DE2 export
            state.status_message.set("DE2 export not yet implemented".to_string());
        }
    });
}

// ============================================================================
// HTML Export Generation
// ============================================================================

/// Generate HTML export of a dialog
///
/// Creates a standalone HTML document with styled dialog tree
fn generate_html(dialog: &Dialog) -> Result<String, String> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<title>Dialog Export</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; background: #f5f5f5; }\n");
    html.push_str(".node { margin: 4px 0; padding: 8px; background: white; border-radius: 4px; border-left: 3px solid #ddd; }\n");
    html.push_str(".node-question { border-left-color: #3b82f6; }\n");
    html.push_str(".node-answer { border-left-color: #22c55e; }\n");
    html.push_str(".node-roll { border-left-color: #f97316; }\n");
    html.push_str(".speaker { color: #4f46e5; font-weight: 500; }\n");
    html.push_str(".text { margin-left: 8px; }\n");
    html.push_str(".meta { color: #9ca3af; font-size: 12px; margin-top: 4px; }\n");
    html.push_str(".children { margin-left: 20px; border-left: 1px solid #e5e7eb; padding-left: 12px; }\n");
    html.push_str("</style>\n</head>\n<body>\n");

    // Dialog info
    let _ = writeln!(html, "<h1>Dialog: {}</h1>", dialog.uuid);
    if let Some(ref synopsis) = dialog.editor_data.synopsis {
        let _ = writeln!(html, "<p><em>{}</em></p>", html_escape(synopsis));
    }
    let _ = writeln!(html, "<p>Nodes: {}</p>", dialog.node_count());

    // Build tree from roots
    for root_uuid in &dialog.root_nodes {
        render_node_html(dialog, root_uuid, &mut html, &mut HashSet::new());
    }

    html.push_str("</body>\n</html>\n");

    Ok(html)
}

fn render_node_html(dialog: &Dialog, uuid: &str, html: &mut String, visited: &mut HashSet<String>) {
    if visited.contains(uuid) {
        return;
    }
    visited.insert(uuid.to_string());

    let Some(node) = dialog.get_node(uuid) else {
        return;
    };

    let class = match node.constructor {
        NodeConstructor::TagQuestion => "node node-question",
        NodeConstructor::TagAnswer => "node node-answer",
        NodeConstructor::ActiveRoll | NodeConstructor::PassiveRoll => "node node-roll",
        _ => "node",
    };

    let _ = writeln!(html, "<div class=\"{class}\">");

    // Type badge
    let _ = write!(html, "<strong>[{}]</strong> ", node.constructor.display_name());

    // Speaker
    if let Some(speaker_idx) = node.speaker
        && speaker_idx >= 0 {
            let _ = write!(html, "<span class=\"speaker\">Speaker {speaker_idx}</span>: ");
        }

    // Text
    if let Some(text_entry) = dialog.get_node_text(node) {
        let text = text_entry.value.as_ref().map_or_else(
            || format!("[{}]", text_entry.handle),
            |s| html_escape(s),
        );
        let _ = writeln!(html, "<span class=\"text\">{text}</span>");
    }

    // Meta info
    html.push_str("<div class=\"meta\">");
    let _ = write!(html, "UUID: {uuid} ");
    if node.end_node {
        html.push_str("[END] ");
    }
    html.push_str("</div>\n");

    // Children
    if !node.children.is_empty() {
        html.push_str("<div class=\"children\">\n");
        for child_uuid in &node.children {
            render_node_html(dialog, child_uuid, html, visited);
        }
        html.push_str("</div>\n");
    }

    html.push_str("</div>\n");
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

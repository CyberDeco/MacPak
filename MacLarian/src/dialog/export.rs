//! Export functionality for dialogs

use std::collections::HashSet;
use super::{Dialog, NodeConstructor};

/// Generate HTML export of a dialog
///
/// Creates a standalone HTML document with styled dialog tree
///
/// # Errors
/// Returns an error message if generation fails.
pub fn generate_html(dialog: &Dialog) -> Result<String, String> {
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
    html.push_str(&format!("<h1>Dialog: {}</h1>\n", dialog.uuid));
    if let Some(ref synopsis) = dialog.editor_data.synopsis {
        html.push_str(&format!("<p><em>{}</em></p>\n", html_escape(synopsis)));
    }
    html.push_str(&format!("<p>Nodes: {}</p>\n", dialog.node_count()));

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

    html.push_str(&format!("<div class=\"{class}\">\n"));

    // Type badge
    html.push_str(&format!("<strong>[{}]</strong> ", node.constructor.display_name()));

    // Speaker
    if let Some(speaker_idx) = node.speaker
        && speaker_idx >= 0 {
            html.push_str(&format!("<span class=\"speaker\">Speaker {speaker_idx}</span>: "));
        }

    // Text
    if let Some(text_entry) = dialog.get_node_text(node) {
        let text = text_entry.value.as_ref().map_or_else(|| format!("[{}]", text_entry.handle), |s| html_escape(s));
        html.push_str(&format!("<span class=\"text\">{text}</span>\n"));
    }

    // Meta info
    html.push_str("<div class=\"meta\">");
    html.push_str(&format!("UUID: {uuid} "));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_generate_html_empty_dialog() {
        let dialog = Dialog::new();
        let html = generate_html(&dialog).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Nodes: 0"));
    }
}

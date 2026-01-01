//! XML and JSON formatting utilities

/// Pretty-print XML content with proper indentation
pub fn format_xml(content: &str) -> String {
    let mut result = String::new();
    let mut indent_level: i32 = 0;
    let indent_str = "    "; // 4 spaces

    let mut chars = content.chars().peekable();
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut text_content = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                let trimmed = text_content.trim();
                if !trimmed.is_empty() {
                    result.push_str(trimmed);
                }
                text_content.clear();

                in_tag = true;
                current_tag.clear();
                current_tag.push(ch);
            }
            '>' => {
                current_tag.push(ch);
                in_tag = false;

                let tag = current_tag.trim();

                if tag.starts_with("<?") || tag.starts_with("<!") {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push_str(tag);
                    result.push('\n');
                } else if tag.starts_with("</") {
                    indent_level = (indent_level - 1).max(0);
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                } else if tag.ends_with("/>") {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                } else {
                    if !result.is_empty() && !result.ends_with('\n') {
                        result.push('\n');
                    }
                    for _ in 0..indent_level {
                        result.push_str(indent_str);
                    }
                    result.push_str(tag);
                    indent_level += 1;
                }

                current_tag.clear();
            }
            _ => {
                if in_tag {
                    current_tag.push(ch);
                } else {
                    text_content.push(ch);
                }
            }
        }
    }

    if !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

/// Pretty-print JSON content with proper indentation
pub fn format_json(content: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| content.to_string()),
        Err(_) => content.to_string(),
    }
}

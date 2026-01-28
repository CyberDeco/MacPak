//! Syntax highlighting for XML and JSON

use floem::peniko::Color as PenikoColor;
use floem::text::{Attrs, AttrsList, FamilyOwned, Weight};
use floem::views::editor::EditorStyle;
use floem::views::editor::id::EditorId;
use floem::views::editor::text::Styling;
use std::borrow::Cow;

/// Token types for syntax highlighting
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    // XML tokens
    XmlTag,         // <tagname>, </tagname>
    XmlAttribute,   // attribute names
    XmlString,      // attribute values in quotes
    XmlComment,     // <!-- comments -->
    XmlDeclaration, // <?xml ... ?>
    XmlCData,       // <![CDATA[ ... ]]>

    // JSON tokens
    JsonKey,     // "key":
    JsonString,  // "value"
    JsonNumber,  // 123, -45.67
    JsonBoolean, // true, false
    JsonNull,    // null
    JsonBracket, // {}, []

    // Common
    Plain,
}

/// A span of text with a specific token type
#[derive(Clone, Debug)]
pub struct TokenSpan {
    pub start: usize,
    pub end: usize,
    pub token_type: TokenType,
}

/// Colors for syntax highlighting (VS Code-inspired)
pub struct SyntaxColors;

impl SyntaxColors {
    // XML colors
    pub const XML_TAG: PenikoColor = PenikoColor::rgba8(86, 156, 214, 255); // Blue
    pub const XML_ATTRIBUTE: PenikoColor = PenikoColor::rgba8(156, 220, 254, 255); // Light cyan
    pub const XML_STRING: PenikoColor = PenikoColor::rgba8(206, 145, 120, 255); // Orange/brown
    pub const XML_COMMENT: PenikoColor = PenikoColor::rgba8(106, 153, 85, 255); // Green
    pub const XML_DECLARATION: PenikoColor = PenikoColor::rgba8(197, 134, 192, 255); // Purple

    // JSON colors
    pub const JSON_KEY: PenikoColor = PenikoColor::rgba8(156, 220, 254, 255); // Light cyan
    pub const JSON_STRING: PenikoColor = PenikoColor::rgba8(206, 145, 120, 255); // Orange/brown
    pub const JSON_NUMBER: PenikoColor = PenikoColor::rgba8(181, 206, 168, 255); // Light green
    pub const JSON_BOOLEAN: PenikoColor = PenikoColor::rgba8(86, 156, 214, 255); // Blue
    pub const JSON_NULL: PenikoColor = PenikoColor::rgba8(86, 156, 214, 255); // Blue
    pub const JSON_BRACKET: PenikoColor = PenikoColor::rgba8(212, 212, 212, 255); // Light gray

    // Default
    pub const PLAIN: PenikoColor = PenikoColor::rgba8(212, 212, 212, 255); // Light gray

    pub fn for_token(token_type: TokenType) -> PenikoColor {
        match token_type {
            TokenType::XmlTag => Self::XML_TAG,
            TokenType::XmlAttribute => Self::XML_ATTRIBUTE,
            TokenType::XmlString => Self::XML_STRING,
            TokenType::XmlComment => Self::XML_COMMENT,
            TokenType::XmlDeclaration => Self::XML_DECLARATION,
            TokenType::XmlCData => Self::XML_STRING,
            TokenType::JsonKey => Self::JSON_KEY,
            TokenType::JsonString => Self::JSON_STRING,
            TokenType::JsonNumber => Self::JSON_NUMBER,
            TokenType::JsonBoolean => Self::JSON_BOOLEAN,
            TokenType::JsonNull => Self::JSON_NULL,
            TokenType::JsonBracket => Self::JSON_BRACKET,
            TokenType::Plain => Self::PLAIN,
        }
    }
}

/// Tokenize XML content for syntax highlighting
pub fn tokenize_xml(text: &str) -> Vec<TokenSpan> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    // Helper to check for safely comparing a string slice
    fn safe_starts_with(text: &str, pos: usize, pattern: &str) -> bool {
        text.get(pos..)
            .map(|s| s.starts_with(pattern))
            .unwrap_or(false)
    }

    // Helper to find pattern end position
    fn find_pattern(bytes: &[u8], start: usize, pattern: &[u8]) -> Option<usize> {
        if pattern.is_empty() {
            return Some(start);
        }
        let mut pos = start;
        while pos + pattern.len() <= bytes.len() {
            if &bytes[pos..pos + pattern.len()] == pattern {
                return Some(pos);
            }
            pos += 1;
        }
        None
    }

    while pos < len {
        // Skip non-ASCII bytes (like BOM)
        if !bytes[pos].is_ascii() {
            pos += 1;
            continue;
        }

        // Check for comment: <!-- ... -->
        if safe_starts_with(text, pos, "<!--") {
            let start = pos;
            pos += 4;
            if let Some(end_pos) = find_pattern(bytes, pos, b"-->") {
                pos = end_pos + 3;
            } else {
                pos = len;
            }
            tokens.push(TokenSpan {
                start,
                end: pos,
                token_type: TokenType::XmlComment,
            });
            continue;
        }

        // Check for CDATA: <![CDATA[ ... ]]>
        if safe_starts_with(text, pos, "<![CDATA[") {
            let start = pos;
            pos += 9;
            if let Some(end_pos) = find_pattern(bytes, pos, b"]]>") {
                pos = end_pos + 3;
            } else {
                pos = len;
            }
            tokens.push(TokenSpan {
                start,
                end: pos,
                token_type: TokenType::XmlCData,
            });
            continue;
        }

        // Check for processing instruction: <?...?>
        if safe_starts_with(text, pos, "<?") {
            let start = pos;
            pos += 2;
            if let Some(end_pos) = find_pattern(bytes, pos, b"?>") {
                pos = end_pos + 2;
            } else {
                pos = len;
            }
            tokens.push(TokenSpan {
                start,
                end: pos,
                token_type: TokenType::XmlDeclaration,
            });
            continue;
        }

        // Check for tag: < ... >
        if bytes[pos] == b'<' {
            let tag_start = pos;
            pos += 1;

            // Skip whitespace
            while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
                pos += 1;
            }

            // Check for closing tag
            if pos < len && bytes[pos] == b'/' {
                pos += 1;
            }

            // Read tag name
            let name_start = pos;
            while pos < len
                && (bytes[pos].is_ascii_alphanumeric()
                    || bytes[pos] == b':'
                    || bytes[pos] == b'_'
                    || bytes[pos] == b'-')
            {
                pos += 1;
            }
            let name_end = pos;

            // Add tag name token (including the < and optional /)
            if name_end > name_start {
                tokens.push(TokenSpan {
                    start: tag_start,
                    end: name_end,
                    token_type: TokenType::XmlTag,
                });
            }

            // Parse attributes until > or />
            while pos < len && bytes[pos] != b'>' {
                // Skip whitespace
                while pos < len
                    && (bytes[pos] == b' '
                        || bytes[pos] == b'\t'
                        || bytes[pos] == b'\n'
                        || bytes[pos] == b'\r')
                {
                    pos += 1;
                }

                if pos >= len || bytes[pos] == b'>' || bytes[pos] == b'/' {
                    break;
                }

                // Read attribute name
                let attr_start = pos;
                while pos < len
                    && (bytes[pos].is_ascii_alphanumeric()
                        || bytes[pos] == b':'
                        || bytes[pos] == b'_'
                        || bytes[pos] == b'-')
                {
                    pos += 1;
                }

                if pos > attr_start {
                    tokens.push(TokenSpan {
                        start: attr_start,
                        end: pos,
                        token_type: TokenType::XmlAttribute,
                    });
                }

                // Skip whitespace and =
                while pos < len && (bytes[pos] == b' ' || bytes[pos] == b'\t' || bytes[pos] == b'=')
                {
                    pos += 1;
                }

                // Read attribute value (quoted string)
                if pos < len && (bytes[pos] == b'"' || bytes[pos] == b'\'') {
                    let quote = bytes[pos];
                    let value_start = pos;
                    pos += 1;
                    while pos < len && bytes[pos] != quote {
                        pos += 1;
                    }
                    if pos < len {
                        pos += 1; // Include closing quote
                    }
                    tokens.push(TokenSpan {
                        start: value_start,
                        end: pos,
                        token_type: TokenType::XmlString,
                    });
                }
            }

            // Handle /> or >
            if pos < len {
                let close_start = pos;
                if bytes[pos] == b'/' {
                    pos += 1;
                }
                if pos < len && bytes[pos] == b'>' {
                    pos += 1;
                }
                tokens.push(TokenSpan {
                    start: close_start,
                    end: pos,
                    token_type: TokenType::XmlTag,
                });
            }
            continue;
        }

        // Plain text - skip to next <
        let start = pos;
        while pos < len && bytes[pos] != b'<' {
            pos += 1;
        }
        if pos > start {
            tokens.push(TokenSpan {
                start,
                end: pos,
                token_type: TokenType::Plain,
            });
        }
    }

    tokens
}

/// Tokenize JSON content for syntax highlighting
pub fn tokenize_json(text: &str) -> Vec<TokenSpan> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    // Helper to check for safely comparing a string slice
    fn safe_starts_with(text: &str, pos: usize, pattern: &str) -> bool {
        text.get(pos..)
            .map(|s| s.starts_with(pattern))
            .unwrap_or(false)
    }

    while pos < len {
        let ch = bytes[pos];

        // Skip whitespace and non-ASCII bytes (like BOM)
        if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || !ch.is_ascii() {
            pos += 1;
            continue;
        }

        // Brackets and colons
        if ch == b'{' || ch == b'}' || ch == b'[' || ch == b']' {
            tokens.push(TokenSpan {
                start: pos,
                end: pos + 1,
                token_type: TokenType::JsonBracket,
            });
            pos += 1;
            continue;
        }

        // Colon and comma (plain)
        if ch == b':' || ch == b',' {
            tokens.push(TokenSpan {
                start: pos,
                end: pos + 1,
                token_type: TokenType::Plain,
            });
            pos += 1;
            continue;
        }

        // String (could be key or value)
        if ch == b'"' {
            let start = pos;
            pos += 1;

            // Read until closing quote, handling escapes
            while pos < len {
                if bytes[pos] == b'\\' && pos + 1 < len {
                    pos += 2; // Skip escaped character
                } else if bytes[pos] == b'"' {
                    pos += 1;
                    break;
                } else {
                    pos += 1;
                }
            }

            // Determine if this is a key (followed by :)
            let mut check_pos = pos;
            while check_pos < len
                && (bytes[check_pos] == b' '
                    || bytes[check_pos] == b'\t'
                    || bytes[check_pos] == b'\n'
                    || bytes[check_pos] == b'\r')
            {
                check_pos += 1;
            }

            let token_type = if check_pos < len && bytes[check_pos] == b':' {
                TokenType::JsonKey
            } else {
                TokenType::JsonString
            };

            tokens.push(TokenSpan {
                start,
                end: pos,
                token_type,
            });
            continue;
        }

        // Numbers
        if ch == b'-' || ch.is_ascii_digit() {
            let start = pos;
            if ch == b'-' {
                pos += 1;
            }
            while pos < len && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
            // Decimal part
            if pos < len && bytes[pos] == b'.' {
                pos += 1;
                while pos < len && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            // Exponent
            if pos < len && (bytes[pos] == b'e' || bytes[pos] == b'E') {
                pos += 1;
                if pos < len && (bytes[pos] == b'+' || bytes[pos] == b'-') {
                    pos += 1;
                }
                while pos < len && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            tokens.push(TokenSpan {
                start,
                end: pos,
                token_type: TokenType::JsonNumber,
            });
            continue;
        }

        // Keywords: true, false, null
        if safe_starts_with(text, pos, "true") {
            tokens.push(TokenSpan {
                start: pos,
                end: pos + 4,
                token_type: TokenType::JsonBoolean,
            });
            pos += 4;
            continue;
        }
        if safe_starts_with(text, pos, "false") {
            tokens.push(TokenSpan {
                start: pos,
                end: pos + 5,
                token_type: TokenType::JsonBoolean,
            });
            pos += 5;
            continue;
        }
        if safe_starts_with(text, pos, "null") {
            tokens.push(TokenSpan {
                start: pos,
                end: pos + 4,
                token_type: TokenType::JsonNull,
            });
            pos += 4;
            continue;
        }

        // Unknown character - skip
        pos += 1;
    }

    tokens
}

/// Custom styling for syntax highlighting
#[derive(Clone)]
pub struct SyntaxStyling {
    id: u64,
    tokens: Vec<TokenSpan>,
    line_offsets: Vec<usize>, // Byte offset where each line starts
    font_size: usize,
}

impl SyntaxStyling {
    pub fn new(text: &str, format: &str) -> Self {
        let tokens = match format.to_uppercase().as_str() {
            "LSX" | "LSF" | "LSFX" | "LOCA" | "XML" => tokenize_xml(text),
            "LSJ" | "JSON" => tokenize_json(text),
            _ => Vec::new(),
        };

        // Compute line offsets
        let mut line_offsets = vec![0];
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                line_offsets.push(i + 1);
            }
        }

        // Generate a unique id based on content hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format.hash(&mut hasher);
        let id = hasher.finish();

        Self {
            id,
            tokens,
            line_offsets,
            font_size: 14,
        }
    }

    /// Get the byte offset for the start of a line
    fn line_start(&self, line: usize) -> usize {
        self.line_offsets.get(line).copied().unwrap_or(0)
    }
}

impl Styling for SyntaxStyling {
    fn id(&self) -> u64 {
        self.id
    }

    fn font_size(&self, _edid: EditorId, _line: usize) -> usize {
        self.font_size
    }

    fn line_height(&self, _edid: EditorId, _line: usize) -> f32 {
        let font_size = self.font_size as f32;
        (1.5 * font_size).round().max(font_size)
    }

    fn font_family(&self, _edid: EditorId, _line: usize) -> Cow<'_, [FamilyOwned]> {
        Cow::Owned(vec![FamilyOwned::Monospace])
    }

    fn weight(&self, _edid: EditorId, _line: usize) -> Weight {
        Weight::NORMAL
    }

    fn italic_style(&self, _edid: EditorId, _line: usize) -> floem::text::Style {
        floem::text::Style::Normal
    }

    fn apply_attr_styles(
        &self,
        _edid: EditorId,
        _style: &EditorStyle,
        line: usize,
        _default: Attrs,
        attrs: &mut AttrsList,
    ) {
        let line_start = self.line_start(line);
        // Get line end from next line offset, or use a large value
        let line_end = self
            .line_offsets
            .get(line + 1)
            .copied()
            .unwrap_or(usize::MAX);

        for span in &self.tokens {
            // Check if span overlaps with line
            if span.end <= line_start || span.start >= line_end {
                continue;
            }

            // Calculate the range within the line
            let span_start_in_line = span.start.saturating_sub(line_start);
            let span_end_in_line = (span.end - line_start).min(line_end - line_start);

            if span_start_in_line < span_end_in_line {
                let color = SyntaxColors::for_token(span.token_type);
                attrs.add_span(
                    span_start_in_line..span_end_in_line,
                    Attrs::new().color(color),
                );
            }
        }
    }
}

//! # Clipboard Module
//!
//! Provides cross-platform clipboard functionality for the Velum Word editor.
//! Supports plain text, rich text (HTML/RTF), Word document fragments, and images.
//!
//! ## Features
//!
//! - Plain text copy/paste
//! - Rich text copy/paste (HTML format for cross-application compatibility)
//! - Word document fragment copy/paste (OOXML format)
//! - Image copy/paste
//! - Cross-platform support (Windows, macOS, Linux)

use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use once_cell::sync::Lazy;
use log::debug;

/// MIME types for clipboard content
pub const MIME_TEXT: &str = "text/plain";
pub const MIME_HTML: &str = "text/html";
pub const MIME_RTF: &str = "application/rtf";
pub const MIME_IMAGE_PNG: &str = "image/png";
pub const MIME_IMAGE_JPEG: &str = "image/jpeg";
pub const MIME_WORD: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

/// Represents different types of clipboard content
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardContent {
    /// Plain text content
    PlainText(String),
    /// Rich text content (HTML format)
    RichText(String),
    /// Word document fragment (OOXML format)
    WordFragment(Vec<u8>),
    /// Image content with format identifier
    Image(ImageClipboardData),
    /// Multiple content types (for rich copy operations)
    Mixed(MixedClipboardData),
    /// No content
    Empty,
}

/// Image data for clipboard operations
#[derive(Debug, Clone, PartialEq)]
pub struct ImageClipboardData {
    /// Image data bytes
    pub data: Vec<u8>,
    /// Image format (png, jpeg, etc.)
    pub format: String,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
}

/// Mixed clipboard data containing multiple content types
#[derive(Debug, Clone, PartialEq)]
pub struct MixedClipboardData {
    /// Plain text version
    pub plain_text: String,
    /// HTML/RTF version for rich text applications
    pub rich_text: Option<String>,
    /// Word fragment for Word compatibility
    pub word_fragment: Option<Vec<u8>>,
}

/// Result of a clipboard operation
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardResult {
    /// Operation succeeded
    Success,
    /// Operation failed with error message
    Failed(String),
    /// Clipboard is empty or doesn't contain expected content
    NoContent,
    /// Operation not supported on current platform
    NotSupported,
}

/// Clipboard content type for detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// Plain text content
    PlainText,
    /// Rich text content
    RichText,
    /// Word document content
    Word,
    /// Image content
    Image,
    /// Unknown content type
    Unknown,
}

/// Document fragment for internal clipboard operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFragment {
    /// Text content of the fragment
    pub text: String,
    /// Paragraph information
    pub paragraphs: Vec<ParagraphInfo>,
    /// Style information
    pub styles: Vec<StyleInfo>,
}

/// Paragraph information within a fragment
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParagraphInfo {
    /// Paragraph text
    pub text: String,
    /// Alignment
    pub alignment: Option<String>,
    /// Indentation
    pub indent_left: Option<f32>,
    pub indent_right: Option<f32>,
    pub indent_first_line: Option<f32>,
    /// Spacing
    pub space_before: Option<f32>,
    pub space_after: Option<f32>,
}

/// Style information for text within a fragment
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleInfo {
    /// Text content with this style
    pub text: String,
    /// Bold
    pub bold: Option<bool>,
    /// Italic
    pub italic: Option<bool>,
    /// Underline
    pub underline: Option<bool>,
    /// Font size
    pub font_size: Option<u16>,
    /// Font family
    pub font_family: Option<String>,
    /// Text color
    pub color: Option<String>,
    /// Background color
    pub background: Option<String>,
}

/// Internal clipboard state
static CLIPBOARD: Lazy<RwLock<Clipboard>> = Lazy::new(|| {
    RwLock::new(Clipboard::new())
});

/// Main clipboard manager
pub struct Clipboard {
    /// Current clipboard content
    content: ClipboardContent,
    /// Clipboard history for undo-style paste
    history: Vec<ClipboardContent>,
    /// Maximum history size
    max_history: usize,
}

impl Clipboard {
    /// Creates a new clipboard manager
    pub fn new() -> Self {
        Clipboard {
            content: ClipboardContent::Empty,
            history: Vec::new(),
            max_history: 10,
        }
    }

    /// Clears the clipboard
    pub fn clear(&mut self) {
        self.content = ClipboardContent::Empty;
    }

    /// Sets plain text content
    pub fn set_text(&mut self, text: &str) {
        self.content = ClipboardContent::PlainText(text.to_string());
        self.add_to_history();
    }

    /// Sets rich text content (HTML format)
    pub fn set_rich_text(&mut self, html: &str, plain_text: &str) {
        self.content = ClipboardContent::Mixed(MixedClipboardData {
            plain_text: plain_text.to_string(),
            rich_text: Some(html.to_string()),
            word_fragment: None,
        });
        self.add_to_history();
    }

    /// Sets Word document fragment
    pub fn set_word_fragment(&mut self, fragment: &[u8], plain_text: &str) {
        self.content = ClipboardContent::Mixed(MixedClipboardData {
            plain_text: plain_text.to_string(),
            rich_text: None,
            word_fragment: Some(fragment.to_vec()),
        });
        self.add_to_history();
    }

    /// Sets image content
    pub fn set_image(&mut self, image: ImageClipboardData) {
        self.content = ClipboardContent::Image(image);
        self.add_to_history();
    }

    /// Gets the current clipboard content
    pub fn get_content(&self) -> &ClipboardContent {
        &self.content
    }

    /// Gets plain text if available
    pub fn get_text(&self) -> Option<&String> {
        match &self.content {
            ClipboardContent::PlainText(text) => Some(text),
            ClipboardContent::Mixed(data) => Some(&data.plain_text),
            ClipboardContent::RichText(_) => None,
            ClipboardContent::WordFragment(_) => None,
            ClipboardContent::Image(_) => None,
            ClipboardContent::Empty => None,
        }
    }

    /// Gets rich text if available
    pub fn get_rich_text(&self) -> Option<&String> {
        match &self.content {
            ClipboardContent::RichText(html) => Some(html),
            ClipboardContent::Mixed(data) => data.rich_text.as_ref(),
            _ => None,
        }
    }

    /// Gets Word fragment if available
    pub fn get_word_fragment(&self) -> Option<&Vec<u8>> {
        match &self.content {
            ClipboardContent::WordFragment(data) => Some(data),
            ClipboardContent::Mixed(data) => data.word_fragment.as_ref(),
            _ => None,
        }
    }

    /// Gets image data if available
    pub fn get_image(&self) -> Option<&ImageClipboardData> {
        match &self.content {
            ClipboardContent::Image(data) => Some(data),
            _ => None,
        }
    }

    /// Detects the content type
    pub fn detect_content_type(&self) -> ContentType {
        match &self.content {
            ClipboardContent::PlainText(_) => ContentType::PlainText,
            ClipboardContent::RichText(_) => ContentType::RichText,
            ClipboardContent::WordFragment(_) => ContentType::Word,
            ClipboardContent::Image(_) => ContentType::Image,
            ClipboardContent::Mixed(data) => {
                if data.word_fragment.is_some() {
                    ContentType::Word
                } else if data.rich_text.is_some() {
                    ContentType::RichText
                } else {
                    ContentType::PlainText
                }
            }
            ClipboardContent::Empty => ContentType::Unknown,
        }
    }

    /// Adds current content to history
    fn add_to_history(&mut self) {
        self.history.push(self.content.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Gets content from history at index
    pub fn get_history(&self, index: usize) -> Option<&ClipboardContent> {
        self.history.get(index)
    }

    /// Gets history size
    pub fn history_size(&self) -> usize {
        self.history.len()
    }
}

/// System clipboard interface
pub struct SystemClipboardInterface;

impl SystemClipboardInterface {
    /// Copies text to the system clipboard
    pub fn copy_text(text: &str) -> ClipboardResult {
        // For FFI purposes, this is implemented by the frontend
        // The Rust core stores content in the internal clipboard
        let mut clipboard = CLIPBOARD.write().unwrap();
        clipboard.set_text(text);
        ClipboardResult::Success
    }

    /// Copies rich text to the system clipboard
    pub fn copy_rich_text(html: &str, plain_text: &str) -> ClipboardResult {
        let mut clipboard = CLIPBOARD.write().unwrap();
        clipboard.set_rich_text(html, plain_text);
        ClipboardResult::Success
    }

    /// Copies Word fragment to the clipboard
    pub fn copy_word_fragment(fragment: &[u8], plain_text: &str) -> ClipboardResult {
        let mut clipboard = CLIPBOARD.write().unwrap();
        clipboard.set_word_fragment(fragment, plain_text);
        ClipboardResult::Success
    }

    /// Copies image to the clipboard
    pub fn copy_image(data: &[u8], format: &str, width: u32, height: u32) -> ClipboardResult {
        let image = ImageClipboardData {
            data: data.to_vec(),
            format: format.to_string(),
            width,
            height,
        };
        let mut clipboard = CLIPBOARD.write().unwrap();
        clipboard.set_image(image);
        ClipboardResult::Success
    }

    /// Pastes text from the system clipboard
    /// Returns the pasted text content
    pub fn paste_text() -> Option<String> {
        let clipboard = CLIPBOARD.read().unwrap();
        clipboard.get_text().cloned()
    }

    /// Pastes rich text from the system clipboard
    /// Returns HTML content
    pub fn paste_rich_text() -> Option<String> {
        let clipboard = CLIPBOARD.read().unwrap();
        clipboard.get_rich_text().cloned()
    }

    /// Checks if the clipboard has text content
    pub fn has_text() -> bool {
        let clipboard = CLIPBOARD.read().unwrap();
        clipboard.get_text().is_some()
    }

    /// Checks if the clipboard has image content
    pub fn has_image() -> bool {
        let clipboard = CLIPBOARD.read().unwrap();
        clipboard.get_image().is_some()
    }

    /// Gets the detected content type
    pub fn get_content_type() -> ContentType {
        let clipboard = CLIPBOARD.read().unwrap();
        clipboard.detect_content_type()
    }

    /// Clears the clipboard
    pub fn clear() {
        let mut clipboard = CLIPBOARD.write().unwrap();
        clipboard.clear();
    }
}

// ==================== HTML/RTF Conversion Utilities ====================

/// HTML to plain text converter
pub fn html_to_plain_text(html: &str) -> String {
    // Simple HTML tag stripping
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity = String::new();

    for c in html.chars() {
        if in_entity {
            entity.push(c);
            if c == ';' {
                // Resolve common entities
                result.push(match entity.as_str() {
                    "&nbsp;" => ' ',
                    "&amp;" => '&',
                    "&lt;" => '<',
                    "&gt;" => '>',
                    "&quot;" => '"',
                    "&apos;" => '\'',
                    "&nbsp;" => ' ',
                    "&copy;" => '©',
                    "&reg;" => '®',
                    "&trade;" => '™',
                    _ => {
                        // Keep unknown entities as-is
                        entity.clone()
                    }
                });
                entity.clear();
                in_entity = false;
            }
        } else if c == '&' {
            in_entity = true;
            entity.clear();
            entity.push(c);
        } else if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            // Preserve newlines for block elements
            if let Some(prev) = result.chars().last() {
                if prev != '\n' && prev != '\r' {
                    result.push('\n');
                }
            }
        } else if !in_tag {
            result.push(c);
        }
    }

    // Clean up whitespace
    let result = result
        .replace("\n\n\n", "\n")
        .replace("\n\n", "\n")
        .trim()
        .to_string();

    result
}

/// Converts HTML to Word-compatible fragment
pub fn html_to_word_fragment(html: &str) -> String {
    // Convert basic HTML to OOXML format
    let mut result = String::new();

    // Parse and convert common HTML elements
    let lines: Vec<&str> = html.split("\n").collect();

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Convert paragraphs
        if trimmed.starts_with("<p") || trimmed.ends_with("</p>") {
            let content = strip_html_tags(trimmed);
            result.push_str(&format!(r#"<w:p><w:r><w:t>{}</w:t></w:r></w:p>"#, escape_xml(&content)));
        }
        // Convert headings
        else if let Some(level) = trimmed.find("<h1>") {
            let content = strip_html_tags(trimmed);
            result.push_str(&format!(r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>"#, escape_xml(&content)));
        }
        else if let Some(level) = trimmed.find("<h2>") {
            let content = strip_html_tags(trimmed);
            result.push_str(&format!(r#"<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>"#, escape_xml(&content)));
        }
        // Convert bold text
        else if trimmed.contains("<b>") || trimmed.contains("<strong>") {
            let content = strip_html_tags(trimmed);
            result.push_str(&format!(r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>{}</w:t></w:r></w:p>"#, escape_xml(&content)));
        }
        // Convert italic text
        else if trimmed.contains("<i>") || trimmed.contains("<em>") {
            let content = strip_html_tags(trimmed);
            result.push_str(&format!(r#"<w:p><w:r><w:rPr><w:i/></w:rPr><w:t>{}</w:t></w:r></w:p>"#, escape_xml(&content)));
        }
        // Regular text
        else {
            let content = strip_html_tags(trimmed);
            if !content.is_empty() {
                result.push_str(&format!(r#"<w:p><w:r><w:t>{}</w:t></w:r></w:p>"#, escape_xml(&content)));
            }
        }
    }

    result
}

/// Strips HTML tags from a string
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity = String::new();

    for c in html.chars() {
        if in_entity {
            entity.push(c);
            if c == ';' {
                result.push_str(&resolve_entity(&entity));
                entity.clear();
                in_entity = false;
            }
        } else if c == '&' {
            in_entity = true;
            entity.clear();
            entity.push(c);
        } else if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }

    result
}

/// Resolves HTML entities
fn resolve_entity(entity: &str) -> String {
    match entity {
        "&nbsp;" => " ".to_string(),
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&apos;" => "'".to_string(),
        "&copy;" => "©".to_string(),
        "&reg;" => "®".to_string(),
        "&trade;" => "™".to_string(),
        _ => entity.to_string(),
    }
}

/// Escapes XML special characters
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
}

/// Creates a complete Word document fragment with proper OOXML structure
pub fn create_word_fragment(
    text: &str,
    styles: Option<Vec<StyleInfo>>,
) -> String {
    let paragraphs: Vec<String> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            format!(
                r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:t>{}</w:t></w:r></w:p>"#,
                escape_xml(line.trim())
            )
        })
        .collect();

    let body = paragraphs.join("\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    {}
  </w:body>
</w:document>"#,
        body
    )
}

/// Converts plain text to HTML
pub fn text_to_html(text: &str) -> String {
    let escaped = text
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\n", "<br/>");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
</head>
<body>
{}
</body>
</html>"#,
        escaped
    )
}

/// Converts document fragment to HTML
pub fn fragment_to_html(fragment: &DocumentFragment) -> String {
    let mut html = String::new();

    for para in &fragment.paragraphs {
        html.push_str("<p>");

        // Apply paragraph formatting
        if let Some(align) = &para.alignment {
            html.push_str(&format!(r#"style="text-align: {};"#, align));
        }

        html.push_str(&escape_html(&para.text));
        html.push_str("</p>");
    }

    // Add style information if available
    if !fragment.styles.is_empty() {
        html.push_str("<style>");
        for style in &fragment.styles {
            if let Some(font_size) = style.font_size {
                html.push_str(&format!(
                    r#".size-{} {{ font-size: {}pt; }}"#,
                    style.font_size.unwrap_or(12),
                    font_size / 2
                ));
            }
            if let Some(font_family) = &style.font_family {
                html.push_str(&format!(
                    r#".family-{{}} {{ font-family: {}; }}"#,
                    font_family
                ));
            }
        }
        html.push_str("</style>");
    }

    html
}

/// Escapes HTML special characters
fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

// ==================== Public API Functions ====================

/// Copies the current selection as plain text
/// This function reads from the global document state
pub fn copy_selection_as_text() -> ClipboardResult {
    use crate::api::get_selection_text;
    let text = get_selection_text();
    if text.is_empty() {
        ClipboardResult::NoContent
    } else {
        SystemClipboardInterface::copy_text(&text)
    }
}

/// Copies the current selection as rich text (HTML format)
/// This function uses the internal clipboard storage
pub fn copy_selection_as_rich_text() -> ClipboardResult {
    use crate::api::get_selection_text;
    let text = get_selection_text();
    if text.is_empty() {
        return ClipboardResult::NoContent;
    }

    let html = text_to_html(&text);
    SystemClipboardInterface::copy_rich_text(&html, &text)
}

/// Copies the current selection as a Word document fragment
pub fn copy_selection_as_word_fragment() -> ClipboardResult {
    use crate::api::get_selection_text;
    let text = get_selection_text();
    if text.is_empty() {
        return ClipboardResult::NoContent;
    }

    let fragment = create_word_fragment(&text, None);
    SystemClipboardInterface::copy_word_fragment(fragment.as_bytes(), &text)
}

/// Pastes plain text at the current cursor position
/// Returns the text content after paste
pub fn paste_text_at_cursor() -> String {
    let text = match SystemClipboardInterface::paste_text() {
        Some(t) => t,
        None => String::new(),
    };
    if text.is_empty() {
        return String::new();
    }
    crate::api::insert_text(0, text)
}

/// Gets the current clipboard content as JSON for FFI
pub fn get_clipboard_content_json() -> String {
    let clipboard = CLIPBOARD.read().unwrap();
    let content_type = clipboard.detect_content_type();

    let json = match clipboard.get_content() {
        ClipboardContent::PlainText(text) => {
            serde_json::json!({
                "type": "plain_text",
                "content": text
            })
        }
        ClipboardContent::RichText(html) => {
            serde_json::json!({
                "type": "rich_text",
                "content": html
            })
        }
        ClipboardContent::WordFragment(_) => {
            serde_json::json!({
                "type": "word_fragment",
                "content": "[binary data]"
            })
        }
        ClipboardContent::Image(img) => {
            serde_json::json!({
                "type": "image",
                "format": img.format,
                "width": img.width,
                "height": img.height,
                "data_size": img.data.len()
            })
        }
        ClipboardContent::Mixed(data) => {
            serde_json::json!({
                "type": "mixed",
                "plain_text": data.plain_text,
                "has_rich_text": data.rich_text.is_some(),
                "has_word_fragment": data.word_fragment.is_some()
            })
        }
        ClipboardContent::Empty => {
            serde_json::json!({
                "type": "empty",
                "content": ""
            })
        }
    };

    serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
}

/// Checks if clipboard has content
pub fn clipboard_has_content() -> bool {
    !matches!(
        SystemClipboardInterface::get_content_type(),
        ContentType::Unknown
    )
}

/// Gets the content type as string
pub fn get_clipboard_type_string() -> String {
    match SystemClipboardInterface::get_content_type() {
        ContentType::PlainText => "text".to_string(),
        ContentType::RichText => "rich_text".to_string(),
        ContentType::Word => "word".to_string(),
        ContentType::Image => "image".to_string(),
        ContentType::Unknown => "unknown".to_string(),
    }
}

/// Clears the clipboard
pub fn clear_clipboard() {
    SystemClipboardInterface::clear();
}

/// Gets the clipboard history size
pub fn get_clipboard_history_size() -> usize {
    let clipboard = CLIPBOARD.read().unwrap();
    clipboard.history_size()
}

/// Gets content from clipboard history
pub fn get_clipboard_history(index: usize) -> Option<String> {
    let clipboard = CLIPBOARD.read().unwrap();
    clipboard.get_history(index).map(|c| format!("{:?}", c))
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_basic() {
        let mut cb = Clipboard::new();

        // Test plain text
        cb.set_text("Hello World");
        assert_eq!(cb.get_text(), Some(&"Hello World".to_string()));
        assert_eq!(cb.detect_content_type(), ContentType::PlainText);
    }

    #[test]
    fn test_clipboard_rich_text() {
        let mut cb = Clipboard::new();

        cb.set_rich_text("<b>Bold</b> text", "Bold text");
        assert_eq!(cb.get_text(), Some(&"Bold text".to_string()));
        assert_eq!(cb.get_rich_text(), Some(&"<b>Bold</b> text".to_string()));
        assert_eq!(cb.detect_content_type(), ContentType::RichText);
    }

    #[test]
    fn test_clipboard_image() {
        let mut cb = Clipboard::new();

        let image = ImageClipboardData {
            data: vec![0x89, 0x50, 0x4E, 0x47], // PNG header
            format: "png".to_string(),
            width: 100,
            height: 50,
        };

        cb.set_image(image.clone());
        assert_eq!(cb.get_image(), Some(&image));
        assert_eq!(cb.detect_content_type(), ContentType::Image);
    }

    #[test]
    fn test_html_to_plain_text() {
        let html = "<p>Hello <b>World</b></p>";
        let plain = html_to_plain_text(html);
        assert!(plain.contains("Hello"));
        assert!(plain.contains("World"));
    }

    #[test]
    fn test_strip_html_tags() {
        let html = "<p>Test <b>bold</b> and <i>italic</i></p>";
        let plain = strip_html_tags(html);
        assert_eq!(plain, "Test bold and italic");
    }

    #[test]
    fn test_escape_xml() {
        let text = "<test>&\"test\"</test>";
        let escaped = escape_xml(text);
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&quot;"));
    }

    #[test]
    fn test_create_word_fragment() {
        let text = "Hello\nWorld";
        let fragment = create_word_fragment(text, None);
        assert!(fragment.contains("w:document"));
        assert!(fragment.contains("Hello"));
        assert!(fragment.contains("World"));
    }

    #[test]
    fn test_text_to_html() {
        let text = "Hello\nWorld";
        let html = text_to_html(text);
        assert!(html.contains("<br/>"));
        assert!(html.contains("Hello"));
        assert!(html.contains("World"));
    }

    #[test]
    fn test_clipboard_history() {
        let mut cb = Clipboard::new();

        cb.set_text("First");
        cb.set_text("Second");
        cb.set_text("Third");

        assert_eq!(cb.history_size(), 3);
        assert!(cb.get_history(0).is_some());
        assert!(cb.get_history(1).is_some());
        assert!(cb.get_history(2).is_some());
    }

    #[test]
    fn test_clipboard_clear() {
        let mut cb = Clipboard::new();
        cb.set_text("Test");
        assert_eq!(cb.detect_content_type(), ContentType::PlainText);

        cb.clear();
        assert_eq!(cb.detect_content_type(), ContentType::Unknown);
    }
}

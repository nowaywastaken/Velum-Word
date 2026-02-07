//! # Table of Contents (TOC) Module
//!
//! Provides functionality for generating and managing Table of Contents
//! from document paragraphs and their associated styles.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Tab leader style for page number dots
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TabLeader {
    /// No leader (space)
    None,
    /// Dotted leader (....)
    Dots,
    /// Dashed leader (----)
    Dashes,
    /// Underlined leader (____)
    Underline,
    /// Thick line leader (████)
    ThickLine,
}

/// Represents a single entry in the Table of Contents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocEntry {
    /// Outline level (1-9, typically 1-3 for most TOCs)
    pub level: u32,
    /// Display text for the entry
    pub text: String,
    /// Page number where this entry appears (1-based)
    pub page_number: u32,
    /// Byte offset in the document where this entry is located
    pub target_offset: usize,
    /// Hyperlink target (anchor ID or URL)
    pub hyperlink: Option<String>,
    /// Paragraph style ID associated with this entry
    pub style_id: Option<String>,
    /// Character index within the paragraph for precise navigation
    pub char_index: usize,
}

/// Styling configuration for a Table of Contents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocStyle {
    /// Maximum outline level to include in TOC
    pub show_levels: u32,
    /// Whether to include page numbers
    pub include_page_numbers: bool,
    /// Whether entries should be clickable hyperlinks
    pub hyperlink_entries: bool,
    /// Leader style between entry text and page number
    pub tab_leader: TabLeader,
    /// Style configuration for each level
    pub toc_level_style: Vec<TocLevelStyle>,
    /// Title of the TOC
    pub title: String,
    /// Hide page numbers for web/viewer display
    pub hide_page_numbers_for_web: bool,
    /// Use outline levels from document structure
    pub use_document_outline: bool,
}

impl Default for TocStyle {
    fn default() -> Self {
        TocStyle {
            show_levels: 3,
            include_page_numbers: true,
            hyperlink_entries: true,
            tab_leader: TabLeader::Dots,
            toc_level_style: Vec::new(),
            title: "Table of Contents".to_string(),
            hide_page_numbers_for_web: false,
            use_document_outline: true,
        }
    }
}

/// Styling for a specific TOC level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocLevelStyle {
    /// Outline level this style applies to
    pub level: u32,
    /// Paragraph style ID that maps to this level
    pub paragraph_style: String,
    /// Character/text style ID for the entry text
    pub text_style: String,
    /// Indentation in points (1/72 inch)
    pub indent_points: f32,
    /// Font size in points
    pub font_size: Option<f32>,
    /// Bold formatting
    pub bold: bool,
    /// Italic formatting
    pub italic: bool,
    /// Text color (hex RGB)
    pub color: Option<String>,
}

impl TocLevelStyle {
    /// Create a default level style for the given level
    pub fn new(level: u32) -> Self {
        TocLevelStyle {
            level,
            paragraph_style: format!("Heading{}", level),
            text_style: format!("Heading{}Char", level),
            indent_points: level as f32 * 720.0, // 720 twips = 0.5 inch per level
            font_size: Some(12.0 + (4 - level.min(9)) as f32), // Larger for lower levels
            bold: level == 1,
            italic: false,
            color: None,
        }
    }
}

/// Builder for constructing Table of Contents from a document
#[derive(Debug, Clone)]
pub struct TocBuilder {
    /// TOC styling configuration
    toc_style: TocStyle,
    /// Document entries (paragraphs with positions)
    document_entries: Vec<DocumentEntry>,
    /// Collected TOC entries
    entries: Vec<TocEntry>,
    /// Map of style ID to outline level
    style_to_level: HashMap<String, u32>,
    /// Page calculations
    page_height: f32,
    /// Current position tracker
    current_offset: usize,
    /// Current page number
    current_page: u32,
    /// Current vertical position on page
    current_y: f32,
    /// Line height used for page calculations
    line_height: f32,
    /// Paragraph styles from the document
    paragraph_styles: HashMap<String, TocLevelStyle>,
}

/// A document entry representing a paragraph for TOC generation
#[derive(Debug, Clone)]
pub struct DocumentEntry {
    /// Text content of the paragraph
    pub text: String,
    /// Byte offset in the document
    pub offset: usize,
    /// Character index within the document
    pub char_index: usize,
    /// Paragraph style ID
    pub style_id: Option<String>,
    /// Outline level (if specified in paragraph properties)
    pub outline_level: Option<u32>,
    /// Height of the paragraph in layout units
    pub height: f32,
    /// Whether this is a heading-like paragraph
    pub is_heading: bool,
}

/// Result of TOC generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocResult {
    /// All TOC entries
    pub entries: Vec<TocEntry>,
    /// Total number of pages covered
    pub total_pages: u32,
    /// Whether generation was successful
    pub success: bool,
    /// Any error message
    pub error_message: Option<String>,
}

/// Error type for TOC operations
#[derive(Debug, Clone, PartialEq)]
pub enum TocError {
    /// Document is empty
    EmptyDocument,
    /// Invalid outline level
    InvalidOutlineLevel(u32),
    /// Style not found
    StyleNotFound(String),
    /// Page calculation error
    PageCalculationError,
}

impl std::fmt::Display for TocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TocError::EmptyDocument => write!(f, "Document is empty"),
            TocError::InvalidOutlineLevel(level) => write!(f, "Invalid outline level: {}", level),
            TocError::StyleNotFound(style) => write!(f, "Style not found: {}", style),
            TocError::PageCalculationError => write!(f, "Error calculating page positions"),
        }
    }
}

impl std::error::Error for TocError {}

impl TocBuilder {
    /// Create a new TOC builder with default style
    pub fn new() -> Self {
        TocBuilder {
            toc_style: TocStyle::default(),
            document_entries: Vec::new(),
            entries: Vec::new(),
            style_to_level: HashMap::new(),
            page_height: 792.0, // Default letter size height in points
            current_offset: 0,
            current_page: 1,
            current_y: 72.0, // Start 1 inch from top
            line_height: 12.0,
            paragraph_styles: HashMap::new(),
        }
    }

    /// Create a new TOC builder with custom style
    pub fn with_style(style: TocStyle) -> Self {
        let mut builder = TocBuilder {
            toc_style: style,
            document_entries: Vec::new(),
            entries: Vec::new(),
            style_to_level: HashMap::new(),
            page_height: 792.0,
            current_offset: 0,
            current_page: 1,
            current_y: 72.0,
            line_height: 12.0,
            paragraph_styles: HashMap::new(),
        };

        // Initialize level styles
        for level_style in &builder.toc_style.toc_level_style {
            builder.paragraph_styles.insert(
                level_style.paragraph_style.clone(),
                level_style.clone(),
            );
        }

        builder
    }

    /// Set the page height for page number calculations
    #[inline]
    pub fn set_page_height(&mut self, height: f32) {
        self.page_height = height;
    }

    /// Set the line height for layout calculations
    #[inline]
    pub fn set_line_height(&mut self, line_height: f32) {
        self.line_height = line_height;
    }

    /// Add a style-to-level mapping
    pub fn add_style_level_mapping(&mut self, style_id: &str, level: u32) {
        if level == 0 || level > 9 {
            return;
        }
        self.style_to_level.insert(style_id.to_string(), level);
    }

    /// Add a document paragraph entry for TOC generation
    pub fn add_entry(&mut self, entry: DocumentEntry) {
        self.document_entries.push(entry);
    }

    /// Add a paragraph entry from text and style
    pub fn add_paragraph(&mut self, text: &str, style_id: Option<&str>, height: f32) {
        let entry = DocumentEntry {
            text: text.to_string(),
            offset: self.current_offset,
            char_index: self.current_offset,
            style_id: style_id.map(|s| s.to_string()),
            outline_level: style_id.and_then(|s| self.style_to_level.get(s).copied()),
            height,
            is_heading: style_id.map(|s| {
                s.starts_with("Heading") || s.starts_with("Heading ")
            }).unwrap_or(false),
        };
        self.current_offset += text.len() + 1; // +1 for newline
        self.document_entries.push(entry);
    }

    /// Add multiple paragraphs at once
    pub fn add_paragraphs(&mut self, paragraphs: &[(&str, Option<&str>, f32)]) {
        for (text, style_id, height) in paragraphs {
            self.add_paragraph(text, *style_id, *height);
        }
    }

    /// Set custom level styles
    pub fn set_level_styles(&mut self, styles: Vec<TocLevelStyle>) {
        self.toc_style.toc_level_style = styles.clone();
        self.paragraph_styles.clear();
        for style in styles {
            self.paragraph_styles.insert(style.paragraph_style.clone(), style);
        }
    }

    /// Determine the outline level for a paragraph
    fn determine_level(&self, entry: &DocumentEntry) -> Option<u32> {
        // First check explicit outline level
        if let Some(level) = entry.outline_level {
            return Some(level.min(9));
        }

        // Check style-based level
        if let Some(ref style_id) = entry.style_id {
            if let Some(level) = self.style_to_level.get(style_id) {
                return Some(*level);
            }

            // Try extracting level from Heading1, Heading2, etc.
            if style_id.starts_with("Heading") {
                let suffix = &style_id["Heading".len()..];
                if let Ok(level) = suffix.parse::<u32>() {
                    if level >= 1 && level <= 9 {
                        return Some(level);
                    }
                }
            }
        }

        None
    }

    /// Calculate page number for an entry based on its offset
    fn calculate_page(&self, entry: &DocumentEntry) -> u32 {
        // Simple page calculation based on document position
        // In a real implementation, this would use actual page layout info
        let total_height: f32 = self.document_entries
            .iter()
            .take_while(|e| e.offset <= entry.offset)
            .map(|e| e.height)
            .sum();

        let page_height_available = self.page_height - 144.0; // Account for margins
        ((total_height as f32) / page_height_available).floor() as u32 + 1
    }

    /// Generate the TOC entries from collected document entries
    pub fn build(&mut self) -> TocResult {
        self.entries.clear();

        if self.document_entries.is_empty() {
            return TocResult {
                entries: Vec::new(),
                total_pages: 0,
                success: false,
                error_message: Some("No document entries to build TOC from".to_string()),
            };
        }

        // Determine max level to include
        let max_level = self.toc_style.show_levels;

        // Process each document entry
        for entry in &self.document_entries {
            // Determine the level for this entry
            let level = match self.determine_level(entry) {
                Some(l) if l <= max_level => l,
                _ => continue, // Skip non-heading entries
            };

            // Skip entries with empty text
            if entry.text.trim().is_empty() {
                continue;
            }

            // Calculate page number
            let page_number = self.calculate_page(entry);

            // Generate hyperlink target
            let hyperlink = if self.toc_style.hyperlink_entries {
                Some(format!("#toc-{}", self.entries.len()))
            } else {
                None
            };

            // Create TOC entry
            let toc_entry = TocEntry {
                level,
                text: entry.text.trim().to_string(),
                page_number,
                target_offset: entry.offset,
                hyperlink,
                style_id: entry.style_id.clone(),
                char_index: entry.char_index,
            };

            self.entries.push(toc_entry);
        }

        // Calculate total pages
        let total_pages = self.entries
            .iter()
            .map(|e| e.page_number)
            .max()
            .unwrap_or(1);

        TocResult {
            entries: self.entries.clone(),
            total_pages,
            success: true,
            error_message: None,
        }
    }

    /// Get the built TOC entries
    pub fn entries(&self) -> &[TocEntry] {
        &self.entries
    }

    /// Get mutable access to built TOC entries
    pub fn entries_mut(&mut self) -> &mut Vec<TocEntry> {
        &mut self.entries
    }

    /// Convert TOC entries to a renderable format
    pub fn to_renderable(&self) -> Vec<RenderedTocEntry> {
        self.entries
            .iter()
            .map(|entry| {
                let level_style = self.paragraph_styles.values()
                    .find(|s| s.level == entry.level)
                    .cloned()
                    .unwrap_or_else(|| TocLevelStyle::new(entry.level));

                RenderedTocEntry {
                    level: entry.level,
                    text: &entry.text,
                    page_number: entry.page_number,
                    indent: level_style.indent_points,
                    hyperlink: entry.hyperlink.as_deref(),
                    font_size: level_style.font_size,
                    bold: level_style.bold,
                    italic: level_style.italic,
                    color: level_style.color.clone(),
                }
            })
            .collect()
    }

    /// Clear all entries and start fresh
    pub fn clear(&mut self) {
        self.document_entries.clear();
        self.entries.clear();
        self.current_offset = 0;
    }

    /// Generate HTML representation of the TOC
    pub fn to_html(&self) -> String {
        if self.entries.is_empty() {
            return String::from("<p>No table of contents available.</p>");
        }

        let mut html = format!("<h1 class=\"toc-title\">{}</h1>\n", self.toc_style.title);
        html.push_str("<ul class=\"toc\">\n");

        let mut current_level = 0;

        for entry in &self.entries {
            if entry.level > current_level {
                while entry.level > current_level {
                    html.push_str("<ul>\n");
                    current_level += 1;
                }
            } else if entry.level < current_level {
                while entry.level < current_level {
                    html.push_str("</ul>\n");
                    current_level -= 1;
                }
            }

            let indent = (entry.level as usize).saturating_sub(1) * 24;
            let hyperlink = entry.hyperlink.as_deref().unwrap_or("");
            let page_number = if self.toc_style.include_page_numbers && !self.toc_style.hide_page_numbers_for_web {
                let leader = match self.toc_style.tab_leader {
                    TabLeader::Dots => " ........................................ ",
                    TabLeader::Dashes => " ------------------------------------ ",
                    TabLeader::Underline => " ____________________________________ ",
                    TabLeader::ThickLine => " ════════════════════════════════════ ",
                    TabLeader::None => "   ",
                };
                format!("{}{}{}", leader, entry.page_number, "</a>")
            } else {
                String::new()
            };

            html.push_str(&format!(
                r#"<li class="toc-level-{}" style="margin-left: {}px;">
    <a href="{}" class="toc-link">{}{}</a>
</li>
"#,
                entry.level,
                indent,
                hyperlink,
                entry.text,
                page_number
            ));
        }

        while current_level > 0 {
            html.push_str("</ul>\n");
            current_level -= 1;
        }

        html.push_str("</ul>");

        html
    }

    /// Generate plain text representation of the TOC
    pub fn to_text(&self) -> String {
        if self.entries.is_empty() {
            return String::from("No table of contents available.\n");
        }

        let mut text = format!("{}\n{}\n\n", self.toc_style.title, "=".repeat(self.toc_style.title.len()));

        for entry in &self.entries {
            let indent = "    ".repeat((entry.level as usize).saturating_sub(1));
            let dots = match self.toc_style.tab_leader {
                TabLeader::Dots => ".",
                TabLeader::Dashes => "-",
                TabLeader::Underline => "_",
                TabLeader::ThickLine => "=",
                TabLeader::None => " ",
            };

            let dot_line = if self.toc_style.include_page_numbers {
                let max_width = 60;
                let text_len = indent.len() + entry.text.len() + 8; // +8 for page number space
                let dot_count = if text_len < max_width {
                    (max_width - text_len) / dots.len()
                } else {
                    2
                };
                dots.repeat(dot_count)
            } else {
                String::new()
            };

            let page_str = if self.toc_style.include_page_numbers {
                format!(" {}", entry.page_number)
            } else {
                String::new()
            };

            text.push_str(&format!("{}{}{}{}\n", indent, entry.text, dot_line, page_str));
        }

        text
    }
}

impl Default for TocBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Renderable TOC entry for UI rendering
#[derive(Debug, Clone)]
pub struct RenderedTocEntry<'a> {
    /// Outline level
    pub level: u32,
    /// Entry text
    pub text: &'a str,
    /// Page number
    pub page_number: u32,
    /// Indentation in points
    pub indent: f32,
    /// Hyperlink target
    pub hyperlink: Option<&'a str>,
    /// Font size in points
    pub font_size: Option<f32>,
    /// Bold formatting
    pub bold: bool,
    /// Italic formatting
    pub italic: bool,
    /// Text color
    pub color: Option<String>,
}

/// Configuration for TOC generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocConfig {
    /// Maximum levels to include
    pub max_levels: u32,
    /// Minimum entries to trigger TOC generation
    pub min_entries: usize,
    /// Skip entries shorter than this
    #[allow(dead_code)]
    pub min_entry_length: usize,
    /// Include page numbers
    pub include_page_numbers: bool,
    /// Generate hyperlinks
    pub enable_hyperlinks: bool,
    /// Custom title
    pub title: Option<String>,
}

impl Default for TocConfig {
    fn default() -> Self {
        TocConfig {
            max_levels: 3,
            min_entries: 3,
            min_entry_length: 1,
            include_page_numbers: true,
            enable_hyperlinks: true,
            title: None,
        }
    }
}

/// Utility functions for TOC operations
pub mod utils {
    use super::*;

    /// Extract heading text from a paragraph
    pub fn extract_heading_text(text: &str) -> String {
        // Remove common heading prefixes like "1.", "1.1", "I.", etc.
        let re = regex::Regex::new(r"^[\d.]+\s+|[IVXLC]+\.\s+|[A-Z]\.\s+").unwrap();
        re.replace(text, "").trim().to_string()
    }

    /// Check if text looks like a heading
    pub fn is_heading_like(text: &str, max_length: usize) -> bool {
        if text.trim().is_empty() {
            return false;
        }
        if text.len() > max_length {
            return false;
        }
        // Check for common heading patterns
        let patterns = [
            r"^#{1,6}\s",           // Markdown style
            r"^[\d]+\.[\s\d]",       // Numbered (1., 1.1)
            r"^Chapter\s+\d",        // Chapter headings
        ];

        let combined = patterns.join("|");
        let re = regex::Regex::new(&combined).unwrap();
        re.is_match(text)  // Returns true if it DOES match heading patterns
    }

    /// Estimate paragraph height based on text
    pub fn estimate_paragraph_height(text: &str, line_height: f32, chars_per_line: usize) -> f32 {
        let lines = if chars_per_line > 0 {
            (text.chars().count() as f32 / chars_per_line as f32).ceil() as usize
        } else {
            1
        };
        lines as f32 * line_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_builder_new() {
        let builder = TocBuilder::new();
        assert!(builder.entries.is_empty());
        assert_eq!(builder.toc_style.show_levels, 3);
    }

    #[test]
    fn test_toc_builder_with_style() {
        let style = TocStyle {
            show_levels: 5,
            include_page_numbers: true,
            hyperlink_entries: false,
            tab_leader: TabLeader::Dashes,
            toc_level_style: Vec::new(),
            title: "Custom TOC".to_string(),
            hide_page_numbers_for_web: false,
            use_document_outline: false,
        };
        let builder = TocBuilder::with_style(style);
        assert_eq!(builder.toc_style.show_levels, 5);
        assert_eq!(builder.toc_style.tab_leader, TabLeader::Dashes);
    }

    #[test]
    fn test_add_paragraph() {
        let mut builder = TocBuilder::new();
        builder.add_paragraph("Introduction", Some("Heading1"), 24.0);
        builder.add_paragraph("Background", Some("Heading2"), 24.0);
        builder.add_paragraph("This is regular text.", Some("Normal"), 12.0);

        assert_eq!(builder.document_entries.len(), 3);
        assert_eq!(builder.current_offset, "Introduction".len() + 1 + "Background".len() + 1 + "This is regular text.".len() + 1);
    }

    #[test]
    fn test_determine_level_heading_style() {
        let mut builder = TocBuilder::new();

        let entry1 = DocumentEntry {
            text: "Chapter 1".to_string(),
            offset: 0,
            char_index: 0,
            style_id: Some("Heading1".to_string()),
            outline_level: None,
            height: 24.0,
            is_heading: true,
        };

        let entry2 = DocumentEntry {
            text: "Section 1.1".to_string(),
            offset: 10,
            char_index: 10,
            style_id: Some("Heading2".to_string()),
            outline_level: None,
            height: 24.0,
            is_heading: true,
        };

        assert_eq!(builder.determine_level(&entry1), Some(1));
        assert_eq!(builder.determine_level(&entry2), Some(2));
    }

    #[test]
    fn test_determine_level_outline_property() {
        let mut builder = TocBuilder::new();

        let entry = DocumentEntry {
            text: "Custom Heading".to_string(),
            offset: 0,
            char_index: 0,
            style_id: Some("MyStyle".to_string()),
            outline_level: Some(4),
            height: 24.0,
            is_heading: true,
        };

        assert_eq!(builder.determine_level(&entry), Some(4));
    }

    #[test]
    fn test_determine_level_invalid() {
        let mut builder = TocBuilder::new();

        // Regular paragraph without heading style
        let entry = DocumentEntry {
            text: "Just text".to_string(),
            offset: 0,
            char_index: 0,
            style_id: Some("Normal".to_string()),
            outline_level: None,
            height: 12.0,
            is_heading: false,
        };

        assert_eq!(builder.determine_level(&entry), None);
    }

    #[test]
    fn test_build_empty_document() {
        let mut builder = TocBuilder::new();
        let result = builder.build();

        assert!(!result.success);
        assert!(result.entries.is_empty());
        assert_eq!(result.error_message, Some("No document entries to build TOC from".to_string()));
    }

    #[test]
    fn test_build_with_headings() {
        let mut builder = TocBuilder::new();

        builder.add_paragraph("Chapter 1: Introduction", Some("Heading1"), 24.0);
        builder.add_paragraph("This is the introduction text.", Some("Normal"), 48.0);
        builder.add_paragraph("Section 1.1: Background", Some("Heading2"), 24.0);
        builder.add_paragraph("Background information here.", Some("Normal"), 36.0);

        let result = builder.build();

        assert!(result.success);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].text, "Chapter 1: Introduction");
        assert_eq!(result.entries[0].level, 1);
        assert_eq!(result.entries[1].text, "Section 1.1: Background");
        assert_eq!(result.entries[1].level, 2);
    }

    #[test]
    fn test_build_filters_by_level() {
        let mut builder = TocBuilder::new();
        builder.toc_style.show_levels = 2;

        builder.add_paragraph("Level 1", Some("Heading1"), 24.0);
        builder.add_paragraph("Level 2", Some("Heading2"), 24.0);
        builder.add_paragraph("Level 3", Some("Heading3"), 24.0);

        let result = builder.build();

        assert_eq!(result.entries.len(), 2);
        assert!(result.entries.iter().all(|e| e.level <= 2));
    }

    #[test]
    fn test_build_with_hyperlinks() {
        let mut builder = TocBuilder::new();
        builder.toc_style.hyperlink_entries = true;

        builder.add_paragraph("First Heading", Some("Heading1"), 24.0);

        let result = builder.build();

        assert!(result.success);
        assert!(result.entries[0].hyperlink.is_some());
        assert_eq!(result.entries[0].hyperlink, Some("#toc-0".to_string()));
    }

    #[test]
    fn test_build_without_hyperlinks() {
        let mut builder = TocBuilder::new();
        builder.toc_style.hyperlink_entries = false;

        builder.add_paragraph("First Heading", Some("Heading1"), 24.0);

        let result = builder.build();

        assert!(result.success);
        assert!(result.entries[0].hyperlink.is_none());
    }

    #[test]
    fn test_toc_level_style_new() {
        let style = TocLevelStyle::new(1);

        assert_eq!(style.level, 1);
        assert_eq!(style.paragraph_style, "Heading1");
        assert_eq!(style.text_style, "Heading1Char");
        assert_eq!(style.indent_points, 720.0);
        // font_size = 12.0 + (4 - 1) = 15.0 for level 1
        assert_eq!(style.font_size, Some(15.0));
        assert!(style.bold);
    }

    #[test]
    fn test_toc_style_default() {
        let style = TocStyle::default();

        assert_eq!(style.show_levels, 3);
        assert!(style.include_page_numbers);
        assert!(style.hyperlink_entries);
        assert_eq!(style.tab_leader, TabLeader::Dots);
        assert_eq!(style.title, "Table of Contents");
    }

    #[test]
    fn test_tab_leader_variants() {
        assert_eq!(TabLeader::None, TabLeader::None);
        assert_eq!(TabLeader::Dots, TabLeader::Dots);
        assert_eq!(TabLeader::Dashes, TabLeader::Dashes);
        assert_eq!(TabLeader::Underline, TabLeader::Underline);
        assert_eq!(TabLeader::ThickLine, TabLeader::ThickLine);
    }

    #[test]
    fn test_toc_entry_serialization() {
        let entry = TocEntry {
            level: 1,
            text: "Introduction".to_string(),
            page_number: 1,
            target_offset: 0,
            hyperlink: Some("#intro".to_string()),
            style_id: Some("Heading1".to_string()),
            char_index: 0,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TocEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_add_style_level_mapping() {
        let mut builder = TocBuilder::new();
        builder.add_style_level_mapping("Title", 1);
        builder.add_style_level_mapping("Subtitle", 2);

        let entry = DocumentEntry {
            text: "My Title".to_string(),
            offset: 0,
            char_index: 0,
            style_id: Some("Title".to_string()),
            outline_level: None,
            height: 24.0,
            is_heading: true,
        };

        assert_eq!(builder.determine_level(&entry), Some(1));
    }

    #[test]
    fn test_clear_builder() {
        let mut builder = TocBuilder::new();
        builder.add_paragraph("Heading", Some("Heading1"), 24.0);

        assert!(!builder.document_entries.is_empty());

        builder.clear();

        assert!(builder.document_entries.is_empty());
        assert!(builder.entries.is_empty());
        assert_eq!(builder.current_offset, 0);
    }

    #[test]
    fn test_add_paragraphs_batch() {
        let mut builder = TocBuilder::new();
        builder.add_paragraphs(&[
            ("Chapter 1", Some("Heading1"), 24.0),
            ("Section 1.1", Some("Heading2"), 24.0),
            ("Section 1.2", Some("Heading2"), 24.0),
        ]);

        assert_eq!(builder.document_entries.len(), 3);
    }

    #[test]
    fn test_to_html() {
        let mut builder = TocBuilder::new();
        builder.add_paragraph("Introduction", Some("Heading1"), 24.0);
        builder.build(); // Must call build() first

        let html = builder.to_html();

        assert!(html.contains("Table of Contents"));
        assert!(html.contains("<ul class=\"toc\">"));
        assert!(html.contains("Introduction"));
    }

    #[test]
    fn test_to_text() {
        let mut builder = TocBuilder::new();
        builder.add_paragraph("Introduction", Some("Heading1"), 24.0);
        builder.build(); // Must call build() first

        let text = builder.to_text();

        // Default title is "Table of Contents"
        assert!(text.contains("Table of Contents"));
        assert!(text.contains("Introduction"));
        assert!(text.contains("=="));
    }

    #[test]
    fn test_to_renderable() {
        let mut builder = TocBuilder::new();
        builder.add_paragraph("Introduction", Some("Heading1"), 24.0);
        builder.build();

        let rendered = builder.to_renderable();

        assert_eq!(rendered.len(), 1);
        assert_eq!(rendered[0].text, "Introduction");
        assert_eq!(rendered[0].level, 1);
    }

    #[test]
    fn test_entries_clone() {
        let mut builder = TocBuilder::new();
        builder.add_paragraph("Test", Some("Heading1"), 24.0);
        builder.build();

        let entries = builder.entries();
        assert_eq!(entries.len(), 1);

        let entries2 = builder.entries.clone();
        assert_eq!(entries, &entries2[..]);
    }

    #[test]
    fn test_invalid_outline_level_capped() {
        let mut builder = TocBuilder::new();
        builder.toc_style.show_levels = 2;

        let entry = DocumentEntry {
            text: "Deep Heading".to_string(),
            offset: 0,
            char_index: 0,
            style_id: Some("Heading5".to_string()),
            outline_level: None,
            height: 24.0,
            is_heading: true,
        };

        // Level 5 should be filtered out since max is 2
        builder.document_entries.push(entry);
        let result = builder.build();

        assert!(result.entries.is_empty());
    }

    #[test]
    fn test_empty_paragraphs_skipped() {
        let mut builder = TocBuilder::new();
        builder.add_paragraph("", Some("Heading1"), 24.0);
        builder.add_paragraph("   ", Some("Heading1"), 24.0);
        builder.add_paragraph("Valid", Some("Heading1"), 24.0);

        let result = builder.build();

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].text, "Valid");
    }

    #[test]
    fn test_set_level_styles() {
        let mut builder = TocBuilder::new();
        builder.set_level_styles(vec![
            TocLevelStyle::new(1),
            TocLevelStyle::new(2),
        ]);

        assert_eq!(builder.paragraph_styles.len(), 2);
    }

    #[test]
    fn test_toc_config_default() {
        let config = TocConfig::default();

        assert_eq!(config.max_levels, 3);
        assert_eq!(config.min_entries, 3);
        assert!(config.include_page_numbers);
        assert!(config.enable_hyperlinks);
    }

    #[test]
    fn test_utils_extract_heading_text() {
        use super::utils::*;

        assert_eq!(extract_heading_text("1. Introduction"), "Introduction");
        assert_eq!(extract_heading_text("1.1 Background"), "Background");
        assert_eq!(extract_heading_text("I. Overview"), "Overview");
        // "Chapter" pattern is not in the regex, so it won't be stripped
        assert_eq!(extract_heading_text("Chapter 1 Getting Started"), "Chapter 1 Getting Started");
        assert_eq!(extract_heading_text("Already Clean"), "Already Clean");
    }

    #[test]
    fn test_utils_is_heading_like() {
        use super::utils::*;

        // Plain text without prefix is NOT heading-like
        assert!(!is_heading_like("Introduction", 50));
        // Markdown and numbered prefixes ARE heading-like
        assert!(is_heading_like("# Heading", 50));
        assert!(is_heading_like("1. Overview", 50));
        assert!(!is_heading_like("", 50));
        assert!(!is_heading_like("This is a very long text that is definitely not a heading because it exceeds the maximum length allowed", 50));
    }

    #[test]
    fn test_utils_estimate_paragraph_height() {
        use super::utils::*;

        let height = estimate_paragraph_height("Short text", 12.0, 80);
        assert!(height > 0.0);

        let height = estimate_paragraph_height("This is a longer paragraph with more text that should span multiple lines", 12.0, 20);
        assert!(height > 12.0);
    }

    #[test]
    fn test_toc_error_display() {
        assert_eq!(TocError::EmptyDocument.to_string(), "Document is empty");
        assert_eq!(TocError::InvalidOutlineLevel(10).to_string(), "Invalid outline level: 10");
        let style_err = TocError::StyleNotFound("Missing".to_string());
        assert_eq!(style_err.to_string(), "Style not found: Missing");
        assert_eq!(TocError::PageCalculationError.to_string(), "Error calculating page positions");
    }
}

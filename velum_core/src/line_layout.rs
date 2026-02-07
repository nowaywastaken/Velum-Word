//! # Line Layout Module
//!
//! Provides higher-level text layout functionality including paragraph layout
//! and bidirectional text support.

use crate::line_breaking::{BreakType, LineBreaker};
use serde::{Deserialize, Serialize};

/// Line spacing rule enumeration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LineSpacingRule {
    Single,     // 单倍行距 (1.0)
    OneAndHalf, // 1.5倍行距
    Double,     // 2倍行距
    AtLeast,    // 最小值
    Exactly,    // 固定值
    Multiple,   // 多倍行距 (如 1.25)
}

/// Text alignment enumeration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Alignment {
    Left,
    Right,
    Center,
    Justify,
}

/// Default implementation for Alignment
impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}

/// Default implementation for LineSpacingRule
impl Default for LineSpacingRule {
    fn default() -> Self {
        LineSpacingRule::Single
    }
}

/// Represents a line with visual layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutLine {
    /// Start byte offset in original text
    pub start: usize,
    /// End byte offset in original text
    pub end: usize,
    /// Width of the line in abstract units
    pub width: f32,
    /// Type of break
    pub break_type: String,
    /// Visual order for bidirectional text (None if LTR)
    pub visual_order: Option<Vec<(usize, usize)>>,
    /// Whether this line contains bidirectional text
    pub is_bidi: bool,
}

/// Represents layout information for a single line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineLayoutInfo {
    /// Line index (0-based)
    pub line_number: usize,
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Line width
    pub width: f32,
    /// Break type as string
    pub break_type: String,
    /// Character count on line
    pub char_count: usize,
    /// Whether line contains bidirectional text
    pub is_bidi: bool,
    /// Trailing whitespace width
    pub trailing_whitespace: f32,
    /// Left offset for indentation
    pub offset_x: f32,
    /// Actual line height used
    pub line_height: f32,
}

/// Paragraph properties for layout customization
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ParagraphProperties {
    /// Left indent in twips (1/1440 of an inch)
    pub indent_left: f32,
    /// Right indent in twips
    pub indent_right: f32,
    /// First line indent in twips (can be negative for hanging indent)
    pub indent_first_line: f32,
    /// Space before paragraph in twips
    pub space_before: f32,
    /// Space after paragraph in twips
    pub space_after: f32,
    /// Line spacing value (interpretation depends on line_spacing_rule)
    pub line_spacing: f32,
    /// Line spacing rule
    pub line_spacing_rule: LineSpacingRule,
    /// Text alignment
    pub alignment: Alignment,
}

impl Default for ParagraphProperties {
    fn default() -> Self {
        ParagraphProperties {
            indent_left: 0.0,
            indent_right: 0.0,
            indent_first_line: 0.0,
            space_before: 0.0,
            space_after: 0.0,
            line_spacing: 1.0,
            line_spacing_rule: LineSpacingRule::Single,
            alignment: Alignment::default(),
        }
    }
}

/// Helper methods for ParagraphProperties
impl ParagraphProperties {
    /// Creates paragraph properties with left and right indent
    #[inline]
    pub fn with_indent(left: f32, right: f32, first_line: f32) -> Self {
        ParagraphProperties {
            indent_left: left,
            indent_right: right,
            indent_first_line: first_line,
            ..Default::default()
        }
    }

    /// Creates paragraph properties with alignment
    #[inline]
    pub fn with_alignment(alignment: Alignment) -> Self {
        ParagraphProperties {
            alignment,
            ..Default::default()
        }
    }

    /// Creates paragraph properties with line spacing
    #[inline]
    pub fn with_line_spacing(rule: LineSpacingRule, value: f32) -> Self {
        ParagraphProperties {
            line_spacing_rule: rule,
            line_spacing: value,
            ..Default::default()
        }
    }

    /// Creates paragraph properties with full customization
    #[inline]
    pub fn new(
        indent_left: f32,
        indent_right: f32,
        indent_first_line: f32,
        space_before: f32,
        space_after: f32,
        line_spacing: f32,
        line_spacing_rule: LineSpacingRule,
        alignment: Alignment,
    ) -> Self {
        ParagraphProperties {
            indent_left,
            indent_right,
            indent_first_line,
            space_before,
            space_after,
            line_spacing,
            line_spacing_rule,
            alignment,
        }
    }
}

/// Complete paragraph layout result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphLayout {
    /// Original text
    pub text: String,
    /// Maximum line width used
    pub max_width: f32,
    /// Available content width (max_width - left_indent - right_indent)
    pub content_width: f32,
    /// Individual line layouts
    pub lines: Vec<LineLayoutInfo>,
    /// Total height (lines * line_height + space_before + space_after)
    pub total_height: f32,
    /// Base line height (without spacing rules)
    pub base_line_height: f32,
    /// Actual line height used
    pub actual_line_height: f32,
    /// Whether text contains bidirectional content
    pub has_bidi: bool,
    /// Paragraph properties used
    pub properties: ParagraphProperties,
}

/// Complete document layout result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLayout {
    /// All paragraphs
    pub paragraphs: Vec<ParagraphLayout>,
    /// Total width
    pub total_width: f32,
    /// Total height
    pub total_height: f32,
    /// Line height
    pub line_height: f32,
}

/// Configuration for line layout
#[derive(Debug, Clone)]
pub struct LineLayoutConfig {
    /// Line height in abstract units
    pub line_height: f32,
    /// Tab size in spaces
    pub tab_size: usize,
    /// Default font size
    pub font_size: f32,
    /// Enable bidirectional support
    pub bidi_enabled: bool,
    /// Trim trailing whitespace
    pub trim_trailing: bool,
}

impl Default for LineLayoutConfig {
    fn default() -> Self {
        LineLayoutConfig {
            line_height: 1.2,
            tab_size: 4,
            font_size: 14.0,
            bidi_enabled: true,
            trim_trailing: true,
        }
    }
}

/// Main line layout struct
#[derive(Debug, Clone)]
pub struct LineLayout {
    config: LineLayoutConfig,
    breaker: LineBreaker,
}

impl Default for LineLayout {
    fn default() -> Self {
        LineLayout::new()
    }
}

impl LineLayout {
    /// Creates a new line layout with default configuration
    #[inline]
    pub fn new() -> Self {
        LineLayout {
            config: LineLayoutConfig::default(),
            breaker: LineBreaker::new(),
        }
    }

    /// Creates a new line layout with custom configuration
    #[inline]
    pub fn with_config(config: LineLayoutConfig) -> Self {
        LineLayout {
            config,
            breaker: LineBreaker::new(),
        }
    }

    /// Sets the line height
    #[inline]
    pub fn set_line_height(&mut self, height: f32) {
        self.config.line_height = height;
    }

    /// Sets the tab size
    #[inline]
    pub fn set_tab_size(&mut self, size: usize) {
        self.config.tab_size = size;
    }

    /// Enables or disables bidirectional support
    #[inline]
    pub fn set_bidi(&mut self, enabled: bool) {
        self.config.bidi_enabled = enabled;
    }

    /// Calculates the line height based on spacing rule
    fn calculate_line_height(&self, base_height: f32, props: ParagraphProperties) -> f32 {
        match props.line_spacing_rule {
            LineSpacingRule::Single => base_height * 1.0,
            LineSpacingRule::OneAndHalf => base_height * 1.5,
            LineSpacingRule::Double => base_height * 2.0,
            LineSpacingRule::Exactly => props.line_spacing,
            LineSpacingRule::AtLeast => base_height.max(props.line_spacing),
            LineSpacingRule::Multiple => base_height * props.line_spacing,
        }
    }

    /// Calculates the left offset for a line based on indentation
    fn calculate_line_offset(&self, line_index: usize, props: ParagraphProperties) -> f32 {
        let left_indent = props.indent_left;
        let first_line_indent = props.indent_first_line;

        if line_index == 0 {
            // First line: left + first_line indent
            left_indent + first_line_indent
        } else {
            // Subsequent lines: just left indent
            left_indent
        }
    }

    /// Layouts a single paragraph with default properties
    pub fn layout_paragraph(&mut self, text: &str, max_width: f32) -> ParagraphLayout {
        self.layout_paragraph_with_props(text, max_width, ParagraphProperties::default())
    }

    /// Layouts a single paragraph with custom properties
    pub fn layout_paragraph_with_props(
        &mut self,
        text: &str,
        max_width: f32,
        props: ParagraphProperties,
    ) -> ParagraphLayout {
        // Calculate content width (accounting for left and right indent)
        // Convert twips to abstract units (assuming 1440 twips per inch)
        let twips_to_units = max_width / 1440.0;
        let left_indent_units = props.indent_left * twips_to_units;
        let right_indent_units = props.indent_right * twips_to_units;
        let content_width = max_width - left_indent_units - right_indent_units;

        // Set breaker max width to content width
        self.breaker.set_max_width(content_width);

        let lines = self.breaker.break_lines(text, None);
        let mut layout_lines = Vec::new();

        let mut has_bidi = false;
        let mut char_offset = 0usize;

        // Calculate base line height
        let base_line_height = self.config.line_height * self.config.font_size;

        // Calculate actual line height based on spacing rule
        let actual_line_height = self.calculate_line_height(base_line_height, props);

        for (i, line) in lines.iter().enumerate() {
            if line.is_empty() {
                layout_lines.push(LineLayoutInfo {
                    line_number: i,
                    start: char_offset,
                    end: char_offset,
                    width: 0.0,
                    break_type: "HardBreak".to_string(),
                    char_count: 0,
                    is_bidi: false,
                    trailing_whitespace: 0.0,
                    offset_x: left_indent_units,
                    line_height: actual_line_height,
                });
                continue;
            }

            let line_text = &text[line.start..line.end];
            let char_count = line_text.chars().count();

            // Check for bidirectional text
            let is_bidi = if self.config.bidi_enabled {
                let has_rtl = line_text.chars().any(|c| {
                    matches!(
                        c,
                        '\u{0590}'..='\u{05FF}' |  // Hebrew
                        '\u{0600}'..='\u{06FF}' |  // Arabic
                        '\u{0750}'..='\u{077F}' |  // Arabic Supplement
                        '\u{08A0}'..='\u{08FF}' |  // Arabic Extended-A
                        '\u{FB50}'..='\u{FDFF}' |  // Arabic Presentation Forms-A
                        '\u{FE70}'..='\u{FEFF}' |  // Arabic Presentation Forms-B
                        '\u{10800}'..='\u{10FFF}'  // Private Use Area (some RTL scripts)
                    )
                });
                if has_rtl {
                    has_bidi = true;
                }
                has_rtl
            } else {
                false
            };

            // Calculate trailing whitespace
            let trailing_ws = if self.config.trim_trailing {
                let trimmed: String = line_text
                    .chars()
                    .rev()
                    .take_while(|c| c.is_whitespace())
                    .collect();
                self.breaker
                    .calculate_text_width(&trimmed.chars().rev().collect::<String>())
            } else {
                0.0
            };

            // Calculate line offset based on indentation
            let offset_x = self.calculate_line_offset(i, props);

            let break_type_str = match line.break_type {
                BreakType::HardBreak => "HardBreak",
                BreakType::SoftBreak => "SoftBreak",
                BreakType::Hyphenated => "Hyphenated",
            };

            layout_lines.push(LineLayoutInfo {
                line_number: i,
                start: line.start,
                end: line.end,
                width: line.width,
                break_type: break_type_str.to_string(),
                char_count,
                is_bidi,
                trailing_whitespace: trailing_ws,
                offset_x,
                line_height: actual_line_height,
            });

            char_offset = line.end;
        }

        // Calculate total height: lines * line_height + space_before + space_after
        let space_before_units = props.space_before * twips_to_units;
        let space_after_units = props.space_after * twips_to_units;
        let total_height =
            layout_lines.len() as f32 * actual_line_height + space_before_units + space_after_units;

        ParagraphLayout {
            text: text.to_string(),
            max_width,
            content_width,
            lines: layout_lines,
            total_height,
            base_line_height,
            actual_line_height,
            has_bidi,
            properties: props,
        }
    }

    /// Layouts a full document with multiple paragraphs
    pub fn layout_document(&mut self, text: &str, max_width: f32) -> DocumentLayout {
        self.layout_document_with_props(text, max_width, ParagraphProperties::default())
    }

    /// Layouts a full document with custom paragraph properties
    pub fn layout_document_with_props(
        &mut self,
        text: &str,
        max_width: f32,
        props: ParagraphProperties,
    ) -> DocumentLayout {
        let paragraphs: Vec<&str> = text.split('\n').collect();
        let mut all_paragraphs = Vec::new();
        let mut total_width = 0.0f32;
        let mut total_height = 0.0f32;

        for paragraph in paragraphs {
            let layout = self.layout_paragraph_with_props(paragraph, max_width, props);

            // Track maximum width
            for line in &layout.lines {
                if line.width + line.offset_x > total_width {
                    total_width = line.width + line.offset_x;
                }
            }

            total_height += layout.total_height;
            all_paragraphs.push(layout);
        }

        DocumentLayout {
            paragraphs: all_paragraphs,
            total_width,
            total_height,
            line_height: self.config.line_height * self.config.font_size,
        }
    }

    /// Layouts text and returns JSON string
    pub fn layout_to_json(&mut self, text: &str, max_width: f32) -> String {
        let layout = self.layout_document(text, max_width);
        serde_json::to_string(&layout).unwrap_or_else(|_| "{}".to_string())
    }

    /// Layouts text with properties and returns JSON string
    pub fn layout_to_json_with_props(
        &mut self,
        text: &str,
        max_width: f32,
        props: ParagraphProperties,
    ) -> String {
        let layout = self.layout_document_with_props(text, max_width, props);
        serde_json::to_string(&layout).unwrap_or_else(|_| "{}".to_string())
    }

    /// Calculates the visual order for a bidirectional line
    #[allow(dead_code)]
    pub fn calculate_visual_order(&self, text: &str) -> Vec<(usize, usize)> {
        if text.is_empty() {
            return Vec::new();
        }

        // Simple implementation - returns the text as-is for LTR
        // Full bidirectional reordering would require more complex handling
        vec![(0, text.len())]
    }

    /// Gets the line breaker for direct access
    #[inline]
    pub fn breaker_mut(&mut self) -> &mut LineBreaker {
        &mut self.breaker
    }

    /// Gets the line breaker for read-only access
    #[inline]
    pub fn breaker(&self) -> &LineBreaker {
        &self.breaker
    }
}

/// Utility functions for text measurement
pub mod measure {
    use super::*;

    /// Gets the number of lines needed for text at given width
    pub fn get_line_count(text: &str, max_width: f32) -> usize {
        let mut layout = LineLayout::new();
        layout
            .breaker_mut()
            .break_lines(text, Some(max_width))
            .len()
    }

    /// Gets the height needed for text at given width
    pub fn get_text_height(text: &str, max_width: f32, line_height: f32, font_size: f32) -> f32 {
        let line_count = get_line_count(text, max_width);
        line_count as f32 * line_height * font_size
    }

    /// Gets the total width of text
    pub fn get_text_total_width(text: &str) -> f32 {
        let mut layout = LineLayout::new();
        layout.breaker_mut().calculate_text_width(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_paragraph_layout() {
        let mut layout = LineLayout::new();
        let text = "This is a test paragraph for layout.";
        let result = layout.layout_paragraph(text, 1000.0);

        assert!(result.lines.len() >= 1);
        assert!(result.total_height > 0.0);
    }

    #[test]
    fn test_empty_paragraph() {
        let mut layout = LineLayout::new();
        let text = "";
        let result = layout.layout_paragraph(text, 1000.0);

        assert!(result.lines.is_empty() || result.lines.len() == 0);
    }

    #[test]
    fn test_multiline_paragraph() {
        let mut layout = LineLayout::new();
        let text = "This is a longer paragraph that should definitely require multiple lines to display properly within the given width constraint.";
        // Width 300px ~ 20-30 chars. Text is > 60 chars. Should wrap.
        let result = layout.layout_paragraph(text, 300.0);

        // Should have multiple lines
        assert!(result.lines.len() > 1);
    }

    #[test]
    fn test_paragraph_with_newlines() {
        let mut layout = LineLayout::new();
        let text = "First paragraph.\nSecond paragraph.\nThird paragraph.";
        let result = layout.layout_document(text, 1000.0);

        assert!(result.paragraphs.len() >= 3);
    }

    #[test]
    fn test_cjk_text_layout() {
        let mut layout = LineLayout::new();
        let text = "这是一个测试段落，用于测试中文分行功能是否正常工作。";
        // 500px ~ 30 chars. Text is 20+ chars.
        // Let's make it tight to force wrap?
        // 16px * 10 = 160px.
        let result = layout.layout_paragraph(text, 160.0);

        assert!(result.lines.len() >= 1);
        for line in &result.lines {
            // Allow override for single long words if any, but CJK breaks everywhere usually.
            assert!(line.width <= 160.0 + 50.0);
        }
    }

    #[test]
    fn test_line_height() {
        let mut layout = LineLayout::new();
        layout.set_line_height(1.5);

        let text = "Test text";
        let result = layout.layout_paragraph(text, 1000.0);

        // Total height should be proportional to line height
        assert!(result.total_height > 0.0);
    }

    #[test]
    fn test_json_output() {
        let mut layout = LineLayout::new();
        let text = "Hello world";
        let json = layout.layout_to_json(text, 1000.0);

        assert!(json.starts_with('{'));
        assert!(json.contains("paragraphs"));
    }

    #[test]
    fn test_visual_order() {
        let layout = LineLayout::new();
        // Simple LTR text
        let text = "Hello";
        let order = layout.calculate_visual_order(text);
        assert!(!order.is_empty());
    }

    #[test]
    fn test_line_layout_info() {
        let mut layout = LineLayout::new();
        let text = "Test line";
        let result = layout.layout_paragraph(text, 1000.0);

        if let Some(line) = result.lines.first() {
            assert_eq!(line.line_number, 0);
            assert!(line.width > 0.0);
            assert!(line.char_count > 0);
        }
    }

    #[test]
    fn test_trailing_whitespace() {
        let mut layout = LineLayout::new();
        let text = "Test   ";
        let result = layout.layout_paragraph(text, 1000.0);

        if let Some(line) = result.lines.first() {
            assert!(line.trailing_whitespace >= 0.0);
        }
    }

    // New tests for paragraph properties

    #[test]
    fn test_paragraph_properties_default() {
        let props = ParagraphProperties::default();
        assert_eq!(props.indent_left, 0.0);
        assert_eq!(props.indent_right, 0.0);
        assert_eq!(props.indent_first_line, 0.0);
        assert_eq!(props.alignment, Alignment::Left);
        assert_eq!(props.line_spacing_rule, LineSpacingRule::Single);
    }

    #[test]
    fn test_paragraph_with_indent() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_indent(360.0, 180.0, 360.0); // 0.25", 0.125", 0.25"
        let result = layout.layout_paragraph_with_props("Test paragraph", 1000.0, props);

        // First line should have left + first_line indent
        if let Some(first_line) = result.lines.first() {
            assert!(first_line.offset_x > 0.0);
        }
    }

    #[test]
    fn test_line_spacing_single() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::Single, 1.0);
        let result = layout.layout_paragraph_with_props("Test", 1000.0, props);

        assert_eq!(result.properties.line_spacing_rule, LineSpacingRule::Single);
        assert_eq!(result.actual_line_height, result.base_line_height);
    }

    #[test]
    fn test_line_spacing_double() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::Double, 2.0);
        let result = layout.layout_paragraph_with_props("Test\nLine2\nLine3", 1000.0, props);

        assert_eq!(result.properties.line_spacing_rule, LineSpacingRule::Double);
        assert_eq!(result.actual_line_height, result.base_line_height * 2.0);
    }

    #[test]
    fn test_line_spacing_exactly() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::Exactly, 30.0);
        let result = layout.layout_paragraph_with_props("Test", 1000.0, props);

        assert_eq!(
            result.properties.line_spacing_rule,
            LineSpacingRule::Exactly
        );
        assert_eq!(result.actual_line_height, 30.0);
    }

    #[test]
    fn test_line_spacing_multiple() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::Multiple, 1.5);
        let result = layout.layout_paragraph_with_props("Test", 1000.0, props);

        assert_eq!(
            result.properties.line_spacing_rule,
            LineSpacingRule::Multiple
        );
        assert_eq!(result.actual_line_height, result.base_line_height * 1.5);
    }

    #[test]
    fn test_alignment_left() {
        let props = ParagraphProperties::with_alignment(Alignment::Left);
        assert_eq!(props.alignment, Alignment::Left);
    }

    #[test]
    fn test_alignment_right() {
        let props = ParagraphProperties::with_alignment(Alignment::Right);
        assert_eq!(props.alignment, Alignment::Right);
    }

    #[test]
    fn test_alignment_center() {
        let props = ParagraphProperties::with_alignment(Alignment::Center);
        assert_eq!(props.alignment, Alignment::Center);
    }

    #[test]
    fn test_alignment_justify() {
        let props = ParagraphProperties::with_alignment(Alignment::Justify);
        assert_eq!(props.alignment, Alignment::Justify);
    }

    #[test]
    fn test_paragraph_properties_new() {
        let props = ParagraphProperties::new(
            360.0, // indent_left
            180.0, // indent_right
            360.0, // indent_first_line
            144.0, // space_before (0.1")
            144.0, // space_after (0.1")
            1.5,   // line_spacing
            LineSpacingRule::Multiple,
            Alignment::Center,
        );

        assert_eq!(props.indent_left, 360.0);
        assert_eq!(props.indent_right, 180.0);
        assert_eq!(props.indent_first_line, 360.0);
        assert_eq!(props.space_before, 144.0);
        assert_eq!(props.space_after, 144.0);
        assert_eq!(props.line_spacing, 1.5);
        assert_eq!(props.line_spacing_rule, LineSpacingRule::Multiple);
        assert_eq!(props.alignment, Alignment::Center);
    }

    #[test]
    fn test_content_width_with_indent() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_indent(720.0, 360.0, 0.0); // 0.5", 0.25"
        let result = layout.layout_paragraph_with_props("Test", 1440.0, props);

        // Content width should be max_width - left - right
        let expected_content = 1440.0 - 720.0 * (1440.0 / 1440.0) - 360.0 * (1440.0 / 1440.0);
        // Note: twips_to_units factor is applied, so with max_width=1440.0 and same value in twips,
        // the conversion factor is 1.0
        assert_eq!(result.content_width, expected_content);
    }

    #[test]
    fn test_space_before_after() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::new(
            0.0,
            0.0,
            0.0,
            720.0, // space_before: 0.5"
            720.0, // space_after: 0.5"
            1.0,
            LineSpacingRule::Single,
            Alignment::Left,
        );
        let result = layout.layout_paragraph_with_props("Test", 1000.0, props);

        // Total height should include space_before and space_after
        assert!(result.total_height > result.lines.len() as f32 * result.actual_line_height);
    }

    #[test]
    fn test_first_line_indent_only_on_first_line() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_indent(0.0, 0.0, 500.0); // First line only indent
        let result = layout.layout_paragraph_with_props("Line 1\nLine 2\nLine 3", 1000.0, props);

        if result.lines.len() >= 2 {
            // First line should have indent
            assert!(result.lines[0].offset_x > 0.0);
            // Second line should not have first_line indent
            assert_eq!(result.lines[1].offset_x, 0.0);
        }
    }

    #[test]
    fn test_hanging_indent() {
        let mut layout = LineLayout::new();
        // Negative first line indent = hanging indent
        let props = ParagraphProperties::with_indent(1000.0, 0.0, -500.0);
        let result = layout.layout_paragraph_with_props("Line 1\nLine 2", 2000.0, props);

        // First line should have less indent than subsequent lines
        if result.lines.len() >= 2 {
            assert!(result.lines[0].offset_x < result.lines[1].offset_x);
        }
    }

    #[test]
    fn test_line_spacing_at_least() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::AtLeast, 40.0);
        let result = layout.layout_paragraph_with_props("Test", 1000.0, props);

        assert_eq!(
            result.properties.line_spacing_rule,
            LineSpacingRule::AtLeast
        );
        assert!(result.actual_line_height >= 40.0);
    }

    #[test]
    fn test_line_spacing_at_least_larger_than_base() {
        let mut layout = LineLayout::new();
        layout.set_line_height(1.0);
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::AtLeast, 50.0);
        let result = layout.layout_paragraph_with_props("Test", 1000.0, props);

        // When base is smaller than AtLeast value, use AtLeast value
        assert!(result.actual_line_height >= 50.0);
    }

    #[test]
    fn test_multiline_with_spacing_rules() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::Double, 2.0);
        let result = layout.layout_paragraph_with_props(
            "First line\nSecond line\nThird line",
            1000.0,
            props,
        );

        assert_eq!(result.lines.len(), 3);
        assert_eq!(result.properties.line_spacing_rule, LineSpacingRule::Double);
        assert_eq!(result.actual_line_height, result.base_line_height * 2.0);
    }

    #[test]
    fn test_all_alignments() {
        let alignments = [
            Alignment::Left,
            Alignment::Right,
            Alignment::Center,
            Alignment::Justify,
        ];

        for &align in &alignments {
            let props = ParagraphProperties::with_alignment(align);
            assert_eq!(props.alignment, align);
        }
    }

    #[test]
    fn test_twips_conversion() {
        let mut layout = LineLayout::new();
        // 1440 twips = 1 inch = max_width when max_width=1440
        let props = ParagraphProperties::with_indent(1440.0, 720.0, 360.0);
        let result = layout.layout_paragraph_with_props("Test", 1440.0, props);

        // Conversion factor should be 1.0 when max_width equals twips base
        let expected_offset = 1440.0 + 360.0; // left + first_line
        assert!((result.lines[0].offset_x - expected_offset).abs() < 1.0);
    }

    #[test]
    fn test_empty_lines_preserved() {
        let mut layout = LineLayout::new();
        let text = "Line 1\n\nLine 3";
        let result =
            layout.layout_paragraph_with_props(text, 1000.0, ParagraphProperties::default());

        assert_eq!(result.lines.len(), 3);
        // Empty line should have zero width
        assert_eq!(result.lines[1].width, 0.0);
    }

    #[test]
    fn test_document_with_custom_props() {
        let mut layout = LineLayout::new();
        let props = ParagraphProperties::with_line_spacing(LineSpacingRule::Double, 2.0);
        let text = "Para 1\nPara 2\nPara 3";
        let result = layout.layout_document_with_props(text, 1000.0, props);

        assert_eq!(result.paragraphs.len(), 3);
        for para in &result.paragraphs {
            assert_eq!(para.properties.line_spacing_rule, LineSpacingRule::Double);
        }
    }
}

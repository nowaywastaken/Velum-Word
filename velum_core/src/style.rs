//! # Style Management System
//!
//! Provides support for character and paragraph styles including:
//! - Style definitions with inheritance
//! - Style parsing from OOXML
//! - Style application to text ranges

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

/// Character style information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct CharacterStyle {
    /// Unique style ID
    pub style_id: String,
    /// Style name (localized)
    pub name: String,
    /// Based on parent style
    pub based_on: Option<String>,
    /// Character formatting attributes
    pub formatting: super::piece_tree::TextAttributes,
    /// Whether this is a hidden style
    pub hidden: bool,
    /// Whether this style is locked
    pub locked: bool,
    /// Priority for style pane
    pub priority: u32,
    /// Quick style flag
    pub quick_style: bool,
}

impl CharacterStyle {
    /// Creates a new character style
    pub fn new(style_id: impl Into<String>) -> Self {
        CharacterStyle {
            style_id: style_id.into(),
            name: String::new(),
            based_on: None,
            formatting: super::piece_tree::TextAttributes::new(),
            hidden: false,
            locked: false,
            priority: 0,
            quick_style: false,
        }
    }

    /// Gets the effective formatting (including inherited from parent)
    pub fn get_effective_formatting<'a>(&'a self, style_map: &'a StyleMap) -> super::piece_tree::TextAttributes {
        let mut result = self.formatting.clone();

        // Inherit from parent style
        if let Some(ref parent_id) = self.based_on {
            if let Some(parent_style) = style_map.get_character_style(parent_id) {
                let parent_formatting = parent_style.get_effective_formatting(style_map);
                result = merge_formatting(&parent_formatting, &result);
            }
        }

        result
    }
}

/// Paragraph style information
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ParagraphStyle {
    /// Unique style ID
    pub style_id: String,
    /// Style name (localized)
    pub name: String,
    /// Based on parent style
    pub based_on: Option<String>,
    /// Linked character style (for character formatting)
    pub link: Option<String>,
    /// Paragraph formatting attributes
    pub formatting: super::piece_tree::ParagraphAttributes,
    /// Associated character style ID
    pub character_style_id: Option<String>,
    /// Whether this is a hidden style
    pub hidden: bool,
    /// Whether this style is locked
    pub locked: bool,
    /// Priority for style pane
    pub priority: u32,
    /// Quick style flag
    pub quick_style: bool,
    /// Style type (paragraph, character, linked, table, number)
    pub style_type: StyleType,
}

impl ParagraphStyle {
    /// Creates a new paragraph style
    pub fn new(style_id: impl Into<String>) -> Self {
        ParagraphStyle {
            style_id: style_id.into(),
            name: String::new(),
            based_on: None,
            link: None,
            formatting: super::piece_tree::ParagraphProperties::default(),
            character_style_id: None,
            hidden: false,
            locked: false,
            priority: 0,
            quick_style: false,
            style_type: StyleType::Paragraph,
        }
    }

    /// Gets the effective formatting (including inherited from parent)
    pub fn get_effective_formatting<'a>(
        &'a self,
        style_map: &'a StyleMap,
    ) -> super::piece_tree::ParagraphProperties {
        let mut result = self.formatting.clone();

        // Inherit from parent style
        if let Some(ref parent_id) = self.based_on {
            if let Some(parent_style) = style_map.get_paragraph_style(parent_id) {
                let parent_formatting = parent_style.get_effective_formatting(style_map);
                result = merge_paragraph_formatting(&parent_formatting, &result);
            }
        }

        result
    }
}

/// Style type enumeration
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum StyleType {
    Paragraph,
    Character,
    Linked,
    Table,
    Number,
    /// Custom/unknown style type
    Custom(String),
}

impl Default for StyleType {
    fn default() -> Self {
        StyleType::Paragraph
    }
}

/// Style map containing all document styles
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct StyleMap {
    /// Character styles by ID
    pub character_styles: HashMap<String, CharacterStyle>,
    /// Paragraph styles by ID
    pub paragraph_styles: HashMap<String, ParagraphStyle>,
    /// Default character style
    pub default_character_style: Option<String>,
    /// Default paragraph style
    pub default_paragraph_style: Option<String>,
    /// Document default font
    pub default_font: Option<DefaultFont>,
    /// Numbering styles
    pub numbering_styles: HashMap<String, NumberingStyle>,
}

impl StyleMap {
    /// Creates a new empty style map
    pub fn new() -> Self {
        StyleMap {
            character_styles: HashMap::new(),
            paragraph_styles: HashMap::new(),
            default_character_style: None,
            default_paragraph_style: None,
            default_font: None,
            numbering_styles: HashMap::new(),
        }
    }

    /// Adds a character style
    pub fn add_character_style(&mut self, style: CharacterStyle) {
        self.character_styles.insert(style.style_id.clone(), style);
    }

    /// Adds a paragraph style
    pub fn add_paragraph_style(&mut self, style: ParagraphStyle) {
        self.paragraph_styles.insert(style.style_id.clone(), style);
    }

    /// Gets a character style by ID
    pub fn get_character_style(&self, style_id: &str) -> Option<&CharacterStyle> {
        self.character_styles.get(style_id)
    }

    /// Gets a paragraph style by ID
    pub fn get_paragraph_style(&self, style_id: &str) -> Option<&ParagraphStyle> {
        self.paragraph_styles.get(style_id)
    }

    /// Sets the default character style
    pub fn set_default_character_style(&mut self, style_id: &str) {
        self.default_character_style = Some(style_id.to_string());
    }

    /// Sets the default paragraph style
    pub fn set_default_paragraph_style(&mut self, style_id: &str) {
        self.default_paragraph_style = Some(style_id.to_string());
    }

    /// Gets the default character formatting
    pub fn get_default_character_formatting(&self) -> super::piece_tree::TextAttributes {
        let mut result = super::piece_tree::TextAttributes::new();

        if let Some(ref style_id) = self.default_character_style {
            if let Some(style) = self.character_styles.get(style_id) {
                result = style.get_effective_formatting(self);
            }
        }

        // Apply default font if set
        if let Some(ref default_font) = self.default_font {
            result.font_family = Some(default_font.font_name.clone());
            result.font_size = Some(default_font.font_size);
        }

        result
    }

    /// Gets the default paragraph formatting
    pub fn get_default_paragraph_formatting(&self) -> super::piece_tree::ParagraphProperties {
        let mut result = super::piece_tree::ParagraphProperties::default();

        if let Some(ref style_id) = self.default_paragraph_style {
            if let Some(style) = self.paragraph_styles.get(style_id) {
                result = style.get_effective_formatting(self);
            }
        }

        result
    }

    /// Applies a character style to a text range
    pub fn apply_character_style(
        &self,
        style_id: &str,
    ) -> Option<super::piece_tree::TextAttributes> {
        self.get_character_style(style_id)
            .map(|style| style.get_effective_formatting(self))
    }

    /// Applies a paragraph style to a paragraph
    pub fn apply_paragraph_style(
        &self,
        style_id: &str,
    ) -> Option<super::piece_tree::ParagraphProperties> {
        self.get_paragraph_style(style_id)
            .map(|style| style.get_effective_formatting(self))
    }
}

/// Default font settings
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DefaultFont {
    pub font_name: String,
    pub font_size: u16,      // In half-points (12pt = 24)
    pub font_color: Option<String>,
    pub bold: bool,
    pub italic: bool,
}

/// Numbering style definition
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct NumberingStyle {
    pub num_id: String,
    pub indent_level: u32,
    pub format: NumberingFormat,
    pub start_value: u32,
    pub suffix: NumberingSuffix,
    pub alignment: super::piece_tree::Alignment,
    pub text_after: Option<String>,
    pub text_before: Option<String>,
}

/// Numbering format
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum NumberingFormat {
    Decimal,
    DecimalZero,
    UpperRoman,
    LowerRoman,
    UpperLetter,
    LowerLetter,
    Ordinal,
    NumberInDash,
    JapaneseCounting,
    ChineseCounting,
    ChineseLegalSimplified,
    KoreanDigital,
    KoreanDigital2,
    HebrewNonStandard,
    ArabicAlpha,
    HebrewNonStandard2,
    Custom(String),
}

impl Default for NumberingFormat {
    fn default() -> Self {
        NumberingFormat::Decimal
    }
}

/// Numbering suffix (text after the number)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum NumberingSuffix {
    Tab,
    Space,
    Nothing,
    Custom(String),
}

impl Default for NumberingSuffix {
    fn default() -> Self {
        NumberingSuffix::Space
    }
}

/// Merges two TextAttributes, with source overriding target
fn merge_formatting(
    base: &super::piece_tree::TextAttributes,
    override_: &super::piece_tree::TextAttributes,
) -> super::piece_tree::TextAttributes {
    let mut result = base.clone();

    // Override attributes from override_
    if let Some(val) = override_.bold { result.bold = Some(val); }
    if let Some(val) = override_.italic { result.italic = Some(val); }
    if let Some(val) = override_.underline { result.underline = Some(val); }
    if let Some(val) = override_.strikethrough { result.strikethrough = Some(val); }
    if let Some(val) = override_.double_underline { result.double_underline = Some(val); }
    if let Some(val) = override_.font_size { result.font_size = Some(val); }
    if let Some(val) = override_.font_family.clone() { result.font_family = Some(val); }
    if let Some(val) = override_.font_weight { result.font_weight = Some(val); }
    if let Some(val) = override_.font_stretch { result.font_stretch = Some(val); }
    if let Some(val) = override_.foreground.clone() { result.foreground = Some(val); }
    if let Some(val) = override_.background.clone() { result.background = Some(val); }
    if let Some(val) = override_.highlight.clone() { result.highlight = Some(val); }
    if let Some(val) = override_.superscript { result.superscript = Some(val); }
    if let Some(val) = override_.subscript { result.subscript = Some(val); }
    if let Some(val) = override_.small_caps { result.small_caps = Some(val); }
    if let Some(val) = override_.all_caps { result.all_caps = Some(val); }
    if let Some(val) = override_.shadow { result.shadow = Some(val); }
    if let Some(val) = override_.outline { result.outline = Some(val); }
    if let Some(val) = override_.emboss { result.emboss = Some(val); }
    if let Some(val) = override_.imprint { result.imprint = Some(val); }
    if let Some(val) = override_.character_spacing { result.character_spacing = Some(val); }
    if let Some(val) = override_.character_scale { result.character_scale = Some(val); }
    if let Some(val) = override_.kerning { result.kerning = Some(val); }
    if let Some(val) = override_.position_offset { result.position_offset = Some(val); }
    if let Some(val) = override_.baseline_offset { result.baseline_offset = Some(val); }
    if let Some(val) = override_.border.clone() { result.border = Some(val); }
    if let Some(val) = override_.ruby_text.clone() { result.ruby_text = Some(val); }
    if let Some(val) = override_.ruby_position { result.ruby_position = Some(val); }
    if let Some(val) = override_.language.clone() { result.language = Some(val); }
    if let Some(val) = override_.spelling_errors { result.spelling_errors = Some(val); }

    result
}

/// Merges two ParagraphProperties, with source overriding target
fn merge_paragraph_formatting(
    base: &super::piece_tree::ParagraphProperties,
    override_: &super::piece_tree::ParagraphProperties,
) -> super::piece_tree::ParagraphProperties {
    let mut result = base.clone();

    result.alignment = override_.alignment;
    result.left_indent = override_.left_indent;
    result.right_indent = override_.right_indent;
    result.first_line_indent = override_.first_line_indent;
    result.space_before = override_.space_before;
    result.space_after = override_.space_after;
    result.line_spacing = override_.line_spacing;
    result.line_spacing_rule = override_.line_spacing_rule;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_style_new() {
        let style = CharacterStyle::new("Heading1");
        assert_eq!(style.style_id, "Heading1");
        assert!(style.formatting.is_empty());
    }

    #[test]
    fn test_character_style_inheritance() {
        let mut styles = StyleMap::new();

        // Add base style
        styles.add_character_style({
            let mut style = CharacterStyle::new("Base");
            style.formatting.bold = Some(true);
            style
        });

        // Add child style
        styles.add_character_style({
            let mut style = CharacterStyle::new("Child");
            style.based_on = Some("Base".to_string());
            style.formatting.italic = Some(true);
            style
        });

        // Get child style with inheritance
        let child_style = styles.get_character_style("Child").unwrap();
        let effective = child_style.get_effective_formatting(&styles);

        assert_eq!(effective.bold, Some(true));
        assert_eq!(effective.italic, Some(true));
    }

    #[test]
    fn test_paragraph_style_new() {
        let style = ParagraphStyle::new("Normal");
        assert_eq!(style.style_id, "Normal");
        assert_eq!(style.style_type, StyleType::Paragraph);
    }

    #[test]
    fn test_style_map_defaults() {
        let mut styles = StyleMap::new();

        let default_font = DefaultFont {
            font_name: "Calibri".to_string(),
            font_size: 22, // 11pt
            font_color: None,
            bold: false,
            italic: false,
        };
        styles.default_font = Some(default_font);

        let default_formatting = styles.get_default_character_formatting();
        assert_eq!(default_formatting.font_family, Some("Calibri".to_string()));
        assert_eq!(default_formatting.font_size, Some(22));
    }
}

//! # Footnote and Endnote Module
//!
//! Implements footnote and endnote support including:
//! - Footnote and endnote insertion and display
//! - Multiple numbering formats (Arabic, Roman, Letter, Chinese, Star)
//! - Automatic numbering and renumbering
//! - Footnote area calculation
//! - Cross-references

use crate::drag_selection::DocumentPosition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Footnote identifier
pub type FootnoteId = u32;

/// Endnote identifier
pub type EndnoteId = u32;

/// Footnote/Endnote reference in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteReference {
    /// Unique identifier for this reference
    pub id: FootnoteId,
    /// Reference marker (e.g., "[1]")
    pub marker: String,
    /// Position in the document
    pub position: DocumentPosition,
    /// Whether this is a cross-reference to an existing footnote
    pub is_cross_reference: bool,
    /// ID of the referenced footnote/endnote (for cross-references)
    pub referenced_id: Option<FootnoteId>,
}

impl FootnoteReference {
    /// Creates a new footnote reference
    pub fn new(id: FootnoteId, marker: String, position: DocumentPosition) -> Self {
        FootnoteReference {
            id,
            marker,
            position,
            is_cross_reference: false,
            referenced_id: None,
        }
    }

    /// Creates a cross-reference to an existing footnote
    pub fn cross_reference(id: FootnoteId, marker: String, position: DocumentPosition, referenced_id: FootnoteId) -> Self {
        FootnoteReference {
            id,
            marker,
            position,
            is_cross_reference: true,
            referenced_id: Some(referenced_id),
        }
    }
}

/// Block-level content container for footnotes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockContainer {
    /// Paragraphs in this container
    pub paragraphs: Vec<ParagraphContent>,
}

/// Individual paragraph in footnote content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphContent {
    /// Text content
    pub text: String,
    /// Character offset in original text
    pub char_offset: usize,
    /// Length of text
    pub length: usize,
}

/// Footnote placement location
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FootnotePlacement {
    /// Footer at bottom of each page
    PageBottom,
    /// Beneath the text on each page
    BeneathText,
    /// End of section
    SectionEnd,
    /// End of document
    DocumentEnd,
}

/// Numbering format for footnotes/endnotes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NumberFormat {
    /// 1, 2, 3...
    Arabic,
    /// I, II, III...
    RomanUpper,
    /// i, ii, iii...
    RomanLower,
    /// A, B, C...
    LetterUpper,
    /// a, b, c...
    LetterLower,
    /// 一、二、三...
    Chinese,
    /// *, **, ***...
    Star,
}

impl NumberFormat {
    /// Formats a number according to this format
    pub fn format(&self, n: u32) -> String {
        match self {
            NumberFormat::Arabic => n.to_string(),
            NumberFormat::RomanUpper => to_roman_upper(n),
            NumberFormat::RomanLower => to_roman_lower(n),
            NumberFormat::LetterUpper => to_letter_upper(n),
            NumberFormat::LetterLower => to_letter_lower(n),
            NumberFormat::Chinese => to_chinese(n),
            NumberFormat::Star => to_stars(n),
        }
    }
}

/// Separator type for footnotes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SeparatorType {
    /// Short horizontal line (default)
    ShortLine,
    /// Long horizontal line
    LongLine,
    /// No separator
    None,
    /// Custom text separator
    Custom(String),
}

/// Footnote separator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteSeparator {
    /// Type of separator
    pub separator_type: SeparatorType,
    /// Length in points (for line separators)
    pub length: f32,
    /// Thickness in points
    pub thickness: f32,
    /// Color (optional, uses default if None)
    pub color: Option<String>,
    /// Spacing before separator in points
    pub spacing_before: f32,
    /// Spacing after separator in points
    pub spacing_after: f32,
}

impl Default for FootnoteSeparator {
    fn default() -> Self {
        FootnoteSeparator {
            separator_type: SeparatorType::ShortLine,
            length: 100.0,
            thickness: 0.75,
            color: None,
            spacing_before: 0.0,
            spacing_after: 6.0,
        }
    }
}

/// Footnote continuation settings
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContinuationType {
    /// Continue on next page with continuation separator
    Continue,
    /// Restart numbering on each page
    RestartEachPage,
    /// No continuation
    None,
}

/// Footnote continuation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteContinuation {
    /// Type of continuation
    pub continuation_type: ContinuationType,
    /// Continuation separator text
    pub continuation_separator: String,
    /// Continuation notice text
    pub continuation_notice: String,
}

impl Default for FootnoteContinuation {
    fn default() -> Self {
        FootnoteContinuation {
            continuation_type: ContinuationType::Continue,
            continuation_separator: "— Continued —".to_string(),
            continuation_notice: String::new(),
        }
    }
}

/// Complete footnote definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    /// Unique identifier
    pub id: FootnoteId,
    /// Reference information
    pub reference: FootnoteReference,
    /// Content of the footnote
    pub content: BlockContainer,
    /// Placement of the footnote
    pub placement: FootnotePlacement,
    /// Numbering format
    pub number_format: NumberFormat,
    /// Starting number for this footnote
    pub start_number: u32,
    /// Custom marker (if not using automatic numbering)
    pub custom_marker: Option<String>,
    /// Position on page (calculated during layout)
    pub page_position: Option<(f32, f32)>,
    /// Height required on page (calculated during layout)
    pub required_height: f32,
    /// Whether this footnote spans multiple pages
    pub is_continued: bool,
    /// Continuation reference (if this is a continuation)
    pub continuation_of: Option<FootnoteId>,
}

impl Footnote {
    /// Creates a new footnote with automatic numbering
    pub fn new(
        id: FootnoteId,
        marker: String,
        position: DocumentPosition,
        content: BlockContainer,
    ) -> Self {
        Footnote {
            id,
            reference: FootnoteReference::new(id, marker.clone(), position),
            content,
            placement: FootnotePlacement::PageBottom,
            number_format: NumberFormat::Arabic,
            start_number: 1,
            custom_marker: None,
            page_position: None,
            required_height: 0.0,
            is_continued: false,
            continuation_of: None,
        }
    }

    /// Creates a footnote with custom marker
    pub fn with_custom_marker(
        id: FootnoteId,
        marker: String,
        position: DocumentPosition,
        content: BlockContainer,
    ) -> Self {
        let mut footnote = Footnote::new(id, marker.clone(), position, content);
        footnote.custom_marker = Some(marker);
        footnote
    }

    /// Gets the display marker (custom or formatted number)
    pub fn get_display_marker(&self) -> String {
        self.custom_marker.clone().unwrap_or_else(|| {
            let number = self.start_number;
            self.number_format.format(number)
        })
    }
}

/// Complete endnote definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endnote {
    /// Unique identifier
    pub id: EndnoteId,
    /// Reference information
    pub reference: FootnoteReference,
    /// Content of the endnote
    pub content: BlockContainer,
    /// Placement of the endnote
    pub placement: FootnotePlacement,
    /// Numbering format
    pub number_format: NumberFormat,
    /// Starting number for this endnote
    pub start_number: u32,
    /// Custom marker (if not using automatic numbering)
    pub custom_marker: Option<String>,
    /// Position in document (calculated during layout)
    pub document_position: Option<(f32, f32)>,
    /// Section reference for section-end endnotes
    pub section_id: Option<u32>,
}

impl Endnote {
    /// Creates a new endnote with automatic numbering
    pub fn new(
        id: EndnoteId,
        marker: String,
        position: DocumentPosition,
        content: BlockContainer,
    ) -> Self {
        Endnote {
            id,
            reference: FootnoteReference::new(id, marker.clone(), position),
            content,
            placement: FootnotePlacement::DocumentEnd,
            number_format: NumberFormat::RomanLower,
            start_number: 1,
            custom_marker: None,
            document_position: None,
            section_id: None,
        }
    }

    /// Creates an endnote with custom marker
    pub fn with_custom_marker(
        id: EndnoteId,
        marker: String,
        position: DocumentPosition,
        content: BlockContainer,
    ) -> Self {
        let mut endnote = Endnote::new(id, marker.clone(), position, content);
        endnote.custom_marker = Some(marker);
        endnote
    }

    /// Gets the display marker (custom or formatted number)
    pub fn get_display_marker(&self) -> String {
        self.custom_marker.clone().unwrap_or_else(|| {
            let number = self.start_number;
            self.number_format.format(number)
        })
    }
}

/// Footnote/endnote area on a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteArea {
    /// Y position of the area
    pub y: f32,
    /// Height of the area
    pub height: f32,
    /// Footnotes in this area
    pub footnotes: Vec<FootnoteId>,
    /// Maximum height available
    pub available_height: f32,
    /// Whether footnotes continued from previous page
    pub has_continuation: bool,
    /// Continuation text
    pub continuation_text: String,
}

/// Footnote/endnote reference in document text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceMark {
    /// Type of reference
    pub reference_type: ReferenceType,
    /// Position in document
    pub position: DocumentPosition,
    /// Reference ID
    pub reference_id: FootnoteId,
    /// Display text
    pub display_text: String,
}

/// Type of reference
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Regular footnote
    Footnote,
    /// Endnote
    Endnote,
    /// Cross-reference to footnote
    FootnoteRef,
    /// Cross-reference to endnote
    EndnoteRef,
}

/// Configuration for footnote/endnote appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteConfig {
    /// Footnote numbering format
    pub footnote_number_format: NumberFormat,
    /// Endnote numbering format
    pub endnote_number_format: NumberFormat,
    /// Footnote placement
    pub footnote_placement: FootnotePlacement,
    /// Endnote placement
    pub endnote_placement: FootnotePlacement,
    /// Footnote starting number
    pub footnote_start_number: u32,
    /// Endnote starting number
    pub endnote_start_number: u32,
    /// Footnote separator
    pub footnote_separator: FootnoteSeparator,
    /// Endnote separator
    pub endnote_separator: FootnoteSeparator,
    /// Footnote continuation
    pub footnote_continuation: FootnoteContinuation,
    /// Endnote continuation
    pub endnote_continuation: FootnoteContinuation,
    /// Footnote text style
    pub footnote_text_style: FootnoteTextStyle,
    /// Footnote mark style
    pub footnote_mark_style: FootnoteMarkStyle,
}

/// Text style for footnote content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteTextStyle {
    /// Font size in points
    pub font_size: f32,
    /// Font family
    pub font_family: String,
    /// Text color
    pub text_color: Option<String>,
    /// Character spacing
    pub character_spacing: f32,
    /// Line spacing multiplier
    pub line_spacing: f32,
    /// Left indent in points
    pub indent_left: f32,
    /// Right indent in points
    pub indent_right: f32,
    /// Space before paragraph
    pub space_before: f32,
    /// Space after paragraph
    pub space_after: f32,
}

impl Default for FootnoteTextStyle {
    fn default() -> Self {
        FootnoteTextStyle {
            font_size: 9.0,
            font_family: "Times New Roman".to_string(),
            text_color: None,
            character_spacing: 0.0,
            line_spacing: 1.0,
            indent_left: 0.0,
            indent_right: 0.0,
            space_before: 0.0,
            space_after: 0.0,
        }
    }
}

/// Style for footnote mark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteMarkStyle {
    /// Superscript
    pub superscript: bool,
    /// Font size relative to main text (percentage)
    pub font_size_percent: f32,
    /// Offset from baseline in points
    pub baseline_offset: f32,
    /// Character spacing
    pub character_spacing: f32,
}

impl Default for FootnoteMarkStyle {
    fn default() -> Self {
        FootnoteMarkStyle {
            superscript: true,
            font_size_percent: 70.0,
            baseline_offset: 3.0,
            character_spacing: 0.0,
        }
    }
}

impl Default for FootnoteConfig {
    fn default() -> Self {
        FootnoteConfig {
            footnote_number_format: NumberFormat::Arabic,
            endnote_number_format: NumberFormat::RomanLower,
            footnote_placement: FootnotePlacement::PageBottom,
            endnote_placement: FootnotePlacement::DocumentEnd,
            footnote_start_number: 1,
            endnote_start_number: 1,
            footnote_separator: FootnoteSeparator::default(),
            endnote_separator: FootnoteSeparator::default(),
            footnote_continuation: FootnoteContinuation::default(),
            endnote_continuation: FootnoteContinuation::default(),
            footnote_text_style: FootnoteTextStyle::default(),
            footnote_mark_style: FootnoteMarkStyle::default(),
        }
    }
}

/// Footnote and Endnote Manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteManager {
    /// All footnotes
    footnotes: HashMap<FootnoteId, Footnote>,
    /// All endnotes
    endnotes: HashMap<EndnoteId, Endnote>,
    /// Footnote references in document order
    footnote_references: Vec<FootnoteReference>,
    /// Endnote references in document order
    endnote_references: Vec<FootnoteReference>,
    /// Current footnote ID counter
    current_footnote_id: FootnoteId,
    /// Current endnote ID counter
    current_endnote_id: EndnoteId,
    /// Configuration
    config: FootnoteConfig,
    /// Page height for area calculation
    page_height: f32,
    /// Footer height
    footer_height: f32,
    /// Bottom margin
    bottom_margin: f32,
}

impl FootnoteManager {
    /// Creates a new footnote manager
    pub fn new() -> Self {
        FootnoteManager {
            footnotes: HashMap::new(),
            endnotes: HashMap::new(),
            footnote_references: Vec::new(),
            endnote_references: Vec::new(),
            current_footnote_id: 1,
            current_endnote_id: 1,
            config: FootnoteConfig::default(),
            page_height: 841.89, // A4 default
            footer_height: 50.0,
            bottom_margin: 56.7,
        }
    }

    /// Creates a new footnote manager with config
    pub fn with_config(config: FootnoteConfig) -> Self {
        FootnoteManager {
            footnotes: HashMap::new(),
            endnotes: HashMap::new(),
            footnote_references: Vec::new(),
            endnote_references: Vec::new(),
            current_footnote_id: 1,
            current_endnote_id: 1,
            config,
            page_height: 841.89,
            footer_height: 50.0,
            bottom_margin: 56.7,
        }
    }

    /// Inserts a new footnote
    pub fn insert_footnote(&mut self, content: BlockContainer, position: DocumentPosition) -> FootnoteId {
        let id = self.current_footnote_id;
        self.current_footnote_id += 1;

        let number = self.calculate_footnote_number(id);
        let marker = self.config.footnote_number_format.format(number);

        let reference = FootnoteReference::new(id, marker.clone(), position);
        self.footnote_references.push(reference.clone());

        let mut footnote = Footnote::new(id, marker.clone(), position, content);
        footnote.number_format = self.config.footnote_number_format;
        footnote.start_number = self.config.footnote_start_number;
        footnote.placement = self.config.footnote_placement;

        self.footnotes.insert(id, footnote);
        self.renumber_footnotes();
        id
    }

    /// Inserts a footnote with custom marker
    pub fn insert_footnote_with_marker(
        &mut self,
        content: BlockContainer,
        position: DocumentPosition,
        custom_marker: String,
    ) -> FootnoteId {
        let id = self.current_footnote_id;
        self.current_footnote_id += 1;

        let reference = FootnoteReference::new(id, custom_marker.clone(), position);
        self.footnote_references.push(reference.clone());

        let mut footnote = Footnote::with_custom_marker(id, custom_marker, position, content);
        footnote.placement = self.config.footnote_placement;
        self.footnotes.insert(id, footnote);
        id
    }

    /// Inserts a new endnote
    pub fn insert_endnote(&mut self, content: BlockContainer, position: DocumentPosition) -> EndnoteId {
        let id = self.current_endnote_id;
        self.current_endnote_id += 1;

        let number = self.calculate_endnote_number(id);
        let marker = self.config.endnote_number_format.format(number);

        let reference = FootnoteReference::new(id, marker.clone(), position);
        self.endnote_references.push(reference.clone());

        let mut endnote = Endnote::new(id, marker.clone(), position, content);
        endnote.number_format = self.config.endnote_number_format;
        endnote.start_number = self.config.endnote_start_number;

        self.endnotes.insert(id, endnote);
        self.renumber_endnotes();
        id
    }

    /// Inserts a cross-reference to a footnote
    pub fn insert_footnote_cross_reference(
        &mut self,
        position: DocumentPosition,
        referenced_footnote_id: FootnoteId,
    ) -> FootnoteId {
        let id = self.current_footnote_id;
        self.current_footnote_id += 1;

        let marker = if let Some(footnote) = self.footnotes.get(&referenced_footnote_id) {
            format!("See footnote {}", footnote.get_display_marker())
        } else {
            "[Invalid Reference]".to_string()
        };

        let reference = FootnoteReference::cross_reference(id, marker.clone(), position, referenced_footnote_id);
        self.footnote_references.push(reference.clone());

        let empty_content = BlockContainer { paragraphs: Vec::new() };
        let mut footnote = Footnote::new(id, marker.clone(), position, empty_content);
        footnote.reference = reference;

        self.footnotes.insert(id, footnote);
        id
    }

    /// Inserts a cross-reference to an endnote
    pub fn insert_endnote_cross_reference(
        &mut self,
        position: DocumentPosition,
        referenced_endnote_id: EndnoteId,
    ) -> EndnoteId {
        let id = self.current_endnote_id;
        self.current_endnote_id += 1;

        let marker = if let Some(endnote) = self.endnotes.get(&referenced_endnote_id) {
            format!("See endnote {}", endnote.get_display_marker())
        } else {
            "[Invalid Reference]".to_string()
        };

        let reference = FootnoteReference::cross_reference(id, marker.clone(), position, referenced_endnote_id);
        self.endnote_references.push(reference.clone());

        let empty_content = BlockContainer { paragraphs: Vec::new() };
        let mut endnote = Endnote::new(id, marker.clone(), position, empty_content);
        endnote.reference = reference;

        self.endnotes.insert(id, endnote);
        id
    }

    /// Gets a footnote by ID
    pub fn get_footnote(&self, id: FootnoteId) -> Option<&Footnote> {
        self.footnotes.get(&id)
    }

    /// Gets a mutable footnote by ID
    pub fn get_footnote_mut(&mut self, id: FootnoteId) -> Option<&mut Footnote> {
        self.footnotes.get_mut(&id)
    }

    /// Gets an endnote by ID
    pub fn get_endnote(&self, id: EndnoteId) -> Option<&Endnote> {
        self.endnotes.get(&id)
    }

    /// Gets an endnote by ID (mutable)
    pub fn get_endnote_mut(&mut self, id: EndnoteId) -> Option<&mut Endnote> {
        self.endnotes.get_mut(&id)
    }

    /// Gets all footnotes
    pub fn get_footnotes(&self) -> &HashMap<FootnoteId, Footnote> {
        &self.footnotes
    }

    /// Gets all endnotes
    pub fn get_endnotes(&self) -> &HashMap<EndnoteId, Endnote> {
        &self.endnotes
    }

    /// Gets footnote references in document order
    pub fn get_footnote_references(&self) -> &[FootnoteReference] {
        &self.footnote_references
    }

    /// Gets endnote references in document order
    pub fn get_endnote_references(&self) -> &[FootnoteReference] {
        &self.endnote_references
    }

    /// Updates footnote content
    pub fn update_footnote_content(&mut self, id: FootnoteId, content: BlockContainer) -> bool {
        if let Some(footnote) = self.footnotes.get_mut(&id) {
            footnote.content = content;
            true
        } else {
            false
        }
    }

    /// Updates endnote content
    pub fn update_endnote_content(&mut self, id: EndnoteId, content: BlockContainer) -> bool {
        if let Some(endnote) = self.endnotes.get_mut(&id) {
            endnote.content = content;
            true
        } else {
            false
        }
    }

    /// Deletes a footnote
    pub fn delete_footnote(&mut self, id: FootnoteId) -> bool {
        self.footnote_references.retain(|r| r.id != id);
        self.footnotes.remove(&id).is_some()
    }

    /// Deletes an endnote
    pub fn delete_endnote(&mut self, id: EndnoteId) -> bool {
        self.endnote_references.retain(|r| r.id != id);
        self.endnotes.remove(&id).is_some()
    }

    /// Calculates the footnote number for a given ID (considering deleted footnotes)
    fn calculate_footnote_number(&self, id: FootnoteId) -> u32 {
        let mut count = self.config.footnote_start_number;
        for ref_id in &self.footnote_references {
            if ref_id.id == id {
                break;
            }
            if !ref_id.is_cross_reference {
                count += 1;
            }
        }
        count
    }

    /// Calculates the endnote number for a given ID (considering deleted endnotes)
    fn calculate_endnote_number(&self, id: EndnoteId) -> u32 {
        let mut count = self.config.endnote_start_number;
        for ref_id in &self.endnote_references {
            if ref_id.id == id {
                break;
            }
            if !ref_id.is_cross_reference {
                count += 1;
            }
        }
        count
    }

    /// Renumbers all footnotes based on their order in the document
    pub fn renumber_footnotes(&mut self) {
        let mut current_number = self.config.footnote_start_number;
        let format = self.config.footnote_number_format;

        // Sort references by position
        let mut sorted_references: Vec<_> = self.footnote_references.iter().collect();
        sorted_references.sort_by_key(|r| r.position.char_offset);

        for reference in sorted_references {
            if let Some(footnote) = self.footnotes.get_mut(&reference.id) {
                if !reference.is_cross_reference {
                    footnote.start_number = current_number;
                    footnote.reference.marker = format.format(current_number);
                    current_number += 1;
                }
            }
        }
    }

    /// Renumbers all endnotes based on their order in the document
    pub fn renumber_endnotes(&mut self) {
        let mut current_number = self.config.endnote_start_number;
        let format = self.config.endnote_number_format;

        // Sort references by position
        let mut sorted_references: Vec<_> = self.endnote_references.iter().collect();
        sorted_references.sort_by_key(|r| r.position.char_offset);

        for reference in sorted_references {
            if let Some(endnote) = self.endnotes.get_mut(&reference.id) {
                if !reference.is_cross_reference {
                    endnote.start_number = current_number;
                    endnote.reference.marker = format.format(current_number);
                    current_number += 1;
                }
            }
        }
    }

    /// Sets the numbering format for footnotes
    pub fn set_footnote_number_format(&mut self, format: NumberFormat) {
        self.config.footnote_number_format = format;
        self.renumber_footnotes();
    }

    /// Sets the numbering format for endnotes
    pub fn set_endnote_number_format(&mut self, format: NumberFormat) {
        self.config.endnote_number_format = format;
        self.renumber_endnotes();
    }

    /// Sets the starting number for footnotes
    pub fn set_footnote_start_number(&mut self, start: u32) {
        self.config.footnote_start_number = start;
        self.renumber_footnotes();
    }

    /// Sets the starting number for endnotes
    pub fn set_endnote_start_number(&mut self, start: u32) {
        self.config.endnote_start_number = start;
        self.renumber_endnotes();
    }

    /// Sets page dimensions for footnote area calculation
    pub fn set_page_dimensions(&mut self, page_height: f32, footer_height: f32, bottom_margin: f32) {
        self.page_height = page_height;
        self.footer_height = footer_height;
        self.bottom_margin = bottom_margin;
    }

    /// Calculates the footnote area for a page
    pub fn calculate_footnote_area(&self, footnotes_on_page: &[FootnoteId]) -> FootnoteArea {
        let available_height = self.page_height - self.footer_height - self.bottom_margin;
        let y = self.page_height - self.footer_height - available_height;

        let mut area = FootnoteArea {
            y,
            height: 0.0,
            footnotes: footnotes_on_page.to_vec(),
            available_height,
            has_continuation: false,
            continuation_text: String::new(),
        };

        // Calculate total height needed
        let text_style = &self.config.footnote_text_style;
        let line_height = text_style.font_size * text_style.line_spacing;

        for footnote_id in footnotes_on_page {
            if let Some(footnote) = self.footnotes.get(footnote_id) {
                let paragraph_count = footnote.content.paragraphs.len();
                let paragraph_height = paragraph_count as f32 * line_height
                    + text_style.space_before
                    + text_style.space_after;
                area.height += paragraph_height + 6.0; // Add spacing between footnotes
            }
        }

        // Check if footnotes need continuation
        if area.height > area.available_height {
            area.height = area.available_height;
            area.has_continuation = true;
            area.continuation_text = self.config.footnote_continuation.continuation_separator.clone();
        }

        area
    }

    /// Gets the configuration
    pub fn get_config(&self) -> &FootnoteConfig {
        &self.config
    }

    /// Gets mutable configuration
    pub fn get_config_mut(&mut self) -> &mut FootnoteConfig {
        &mut self.config
    }

    /// Sets the configuration
    pub fn set_config(&mut self, config: FootnoteConfig) {
        self.config = config;
    }

    /// Gets the total count of footnotes
    pub fn footnote_count(&self) -> usize {
        self.footnotes.len()
    }

    /// Gets the total count of endnotes
    pub fn endnote_count(&self) -> usize {
        self.endnotes.len()
    }

    /// Clears all footnotes
    pub fn clear_footnotes(&mut self) {
        self.footnotes.clear();
        self.footnote_references.clear();
        self.current_footnote_id = 1;
    }

    /// Clears all endnotes
    pub fn clear_endnotes(&mut self) {
        self.endnotes.clear();
        self.endnote_references.clear();
        self.current_endnote_id = 1;
    }

    /// Clears all footnotes and endnotes
    pub fn clear_all(&mut self) {
        self.clear_footnotes();
        self.clear_endnotes();
    }
}

impl Default for FootnoteManager {
    fn default() -> Self {
        FootnoteManager::new()
    }
}

// ============ Number Formatting Functions ============

/// Converts a number to uppercase Roman numerals
fn to_roman_upper(n: u32) -> String {
    let roman_numerals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut result = String::new();
    let mut remaining = n;

    for (value, numeral) in roman_numerals {
        while remaining >= value {
            result.push_str(numeral);
            remaining -= value;
        }
    }

    result
}

/// Converts a number to lowercase Roman numerals
fn to_roman_lower(n: u32) -> String {
    to_roman_upper(n).to_lowercase()
}

/// Converts a number to uppercase letters (A, B, C, ...)
fn to_letter_upper(n: u32) -> String {
    if n == 0 {
        return String::new();
    }
    // Handle numbers beyond 26 using double letters (AA, AB, ...)
    let mut result = String::new();
    let mut num = n - 1;

    loop {
        if num < 26 {
            result.insert(0, (b'A' + num as u8) as char);
            break;
        } else {
            result.insert(0, (b'A' + (num % 26) as u8) as char);
            num = num / 26 - 1;
        }
    }

    result
}

/// Converts a number to lowercase letters (a, b, c, ...)
fn to_letter_lower(n: u32) -> String {
    to_letter_upper(n).to_lowercase()
}

/// Converts a number to Chinese numerals (一、二、三...)
fn to_chinese(n: u32) -> String {
    let chinese_digits = [
        "零", "一", "二", "三", "四", "五", "六", "七", "八", "九",
    ];
    let units = ["", "十", "百", "千"];

    if n == 0 {
        return chinese_digits[0].to_string();
    }

    let mut result = String::new();
    let mut num = n;
    let mut has_nonzero = false;

    // Process digits from right to left
    let mut position = 0;
    while num > 0 || position == 0 {
        let digit = num % 10;
        num /= 10;

        if digit > 0 {
            let prefix = format!("{}{}", chinese_digits[digit as usize], units[position]);
            result = prefix + &result;
            has_nonzero = true;
        } else if has_nonzero && position % 4 != 0 {
            // Only add zero between non-zero digits, and not at unit boundaries
            result = "零".to_string() + &result;
        }

        position += 1;
        if position >= 4 && num == 0 {
            break;
        }
    }

    // Simplify consecutive zeros to single zero
    while result.contains("零零") {
        result = result.replace("零零", "零");
    }
    // Remove leading zero if present
    if result.starts_with("零") && result.len() > 1 {
        result = result[4..].to_string(); // Remove "零" and the following unit
    }
    // Remove trailing zero if present
    if result.ends_with("零") {
        result.pop();
    }

    result
}

/// Converts a number to asterisks (*, **, ***...)
fn to_stars(n: u32) -> String {
    // Cycle through 1-9 asterisks, wrapping around
    let star_count = ((n - 1) % 9) + 1;
    "*".repeat(star_count as usize)
}

// ============ OOXML Conversion Functions ============

/// Converts internal footnote to OOXML w:footnote format
pub fn to_ooxml_footnote(footnote: &Footnote) -> String {
    let marker = &footnote.reference.marker;
    let mut xml = format!(
        r#"<w:footnote w:id="{}">
    <w:p>
        <w:r>
            <w:footnoteRef/>
        </w:r>
        <w:r>
            <w:rPr>
                <w:rStyle w:val="FootnoteReference"/>
            </w:rPr>
            <w:t>{}</w:t>
        </w:r>
    </w:p>
"#,
        footnote.id, marker
    );

    // Add content paragraphs
    for para in &footnote.content.paragraphs {
        xml.push_str(&format!(
            r#"    <w:p>
        <w:r>
            <w:t>{}</w:t>
        </w:r>
    </w:p>
"#,
            escape_xml(&para.text)
        ));
    }

    xml.push_str("</w:footnote>");
    xml
}

/// Converts internal endnote to OOXML w:endnote format
pub fn to_ooxml_endnote(endnote: &Endnote) -> String {
    let marker = &endnote.reference.marker;
    let mut xml = format!(
        r#"<w:endnote w:id="{}">
    <w:p>
        <w:r>
            <w:endnoteRef/>
        </w:r>
        <w:r>
            <w:rPr>
                <w:rStyle w:val="EndnoteReference"/>
            </w:rPr>
            <w:t>{}</w:t>
        </w:r>
    </w:p>
"#,
        endnote.id, marker
    );

    // Add content paragraphs
    for para in &endnote.content.paragraphs {
        xml.push_str(&format!(
            r#"    <w:p>
        <w:r>
            <w:t>{}</w:t>
        </w:r>
    </w:p>
"#,
            escape_xml(&para.text)
        ));
    }

    xml.push_str("</w:endnote>");
    xml
}

/// Escapes special XML characters
fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            c => result.push(c),
        }
    }
    result
}

/// Parses OOXML footnote element
pub fn parse_ooxml_footnote(xml: &str) -> Option<Footnote> {
    // Simplified parser - extracts id and content
    let id = extract_attribute(xml, "w:id")?.parse::<FootnoteId>().ok()?;
    let marker = extract_marker_from_footnote(xml)?;

    let content = parse_footnote_content(xml)?;

    let position = DocumentPosition::default();

    Some(Footnote::new(id, marker, position, content))
}

/// Parses OOXML endnote element
pub fn parse_ooxml_endnote(xml: &str) -> Option<Endnote> {
    // Simplified parser - extracts id and content
    let id = extract_attribute(xml, "w:id")?.parse::<EndnoteId>().ok()?;
    let marker = extract_marker_from_endnote(xml)?;

    let content = parse_footnote_content(xml)?;

    let position = DocumentPosition::default();

    Some(Endnote::new(id, marker, position, content))
}

/// Extracts an attribute from XML
fn extract_attribute(xml: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{}="([^"]*)""#, attr);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(xml)?.get(1).map(|m| m.as_str().to_string())
}

/// Extracts marker from footnote XML
fn extract_marker_from_footnote(xml: &str) -> Option<String> {
    let re = regex::Regex::new(r#"<w:t>(.*?)</w:t>"#).ok()?;
    re.captures(xml)?.get(1).map(|m| m.as_str().to_string())
}

/// Extracts marker from endnote XML
fn extract_marker_from_endnote(xml: &str) -> Option<String> {
    let re = regex::Regex::new(r#"<w:t>(.*?)</w:t>"#).ok()?;
    re.captures(xml)?.get(1).map(|m| m.as_str().to_string())
}

/// Parses footnote content from XML
fn parse_footnote_content(xml: &str) -> Option<BlockContainer> {
    let re = regex::Regex::new(r#"<w:p[^>]*>(.*?)</w:p>"#).ok()?;
    let mut paragraphs = Vec::new();

    for cap in re.captures_iter(xml) {
        let para_text = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let text_re = regex::Regex::new(r#"<w:t>(.*?)</w:t>"#).ok()?;
        let text: String = text_re
            .find_iter(&para_text)
            .map(|m| m.as_str().to_string())
            .collect();
        let text_len = text.len();

        paragraphs.push(ParagraphContent {
            text,
            char_offset: 0,
            length: text_len,
        });
    }

    Some(BlockContainer { paragraphs })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test content
    fn test_content(text: &str) -> BlockContainer {
        BlockContainer {
            paragraphs: vec![ParagraphContent {
                text: text.to_string(),
                char_offset: 0,
                length: text.len(),
            }],
        }
    }

    #[test]
    fn test_number_format_arabic() {
        assert_eq!(NumberFormat::Arabic.format(1), "1");
        assert_eq!(NumberFormat::Arabic.format(100), "100");
    }

    #[test]
    fn test_number_format_roman_upper() {
        assert_eq!(NumberFormat::RomanUpper.format(1), "I");
        assert_eq!(NumberFormat::RomanUpper.format(4), "IV");
        assert_eq!(NumberFormat::RomanUpper.format(9), "IX");
        assert_eq!(NumberFormat::RomanUpper.format(100), "C");
        assert_eq!(NumberFormat::RomanUpper.format(3999), "MMMCMXCIX");
    }

    #[test]
    fn test_number_format_roman_lower() {
        assert_eq!(NumberFormat::RomanLower.format(1), "i");
        assert_eq!(NumberFormat::RomanLower.format(4), "iv");
        assert_eq!(NumberFormat::RomanLower.format(9), "ix");
    }

    #[test]
    fn test_number_format_letter_upper() {
        assert_eq!(NumberFormat::LetterUpper.format(1), "A");
        assert_eq!(NumberFormat::LetterUpper.format(26), "Z");
        assert_eq!(NumberFormat::LetterUpper.format(27), "AA");
        assert_eq!(NumberFormat::LetterUpper.format(52), "AZ");
        assert_eq!(NumberFormat::LetterUpper.format(53), "BA");
    }

    #[test]
    fn test_number_format_letter_lower() {
        assert_eq!(NumberFormat::LetterLower.format(1), "a");
        assert_eq!(NumberFormat::LetterLower.format(26), "z");
        assert_eq!(NumberFormat::LetterLower.format(27), "aa");
    }

    #[test]
    fn test_number_format_chinese() {
        assert_eq!(NumberFormat::Chinese.format(1), "一");
        assert_eq!(NumberFormat::Chinese.format(10), "十");
        assert_eq!(NumberFormat::Chinese.format(11), "十一");
        assert_eq!(NumberFormat::Chinese.format(20), "二十");
        assert_eq!(NumberFormat::Chinese.format(21), "二十一");
        assert_eq!(NumberFormat::Chinese.format(100), "一百");
        assert_eq!(NumberFormat::Chinese.format(123), "一百二十三");
    }

    #[test]
    fn test_number_format_star() {
        assert_eq!(NumberFormat::Star.format(1), "*");
        assert_eq!(NumberFormat::Star.format(3), "***");
        assert_eq!(NumberFormat::Star.format(9), "*********");
        assert_eq!(NumberFormat::Star.format(10), "*");
    }

    #[test]
    fn test_footnote_manager_new() {
        let manager = FootnoteManager::new();
        assert_eq!(manager.footnote_count(), 0);
        assert_eq!(manager.endnote_count(), 0);
    }

    #[test]
    fn test_insert_footnote() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);
        let content = test_content("This is a footnote.");

        let id = manager.insert_footnote(content, position);

        assert_eq!(id, 1);
        assert_eq!(manager.footnote_count(), 1);

        let footnote = manager.get_footnote(1).unwrap();
        assert_eq!(footnote.reference.marker, "1");
        assert!(footnote.content.paragraphs[0].text.contains("footnote"));
    }

    #[test]
    fn test_insert_footnote_with_marker() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);
        let content = test_content("Custom marker footnote.");

        let id = manager.insert_footnote_with_marker(content, position, "*".to_string());

        assert_eq!(id, 1);
        assert_eq!(manager.footnote_count(), 1);

        let footnote = manager.get_footnote(1).unwrap();
        assert_eq!(footnote.get_display_marker(), "*");
    }

    #[test]
    fn test_insert_endnote() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(200, 10, 15);
        let content = test_content("This is an endnote.");

        let id = manager.insert_endnote(content, position);

        assert_eq!(id, 1);
        assert_eq!(manager.endnote_count(), 1);

        let endnote = manager.get_endnote(1).unwrap();
        assert_eq!(endnote.reference.marker, "i");
    }

    #[test]
    fn test_renumber_footnotes() {
        let mut manager = FootnoteManager::new();
        let pos1 = DocumentPosition::new(100, 5, 10);
        let pos2 = DocumentPosition::new(200, 8, 12);
        let pos3 = DocumentPosition::new(150, 7, 11); // Inserted out of order

        manager.insert_footnote(test_content("First"), pos1);
        manager.insert_footnote(test_content("Third"), pos3);
        manager.insert_footnote(test_content("Second"), pos2);

        manager.renumber_footnotes();

        let refs = manager.get_footnote_references();
        assert_eq!(refs[0].marker, "1");
        assert_eq!(refs[1].marker, "2");
        assert_eq!(refs[2].marker, "3");
    }

    #[test]
    fn test_insert_footnote_cross_reference() {
        let mut manager = FootnoteManager::new();
        let pos1 = DocumentPosition::new(100, 5, 10);
        let pos2 = DocumentPosition::new(200, 10, 15);

        let footnote_id = manager.insert_footnote(test_content("Original footnote."), pos1);
        let cross_ref_id = manager.insert_footnote_cross_reference(pos2, footnote_id);

        assert_eq!(cross_ref_id, 2);
        let cross_ref = manager.get_footnote(2).unwrap();
        assert!(cross_ref.reference.is_cross_reference);
        assert_eq!(cross_ref.reference.referenced_id, Some(1));
    }

    #[test]
    fn test_delete_footnote() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);

        manager.insert_footnote(test_content("First"), position.clone());
        manager.insert_footnote(test_content("Second"), position.clone());
        assert_eq!(manager.footnote_count(), 2);

        assert!(manager.delete_footnote(1));
        assert_eq!(manager.footnote_count(), 1);
        assert!(manager.get_footnote(1).is_none());
    }

    #[test]
    fn test_update_footnote_content() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);

        let id = manager.insert_footnote(test_content("Original"), position);
        let new_content = test_content("Updated content.");

        assert!(manager.update_footnote_content(id, new_content.clone()));

        let footnote = manager.get_footnote(id).unwrap();
        assert!(footnote.content.paragraphs[0].text.contains("Updated"));
    }

    #[test]
    fn test_set_footnote_number_format() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);

        manager.insert_footnote(test_content("First"), position.clone());
        manager.insert_footnote(test_content("Second"), position.clone());

        manager.set_footnote_number_format(NumberFormat::RomanUpper);

        let refs = manager.get_footnote_references();
        assert_eq!(refs[0].marker, "I");
        assert_eq!(refs[1].marker, "II");
    }

    #[test]
    fn test_calculate_footnote_area() {
        let mut manager = FootnoteManager::new();
        manager.set_page_dimensions(841.89, 50.0, 56.7); // A4 dimensions

        let position = DocumentPosition::new(100, 5, 10);
        let footnote_id = manager.insert_footnote(test_content("Test footnote"), position);

        let area = manager.calculate_footnote_area(&[footnote_id]);
        assert!(area.footnotes.contains(&footnote_id));
        assert!(area.height > 0.0);
        assert!(area.available_height > 0.0);
    }

    #[test]
    fn test_clear_footnotes() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);

        manager.insert_footnote(test_content("First"), position.clone());
        manager.insert_footnote(test_content("Second"), position.clone());
        manager.insert_endnote(test_content("Endnote"), position.clone());

        assert_eq!(manager.footnote_count(), 2);
        assert_eq!(manager.endnote_count(), 1);

        manager.clear_footnotes();

        assert_eq!(manager.footnote_count(), 0);
        assert_eq!(manager.endnote_count(), 1);
    }

    #[test]
    fn test_clear_all() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);

        manager.insert_footnote(test_content("First"), position.clone());
        manager.insert_endnote(test_content("Endnote"), position.clone());

        manager.clear_all();

        assert_eq!(manager.footnote_count(), 0);
        assert_eq!(manager.endnote_count(), 0);
    }

    #[test]
    fn test_config_update() {
        let mut manager = FootnoteManager::new();
        let mut config = FootnoteConfig::default();
        config.footnote_number_format = NumberFormat::RomanLower;
        config.footnote_start_number = 5;
        config.footnote_placement = FootnotePlacement::BeneathText;

        manager.set_config(config);

        let new_config = manager.get_config();
        assert_eq!(new_config.footnote_number_format, NumberFormat::RomanLower);
        assert_eq!(new_config.footnote_start_number, 5);
        assert_eq!(new_config.footnote_placement, FootnotePlacement::BeneathText);
    }

    #[test]
    fn test_footnote_references_order() {
        let mut manager = FootnoteManager::new();

        // Insert footnotes at different positions
        manager.insert_footnote(test_content("Second"), DocumentPosition::new(200, 10, 5));
        manager.insert_footnote(test_content("First"), DocumentPosition::new(100, 5, 5));
        manager.insert_footnote(test_content("Third"), DocumentPosition::new(300, 15, 5));

        let refs = manager.get_footnote_references();
        assert_eq!(refs.len(), 3);

        // They should be in document order
        assert_eq!(refs[0].position.char_offset, 100);
        assert_eq!(refs[1].position.char_offset, 200);
        assert_eq!(refs[2].position.char_offset, 300);
    }

    #[test]
    fn test_ooxml_footnote_conversion() {
        let footnote = Footnote::new(
            1,
            "1".to_string(),
            DocumentPosition::new(100, 5, 10),
            test_content("Test footnote content"),
        );

        let xml = to_ooxml_footnote(&footnote);
        assert!(xml.contains(r#"w:id="1""#));
        assert!(xml.contains("w:footnote"));
        assert!(xml.contains("Test footnote content"));
    }

    #[test]
    fn test_ooxml_endnote_conversion() {
        let endnote = Endnote::new(
            1,
            "i".to_string(),
            DocumentPosition::new(100, 5, 10),
            test_content("Test endnote content"),
        );

        let xml = to_ooxml_endnote(&endnote);
        assert!(xml.contains(r#"w:id="1""#));
        assert!(xml.contains("w:endnote"));
        assert!(xml.contains("Test endnote content"));
    }

    #[test]
    fn test_footnote_with_multiple_paragraphs() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);

        let content = BlockContainer {
            paragraphs: vec![
                ParagraphContent {
                    text: "First paragraph".to_string(),
                    char_offset: 0,
                    length: 15,
                },
                ParagraphContent {
                    text: "Second paragraph".to_string(),
                    char_offset: 16,
                    length: 16,
                },
            ],
        };

        let id = manager.insert_footnote(content, position);
        let footnote = manager.get_footnote(id).unwrap();

        assert_eq!(footnote.content.paragraphs.len(), 2);
        assert_eq!(footnote.content.paragraphs[0].text, "First paragraph");
        assert_eq!(footnote.content.paragraphs[1].text, "Second paragraph");
    }

    #[test]
    fn test_footnote_placement_variants() {
        let placements = [
            FootnotePlacement::PageBottom,
            FootnotePlacement::BeneathText,
            FootnotePlacement::SectionEnd,
            FootnotePlacement::DocumentEnd,
        ];

        for placement in placements {
            let mut manager = FootnoteManager::new();
            let mut config = FootnoteConfig::default();
            config.footnote_placement = placement;
            manager.set_config(config);

            let position = DocumentPosition::new(100, 5, 10);
            let id = manager.insert_footnote(test_content("Test"), position);

            let footnote = manager.get_footnote(id).unwrap();
            assert_eq!(footnote.placement, placement);
        }
    }

    #[test]
    fn test_custom_number_format_sequence() {
        let mut manager = FootnoteManager::new();
        let position = DocumentPosition::new(100, 5, 10);

        // Insert several footnotes
        for i in 1..=5 {
            manager.insert_footnote(test_content(&format!("Footnote {}", i)), position.clone());
        }

        // Change to Roman numerals
        manager.set_footnote_number_format(NumberFormat::RomanUpper);

        let refs = manager.get_footnote_references();
        assert_eq!(refs[0].marker, "I");
        assert_eq!(refs[1].marker, "II");
        assert_eq!(refs[2].marker, "III");
        assert_eq!(refs[3].marker, "IV");
        assert_eq!(refs[4].marker, "V");
    }
}

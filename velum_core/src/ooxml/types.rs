use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content types defined in [Content_Types].xml
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    /// Main document body (word/document.xml)
    MainDocument,
    /// Document styles (word/styles.xml)
    Styles,
    /// Theme colors and fonts (word/theme/theme1.xml)
    Theme,
    /// Document settings (word/settings.xml)
    Settings,
    /// Core properties (docProps/core.xml)
    CoreProperties,
    /// App properties (docProps/app.xml)
    AppProperties,
    /// Web settings (word/webSettings.xml)
    WebSettings,
    /// Numbering definitions (word/numbering.xml)
    Numbering,
    /// Custom XML properties
    CustomXml,
    /// Thumbnail image
    Thumbnail,
    /// Relationships file
    Relationships,
    /// PNG image
    ImagePng,
    /// JPEG image
    ImageJpeg,
    /// GIF image
    ImageGif,
    /// BMP image
    ImageBmp,
    /// WebP image
    ImageWebP,
    /// TIFF image
    ImageTiff,
    /// SVG image
    ImageSvg,
    /// Unknown content type
    Unknown(String),
}

impl ContentType {
    /// Parse content type string into enum
    pub fn from_string(s: &str) -> Self {
        match s {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml" => ContentType::MainDocument,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml" => ContentType::Styles,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.theme+xml" => ContentType::Theme,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml" => ContentType::Settings,
            "application/vnd.openxmlformats-package.core-properties+xml" => ContentType::CoreProperties,
            "application/vnd.openxmlformats-officedocument.extended-properties+xml" => ContentType::AppProperties,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml" => ContentType::WebSettings,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml" => ContentType::Numbering,
            "application/xml" | "application/vnd.openxmlformats-officedocument.customXmlProperties+xml" => ContentType::CustomXml,
            "application/vnd.openxmlformats-package.relationships+xml" => ContentType::Relationships,
            // Image types
            "image/png" => ContentType::ImagePng,
            "image/jpeg" | "image/jpg" => ContentType::ImageJpeg,
            "image/gif" => ContentType::ImageGif,
            "image/bmp" => ContentType::ImageBmp,
            "image/webp" => ContentType::ImageWebP,
            "image/tiff" | "image/tif" => ContentType::ImageTiff,
            "image/svg+xml" => ContentType::ImageSvg,
            _ => ContentType::Unknown(s.to_string()),
        }
    }

    /// Check if this is an image content type
    pub fn is_image(&self) -> bool {
        matches!(self,
            ContentType::ImagePng |
            ContentType::ImageJpeg |
            ContentType::ImageGif |
            ContentType::ImageBmp |
            ContentType::ImageWebP |
            ContentType::ImageTiff |
            ContentType::ImageSvg
        )
    }

    /// Get the part name for this content type
    pub fn default_part_name(&self) -> Option<&'static str> {
        match self {
            ContentType::MainDocument => Some("/word/document.xml"),
            ContentType::Styles => Some("/word/styles.xml"),
            ContentType::Theme => Some("/word/theme/theme1.xml"),
            ContentType::Settings => Some("/word/settings.xml"),
            ContentType::CoreProperties => Some("/docProps/core.xml"),
            ContentType::AppProperties => Some("/docProps/app.xml"),
            ContentType::WebSettings => Some("/word/webSettings.xml"),
            ContentType::Numbering => Some("/word/numbering.xml"),
            _ => None,
        }
    }
}

/// Relationship type constants (ECMA-376)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Main document relationship
    Document,
    /// Styles relationship
    Styles,
    /// Theme relationship
    Theme,
    /// Settings relationship
    Settings,
    /// Core properties relationship
    CoreProperties,
    /// Custom XML relationship
    CustomXml,
    /// Thumbnail relationship
    Thumbnail,
    /// Office document relationship
    OfficeDocument,
    /// Image relationship
    Image,
    /// Unknown relationship type
    Unknown(String),
}

impl RelationshipType {
    /// Parse relationship type string into enum
    pub fn from_string(s: &str) -> Self {
        match s {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" => RelationshipType::OfficeDocument,
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/mainDocument" => RelationshipType::Document,
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" => RelationshipType::Styles,
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" => RelationshipType::Theme,
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" => RelationshipType::Settings,
            "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" => RelationshipType::CoreProperties,
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml" => RelationshipType::CustomXml,
            "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail" => RelationshipType::Thumbnail,
            // Image relationships
            rel if rel.contains("relationships/image") => RelationshipType::Image,
            _ => RelationshipType::Unknown(s.to_string()),
        }
    }

    /// Check if this is an image relationship type
    pub fn is_image(&self) -> bool {
        matches!(self, RelationshipType::Image)
    }
}

/// Represents a relationship between parts in the package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Relationship ID (e.g., "rId1")
    pub id: String,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Target URI (can be relative or absolute)
    pub target: String,
    /// Target mode (Internal or External)
    pub target_mode: Option<String>,
}

/// A part in the OPC package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePart {
    /// Part name (e.g., "/word/document.xml")
    pub name: String,
    /// Content type of the part
    pub content_type: ContentType,
    /// Raw binary data of the part
    pub data: Vec<u8>,
}

/// Represents a parsed paragraph in the document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Paragraph {
    /// Paragraph text content
    pub text: String,
    /// Paragraph properties (indentation, alignment, etc.)
    pub properties: ParagraphProperties,
    /// List of runs in this paragraph
    pub runs: Vec<Run>,
}

/// Properties of a paragraph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParagraphProperties {
    /// Paragraph alignment
    pub alignment: Option<String>,
    /// Left indentation in twips (1/20 of a point)
    pub indent_left: Option<i32>,
    /// Right indentation in twips
    pub indent_right: Option<i32>,
    /// First line indentation in twips
    pub indent_first_line: Option<i32>,
    /// Space before paragraph in twips
    pub spacing_before: Option<i32>,
    /// Space after paragraph in twips
    pub spacing_after: Option<i32>,
    /// Line spacing
    pub spacing_line: Option<i32>,
}

/// Represents a run of text with common formatting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Run {
    /// Text content of the run
    pub text: String,
    /// Run properties
    pub properties: RunProperties,
}

/// Properties of a run (text formatting)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunProperties {
    /// Bold formatting
    pub bold: Option<bool>,
    /// Italic formatting
    pub italic: Option<bool>,
    /// Underline type
    pub underline: Option<String>,
    /// Font size in half-points
    pub font_size: Option<i32>,
    /// Font name
    pub font_name: Option<String>,
    /// Text color (hex RGB)
    pub color: Option<String>,
    /// Background color (hex RGB)
    pub background_color: Option<String>,
}

/// Represents a style definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Style {
    /// Style ID (e.g., "Normal", "Heading1")
    pub id: String,
    /// Style name (e.g., "Normal", "Heading 1")
    pub name: Option<String>,
    /// Style type (paragraph, character, table, number)
    pub style_type: String,
    /// Style ID of the parent style
    pub based_on: Option<String>,
    /// Paragraph properties
    pub paragraph_properties: ParagraphProperties,
    /// Run properties
    pub run_properties: RunProperties,
    /// Whether this is the default style
    pub is_default: bool,
}

/// Theme colors and fonts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Color scheme
    pub colors: HashMap<String, String>,
    /// Font scheme
    pub fonts: ThemeFonts,
}

/// Theme font definitions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeFonts {
    /// Major font (headings)
    pub major_font: String,
    /// Minor font (body text)
    pub minor_font: String,
    /// Symbol font
    pub symbol_font: String,
}

/// Represents an embedded or linked image in the document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentImage {
    /// Unique image ID (e.g., "rId5")
    pub id: String,
    /// Path to image in the package (e.g., "media/image1.png")
    pub path: String,
    /// Original width in EMUs (English Metric Units)
    pub original_width: Option<u32>,
    /// Original height in EMUs
    pub original_height: Option<u32>,
    /// Desired width in EMUs (may differ from original for scaling)
    pub desired_width: Option<u32>,
    /// Desired height in EMUs
    pub desired_height: Option<u32>,
    /// Horizontal scaling percentage (100 = original size)
    pub scale_x: Option<f32>,
    /// Vertical scaling percentage (100 = original size)
    pub scale_y: Option<f32>,
    /// Title of the image
    pub title: Option<String>,
    /// Alt text description
    pub alt_description: Option<String>,
    /// Whether the image is linked rather than embedded
    pub is_linked: bool,
}

/// Blip fill properties for images
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlipFill {
    /// Reference to the image
    pub image_reference: String,
    /// Tile mode for texture fill
    pub tile_mode: Option<String>,
    /// Stretch mode for bitmap fill
    pub stretch_mode: Option<String>,
    /// Source rectangle for partial image
    pub source_rect: Option<SourceRect>,
}

/// Source rectangle for partial image
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceRect {
    /// Top offset
    pub top: f32,
    /// Left offset
    pub left: f32,
    /// Bottom offset
    pub bottom: f32,
    /// Right offset
    pub right: f32,
}

/// Document anchor for positioning floating elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAnchor {
    /// Anchor type (page, paragraph, or character)
    pub anchor_type: String,
    /// Page number for page anchoring (1-based)
    pub page_number: Option<usize>,
    /// Paragraph ID for paragraph anchoring
    pub paragraph_id: Option<String>,
    /// Character position for character anchoring
    pub character_position: Option<usize>,
    /// Horizontal position
    pub horizontal: Option<AnchorPositionSpec>,
    /// Vertical position
    pub vertical: Option<AnchorPositionSpec>,
    /// Allow overlap with text
    pub allow_overlap: bool,
}

/// Anchor position specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorPositionSpec {
    /// Position type (absolute, relative, left, center, right, inside, outside)
    pub position_type: String,
    /// Offset value
    pub offset: f32,
    /// Alignment type
    pub alignment: Option<String>,
}

// ============================================
// Table types
// ============================================

/// Represents a table in the document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Table {
    /// Table rows
    pub rows: Vec<TableRow>,
    /// Table properties (width, alignment, borders, etc.)
    pub properties: TableProperties,
}

/// Table row in a table
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableRow {
    /// Cells in this row
    pub cells: Vec<TableCell>,
    /// Row height
    pub height: Option<u32>,
    /// Row properties
    pub properties: TableRowProperties,
}

/// Table cell within a row
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableCell {
    /// Paragraphs in this cell
    pub paragraphs: Vec<Paragraph>,
    /// Cell width in twips
    pub width: Option<u32>,
    /// Vertical merge information
    pub vertical_merge: Option<i32>,
    /// Horizontal merge information
    pub horizontal_merge: Option<i32>,
    /// Cell properties
    pub properties: TableCellProperties,
}

/// Table properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableProperties {
    /// Table width
    pub width: Option<u32>,
    /// Table alignment (left, center, right, etc.)
    pub alignment: Option<String>,
    /// Table borders
    pub borders: TableBorders,
    /// Table indentation
    pub indent: Option<i32>,
    /// Table layout type (fixed, auto)
    pub layout: Option<String>,
}

/// Table row properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableRowProperties {
    /// Row height
    pub height: Option<u32>,
    /// Row height rule (auto, exact, atLeast)
    pub height_rule: Option<String>,
    /// Table row header (repeat on each page)
    pub is_header: bool,
}

/// Table cell properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableCellProperties {
    /// Cell width
    pub width: Option<u32>,
    /// Vertical alignment (top, center, bottom)
    pub vertical_alignment: Option<String>,
    /// Text direction
    pub text_direction: Option<String>,
    /// Shading/background color
    pub shading_color: Option<String>,
}

/// Table borders
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableBorders {
    /// Top border
    pub top: Option<TableBorder>,
    /// Bottom border
    pub bottom: Option<TableBorder>,
    /// Left border
    pub left: Option<TableBorder>,
    /// Right border
    pub right: Option<TableBorder>,
    /// Inside horizontal borders
    pub inside_horizontal: Option<TableBorder>,
    /// Inside vertical borders
    pub inside_vertical: Option<TableBorder>,
}

/// Individual table border
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableBorder {
    /// Border style (single, double, etc.)
    pub style: Option<String>,
    /// Border size in eighths of a point
    pub size: Option<u32>,
    /// Border color (hex RGB)
    pub color: Option<String>,
}

// ============================================
// Header/Footer types
// ============================================

/// Represents a header in the document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Header {
    /// Header ID (e.g., "rId1")
    pub id: String,
    /// Header type (default, first, even, odd)
    pub header_type: String,
    /// Paragraphs in header
    pub paragraphs: Vec<Paragraph>,
    /// Images in header
    pub images: Vec<DocumentImage>,
}

/// Represents a footer in the document
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Footer {
    /// Footer ID (e.g., "rId2")
    pub id: String,
    /// Footer type (default, first, even, odd)
    pub footer_type: String,
    /// Paragraphs in footer
    pub paragraphs: Vec<Paragraph>,
    /// Images in footer
    pub images: Vec<DocumentImage>,
}

// ============================================
// Footnote/Endnote types
// ============================================

/// Represents a footnote
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Footnote {
    /// Footnote ID
    pub id: String,
    /// Footnote type (separator, continuationSeparator, etc.)
    pub footnote_type: Option<String>,
    /// Paragraphs in footnote
    pub paragraphs: Vec<Paragraph>,
}

/// Represents an endnote
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Endnote {
    /// Endnote ID
    pub id: String,
    /// Endnote type (separator, continuationSeparator, etc.)
    pub endnote_type: Option<String>,
    /// Paragraphs in endnote
    pub paragraphs: Vec<Paragraph>,
}

// ============================================
// Numbering types
// ============================================

/// Numbering definition (list styles)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Numbering {
    /// Abstract numbering definitions
    pub abstract_num_defs: Vec<AbstractNumDef>,
    /// Numbering instances
    pub num_instances: Vec<NumInstance>,
}

/// Abstract numbering definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbstractNumDef {
    /// Abstract numbering ID
    pub abstract_num_id: String,
    /// List level definitions
    pub levels: Vec<ListLevel>,
}

/// A list level within an abstract numbering
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListLevel {
    /// Level index (0-8)
    pub level: u32,
    /// Format (bullet, decimal, lowerLetter, etc.)
    pub format: String,
    /// Text pattern for level (e.g., "%1.")
    pub text: String,
    /// Starting value
    pub start_value: u32,
    /// Paragraph properties for this level
    pub paragraph_properties: ParagraphProperties,
    /// Run properties for this level
    pub run_properties: RunProperties,
}

/// Numbering instance (actual list using an abstract definition)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NumInstance {
    /// Numbering instance ID
    pub num_id: String,
    /// Reference to abstract numbering
    pub abstract_num_id: String,
    /// Override for level overrides
    pub overrides: Vec<LevelOverride>,
}

/// Level override for a specific list level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LevelOverride {
    /// Level index
    pub level: u32,
    /// Override start value
    pub start_value: Option<u32>,
    /// Override text
    pub text: Option<String>,
}

// ============================================
// Content Control (SDT) types
// ============================================

/// Content control (Structured Document Tag)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentControl {
    /// Tag name
    pub tag: Option<String>,
    /// Alias
    pub alias: Option<String>,
    /// Content control type
    pub sdt_type: String,
    /// Properties
    pub properties: ContentControlProperties,
    /// Content (paragraphs)
    pub content: Vec<Paragraph>,
}

/// Content control properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentControlProperties {
    /// Placeholder text
    pub placeholder_text: Option<String>,
    /// Data binding
    pub data_binding: Option<String>,
    /// Color
    pub color: Option<String>,
    /// ID
    pub id: Option<String>,
    /// Is temporary
    pub is_temporary: bool,
}

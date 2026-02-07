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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let rel = Relationship {
            id: "rId1".to_string(),
            relationship_type: RelationshipType::Document,
            target: "word/document.xml".to_string(),
            target_mode: Some("Internal".to_string()),
        };
        assert_eq!(rel.id, "rId1");
        assert_eq!(rel.target, "word/document.xml");
    }

    #[test]
    fn test_package_part_creation() {
        let part = PackagePart {
            name: "/word/document.xml".to_string(),
            content_type: ContentType::MainDocument,
            data: b"<test>data</test>".to_vec(),
        };
        assert_eq!(part.name, "/word/document.xml");
        assert_eq!(part.data.len(), 17);
    }

    #[test]
    fn test_paragraph_default() {
        let para = Paragraph::default();
        assert!(para.text.is_empty());
        assert!(para.runs.is_empty());
    }

    #[test]
    fn test_paragraph_with_runs() {
        let mut para = Paragraph::default();
        para.text = "Hello World".to_string();
        para.runs.push(Run {
            text: "Hello".to_string(),
            properties: RunProperties::default(),
        });
        assert_eq!(para.runs.len(), 1);
    }

    #[test]
    fn test_run_properties_default() {
        let props = RunProperties::default();
        assert!(props.bold.is_none());
        assert!(props.italic.is_none());
    }

    #[test]
    fn test_run_properties_with_values() {
        let mut props = RunProperties::default();
        props.bold = Some(true);
        props.italic = Some(true);
        props.font_size = Some(24);
        props.font_name = Some("Arial".to_string());
        props.color = Some("#FF0000".to_string());

        assert_eq!(props.bold, Some(true));
        assert_eq!(props.italic, Some(true));
        assert_eq!(props.font_size, Some(24));
    }

    #[test]
    fn test_style_default() {
        let style = Style::default();
        assert!(style.id.is_empty());
        assert!(!style.is_default); // is_default is bool, default is false
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert!(theme.name.is_empty());
        assert!(theme.colors.is_empty());
    }

    #[test]
    fn test_theme_with_values() {
        let mut theme = Theme::default();
        theme.name = "Office Theme".to_string();
        theme.colors.insert("dk1".to_string(), "#000000".to_string());
        theme.colors.insert("lt1".to_string(), "#FFFFFF".to_string());

        assert_eq!(theme.colors.len(), 2);
    }

    #[test]
    fn test_table_creation() {
        let table = Table::default();
        assert!(table.rows.is_empty());
    }

    #[test]
    fn test_table_with_rows() {
        let mut table = Table::default();
        table.rows.push(TableRow::default());
        table.rows.push(TableRow::default());
        assert_eq!(table.rows.len(), 2);
    }

    #[test]
    fn test_table_cell_default() {
        let cell = TableCell::default();
        assert!(cell.paragraphs.is_empty());
        assert!(cell.properties.shading_color.is_none());
    }

    #[test]
    fn test_table_borders_default() {
        let borders = TableBorders::default();
        assert!(borders.top.is_none());
        assert!(borders.bottom.is_none());
    }

    #[test]
    fn test_table_border_creation() {
        let border = TableBorder {
            style: Some("single".to_string()),
            size: Some(4),
            color: Some("#000000".to_string()),
        };
        assert_eq!(border.style, Some("single".to_string()));
    }

    #[test]
    fn test_header_footer_default() {
        let header = Header::default();
        assert!(header.paragraphs.is_empty());

        let footer = Footer::default();
        assert!(footer.paragraphs.is_empty());
    }

    #[test]
    fn test_footnote_endnote_default() {
        let footnote = Footnote::default();
        assert!(footnote.paragraphs.is_empty());
        assert!(footnote.id.is_empty());

        let endnote = Endnote::default();
        assert!(endnote.paragraphs.is_empty());
    }

    #[test]
    fn test_numbering_default() {
        let num = Numbering::default();
        assert!(num.abstract_num_defs.is_empty());
        assert!(num.num_instances.is_empty());
    }

    #[test]
    fn test_abstract_num_def() {
        let def = AbstractNumDef {
            abstract_num_id: "0".to_string(),
            levels: vec![],
        };
        assert_eq!(def.abstract_num_id, "0");
    }

    #[test]
    fn test_list_level() {
        let level = ListLevel {
            level: 0,
            format: "bullet".to_string(),
            text: "·".to_string(),
            start_value: 1,
            paragraph_properties: ParagraphProperties::default(),
            run_properties: RunProperties::default(),
        };
        assert_eq!(level.level, 0);
        assert_eq!(level.format, "bullet");
    }

    #[test]
    fn test_num_instance() {
        let instance = NumInstance {
            num_id: "1".to_string(),
            abstract_num_id: "0".to_string(),
            overrides: vec![],
        };
        assert_eq!(instance.num_id, "1");
    }

    #[test]
    fn test_level_override() {
        let override_ = LevelOverride {
            level: 0,
            start_value: Some(5),
            text: Some("5.".to_string()),
        };
        assert_eq!(override_.level, 0);
        assert_eq!(override_.start_value, Some(5));
    }

    #[test]
    fn test_content_control_default() {
        let cc = ContentControl::default();
        assert!(cc.tag.is_none());
        assert!(cc.content.is_empty());
    }

    #[test]
    fn test_content_control_properties_default() {
        let props = ContentControlProperties::default();
        assert!(props.placeholder_text.is_none());
        assert!(!props.is_temporary);
    }

    #[test]
    fn test_document_image_default() {
        let img = DocumentImage::default();
        assert!(img.id.is_empty());
        assert!(img.path.is_empty());
    }

    #[test]
    fn test_blip_fill_default() {
        let blip = BlipFill::default();
        assert!(blip.image_reference.is_empty());
        assert!(blip.tile_mode.is_none());
        assert!(blip.stretch_mode.is_none());
        assert!(blip.source_rect.is_none());
    }

    #[test]
    fn test_source_rect_default() {
        let rect = SourceRect::default();
        assert_eq!(rect.top, 0.0);
        assert_eq!(rect.left, 0.0);
        assert_eq!(rect.bottom, 0.0);
        assert_eq!(rect.right, 0.0);
    }

    #[test]
    fn test_document_anchor_default() {
        let anchor = DocumentAnchor {
            anchor_type: "page".to_string(),
            page_number: None,
            paragraph_id: None,
            character_position: None,
            horizontal: None,
            vertical: None,
            allow_overlap: false,
        };
        assert_eq!(anchor.anchor_type, "page");
        assert!(!anchor.allow_overlap);
    }

    #[test]
    fn test_paragraph_properties_default() {
        let props = ParagraphProperties::default();
        assert!(props.alignment.is_none());
        assert!(props.indent_left.is_none());
    }

    #[test]
    fn test_theme_fonts_default() {
        let fonts = ThemeFonts::default();
        assert!(fonts.major_font.is_empty());
        assert!(fonts.minor_font.is_empty());
    }

    #[test]
    fn test_theme_fonts_with_values() {
        let mut fonts = ThemeFonts::default();
        fonts.major_font = "Calibri".to_string();
        fonts.minor_font = "宋体".to_string();
        fonts.symbol_font = "Symbol".to_string();

        assert_eq!(fonts.major_font, "Calibri");
    }
}

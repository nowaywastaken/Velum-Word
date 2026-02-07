//! OOXML (Office Open XML) parsing module for Word documents (.docx)
//!
//! This module implements the OPC (Open Packaging Conventions) standard for reading
//! and parsing Microsoft Word documents. It provides functionality to extract text,
//! styles, and metadata from .docx files.
//!
//! # Example
//!
//! ```rust
//! use velum_core::ooxml::{parse_ooxml, ParsedDocument};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let file_data = std::fs::read("document.docx")?;
//!     let document = parse_ooxml(&file_data)?;
//!     println!("Extracted text: {}", document.text);
//!     Ok(())
//! }
//! ```

mod error;
mod types;
mod opc;
mod document;
mod converter;
mod serializer;

pub use error::OoxmlError;
pub use converter::ooxml_to_piece_tree;
pub use serializer::{
    DocxSerializer,
    ExportOptions,
    ExportFormat,
    piece_tree_to_word_document,
};
pub use types::{
    ContentType,
    Paragraph,
    ParagraphProperties,
    Relationship,
    RelationshipType,
    Run,
    RunProperties,
    Style,
    Theme,
    ThemeFonts,
    PackagePart,
    DocumentImage,
    BlipFill,
    SourceRect,
    DocumentAnchor,
    AnchorPositionSpec,
    // Table types
    Table,
    TableRow,
    TableCell,
    TableProperties,
    TableRowProperties,
    TableCellProperties,
    TableBorders,
    TableBorder,
    // Header/Footer types
    Header,
    Footer,
    // Footnote/Endnote types
    Footnote,
    Endnote,
    // Numbering types
    Numbering,
    AbstractNumDef,
    ListLevel,
    NumInstance,
    LevelOverride,
    // Content Control types
    ContentControl,
    ContentControlProperties,
};
pub use opc::OpcPackage;
pub use document::WordDocument;

/// Serializable document structure for UI consumption
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedDocument {
    /// Plain text content extracted from the document
    pub text: String,

    /// Document styles indexed by style ID
    #[serde(default)]
    pub styles: std::collections::HashMap<String, Style>,

    /// Number of paragraphs in the document
    pub paragraph_count: usize,

    /// Number of characters in the document
    pub char_count: usize,

    /// Number of words in the document
    pub word_count: usize,

    /// Document title from core properties
    #[serde(default)]
    pub title: Option<String>,

    /// Document author from core properties
    #[serde(default)]
    pub author: Option<String>,

    /// Creation date from core properties
    #[serde(default)]
    pub created_at: Option<String>,

    /// Modification date from core properties
    #[serde(default)]
    pub modified_at: Option<String>,

    /// Theme colors (if available)
    #[serde(default)]
    pub theme: Option<Theme>,

    /// Tables in the document
    #[serde(default)]
    pub tables: Vec<Table>,

    /// Images in the document
    #[serde(default)]
    pub images: Vec<DocumentImage>,

    /// Headers in the document
    #[serde(default)]
    pub headers: Vec<Header>,

    /// Footers in the document
    #[serde(default)]
    pub footers: Vec<Footer>,

    /// Footnotes in the document
    #[serde(default)]
    pub footnotes: Vec<Footnote>,

    /// Endnotes in the document
    #[serde(default)]
    pub endnotes: Vec<Endnote>,

    /// Numbering definitions (list styles)
    #[serde(default)]
    pub numbering: Vec<Numbering>,
}

impl Default for ParsedDocument {
    fn default() -> Self {
        ParsedDocument {
            text: String::new(),
            styles: std::collections::HashMap::new(),
            paragraph_count: 0,
            char_count: 0,
            word_count: 0,
            title: None,
            author: None,
            created_at: None,
            modified_at: None,
            theme: None,
            tables: Vec::new(),
            images: Vec::new(),
            headers: Vec::new(),
            footers: Vec::new(),
            footnotes: Vec::new(),
            endnotes: Vec::new(),
            numbering: Vec::new(),
        }
    }
}

/// Parse OOXML document data and return structured content
///
/// This function takes raw .docx file bytes and parses them according to the
/// ECMA-376 OOXML standard. It extracts text content, styles, and metadata.
///
/// # Arguments
///
/// * `file_data` - Raw bytes of the .docx file
///
/// # Returns
///
/// Returns `Ok(ParsedDocument)` containing extracted content, or `Err(OoxmlError)` on failure.
///
/// # Errors
///
/// Returns `OoxmlError` if:
/// - The file is not a valid ZIP archive
/// - Required parts are missing (e.g., [Content_Types].xml)
/// - XML parsing fails
/// - Content types are invalid
pub fn parse_ooxml(file_data: &[u8]) -> Result<ParsedDocument, OoxmlError> {
    // Parse the OPC package
    let package = OpcPackage::new(file_data)?;
    
    // Parse the Word document
    let word_doc = WordDocument::parse(&package)?;
    
    // Calculate statistics
    let char_count = word_doc.text.chars().count();
    let word_count = word_doc.text.split_whitespace().count();
    
    // Extract core properties
    let (title, author, created_at, modified_at) = if let Some(props) = &word_doc.core_properties {
        (
            props.title.clone(),
            props.creator.clone(),
            props.created.clone(),
            props.modified.clone(),
        )
    } else {
        (None, None, None, None)
    };
    
    Ok(ParsedDocument {
        text: word_doc.text,
        styles: word_doc.styles,
        paragraph_count: word_doc.paragraphs.len(),
        char_count,
        word_count,
        title,
        author,
        created_at,
        modified_at,
        theme: word_doc.theme,
        tables: word_doc.tables,
        images: word_doc.images,
        headers: word_doc.headers,
        footers: word_doc.footers,
        footnotes: word_doc.footnotes,
        endnotes: word_doc.endnotes,
        numbering: word_doc.numbering,
    })
}

/// Parse OOXML document from file path
///
/// # Arguments
///
/// * `file_path` - Path to the .docx file
///
/// # Returns
///
/// Returns `Ok(ParsedDocument)` or `Err(OoxmlError)`
pub fn parse_ooxml_from_file(file_path: &str) -> Result<ParsedDocument, OoxmlError> {
    let file_data = std::fs::read(file_path)?;
    parse_ooxml(&file_data)
}

/// Serialize ParsedDocument to JSON string
///
/// # Arguments
///
/// * `document` - The parsed document to serialize
///
/// # Returns
///
/// JSON string representation
pub fn document_to_json(document: &ParsedDocument) -> Result<String, OoxmlError> {
    serde_json::to_string(document)
        .map_err(|e| OoxmlError::ParseError(e.to_string()))
}

/// Deserialize ParsedDocument from JSON string
///
/// # Arguments
///
/// * `json` - JSON string representation
///
/// # Returns
///
/// ParsedDocument or error
pub fn document_from_json(json: &str) -> Result<ParsedDocument, OoxmlError> {
    serde_json::from_str(json)
        .map_err(|e| OoxmlError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_parse_ooxml_empty() {
        // Test with empty data - should fail with zip error
        let result = parse_ooxml(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_content_type_parsing() {
        // Test content type enum
        let ct = ContentType::from_string("application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml");
        assert_eq!(ct, ContentType::MainDocument);
        
        let ct = ContentType::from_string("application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml");
        assert_eq!(ct, ContentType::Styles);
        
        let ct = ContentType::from_string("unknown/type");
        assert_eq!(ct, ContentType::Unknown("unknown/type".to_string()));
    }

    #[test]
    fn test_relationship_type_parsing() {
        // Test relationship type enum
        let rt = RelationshipType::from_string("http://schemas.openxmlformats.org/officeDocument/2006/relationships/mainDocument");
        assert_eq!(rt, RelationshipType::Document);
        
        let rt = RelationshipType::from_string("http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles");
        assert_eq!(rt, RelationshipType::Styles);
        
        let rt = RelationshipType::from_string("unknown/type");
        assert_eq!(rt, RelationshipType::Unknown("unknown/type".to_string()));
    }

    #[test]
    fn test_parsed_document_serialization() {
        let doc = ParsedDocument {
            text: "Hello World".to_string(),
            styles: std::collections::HashMap::new(),
            paragraph_count: 1,
            char_count: 11,
            word_count: 2,
            title: Some("Test Document".to_string()),
            author: Some("Test Author".to_string()),
            created_at: None,
            modified_at: None,
            theme: None,
            tables: Vec::new(),
            images: Vec::new(),
            headers: Vec::new(),
            footers: Vec::new(),
            footnotes: Vec::new(),
            endnotes: Vec::new(),
            numbering: Vec::new(),
        };

        let json = document_to_json(&doc).unwrap();
        let parsed = document_from_json(&json).unwrap();

        assert_eq!(parsed.text, "Hello World");
        assert_eq!(parsed.paragraph_count, 1);
        assert_eq!(parsed.title, Some("Test Document".to_string()));
    }
}

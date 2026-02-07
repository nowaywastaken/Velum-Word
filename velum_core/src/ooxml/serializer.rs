//! DOCX Serializer - Exports PieceTree documents to OOXML format
//!
//! This module provides functionality to save Word documents in DOCX format.
//! It converts PieceTree data structures to OOXML XML files and packages
//! them into a valid ZIP archive according to ECMA-376 standards.

use std::collections::HashMap;
use std::io::{Cursor, Write};
use zip::write::FileOptions;
use zip::ZipWriter;

use super::error::OoxmlError;
use super::opc::OpcPackage;
use super::types::{
    ContentType, Paragraph, ParagraphProperties, Relationship, RelationshipType,
    Run, RunProperties, Style, Theme, ThemeFonts,
};
use crate::piece_tree::{PieceTree, TextAttributes};

/// DOCX 序列化器
pub struct DocxSerializer {
    package: OpcPackage,
    document: WordDocument,
}

/// 导出选项
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_images: bool,
    pub include_styles: bool,
    pub include_theme: bool,
}

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Docx,
    Docm,
    FlatOxml,
}

/// Represents an image to be embedded in the document
#[derive(Debug, Clone)]
pub struct ExportImage {
    /// Unique ID for referencing the image
    pub id: String,
    /// File path within the media folder
    pub path: String,
    /// Image data bytes
    pub data: Vec<u8>,
    /// MIME type of the image
    pub mime_type: String,
}

/// Serialized part to be written to the ZIP archive
#[derive(Debug, Clone)]
pub struct SerializedPart {
    /// Path within the archive (e.g., "/word/document.xml")
    pub path: String,
    /// Content type for Content_Types.xml
    pub content_type: ContentType,
    /// XML content bytes
    pub data: Vec<u8>,
    /// Relationships for this part
    pub relationships: Vec<Relationship>,
}

/// Serialized document data ready for ZIP packaging
#[derive(Debug, Clone)]
pub struct SerializedDocument {
    /// All parts to be written to the archive
    pub parts: Vec<SerializedPart>,
    /// Root relationships
    pub root_relationships: Vec<Relationship>,
    /// Images to be embedded
    pub images: Vec<ExportImage>,
    /// Content types map
    pub content_types: HashMap<String, ContentType>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            format: ExportFormat::Docx,
            include_images: true,
            include_styles: true,
            include_theme: true,
        }
    }
}

impl DocxSerializer {
    /// Create a new serializer from an OPC package
    pub fn new(package: OpcPackage, document: WordDocument) -> Self {
        DocxSerializer { package, document }
    }

    /// Export the document to DOCX format bytes
    pub fn export_docx(&self, options: Option<ExportOptions>) -> Result<Vec<u8>, OoxmlError> {
        let options = options.unwrap_or_default();
        let serialized = self.serialize(options.clone())?;
        self.package_to_zip(&serialized, options)
    }

    /// Export the document to a file
    pub fn export_to_file(
        &self,
        path: &str,
        options: Option<ExportOptions>,
    ) -> Result<(), OoxmlError> {
        let docx_data = self.export_docx(options)?;
        std::fs::write(path, &docx_data)?;
        Ok(())
    }

    /// Serialize the document to an intermediate representation
    fn serialize(&self, options: ExportOptions) -> Result<SerializedDocument, OoxmlError> {
        let mut parts = Vec::new();
        let images = Vec::new();
        let mut content_types = HashMap::new();
        let mut root_relationships = Vec::new();

        // Generate root relationships
        root_relationships.push(Relationship {
            id: "rId1".to_string(),
            relationship_type: RelationshipType::OfficeDocument,
            target: "word/document.xml".to_string(),
            target_mode: None,
        });

        if options.include_styles {
            root_relationships.push(Relationship {
                id: "rId2".to_string(),
                relationship_type: RelationshipType::Styles,
                target: "word/styles.xml".to_string(),
                target_mode: None,
            });
        }

        root_relationships.push(Relationship {
            id: "rId3".to_string(),
            relationship_type: RelationshipType::CoreProperties,
            target: "docProps/core.xml".to_string(),
            target_mode: None,
        });

        root_relationships.push(Relationship {
            id: "rId4".to_string(),
            relationship_type: RelationshipType::Unknown("http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties".to_string()),
            target: "docProps/app.xml".to_string(),
            target_mode: None,
        });

        // Serialize main document
        let document_part = self.serialize_document(&self.document)?;
        parts.push(document_part);
        content_types.insert(
            "/word/document.xml".to_string(),
            ContentType::MainDocument,
        );

        // Serialize styles if requested
        if options.include_styles {
            let styles_part = self.serialize_styles(&self.document.styles)?;
            parts.push(styles_part);
            content_types.insert(
                "/word/styles.xml".to_string(),
                ContentType::Styles,
            );
        }

        // Serialize core properties
        let core_part = self.serialize_core_properties(&self.document);
        parts.push(core_part);
        content_types.insert(
            "/docProps/core.xml".to_string(),
            ContentType::CoreProperties,
        );

        // Serialize app properties
        let app_part = Self::serialize_app_properties();
        parts.push(app_part);
        content_types.insert(
            "/docProps/app.xml".to_string(),
            ContentType::AppProperties,
        );

        // Serialize theme if requested and available
        if options.include_theme {
            if let Some(ref theme) = self.document.theme {
                let theme_part = self.serialize_theme(theme);
                parts.push(theme_part);
                content_types.insert(
                    "/word/theme/theme1.xml".to_string(),
                    ContentType::Theme,
                );
            }
        }

        // Add default content types
        content_types.insert("/rels".to_string(), ContentType::Relationships);
        content_types.insert(".rels".to_string(), ContentType::Relationships);

        Ok(SerializedDocument {
            parts,
            root_relationships,
            images,
            content_types,
        })
    }

    /// Serialize the main document body
    fn serialize_document(&self, document: &WordDocument) -> Result<SerializedPart, OoxmlError> {
        let mut body = String::new();

        // Document header
        body.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        body.push_str(r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#);
        body.push_str(r#"<w:body>"#);

        // Serialize each paragraph
        for para in &document.paragraphs {
            body.push_str(&self.serialize_paragraph(para)?);
        }

        // End document body
        body.push_str(r#"</w:body>"#);
        body.push_str(r#"</w:document>"#);

        Ok(SerializedPart {
            path: "/word/document.xml".to_string(),
            content_type: ContentType::MainDocument,
            data: body.into_bytes(),
            relationships: Vec::new(),
        })
    }

    /// Serialize a single paragraph
    fn serialize_paragraph(&self, para: &Paragraph) -> Result<String, OoxmlError> {
        let mut xml = String::new();

        xml.push_str("<w:p>");

        // Serialize paragraph properties
        xml.push_str(&self.serialize_paragraph_properties(&para.properties));

        // Serialize runs
        for run in &para.runs {
            xml.push_str(&self.serialize_run(run)?);
        }

        xml.push_str("</w:p>");

        Ok(xml)
    }

    /// Serialize paragraph properties
    fn serialize_paragraph_properties(&self, props: &ParagraphProperties) -> String {
        let mut xml = String::new();

        if props.indent_left.is_some()
            || props.indent_right.is_some()
            || props.indent_first_line.is_some()
            || props.spacing_before.is_some()
            || props.spacing_after.is_some()
            || props.spacing_line.is_some()
            || props.alignment.is_some()
        {
            xml.push_str("<w:pPr>");

            if let Some(ref align) = props.alignment {
                xml.push_str(&format!(r#"<w:jc w:val="{}"/>"#, escape_xml_attr(align)));
            }

            if let Some(left) = props.indent_left {
                xml.push_str(&format!(r#"<w:ind w:left="{}"/>"#, left));
            }

            if let Some(right) = props.indent_right {
                xml.push_str(&format!(r#"<w:ind w:right="{}"/>"#, right));
            }

            if let Some(first) = props.indent_first_line {
                xml.push_str(&format!(r#"<w:ind w:firstLine="{}"/>"#, first));
            }

            if let Some(before) = props.spacing_before {
                xml.push_str(&format!(r#"<w:spacing w:before="{}"/>"#, before));
            }

            if let Some(after) = props.spacing_after {
                xml.push_str(&format!(r#"<w:spacing w:after="{}"/>"#, after));
            }

            if let Some(line) = props.spacing_line {
                xml.push_str(&format!(r#"<w:spacing w:line="{}"/>"#, line));
            }

            xml.push_str("</w:pPr>");
        }

        xml
    }

    /// Serialize a run
    fn serialize_run(&self, run: &Run) -> Result<String, OoxmlError> {
        let mut xml = String::new();

        xml.push_str("<w:r>");

        // Serialize run properties
        xml.push_str(&self.serialize_run_properties(&run.properties));

        // Serialize text
        if !run.text.is_empty() {
            xml.push_str(&format!(
                "<w:t>{}</w:t>",
                escape_xml_text(&run.text)
            ));
        }

        xml.push_str("</w:r>");

        Ok(xml)
    }

    /// Serialize run properties
    fn serialize_run_properties(&self, props: &RunProperties) -> String {
        let mut xml = String::new();

        if props.bold.is_some()
            || props.italic.is_some()
            || props.underline.is_some()
            || props.font_size.is_some()
            || props.font_name.is_some()
            || props.color.is_some()
            || props.background_color.is_some()
        {
            xml.push_str("<w:rPr>");

            if let Some(bold) = props.bold {
                xml.push_str(&format!(r#"<w:b w:val="{}"/>"#, if bold { "1" } else { "0" }));
            }

            if let Some(italic) = props.italic {
                xml.push_str(&format!(r#"<w:i w:val="{}"/>"#, if italic { "1" } else { "0" }));
            }

            if let Some(ref underline) = props.underline {
                xml.push_str(&format!(r#"<w:u w:val="{}"/>"#, escape_xml_attr(underline)));
            }

            if let Some(size) = props.font_size {
                // Word uses half-points, so multiply by 2
                xml.push_str(&format!(r#"<w:sz w:val="{}"/>"#, size * 2));
            }

            if let Some(ref name) = props.font_name {
                xml.push_str(&format!(r#"<w:rFonts w:ascii="{}"/>"#, escape_xml_attr(name)));
            }

            if let Some(ref color) = props.color {
                xml.push_str(&format!(r#"<w:color w:val="{}"/>"#, escape_xml_attr(color)));
            }

            if let Some(ref bg_color) = props.background_color {
                xml.push_str(&format!(r#"<w:shd w:fill="{}"/>"#, escape_xml_attr(bg_color)));
            }

            xml.push_str("</w:rPr>");
        }

        xml
    }

    /// Serialize styles
    fn serialize_styles(&self, styles: &HashMap<String, Style>) -> Result<SerializedPart, OoxmlError> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(
            r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
        );

        // Serialize each style
        for (_, style) in styles {
            xml.push_str(&self.serialize_style(style)?);
        }

        // Add default styles if none exist
        if styles.is_empty() {
            xml.push_str(&self.get_default_styles());
        }

        xml.push_str("</w:styles>");

        Ok(SerializedPart {
            path: "/word/styles.xml".to_string(),
            content_type: ContentType::Styles,
            data: xml.into_bytes(),
            relationships: Vec::new(),
        })
    }

    /// Serialize a single style
    fn serialize_style(&self, style: &Style) -> Result<String, OoxmlError> {
        let mut xml = String::new();

        xml.push_str(&format!(r#"<w:style w:styleId="{}" w:type="{}""#,
            escape_xml_attr(&style.id),
            escape_xml_attr(&style.style_type)));

        if style.is_default {
            xml.push_str(r#" w:default="1""#);
        }

        xml.push_str(">");

        // Style name
        if let Some(ref name) = style.name {
            xml.push_str(&format!(r#"<w:name w:val="{}"/>"#, escape_xml_attr(name)));
        }

        // Based on
        if let Some(ref based_on) = style.based_on {
            xml.push_str(&format!(r#"<w:basedOn w:val="{}"/>"#, escape_xml_attr(based_on)));
        }

        // Paragraph properties
        xml.push_str(&self.serialize_paragraph_properties(&style.paragraph_properties));

        // Run properties
        xml.push_str(&self.serialize_run_properties(&style.run_properties));

        xml.push_str("</w:style>");

        Ok(xml)
    }

    /// Get default styles for a new document
    fn get_default_styles(&self) -> String {
        r#"<w:style w:styleId="Normal" w:type="paragraph" w:default="1">
            <w:name w:val="Normal"/>
            <w:rPr>
                <w:sz w:val="22"/>
                <w:lang w:val="en-US"/>
            </w:rPr>
        </w:style>
        <w:style w:styleId="Heading1" w:type="paragraph">
            <w:name w:val="Heading 1"/>
            <w:basedOn w:val="Normal"/>
            <w:pPr>
                <w:spacing w:before="240" w:after="60"/>
            </w:pPr>
            <w:rPr>
                <w:b w:val="1"/>
                <w:sz w:val="32"/>
            </w:rPr>
        </w:style>
        <w:style w:styleId="Heading2" w:type="paragraph">
            <w:name w:val="Heading 2"/>
            <w:basedOn w:val="Heading1"/>
            <w:pPr>
                <w:spacing w:before="200" w:after="40"/>
            </w:pPr>
            <w:rPr>
                <w:b w:val="1"/>
                <w:sz w:val="26"/>
            </w:rPr>
        </w:style>"#
            .to_string()
    }

    /// Serialize core properties
    fn serialize_core_properties(&self, document: &WordDocument) -> SerializedPart {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(
            r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
            xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:dcterms="http://purl.org/dc/terms/"
            xmlns:dcmitype="http://purl.org/dc/dcmitype/"
            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
        );

        // Title
        if let Some(ref title) = document.core_properties.as_ref().and_then(|p| p.title.as_ref()) {
            xml.push_str(&format!(r#"<dc:title>{}</dc:title>"#, escape_xml_text(title)));
        } else {
            xml.push_str("<dc:title/>");
        }

        // Creator
        if let Some(ref creator) = document.core_properties.as_ref().and_then(|p| p.creator.as_ref()) {
            xml.push_str(&format!(r#"<dc:creator>{}</dc:creator>"#, escape_xml_text(creator)));
        }

        // Created
        if let Some(ref created) = document.core_properties.as_ref().and_then(|p| p.created.as_ref()) {
            xml.push_str(&format!(
                r#"<dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>"#,
                created
            ));
        } else {
            let now = chrono::Utc::now();
            xml.push_str(&format!(
                r#"<dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>"#,
                now.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
            ));
        }

        // Modified
        if let Some(ref modified) = document.core_properties.as_ref().and_then(|p| p.modified.as_ref()) {
            xml.push_str(&format!(
                r#"<dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>"#,
                modified
            ));
        } else {
            let now = chrono::Utc::now();
            xml.push_str(&format!(
                r#"<dcterms:modified xsi:type="dcterms:W3CDTF">{}</dcterms:modified>"#,
                now.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
            ));
        }

        xml.push_str("</cp:coreProperties>");

        SerializedPart {
            path: "/docProps/core.xml".to_string(),
            content_type: ContentType::CoreProperties,
            data: xml.into_bytes(),
            relationships: Vec::new(),
        }
    }

    /// Serialize app properties
    fn serialize_app_properties() -> SerializedPart {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        <Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
            <Application>Velum</Application>
            <AppVersion>1.0</AppVersion>
        </Properties>"#
            .to_string();

        SerializedPart {
            path: "/docProps/app.xml".to_string(),
            content_type: ContentType::AppProperties,
            data: xml.into_bytes(),
            relationships: Vec::new(),
        }
    }

    /// Serialize theme
    fn serialize_theme(&self, theme: &Theme) -> SerializedPart {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(
            r#"<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">"#,
        );

        // Theme colors
        xml.push_str("<a:clrScheme name=\"Office\">");
        xml.push_str("<a:dk1><a:srgbClr val=\"000000\"/></a:dk1>");
        xml.push_str("<a:lt1><a:srgbClr val=\"FFFFFF\"/></a:lt1>");
        xml.push_str("<a:dk2><a:srgbClr val=\"1F497D\"/></a:dk2>");
        xml.push_str("<a:lt2><a:srgbClr val=\"EEECE1\"/></a:lt2>");
        xml.push_str("<a:accent1><a:srgbClr val=\"4F81BD\"/></a:accent1>");
        xml.push_str("<a:accent2><a:srgbClr val=\"C0504D\"/></a:accent2>");
        xml.push_str("<a:accent3><a:srgbClr val=\"9BBB59\"/></a:accent3>");
        xml.push_str("<a:accent4><a:srgbClr val=\"8064A2\"/></a:accent4>");
        xml.push_str("<a:accent5><a:srgbClr val=\"4BACC6\"/></a:accent5>");
        xml.push_str("<a:accent6><a:srgbClr val=\"F79646\"/></a:accent6>");
        xml.push_str("<a:hlink><a:srgbClr val=\"0000FF\"/></a:hlink>");
        xml.push_str("<a:folHlink><a:srgbClr val=\"800080\"/></a:folHlink>");
        xml.push_str("</a:clrScheme>");

        // Theme fonts
        xml.push_str(&format!(
            r#"<a:fontScheme name="Office">
            <a:majorFont>
                <a:latin typeface="{}"/>
                <a:ea typeface="{}"/>
            </a:majorFont>
            <a:minorFont>
                <a:latin typeface="{}"/>
                <a:ea typeface="{}"/>
            </a:minorFont>
        </a:fontScheme>"#,
            escape_xml_attr(&theme.fonts.major_font),
            escape_xml_attr(&theme.fonts.major_font),
            escape_xml_attr(&theme.fonts.minor_font),
            escape_xml_attr(&theme.fonts.minor_font)
        ));

        xml.push_str("</a:theme>");

        SerializedPart {
            path: "/word/theme/theme1.xml".to_string(),
            content_type: ContentType::Theme,
            data: xml.into_bytes(),
            relationships: Vec::new(),
        }
    }

    /// Package the serialized document into a ZIP archive
    fn package_to_zip(
        &self,
        serialized: &SerializedDocument,
        _options: ExportOptions,
    ) -> Result<Vec<u8>, OoxmlError> {
        let mut writer = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut writer);

            let zip_options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .compression_level(Some(9));

            // Write [Content_Types].xml
            let content_types_xml = self.generate_content_types_xml(&serialized.content_types);
            zip.start_file("[Content_Types].xml", zip_options)?;
            zip.write_all(&content_types_xml)?;

            // Write root relationships
            let rels_xml = self.generate_relationships_xml(&serialized.root_relationships, "");
            zip.start_file("_rels/.rels", zip_options)?;
            zip.write_all(&rels_xml)?;

            // Write document relationships
            let doc_rels = self.generate_document_relationships(serialized);
            zip.start_file("word/_rels/document.xml.rels", zip_options)?;
            zip.write_all(&doc_rels)?;

            // Write all parts
            for part in &serialized.parts {
                zip.start_file(&part.path[1..], zip_options)?; // Remove leading slash
                zip.write_all(&part.data)?;
            }

            // Write images if any
            for image in &serialized.images {
                zip.start_file(&image.path, zip_options)?;
                zip.write_all(&image.data)?;
            }

            // Finish ZIP
            zip.finish()?;
        }

        Ok(writer.into_inner())
    }

    /// Generate Content_Types.xml
    fn generate_content_types_xml(
        &self,
        content_types: &HashMap<String, ContentType>,
    ) -> Vec<u8> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
        );

        // Default types for common extensions
        xml.push_str(r#"<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>"#);
        xml.push_str(r#"<Default Extension="xml" ContentType="application/xml"/>"#);

        // Override types
        for (part_name, content_type) in content_types {
            if part_name.starts_with("/") {
                let type_str = match content_type {
                    ContentType::MainDocument => "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
                    ContentType::Styles => "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                    ContentType::Theme => "application/vnd.openxmlformats-officedocument.theme+xml",
                    ContentType::Settings => "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
                    ContentType::CoreProperties => "application/vnd.openxmlformats-package.core-properties+xml",
                    ContentType::AppProperties => "application/vnd.openxmlformats-officedocument.extended-properties+xml",
                    ContentType::Numbering => "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                    ContentType::WebSettings => "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml",
                    ContentType::ImagePng => "image/png",
                    ContentType::ImageJpeg => "image/jpeg",
                    ContentType::ImageGif => "image/gif",
                    ContentType::ImageBmp => "image/bmp",
                    ContentType::ImageSvg => "image/svg+xml",
                    _ => "application/xml",
                };
                xml.push_str(&format!(
                    r#"<Override PartName="{}" ContentType="{}"/>"#,
                    part_name, type_str
                ));
            }
        }

        xml.push_str("</Types>");

        xml.into_bytes()
    }

    /// Generate relationships XML
    fn generate_relationships_xml(
        &self,
        relationships: &[Relationship],
        base_path: &str,
    ) -> Vec<u8> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );

        for rel in relationships {
            let target = if base_path.is_empty() {
                rel.target.clone()
            } else {
                format!("{}/{}", base_path, rel.target)
            };
            let type_uri = match &rel.relationship_type {
                RelationshipType::OfficeDocument => "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument".to_string(),
                RelationshipType::Document => "http://schemas.openxmlformats.org/officeDocument/2006/relationships/mainDocument".to_string(),
                RelationshipType::Styles => "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles".to_string(),
                RelationshipType::Theme => "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme".to_string(),
                RelationshipType::Settings => "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings".to_string(),
                RelationshipType::CoreProperties => "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties".to_string(),
                RelationshipType::Image => "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image".to_string(),
                RelationshipType::Unknown(uri) => uri.clone(),
                _ => "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument".to_string(),
            };
            xml.push_str(&format!(
                r#"<Relationship Id="{}" Type="{}" Target="{}"/>"#,
                escape_xml_attr(&rel.id),
                type_uri,
                escape_xml_attr(&target)
            ));
        }

        xml.push_str("</Relationships>");

        xml.into_bytes()
    }

    /// Generate document relationships
    fn generate_document_relationships(&self, serialized: &SerializedDocument) -> Vec<u8> {
        let mut relationships = Vec::new();

        // Add theme relationship if theme is included
        if serialized.content_types.contains_key("/word/theme/theme1.xml") {
            relationships.push(Relationship {
                id: "rIdTheme".to_string(),
                relationship_type: RelationshipType::Theme,
                target: "../theme/theme1.xml".to_string(),
                target_mode: None,
            });
        }

        // Add image relationships
        for (i, image) in serialized.images.iter().enumerate() {
            relationships.push(Relationship {
                id: format!("rIdImage{}", i + 1),
                relationship_type: RelationshipType::Image,
                target: format!("media/{}", image.path),
                target_mode: None,
            });
        }

        self.generate_relationships_xml(&relationships, "word")
    }
}

/// Convert PieceTree to WordDocument for serialization
pub fn piece_tree_to_word_document(tree: &PieceTree) -> WordDocument {
    let mut paragraphs = Vec::new();
    let mut current_para = Paragraph::default();

    // Process all pieces
    for piece in &tree.pieces {
        let buffer_idx = PieceTree::buffer_idx(&piece.buffer_id);
        if let Some(buffer) = tree.buffers.get(buffer_idx) {
            let piece_text = if piece.start + piece.length <= buffer.len() {
                &buffer[piece.start..piece.start + piece.length]
            } else {
                ""
            };

            // Split by newlines to create paragraphs
            for part in piece_text.split('\n') {
                if !current_para.text.is_empty() {
                    paragraphs.push(current_para.clone());
                }
                current_para = Paragraph::default();
                current_para.text = part.to_string();

                // Create run with piece attributes
                let mut run = Run::default();
                run.text = part.to_string();

                // Convert TextAttributes to RunProperties
                if let Some(ref attrs) = piece.attributes {
                    run.properties = convert_attrs_to_run_props(attrs);
                }

                current_para.runs.push(run);
            }
        }
    }

    // Add last paragraph if not empty
    if !current_para.text.is_empty() || !current_para.runs.is_empty() {
        paragraphs.push(current_para);
    }

    // Build full text
    let text = paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect::<Vec<_>>()
        .join("\n");

    WordDocument {
        text,
        paragraphs,
        styles: HashMap::new(),
        theme: Some(create_default_theme()),
        core_properties: Some(CoreProperties::default()),
    }
}

/// Convert TextAttributes to RunProperties
fn convert_attrs_to_run_props(attrs: &TextAttributes) -> RunProperties {
    RunProperties {
        bold: attrs.bold,
        italic: attrs.italic,
        underline: attrs.underline.map(|u| if u { "single".to_string() } else { "none".to_string() }),
        font_size: attrs.font_size.map(|s| s as i32),
        font_name: attrs.font_family.clone(),
        color: attrs.foreground.clone(),
        background_color: attrs.background.clone(),
    }
}

/// Create a default theme
fn create_default_theme() -> Theme {
    Theme {
        name: "Office Theme".to_string(),
        colors: HashMap::new(),
        fonts: ThemeFonts {
            major_font: "Calibri".to_string(),
            minor_font: "Calibri".to_string(),
            symbol_font: "Symbol".to_string(),
        },
    }
}

/// Core properties helper
#[derive(Debug, Clone, Default)]
pub struct CoreProperties {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub keywords: Option<String>,
    pub description: Option<String>,
    pub last_modified_by: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

/// Word document structure for serialization
#[derive(Debug, Clone, Default)]
pub struct WordDocument {
    pub text: String,
    pub paragraphs: Vec<Paragraph>,
    pub styles: HashMap<String, Style>,
    pub theme: Option<Theme>,
    pub core_properties: Option<CoreProperties>,
}

/// Escape special XML characters in text content
fn escape_xml_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape special XML characters in attribute values
fn escape_xml_attr(attr: &str) -> String {
    escape_xml_text(attr)
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_serialize_empty_document() {
        let doc = WordDocument::default();
        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(None);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_serialize_simple_document() {
        let mut doc = WordDocument::default();
        doc.text = "Hello World".to_string();

        let para = Paragraph {
            text: "Hello World".to_string(),
            properties: ParagraphProperties::default(),
            runs: vec![Run {
                text: "Hello World".to_string(),
                properties: RunProperties::default(),
            }],
        };
        doc.paragraphs.push(para);

        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(None);
        assert!(result.is_ok());
        let data = result.unwrap();

        // Verify ZIP structure
        assert!(data.starts_with(b"PK")); // ZIP magic bytes
    }

    #[test]
    fn test_serialize_with_formatted_text() {
        let mut doc = WordDocument::default();
        doc.text = "Bold and Italic".to_string();

        let run = Run {
            text: "Bold".to_string(),
            properties: RunProperties {
                bold: Some(true),
                ..Default::default()
            },
        };

        let para = Paragraph {
            text: "Bold and Italic".to_string(),
            properties: ParagraphProperties::default(),
            runs: vec![run],
        };
        doc.paragraphs.push(para);

        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_with_styles() {
        let mut doc = WordDocument::default();
        doc.text = "Heading".to_string();

        let mut style = Style::default();
        style.id = "Heading1".to_string();
        style.name = Some("Heading 1".to_string());
        style.style_type = "paragraph".to_string();
        style.run_properties.bold = Some(true);
        style.run_properties.font_size = Some(32);

        doc.styles.insert("Heading1".to_string(), style);

        let para = Paragraph {
            text: "Heading".to_string(),
            properties: ParagraphProperties::default(),
            runs: vec![Run {
                text: "Heading".to_string(),
                properties: RunProperties::default(),
            }],
        };
        doc.paragraphs.push(para);

        let options = ExportOptions {
            format: ExportFormat::Docx,
            include_images: true,
            include_styles: true,
            include_theme: true,
        };

        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(Some(options));
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_multiple_paragraphs() {
        let mut doc = WordDocument::default();
        doc.text = "Paragraph 1\nParagraph 2\nParagraph 3".to_string();

        for i in 1..=3 {
            let para = Paragraph {
                text: format!("Paragraph {}", i),
                properties: ParagraphProperties::default(),
                runs: vec![Run {
                    text: format!("Paragraph {}", i),
                    properties: RunProperties::default(),
                }],
            };
            doc.paragraphs.push(para);
        }

        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(None);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.len() > 100);
    }

    #[test]
    fn test_serialize_with_special_characters() {
        let mut doc = WordDocument::default();
        doc.text = "Special chars: <>&\"'".to_string();

        let para = Paragraph {
            text: "Special chars: <>&\"'".to_string(),
            properties: ParagraphProperties::default(),
            runs: vec![Run {
                text: "Special chars: <>&\"'".to_string(),
                properties: RunProperties::default(),
            }],
        };
        doc.paragraphs.push(para);

        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_serialize_export_options() {
        let doc = WordDocument::default();

        // Test with minimal options
        let options = ExportOptions {
            format: ExportFormat::Docx,
            include_images: false,
            include_styles: false,
            include_theme: false,
        };

        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(Some(options));
        assert!(result.is_ok());
    }

    #[test]
    fn test_content_types_generation() {
        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: WordDocument::default(),
        };

        let mut content_types = HashMap::new();
        content_types.insert("/word/document.xml".to_string(), ContentType::MainDocument);
        content_types.insert("/word/styles.xml".to_string(), ContentType::Styles);
        content_types.insert("/docProps/core.xml".to_string(), ContentType::CoreProperties);

        let xml = serializer.generate_content_types_xml(&content_types);
        let xml_str = String::from_utf8_lossy(&xml);

        assert!(xml_str.contains("word/document.xml"));
        assert!(xml_str.contains("word/styles.xml"));
        assert!(xml_str.contains("docProps/core.xml"));
    }

    #[test]
    fn test_piece_tree_conversion() {
        let tree = PieceTree::new("Line 1\nLine 2\nLine 3".to_string());
        let doc = piece_tree_to_word_document(&tree);

        assert_eq!(doc.text, "Line 1\nLine 2\nLine 3");
        assert_eq!(doc.paragraphs.len(), 3);
        assert_eq!(doc.paragraphs[0].text, "Line 1");
        assert_eq!(doc.paragraphs[1].text, "Line 2");
        assert_eq!(doc.paragraphs[2].text, "Line 3");
    }

    #[test]
    fn test_piece_tree_with_attributes() {
        let mut tree = PieceTree::new("".to_string());
        tree.insert(0, "Bold".to_string());

        // Apply bold formatting by modifying the piece
        if let Some(piece) = tree.pieces.first_mut() {
            piece.attributes = Some(TextAttributes {
                bold: Some(true),
                ..Default::default()
            });
        }

        let doc = piece_tree_to_word_document(&tree);
        assert!(!doc.paragraphs.is_empty());
    }

    #[test]
    fn test_xml_escaping() {
        assert_eq!(escape_xml_text("a<b>c&d"), "a&lt;b&gt;c&amp;d");
        assert_eq!(escape_xml_attr("a\"b'c"), "a&quot;b&apos;c");
    }

    #[test]
    fn test_export_to_file() {
        let doc = WordDocument::default();
        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let temp_path = PathBuf::from("/tmp/test_export.docx");
        let result = serializer.export_to_file(temp_path.to_str().unwrap(), None);

        assert!(result.is_ok());

        // Clean up
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_flat_ooxml_format() {
        let options = ExportOptions {
            format: ExportFormat::FlatOxml,
            ..Default::default()
        };

        let doc = WordDocument::default();
        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(Some(options));
        // Flat OOXML is just the document XML without ZIP packaging
        // For now, it produces the same output as Docx
        assert!(result.is_ok());
    }

    #[test]
    fn test_relationships_generation() {
        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: WordDocument::default(),
        };

        let relationships = vec![
            Relationship {
                id: "rId1".to_string(),
                relationship_type: RelationshipType::OfficeDocument,
                target: "word/document.xml".to_string(),
                target_mode: None,
            },
            Relationship {
                id: "rId2".to_string(),
                relationship_type: RelationshipType::Styles,
                target: "word/styles.xml".to_string(),
                target_mode: None,
            },
        ];

        let xml = serializer.generate_relationships_xml(&relationships, "");
        let xml_str = String::from_utf8_lossy(&xml);

        assert!(xml_str.contains("rId1"));
        assert!(xml_str.contains("rId2"));
        assert!(xml_str.contains("document.xml"));
        assert!(xml_str.contains("styles.xml"));
    }

    #[test]
    fn test_large_document() {
        let mut doc = WordDocument::default();
        let mut text = String::new();

        // Create 100 paragraphs
        for i in 0..100 {
            text.push_str(&format!("This is paragraph {}.\n", i));
            let para = Paragraph {
                text: format!("This is paragraph {}.", i),
                properties: ParagraphProperties::default(),
                runs: vec![Run {
                    text: format!("This is paragraph {}.", i),
                    properties: RunProperties::default(),
                }],
            };
            doc.paragraphs.push(para);
        }
        doc.text = text.trim_end().to_string();

        let serializer = DocxSerializer {
            package: OpcPackage::new(&[]).unwrap_or_default(),
            document: doc,
        };

        let result = serializer.export_docx(None);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(data.len() > 5000);
    }
}

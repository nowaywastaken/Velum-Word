//! WordProcessingML document parser

use std::collections::HashMap;

use super::opc::OpcPackage;
use super::types::{
    Paragraph, ParagraphProperties, Run, RunProperties, Style, Theme, ThemeFonts,
    Table, TableRow, TableCell, TableProperties, TableRowProperties,
    TableBorders, TableBorder, Header, Footer, Footnote, Endnote, Numbering,
    AbstractNumDef, ListLevel, NumInstance, DocumentImage,
};
use super::error::OoxmlError;

/// WordProcessingML document parser
#[derive(Debug, Clone)]
pub struct WordDocument {
    /// Extracted text content
    pub text: String,
    /// Parsed paragraphs
    pub paragraphs: Vec<Paragraph>,
    /// Document styles indexed by style ID
    pub styles: HashMap<String, Style>,
    /// Document theme (colors/fonts)
    pub theme: Option<Theme>,
    /// Core properties (title, author, etc.)
    pub core_properties: Option<CoreProperties>,
    /// Tables in the document
    pub tables: Vec<Table>,
    /// Images in the document
    pub images: Vec<DocumentImage>,
    /// Headers in the document
    pub headers: Vec<Header>,
    /// Footers in the document
    pub footers: Vec<Footer>,
    /// Footnotes in the document
    pub footnotes: Vec<Footnote>,
    /// Endnotes in the document
    pub endnotes: Vec<Endnote>,
    /// Numbering definitions (list styles)
    pub numbering: Vec<Numbering>,
}

/// Core document properties
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

impl WordDocument {
    /// Create a new WordDocument by parsing the OPC package
    pub fn parse(package: &OpcPackage) -> Result<Self, OoxmlError> {
        let mut document = WordDocument {
            text: String::new(),
            paragraphs: Vec::new(),
            styles: HashMap::new(),
            theme: None,
            core_properties: None,
            tables: Vec::new(),
            images: Vec::new(),
            headers: Vec::new(),
            footers: Vec::new(),
            footnotes: Vec::new(),
            endnotes: Vec::new(),
            numbering: Vec::new(),
        };

        document.parse_main_document(package)?;
        document.parse_styles(package)?;
        document.parse_theme(package)?;
        document.parse_core_properties(package)?;
        document.parse_numbering(package)?;
        document.parse_headers_footers(package)?;
        document.parse_footnotes_endnotes(package)?;

        Ok(document)
    }

    /// Parse the main document body (word/document.xml)
    fn parse_main_document(&mut self, package: &OpcPackage) -> Result<(), OoxmlError> {
        let main_part_name = "/word/document.xml".to_string();

        let main_part = package.get_part(&main_part_name)
            .ok_or_else(|| OoxmlError::PartNotFound(main_part_name.clone()))?;

        let xml_str = String::from_utf8_lossy(&main_part.data);

        // First, extract and parse all tables
        self.parse_tables(&xml_str, package);

        // Then parse paragraphs (excluding those inside tables)
        // We need to handle paragraphs outside tables
        let para_pattern = regex::Regex::new(r#"<w:p[^>]*>(.*?)</w:p>"#).unwrap();

        // Track positions to skip table content
        let table_pattern = regex::Regex::new(r#"<w:tbl[^>]*>.*?</w:tbl>"#).unwrap();
        let mut last_end = 0usize;

        for table_cap in table_pattern.captures(&xml_str) {
            let table_range = match table_cap.get(0) {
                Some(m) => m.start()..m.end(),
                None => continue,
            };

            // Parse paragraphs before this table
            let before_table = &xml_str[last_end..table_range.start];
            for para_cap in para_pattern.captures(before_table) {
                if let Some(para_xml) = para_cap.get(1) {
                    if let Some(para) = self.parse_paragraph(para_xml.as_str()) {
                        self.paragraphs.push(para);
                    }
                }
            }

            // Skip table content (already parsed)
            last_end = table_range.end;
        }

        // Parse paragraphs after last table
        let after_tables = &xml_str[last_end..];
        for para_cap in para_pattern.captures(after_tables) {
            if let Some(para_xml) = para_cap.get(1) {
                if let Some(para) = self.parse_paragraph(para_xml.as_str()) {
                    self.paragraphs.push(para);
                }
            }
        }

        // Parse inline images in the document
        self.parse_inline_images(&xml_str, package);

        self.text = self.paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(())
    }

    /// Parse a single paragraph from XML
    fn parse_paragraph(&self, para_xml: &str) -> Option<Paragraph> {
        let mut paragraph = Paragraph::default();

        // Parse runs within paragraph
        let run_pattern = regex::Regex::new(r#"<w:r[^>]*>(.*?)</w:r>"#).unwrap();
        for run_cap in run_pattern.captures(para_xml) {
            let run_xml = match run_cap.get(1) {
                Some(m) => m.as_str(),
                None => continue,
            };

            let mut run = Run::default();

            // Parse text in run
            let text_pattern = regex::Regex::new(r#"<w:t[^>]*>([^<]*)</w:t>"#).unwrap();
            for text_cap in text_pattern.captures(run_xml) {
                if let Some(text_match) = text_cap.get(1) {
                    run.text = text_match.as_str().to_string();
                    break;
                }
            }

            // Parse run properties
            let rpr_pattern = regex::Regex::new(r#"<w:rPr[^>]*>(.*?)</w:rPr>"#).unwrap();
            if let Some(rpr_cap) = rpr_pattern.captures(run_xml) {
                if let Some(rpr_xml) = rpr_cap.get(1) {
                    Self::parse_run_properties(rpr_xml.as_str(), &mut run.properties);
                }
            }

            if !run.text.is_empty() || !run.properties.is_default() {
                paragraph.runs.push(run);
            }
        }

        if paragraph.runs.is_empty() {
            return None;
        }

        paragraph.text = paragraph.runs
            .iter()
            .map(|r| r.text.clone())
            .collect();
        Some(paragraph)
    }

    /// Parse tables from document XML
    fn parse_tables(&mut self, xml_str: &str, _package: &OpcPackage) {
        let table_pattern = regex::Regex::new(r#"<w:tbl[^>]*>(.*?)</w:tbl>"#).unwrap();

        for table_cap in table_pattern.captures(xml_str) {
            let table_xml = match table_cap.get(1) {
                Some(m) => m.as_str(),
                None => continue,
            };

            let mut table = Table::default();
            table.properties = self.parse_table_properties(table_xml);

            // Parse table rows
            let row_pattern = regex::Regex::new(r#"<w:tr[^>]*>(.*?)</w:tr>"#).unwrap();
            for row_cap in row_pattern.captures(table_xml) {
                let row_xml = match row_cap.get(1) {
                    Some(m) => m.as_str(),
                    None => continue,
                };

                let mut row = TableRow::default();
                row.properties = self.parse_table_row_properties(row_xml);

                // Parse table cells
                let cell_pattern = regex::Regex::new(r#"<w:tc[^>]*>(.*?)</w:tc>"#).unwrap();
                for cell_cap in cell_pattern.captures(row_xml) {
                    let cell_xml = match cell_cap.get(1) {
                        Some(m) => m.as_str(),
                        None => continue,
                    };

                    let cell = self.parse_table_cell(cell_xml);
                    row.cells.push(cell);
                }

                if !row.cells.is_empty() {
                    table.rows.push(row);
                }
            }

            if !table.rows.is_empty() {
                self.tables.push(table);
            }
        }
    }

    /// Parse table properties from XML
    fn parse_table_properties(&self, table_xml: &str) -> TableProperties {
        let mut props = TableProperties::default();

        // Parse table width
        if let Some(caps) = regex::Regex::new(r#"<w:tblW[^>]*w:w="([^"]*)""#).unwrap().captures(table_xml) {
            if let Some(m) = caps.get(1) {
                props.width = m.as_str().parse().ok();
            }
        }

        // Parse table alignment
        if let Some(caps) = regex::Regex::new(r#"<w:jc[^>]*w:val="([^"]*)""#).unwrap().captures(table_xml) {
            if let Some(m) = caps.get(1) {
                props.alignment = Some(m.as_str().to_string());
            }
        }

        // Parse table indent
        if let Some(caps) = regex::Regex::new(r#"<w:tblInd[^>]*w:w="([^"]*)""#).unwrap().captures(table_xml) {
            if let Some(m) = caps.get(1) {
                props.indent = m.as_str().parse().ok();
            }
        }

        // Parse table layout
        if let Some(caps) = regex::Regex::new(r#"<w:tblLayout[^>]*w:type="([^"]*)""#).unwrap().captures(table_xml) {
            if let Some(m) = caps.get(1) {
                props.layout = Some(m.as_str().to_string());
            }
        }

        // Parse table borders
        props.borders = self.parse_table_borders(table_xml);

        props
    }

    /// Parse table borders from XML
    fn parse_table_borders(&self, table_xml: &str) -> TableBorders {
        let mut borders = TableBorders::default();

        // Helper to parse individual border
        let parse_border = |xml: &str, tag: &str| -> Option<TableBorder> {
            // Simplified border parsing
            if let Some(caps) = regex::Regex::new(&format!(r#"<w:{}[^>]*w:val="([^"]*)"[^>]*w:sz="([^"]*)"[^>]*w:fill="([^"]*)""#, tag)).unwrap().captures(xml) {
                return Some(TableBorder {
                    style: caps.get(1).map(|m| m.as_str().to_string()),
                    size: caps.get(2).and_then(|m| m.as_str().parse().ok()),
                    color: caps.get(3).map(|m| {
                        let color = m.as_str().to_string();
                        if color.is_empty() { None } else { Some(color) }
                    }).unwrap_or(None),
                });
            }
            None
        };

        borders.top = parse_border(table_xml, "top");
        borders.bottom = parse_border(table_xml, "bottom");
        borders.left = parse_border(table_xml, "left");
        borders.right = parse_border(table_xml, "right");
        borders.inside_horizontal = parse_border(table_xml, "insideH");
        borders.inside_vertical = parse_border(table_xml, "insideV");

        borders
    }

    /// Parse table row properties from XML
    fn parse_table_row_properties(&self, row_xml: &str) -> TableRowProperties {
        let mut props = TableRowProperties::default();

        // Parse row height
        if let Some(caps) = regex::Regex::new(r#"<w:trHeight[^>]*w:h="([^"]*)""#).unwrap().captures(row_xml) {
            if let Some(m) = caps.get(1) {
                props.height = m.as_str().parse().ok();
            }
        }

        // Parse height rule
        if let Some(caps) = regex::Regex::new(r#"<w:trHeight[^>]*w:hrule="([^"]*)""#).unwrap().captures(row_xml) {
            if let Some(m) = caps.get(1) {
                props.height_rule = Some(m.as_str().to_string());
            }
        }

        // Check if header row
        props.is_header = row_xml.contains("<w:tblHeader");

        props
    }

    /// Parse table cell from XML
    fn parse_table_cell(&self, cell_xml: &str) -> TableCell {
        let mut cell = TableCell::default();

        // Parse cell width
        if let Some(caps) = regex::Regex::new(r#"<w:tcW[^>]*w:w="([^"]*)""#).unwrap().captures(cell_xml) {
            if let Some(m) = caps.get(1) {
                cell.width = m.as_str().parse().ok();
            }
        }

        // Parse vertical merge
        if let Some(caps) = regex::Regex::new(r#"<w:vMerge[^>]*w:val="([^"]*)""#).unwrap().captures(cell_xml) {
            if let Some(m) = caps.get(1) {
                cell.vertical_merge = Some(match m.as_str() {
                    "restart" => 1,
                    "continue" => -1,
                    _ => 0,
                });
            }
        }

        // Parse horizontal merge
        if let Some(caps) = regex::Regex::new(r#"<w:hMerge[^>]*w:val="([^"]*)""#).unwrap().captures(cell_xml) {
            if let Some(m) = caps.get(1) {
                cell.horizontal_merge = Some(match m.as_str() {
                    "restart" => 1,
                    "continue" => -1,
                    _ => 0,
                });
            }
        }

        // Parse paragraphs in cell
        let para_pattern = regex::Regex::new(r#"<w:p[^>]*>(.*?)</w:p>"#).unwrap();
        for para_cap in para_pattern.captures(cell_xml) {
            if let Some(para_xml) = para_cap.get(1) {
                if let Some(para) = self.parse_paragraph(para_xml.as_str()) {
                    cell.paragraphs.push(para);
                }
            }
        }

        cell
    }

    /// Parse inline images from document XML
    fn parse_inline_images(&mut self, xml_str: &str, package: &OpcPackage) {
        // Parse drawing elements with inline pictures
        let drawing_pattern = regex::Regex::new(
            r#"<w:drawing[^>]*>.*?<wp:inline[^>]*>.*?(?:<wp:blipFill>.*?<a:blip[^>]*r:embed="([^"]*)"[^>]*>.*?</wp:blipFill>).*?</wp:inline>.*?</w:drawing>"#
        ).unwrap();

        for cap in drawing_pattern.captures(xml_str) {
            if let Some(embed_id) = cap.get(1) {
                let embed_id = embed_id.as_str().to_string();

                // Look up the image in relationships
                let image = self.resolve_image_reference(package, &embed_id);
                if let Some(img) = image {
                    self.images.push(img);
                }
            }
        }
    }

    /// Resolve image reference from relationships
    fn resolve_image_reference(&self, package: &OpcPackage, embed_id: &str) -> Option<DocumentImage> {
        // Get document relationships
        let doc_rels_part = "/word/_rels/document.xml.rels";

        let rels_part = if let Some(part) = package.get_part(doc_rels_part) {
            part
        } else {
            return None;
        };

        let rels_xml = String::from_utf8_lossy(&rels_part.data);

        // Find the relationship with this ID
        // Escape special regex characters in embed_id
        let escaped_id = regex::escape(embed_id);
        let rel_pattern = format!(
            r#"<Relationship[^>]*Id="{}"[^>]*Target="([^"]*)""#,
            escaped_id
        );

        if let Some(caps) = regex::Regex::new(&rel_pattern).unwrap().captures(&rels_xml) {
            if let Some(target) = caps.get(1) {
                let target = target.as_str().to_string();
                let image_path = format!("/word/{}", target);

                // Get the image part
                if let Some(image_part) = package.get_part(&image_path) {
                    let _content_type = &image_part.content_type;

                    return Some(DocumentImage {
                        id: embed_id.to_string(),
                        path: target,
                        original_width: None,
                        original_height: None,
                        desired_width: None,
                        desired_height: None,
                        scale_x: None,
                        scale_y: None,
                        title: None,
                        alt_description: None,
                        is_linked: false,
                    });
                }
            }
        }

        None
    }

    /// Parse run properties from XML
    fn parse_run_properties(xml: &str, props: &mut RunProperties) {
        // Bold
        if let Some(caps) = regex::Regex::new(r#"<w:b[^>]*val="([^"]*)""#).unwrap().captures(xml) {
            if let Some(m) = caps.get(1) {
                props.bold = Some(m.as_str() != "0");
            }
        }
        
        // Italic
        if let Some(caps) = regex::Regex::new(r#"<w:i[^>]*val="([^"]*)""#).unwrap().captures(xml) {
            if let Some(m) = caps.get(1) {
                props.italic = Some(m.as_str() != "0");
            }
        }
        
        // Underline
        if let Some(caps) = regex::Regex::new(r#"<w:u[^>]*val="([^"]*)""#).unwrap().captures(xml) {
            if let Some(m) = caps.get(1) {
                props.underline = Some(m.as_str().to_string());
            }
        }
        
        // Font size
        if let Some(caps) = regex::Regex::new(r#"<w:sz[^>]*val="(\d+)""#).unwrap().captures(xml) {
            if let Some(m) = caps.get(1) {
                if let Ok(size) = m.as_str().parse::<i32>() {
                    props.font_size = Some(size / 2);
                }
            }
        }
        
        // Color
        if let Some(caps) = regex::Regex::new(r#"<w:color[^>]*val="([^"]*)""#).unwrap().captures(xml) {
            if let Some(m) = caps.get(1) {
                props.color = Some(m.as_str().to_string());
            }
        }
        
        // Font name
        if let Some(caps) = regex::Regex::new(r#"<w:rFonts[^>]*w:ascii="([^"]*)""#).unwrap().captures(xml) {
            if let Some(m) = caps.get(1) {
                props.font_name = Some(m.as_str().to_string());
            }
        }
    }

    /// Parse styles (word/styles.xml)
    fn parse_styles(&mut self, package: &OpcPackage) -> Result<(), OoxmlError> {
        let styles_part_name = "/word/styles.xml";
        
        let styles_part = if let Some(part) = package.get_part(styles_part_name) {
            part
        } else {
            return Ok(());
        };

        let xml_str = String::from_utf8_lossy(&styles_part.data);
        
        // Parse style elements
        let style_pattern = regex::Regex::new(
            r#"<w:style[^>]*w:styleId="([^"]*)"[^>]*w:type="([^"]*)"[^>]*>(.*?)</w:style>"#
        ).unwrap();
        
        for cap in style_pattern.captures(&xml_str) {
            let style_id = match cap.get(1) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };
            
            let style_type = match cap.get(2) {
                Some(m) => m.as_str().to_string(),
                None => "paragraph".to_string(),
            };
            
            let style_xml = match cap.get(3) {
                Some(m) => m.as_str(),
                None => "",
            };
            
            let mut style = Style {
                id: style_id.clone(),
                name: None,
                style_type,
                based_on: None,
                paragraph_properties: ParagraphProperties::default(),
                run_properties: RunProperties::default(),
                is_default: false,
            };
            
            // Get style name
            if let Some(name_cap) = regex::Regex::new(r#"<w:name[^>]*w:val="([^"]*)""#).unwrap().captures(style_xml) {
                if let Some(m) = name_cap.get(1) {
                    style.name = Some(m.as_str().to_string());
                }
            }
            
            // Get basedOn
            if let Some(based_cap) = regex::Regex::new(r#"<w:basedOn[^>]*w:val="([^"]*)""#).unwrap().captures(style_xml) {
                if let Some(m) = based_cap.get(1) {
                    style.based_on = Some(m.as_str().to_string());
                }
            }
            
            // Check if default
            if regex::Regex::new(r#"w:default="1""#).unwrap().is_match(style_xml) {
                style.is_default = true;
            }
            
            self.styles.insert(style_id, style);
        }

        Ok(())
    }

    /// Parse theme (word/theme/theme1.xml)
    fn parse_theme(&mut self, package: &OpcPackage) -> Result<(), OoxmlError> {
        let theme_part_names = ["/word/theme/theme1.xml", "/word/theme/theme.xml", "/word/themes/theme1.xml"];
        
        let theme_part = theme_part_names.iter()
            .find(|&name| package.get_part(name).is_some())
            .and_then(|name| package.get_part(name));

        if theme_part.is_none() {
            return Ok(());
        }

        let _theme_part = theme_part.unwrap();
        let theme = Theme {
            name: "Office Theme".to_string(),
            colors: HashMap::new(),
            fonts: ThemeFonts {
                major_font: "Calibri".to_string(),
                minor_font: "Calibri".to_string(),
                symbol_font: "Symbol".to_string(),
            },
        };

        self.theme = Some(theme);
        Ok(())
    }

    /// Parse core properties (docProps/core.xml)
    fn parse_core_properties(&mut self, package: &OpcPackage) -> Result<(), OoxmlError> {
        let core_part_name = "/docProps/core.xml";
        
        let core_part = if let Some(part) = package.get_part(core_part_name) {
            part
        } else {
            return Ok(());
        };

        let xml_str = String::from_utf8_lossy(&core_part.data);
        let mut props = CoreProperties::default();

        // Parse title
        if let Some(caps) = regex::Regex::new(r#"<dc:title[^>]*>([^<]*)</dc:title>"#).unwrap().captures(&xml_str) {
            if let Some(m) = caps.get(1) {
                props.title = Some(m.as_str().to_string());
            }
        }
        
        // Parse creator
        if let Some(caps) = regex::Regex::new(r#"<dc:creator[^>]*>([^<]*)</dc:creator>"#).unwrap().captures(&xml_str) {
            if let Some(m) = caps.get(1) {
                props.creator = Some(m.as_str().to_string());
            }
        }
        
        // Parse created
        if let Some(caps) = regex::Regex::new(r#"<dcterms:created[^>]*>([^<]*)</dcterms:created>"#).unwrap().captures(&xml_str) {
            if let Some(m) = caps.get(1) {
                props.created = Some(m.as_str().to_string());
            }
        }
        
        // Parse modified
        if let Some(caps) = regex::Regex::new(r#"<dcterms:modified[^>]*>([^<]*)</dcterms:modified>"#).unwrap().captures(&xml_str) {
            if let Some(m) = caps.get(1) {
                props.modified = Some(m.as_str().to_string());
            }
        }

        self.core_properties = Some(props);
        Ok(())
    }

    /// Parse numbering definitions (word/numbering.xml)
    fn parse_numbering(&mut self, package: &OpcPackage) -> Result<(), OoxmlError> {
        let numbering_part_name = "/word/numbering.xml";

        let numbering_part = if let Some(part) = package.get_part(numbering_part_name) {
            part
        } else {
            return Ok(());
        };

        let xml_str = String::from_utf8_lossy(&numbering_part.data);
        let mut numbering = Numbering::default();

        // Parse abstract numbering definitions
        let abstract_num_pattern = regex::Regex::new(
            r#"<w:abstractNum[^>]*w:abstractNumId="([^"]*)"[^>]*>(.*?)</w:abstractNum>"#
        ).unwrap();

        for cap in abstract_num_pattern.captures(&xml_str) {
            let abstract_num_id = match cap.get(1) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };

            let abstract_num_xml = match cap.get(2) {
                Some(m) => m.as_str(),
                None => continue,
            };

            let mut abstract_num = AbstractNumDef {
                abstract_num_id,
                levels: Vec::new(),
            };

            // Parse list levels (lvl)
            let lvl_pattern = regex::Regex::new(r#"<w:lvl[^>]*w:ilvl="([^"]*)"[^>]*>(.*?)</w:lvl>"#).unwrap();
            for lvl_cap in lvl_pattern.captures(abstract_num_xml) {
                let level_idx = match lvl_cap.get(1) {
                    Some(m) => m.as_str().parse().unwrap_or(0),
                    None => 0,
                };

                let lvl_xml = match lvl_cap.get(2) {
                    Some(m) => m.as_str(),
                    None => continue,
                };

                let mut level = ListLevel {
                    level: level_idx,
                    format: String::new(),
                    text: String::new(),
                    start_value: 1,
                    paragraph_properties: ParagraphProperties::default(),
                    run_properties: RunProperties::default(),
                };

                // Parse format
                if let Some(caps) = regex::Regex::new(r#"<w:numFmt[^>]*w:val="([^"]*)""#).unwrap().captures(lvl_xml) {
                    if let Some(m) = caps.get(1) {
                        level.format = m.as_str().to_string();
                    }
                }

                // Parse text
                if let Some(caps) = regex::Regex::new(r#"<w:lvlText[^>]*w:val="([^"]*)""#).unwrap().captures(lvl_xml) {
                    if let Some(m) = caps.get(1) {
                        level.text = m.as_str().to_string();
                    }
                }

                // Parse start value
                if let Some(caps) = regex::Regex::new(r#"<w:startOverride[^>]*w:val="([^"]*)""#).unwrap().captures(lvl_xml) {
                    if let Some(m) = caps.get(1) {
                        level.start_value = m.as_str().parse().unwrap_or(1);
                    }
                }

                abstract_num.levels.push(level);
            }

            numbering.abstract_num_defs.push(abstract_num);
        }

        // Parse numbering instances
        let num_pattern = regex::Regex::new(r#"<w:num[^>]*w:numId="([^"]*)"[^>]*>(.*?)</w:num>"#).unwrap();
        for cap in num_pattern.captures(&xml_str) {
            let num_id = match cap.get(1) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };

            let num_xml = match cap.get(2) {
                Some(m) => m.as_str(),
                None => continue,
            };

            let mut num_instance = NumInstance {
                num_id,
                abstract_num_id: String::new(),
                overrides: Vec::new(),
            };

            // Parse abstract num ID reference
            if let Some(caps) = regex::Regex::new(r#"<w:abstractNumId[^>]*w:val="([^"]*)""#).unwrap().captures(num_xml) {
                if let Some(m) = caps.get(1) {
                    num_instance.abstract_num_id = m.as_str().to_string();
                }
            }

            numbering.num_instances.push(num_instance);
        }

        self.numbering.push(numbering);
        Ok(())
    }

    /// Parse headers and footers
    fn parse_headers_footers(&mut self, package: &OpcPackage) -> Result<(), OoxmlError> {
        // Get document relationships to find header/footer references
        let doc_rels_part = "/word/_rels/document.xml.rels";

        let rels_part = if let Some(part) = package.get_part(doc_rels_part) {
            part
        } else {
            return Ok(());
        };

        let rels_xml = String::from_utf8_lossy(&rels_part.data);

        // Header pattern in relationships
        let header_pattern = regex::Regex::new(
            r#"<Relationship[^>]*Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"[^>]*Id="([^"]*)"[^>]*Target="([^"]*)""#
        ).unwrap();

        for cap in header_pattern.captures(&rels_xml) {
            let header_id = match cap.get(1) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };

            let target = match cap.get(2) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };

            let header_part_name = format!("/word/{}", target);
            if let Some(header_part) = package.get_part(&header_part_name) {
                let header_xml = String::from_utf8_lossy(&header_part.data);
                let paragraphs = self.parse_header_footer_content(&header_xml);

                let header = Header {
                    id: header_id,
                    header_type: Self::determine_header_type(&target),
                    paragraphs,
                    images: Vec::new(),
                };
                self.headers.push(header);
            }
        }

        // Footer pattern in relationships
        let footer_pattern = regex::Regex::new(
            r#"<Relationship[^>]*Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"[^>]*Id="([^"]*)"[^>]*Target="([^"]*)""#
        ).unwrap();

        for cap in footer_pattern.captures(&rels_xml) {
            let footer_id = match cap.get(1) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };

            let target = match cap.get(2) {
                Some(m) => m.as_str().to_string(),
                None => continue,
            };

            let footer_part_name = format!("/word/{}", target);
            if let Some(footer_part) = package.get_part(&footer_part_name) {
                let footer_xml = String::from_utf8_lossy(&footer_part.data);
                let paragraphs = self.parse_header_footer_content(&footer_xml);

                let footer = Footer {
                    id: footer_id,
                    footer_type: Self::determine_footer_type(&target),
                    paragraphs,
                    images: Vec::new(),
                };
                self.footers.push(footer);
            }
        }

        Ok(())
    }

    /// Determine header type from target filename
    fn determine_header_type(target: &str) -> String {
        if target.contains("header1") {
            "first".to_string()
        } else if target.contains("even") {
            "even".to_string()
        } else {
            "default".to_string()
        }
    }

    /// Determine footer type from target filename
    fn determine_footer_type(target: &str) -> String {
        if target.contains("footer1") {
            "first".to_string()
        } else if target.contains("even") {
            "even".to_string()
        } else {
            "default".to_string()
        }
    }

    /// Parse content from header/footer XML
    fn parse_header_footer_content(&self, xml_str: &str) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();

        let para_pattern = regex::Regex::new(r#"<w:p[^>]*>(.*?)</w:p>"#).unwrap();
        for para_cap in para_pattern.captures(xml_str) {
            if let Some(para_xml) = para_cap.get(1) {
                if let Some(para) = self.parse_paragraph(para_xml.as_str()) {
                    paragraphs.push(para);
                }
            }
        }

        paragraphs
    }

    /// Parse footnotes and endnotes
    fn parse_footnotes_endnotes(&mut self, package: &OpcPackage) -> Result<(), OoxmlError> {
        // Parse footnotes
        let footnote_part_names = ["/word/footnotes.xml", "/word/footnote.xml"];

        for part_name in footnote_part_names.iter() {
            if let Some(footnote_part) = package.get_part(part_name) {
                let xml_str = String::from_utf8_lossy(&footnote_part.data);

                let footnote_pattern = regex::Regex::new(
                    r#"<w:footnote[^>]*w:id="([^"]*)"[^>]*>(.*?)</w:footnote>"#
                ).unwrap();

                for cap in footnote_pattern.captures(&xml_str) {
                    let footnote_id = match cap.get(1) {
                        Some(m) => m.as_str().to_string(),
                        None => continue,
                    };

                    let footnote_xml = match cap.get(2) {
                        Some(m) => m.as_str(),
                        None => continue,
                    };

                    let mut footnote = Footnote {
                        id: footnote_id,
                        footnote_type: None,
                        paragraphs: Vec::new(),
                    };

                    // Determine footnote type
                    if footnote_xml.contains("<w:footnoteRef") {
                        footnote.footnote_type = Some("normal".to_string());
                    } else if footnote_xml.contains("<w:separator") {
                        footnote.footnote_type = Some("separator".to_string());
                    } else if footnote_xml.contains("<w:continuationSeparator") {
                        footnote.footnote_type = Some("continuationSeparator".to_string());
                    }

                    // Parse paragraphs in footnote
                    let para_pattern = regex::Regex::new(r#"<w:p[^>]*>(.*?)</w:p>"#).unwrap();
                    for para_cap in para_pattern.captures(footnote_xml) {
                        if let Some(para_xml) = para_cap.get(1) {
                            if let Some(para) = self.parse_paragraph(para_xml.as_str()) {
                                footnote.paragraphs.push(para);
                            }
                        }
                    }

                    self.footnotes.push(footnote);
                }
            }
        }

        // Parse endnotes
        let endnote_part_names = ["/word/endnotes.xml", "/word/endnote.xml"];

        for part_name in endnote_part_names.iter() {
            if let Some(endnote_part) = package.get_part(part_name) {
                let xml_str = String::from_utf8_lossy(&endnote_part.data);

                let endnote_pattern = regex::Regex::new(
                    r#"<w:endnote[^>]*w:id="([^"]*)"[^>]*>(.*?)</w:endnote>"#
                ).unwrap();

                for cap in endnote_pattern.captures(&xml_str) {
                    let endnote_id = match cap.get(1) {
                        Some(m) => m.as_str().to_string(),
                        None => continue,
                    };

                    let endnote_xml = match cap.get(2) {
                        Some(m) => m.as_str(),
                        None => continue,
                    };

                    let mut endnote = Endnote {
                        id: endnote_id,
                        endnote_type: None,
                        paragraphs: Vec::new(),
                    };

                    // Determine endnote type
                    if endnote_xml.contains("<w:endnoteRef") {
                        endnote.endnote_type = Some("normal".to_string());
                    } else if endnote_xml.contains("<w:separator") {
                        endnote.endnote_type = Some("separator".to_string());
                    } else if endnote_xml.contains("<w:continuationSeparator") {
                        endnote.endnote_type = Some("continuationSeparator".to_string());
                    }

                    // Parse paragraphs in endnote
                    for para_cap in endnote_pattern.captures(endnote_xml) {
                        if let Some(para_xml) = para_cap.get(1) {
                            if let Some(para) = self.parse_paragraph(para_xml.as_str()) {
                                endnote.paragraphs.push(para);
                            }
                        }
                    }

                    self.endnotes.push(endnote);
                }
            }
        }

        Ok(())
    }
}

impl RunProperties {
    /// Check if properties are default (no formatting)
    fn is_default(&self) -> bool {
        self.bold.is_none() 
            && self.italic.is_none() 
            && self.underline.is_none() 
            && self.font_size.is_none() 
            && self.font_name.is_none() 
            && self.color.is_none() 
            && self.background_color.is_none()
    }
}

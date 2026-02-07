//! OPC (Open Packaging Conventions) Package Reader
//! Reads and parses ZIP-based Office Open XML documents (.docx, .xlsx, .pptx)

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};
use zip::result::ZipResult;
use zip::ZipArchive;

use super::error::OoxmlError;
use super::types::{ContentType, PackagePart, Relationship, RelationshipType};

/// OPC Package Reader
#[derive(Debug, Clone, Default)]
pub struct OpcPackage {
    /// All parts in the package indexed by part name
    pub parts: HashMap<String, PackagePart>,
    /// All content types indexed by part name
    pub content_types: HashMap<String, ContentType>,
    /// Root relationships (_rels/.rels)
    pub root_relationships: Vec<Relationship>,
    /// Relationships indexed by source part name
    pub relationships: HashMap<String, Vec<Relationship>>,
}

impl OpcPackage {
    /// Create a new OpcPackage from ZIP file data
    pub fn new(file_data: &[u8]) -> Result<Self, OoxmlError> {
        let reader = Cursor::new(file_data);
        let mut archive = ZipArchive::new(reader)?;

        let mut package = OpcPackage {
            parts: HashMap::new(),
            content_types: HashMap::new(),
            root_relationships: Vec::new(),
            relationships: HashMap::new(),
        };

        // Parse [Content_Types].xml
        package.parse_content_types(&mut archive)?;

        // Parse _rels/.rels for root relationships
        package.parse_root_relationships(&mut archive)?;

        // Parse relationships files for each part
        package.parse_all_relationships(&mut archive)?;

        // Extract all parts from the archive
        package.extract_parts(&mut archive)?;

        Ok(package)
    }

    /// Helper function to read file content from archive (tries multiple path variants)
    fn read_file_from_archive<R: Read + Seek>(
        archive: &mut ZipArchive<R>,
        paths: &[&str],
    ) -> Option<Vec<u8>> {
        for path in paths {
            if let Ok(mut file) = archive.by_name(path) {
                let mut data = Vec::new();
                if file.read_to_end(&mut data).is_ok() {
                    return Some(data);
                }
            }
            // Try with leading slash
            let alt_path = format!("/{}", path);
            if let Ok(mut file) = archive.by_name(&alt_path) {
                let mut data = Vec::new();
                if file.read_to_end(&mut data).is_ok() {
                    return Some(data);
                }
            }
        }
        None
    }

    /// Parse [Content_Types].xml to get content types for all parts
    fn parse_content_types<R: Read + Seek>(&mut self, archive: &mut ZipArchive<R>) -> ZipResult<()> {
        if let Some(xml_data) = Self::read_file_from_archive(archive, &["[Content_Types].xml"]) {
            self.parse_content_types_xml(&xml_data);
        }
        Ok(())
    }

    /// Parse the content types XML data using regex
    fn parse_content_types_xml(&mut self, xml_data: &[u8]) {
        let xml_str = String::from_utf8_lossy(xml_data);
        
        // Parse Override elements
        // <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
        let override_pattern = regex::Regex::new(r#"<Override\s+PartName="([^"]+)"\s+ContentType="([^"]+)"\s*/>"#).unwrap();
        
        for cap in override_pattern.captures_iter(&xml_str) {
            let part_name = cap[1].to_string();
            let content_type_str = cap[2].to_string();
            
            let content_type = ContentType::from_string(&content_type_str);
            self.content_types.insert(part_name, content_type);
        }

        // Parse Default elements
        // <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
        let default_pattern = regex::Regex::new(r#"<Default\s+Extension="([^"]+)"\s+ContentType="([^"]+)"\s*/>"#).unwrap();
        
        for cap in default_pattern.captures_iter(&xml_str) {
            let extension = cap[1].to_string();
            let content_type_str = cap[2].to_string();
            
            let content_type = ContentType::from_string(&content_type_str);
            self.content_types.insert(format!("/{}", extension), content_type);
        }
    }

    /// Parse _rels/.rels for root package relationships
    fn parse_root_relationships<R: Read + Seek>(&mut self, archive: &mut ZipArchive<R>) -> ZipResult<()> {
        if let Some(xml_data) = Self::read_file_from_archive(archive, &["_rels/.rels"]) {
            self.root_relationships = Self::parse_relationships_xml(&xml_data);
        }
        Ok(())
    }

    /// Parse relationships XML using regex
    fn parse_relationships_xml(xml_data: &[u8]) -> Vec<Relationship> {
        let xml_str = String::from_utf8_lossy(xml_data);
        let mut relationships = Vec::new();
        
        // <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
        let rel_pattern = regex::Regex::new(r#"<Relationship\s+Id="([^"]+)"\s+Type="([^"]+)"\s+Target="([^"]+)"\s*/>"#).unwrap();
        
        for cap in rel_pattern.captures_iter(&xml_str) {
            let id = cap[1].to_string();
            let type_uri = cap[2].to_string();
            let target = cap[3].to_string();
            
            let rel_type = RelationshipType::from_string(&type_uri);
            
            relationships.push(Relationship {
                id,
                relationship_type: rel_type,
                target,
                target_mode: None,
            });
        }

        relationships
    }

    /// Parse relationships files for each part
    fn parse_all_relationships<R: Read + Seek>(&mut self, archive: &mut ZipArchive<R>) -> ZipResult<()> {
        // Find all .rels files by looking at the content types
        let mut rel_files: Vec<String> = self.content_types.keys()
            .filter(|name| name.ends_with(".rels"))
            .cloned()
            .collect();

        // Also check for word/_rels/*.rels files
        for part_name in self.content_types.keys().cloned().collect::<Vec<_>>() {
            if part_name.starts_with("word/") && !part_name.ends_with(".rels") {
                // This is a part that might have relationships
                let rel_path = format!("{}/_rels/{}.rels", 
                    part_name.rsplit_once('/').map_or("", |(p, _)| p),
                    part_name.rsplit('/').next().unwrap_or(&part_name));
                if !rel_files.contains(&rel_path) {
                    rel_files.push(rel_path);
                }
            }
        }

        for rel_file in rel_files {
            if let Some(xml_data) = Self::read_file_from_archive(archive, &[&rel_file]) {
                let relationships = Self::parse_relationships_xml(&xml_data);
                if !relationships.is_empty() {
                    // Store relationships keyed by the source part (derive from .rels path)
                    let source_part = rel_file
                        .strip_suffix(".rels")
                        .unwrap_or(&rel_file)
                        .replace("/_rels/", "/");
                    self.relationships.insert(source_part, relationships);
                }
            }
        }

        Ok(())
    }

    /// Extract all parts from the archive
    fn extract_parts<R: Read + Seek>(&mut self, archive: &mut ZipArchive<R>) -> ZipResult<()> {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            // Skip special files
            if name.starts_with('_') || name == "[Content_Types].xml" {
                continue;
            }

            // Get content type from our map
            let content_type = self.content_types.get(&name).cloned();

            if let Some(ct) = content_type {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;

                self.parts.insert(name.clone(), PackagePart {
                    name: name.clone(),
                    content_type: ct,
                    data,
                });
            }
        }

        Ok(())
    }

    /// Get a part by name
    pub fn get_part(&self, name: &str) -> Option<&PackagePart> {
        self.parts.get(name)
    }

    /// Get content type for a part
    pub fn get_content_type(&self, name: &str) -> Option<ContentType> {
        self.content_types.get(name).cloned()
    }

    /// Get relationships for a source part
    pub fn get_relationships(&self, source: &str) -> Option<&Vec<Relationship>> {
        self.relationships.get(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relationships_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
    <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="word/styles.xml"/>
</Relationships>"#;

        let relationships = OpcPackage::parse_relationships_xml(xml.as_bytes());
        assert_eq!(relationships.len(), 2);
        assert_eq!(relationships[0].id, "rId1");
        assert_eq!(relationships[0].target, "word/document.xml");
    }
}

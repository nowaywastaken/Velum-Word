use crate::piece_tree::{PieceTree, Piece, BufferId, TextAttributes};
use crate::ooxml::document::WordDocument;
use crate::ooxml::types::RunProperties;

/// Converts a parsed WordDocument into a PieceTree
pub fn ooxml_to_piece_tree(doc: &WordDocument) -> PieceTree {
    let mut combined_text = String::new();
    let mut pieces = Vec::new();
    
    let mut current_offset = 0;

    for (i, paragraph) in doc.paragraphs.iter().enumerate() {
        // Process each run in the paragraph
        for run in &paragraph.runs {
            if run.text.is_empty() {
                continue;
            }

            let text = &run.text;
            let len = text.len(); // Byte length
            let char_len = text.chars().count();
            
            // Append text to the "buffer" string
            combined_text.push_str(text);
            
            // Create a piece for this run
            let attributes = convert_run_properties(&run.properties);
            
            pieces.push(Piece::new_with_attrs(
                current_offset,
                len,
                BufferId::ORIGINAL, // We assume all loaded text is "Original"
                char_len,
                Some(attributes),
            ));
            
            current_offset += len;
        }

        // Add newline between paragraphs (except possibly the last one, but usually documents end with newline)
        if i < doc.paragraphs.len() - 1 {
            combined_text.push('\n');
            pieces.push(Piece::new(
                current_offset,
                1,
                BufferId::ORIGINAL,
                1,
            ));
            current_offset += 1;
        }
    }

    // Construct the PieceTree using the public constructor
    // The buffer list should contain the combined text.
    // PieceTree logic usually expects buffers[0] to be the initial loaded content.
    let buffers = vec![combined_text];
    
    PieceTree::from_loaded_data(pieces, buffers)
}

/// Convert OOXML RunProperties to PieceTree TextAttributes
fn convert_run_properties(props: &RunProperties) -> TextAttributes {
    let mut attrs = TextAttributes::default();
    
    attrs.bold = props.bold;
    attrs.italic = props.italic;
    
    // Underline mapping
    if let Some(u) = &props.underline {
        if u != "none" {
            attrs.underline = Some(true);
        }
    }
    
    // Font size: OOXML in half-points -> Points (u16)
    if let Some(half_pts) = props.font_size {
        if half_pts > 0 {
            attrs.font_size = Some((half_pts / 2) as u16);
        }
    }
    
    // Font family
    attrs.font_family = props.font_name.clone();
    
    // Colors
    attrs.foreground = props.color.clone().map(|c| if !c.starts_with("#") { format!("#{}", c) } else { c });
    attrs.background = props.background_color.clone().map(|c| if !c.starts_with("#") { format!("#{}", c) } else { c });

    attrs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ooxml::types::{Paragraph, Run};
    use crate::ooxml::document::WordDocument;
    use std::collections::HashMap;

    #[test]
    fn test_ooxml_to_piece_tree_conversion() {
        let mut doc = WordDocument {
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

        // Create a paragraph with mixed formatting
        let mut p1 = Paragraph::default();
        
        let mut r1 = Run::default();
        r1.text = "Hello ".to_string();
        r1.properties.bold = Some(true);
        
        let mut r2 = Run::default();
        r2.text = "World".to_string();
        r2.properties.italic = Some(true);
        r2.properties.font_size = Some(24); // 12pt
        
        p1.runs.push(r1);
        p1.runs.push(r2);
        
        doc.paragraphs.push(p1);
        
        // Convert
        let pt = ooxml_to_piece_tree(&doc);
        
        // Verify text content
        assert_eq!(pt.get_text(), "Hello World"); // Note: might have trailing newline if logic adds it
        // Wait, logic adds newline only if i < len - 1. P1 is last, so no newline.
        
        // Verify pieces
        assert_eq!(pt.pieces.len(), 2);
        
        // Piece 1: "Hello " (bold)
        let p1 = &pt.pieces[0];
        assert_eq!(p1.length, 6);
        assert!(p1.attributes.as_ref().unwrap().bold.unwrap());
        
        // Piece 2: "World" (italic, size 12)
        let p2 = &pt.pieces[1];
        assert_eq!(p2.length, 5);
        assert!(p2.attributes.as_ref().unwrap().italic.unwrap());
        assert_eq!(p2.attributes.as_ref().unwrap().font_size.unwrap(), 12);
    }
}

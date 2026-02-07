use crate::piece_tree::{PieceTree, TextAttributes, Piece};
use crate::find::SearchOptions;
use crate::page_layout::{PageConfig, PageLayout};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Document metadata structure
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub created_at: u64,
    pub modified_at: u64,
    pub word_count: usize,
    pub char_count: usize,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        DocumentMetadata {
            title: "Untitled Document".to_string(),
            author: "".to_string(),
            created_at: now,
            modified_at: now,
            word_count: 0,
            char_count: 0,
        }
    }
}

/// Document structure containing both content and metadata
pub struct Document {
    pub content: PieceTree,
    pub metadata: DocumentMetadata,
}

impl Document {
    pub fn empty() -> Self {
        Document {
            content: PieceTree::empty(),
            metadata: DocumentMetadata::default(),
        }
    }

    pub fn new(content: String) -> Self {
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        let mut metadata = DocumentMetadata::default();
        metadata.char_count = char_count;
        metadata.word_count = word_count;
        
        Document {
            content: PieceTree::new(content),
            metadata,
        }
    }

    pub fn update_metadata(&mut self) {
        let text = self.content.get_text();
        self.metadata.char_count = text.chars().count();
        self.metadata.word_count = text.split_whitespace().count();
        self.metadata.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

static DOCUMENT: Lazy<RwLock<Document>> = Lazy::new(|| RwLock::new(Document::empty()));

pub fn hello_velum() -> String {
    "Hello from Velum Core (Rust)!".to_string()
}

pub fn get_sample_document() -> String {
    let mut doc = DOCUMENT.write().unwrap();
    *doc = Document::new("Welcome to Velum.".to_string());
    doc.content.insert(16, " This is Microsoft Word 1:1 replica project.".to_string());
    doc.update_metadata();
    doc.content.get_text()
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

// 创建空文档
pub fn create_empty_document() -> String {
    let mut doc = DOCUMENT.write().unwrap();
    *doc = Document::empty();
    doc.content.get_text()
}

// 在指定位置插入文本
pub fn insert_text(offset: usize, new_text: String) -> String {
    let mut doc = DOCUMENT.write().unwrap();
    doc.content.insert(offset, new_text);
    doc.update_metadata();
    doc.content.get_text()
}

// 删除指定范围文本
pub fn delete_text(offset: usize, length: usize) -> String {
    let mut doc = DOCUMENT.write().unwrap();
    doc.content.delete(offset, length);
    doc.update_metadata();
    doc.content.get_text()
}

// 获取文本范围
pub fn get_text_range(offset: usize, length: usize) -> String {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_text_range(offset, length)
}

// 获取行数统计
pub fn get_line_count() -> usize {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_line_count()
}

// 获取指定行内容
pub fn get_line_content(line_number: usize) -> Option<String> {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_line(line_number)
}

// 获取指定行的字符偏移量
pub fn get_offset_at_line(line_number: usize) -> usize {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_offset_at_line(line_number)
}

// 获取完整文本
pub fn get_full_text() -> String {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_text()
}

// 撤销
pub fn undo() -> String {
    let mut doc = DOCUMENT.write().unwrap();
    doc.content.undo();
    doc.update_metadata();
    doc.content.get_text()
}

// 重做
pub fn redo() -> String {
    let mut doc = DOCUMENT.write().unwrap();
    doc.content.redo();
    doc.update_metadata();
    doc.content.get_text()
}

// 检查是否可以撤销
pub fn can_undo() -> bool {
    let doc = DOCUMENT.read().unwrap();
    doc.content.can_undo()
}

// 检查是否可以重做
pub fn can_redo() -> bool {
    let doc = DOCUMENT.read().unwrap();
    doc.content.can_redo()
}

// ==================== Document Metadata APIs ====================

// 获取文档标题
pub fn get_document_title() -> String {
    let doc = DOCUMENT.read().unwrap();
    doc.metadata.title.clone()
}

// 设置文档标题
pub fn set_document_title(title: String) {
    let mut doc = DOCUMENT.write().unwrap();
    doc.metadata.title = title;
    doc.metadata.modified_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
}

// 获取文档作者
pub fn get_document_author() -> String {
    let doc = DOCUMENT.read().unwrap();
    doc.metadata.author.clone()
}

// 设置文档作者
pub fn set_document_author(author: String) {
    let mut doc = DOCUMENT.write().unwrap();
    doc.metadata.author = author;
}

// 获取创建时间
pub fn get_document_created_at() -> u64 {
    let doc = DOCUMENT.read().unwrap();
    doc.metadata.created_at
}

// 获取修改时间
pub fn get_document_modified_at() -> u64 {
    let doc = DOCUMENT.read().unwrap();
    doc.metadata.modified_at
}

// 获取字数统计
pub fn get_word_count() -> usize {
    let doc = DOCUMENT.read().unwrap();
    doc.metadata.word_count
}

// 获取字符统计
pub fn get_char_count() -> usize {
    let doc = DOCUMENT.read().unwrap();
    doc.metadata.char_count
}

// 获取当前光标位置的行列号 (1-indexed)
pub fn get_cursor_position(char_offset: usize) -> (usize, usize) {
    let doc = DOCUMENT.read().unwrap();
    doc.content.move_to(char_offset)
}

// ==================== Selection APIs ====================

/// Gets the selection anchor position (i32 for FFI)
pub fn get_selection_anchor() -> i32 {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_selection_anchor() as i32
}

/// Gets the selection active position (i32 for FFI)
pub fn get_selection_active() -> i32 {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_selection_active() as i32
}

/// Sets the selection with anchor and active positions
pub fn set_selection(anchor: i32, active: i32) {
    let mut doc = DOCUMENT.write().unwrap();
    let anchor = anchor.max(0) as usize;
    let active = active.max(0) as usize;
    doc.content.set_selection(anchor, active);
}

/// Gets the selected text content
pub fn get_selection_text() -> String {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_selection_text()
}

/// Moves the selection to the specified position (collapses to cursor)
pub fn move_selection_to(offset: i32) {
    let mut doc = DOCUMENT.write().unwrap();
    let offset = offset.max(0) as usize;
    doc.content.move_selection_to(offset);
}

/// Clears the selection by collapsing to the end of the document
pub fn clear_selection() {
    let mut doc = DOCUMENT.write().unwrap();
    doc.content.clear_selection();
}

/// Returns true if there is a non-empty selection
pub fn has_selection() -> bool {
    let doc = DOCUMENT.read().unwrap();
    doc.content.has_selection()
}

/// Gets the selection range as (start, end)
pub fn get_selection_range() -> (i32, i32) {
    let doc = DOCUMENT.read().unwrap();
    let (start, end) = doc.content.get_selection_range();
    (start as i32, end as i32)
}

// ==================== Find and Replace APIs ====================

/// Finds text with options and returns JSON result
/// # Arguments
/// * `query` - Text to find
/// * `options_json` - JSON serialized SearchOptions
/// # Returns
/// JSON serialized SearchResultSet
pub fn find_text(query: &str, options_json: &str) -> String {
    let doc = DOCUMENT.read().unwrap();
    doc.content.find_text_json(query, options_json)
}

/// Replaces text and returns the number of replacements made
/// # Arguments
/// * `find` - Text to find
/// * `replace` - Replacement text
/// * `all` - If true, replace all; otherwise replace only the first
/// # Returns
/// Number of replacements made
pub fn replace_text(find: &str, replace: &str, all: bool) -> i32 {
    let mut doc = DOCUMENT.write().unwrap();
    doc.content.replace_text_json(find, replace, all)
}

/// Gets the count of matches for a query
/// # Arguments
/// * `query` - Text to find
/// # Returns
/// Number of matches found
pub fn get_match_count(query: &str) -> i32 {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_match_count(query)
}

/// Advanced find with full options (JSON input/output)
pub fn find_with_options(options_json: &str) -> String {
    let doc = DOCUMENT.read().unwrap();
    let options: Result<SearchOptions, _> = serde_json::from_str(options_json);
    match options {
        Ok(options) => {
            let results = doc.content.find_all(&options);
            serde_json::to_string(&results).unwrap_or_else(|_| "{}".to_string())
        }
        Err(e) => format!(r#"{{"error": "{}"}}"#, e),
    }
}

/// Find next match from current cursor position
pub fn find_next(query: &str) -> String {
    let doc = DOCUMENT.read().unwrap();
    let options = SearchOptions {
        query: query.to_string(),
        ..Default::default()
    };
    let result = doc.content.find_next(&options, doc.content.get_selection_active());
    match result {
        Some(r) => serde_json::to_string(&r).unwrap_or_else(|_| "{}".to_string()),
        None => "{}".to_string(),
    }
}

/// Find previous match from current cursor position
pub fn find_previous(query: &str) -> String {
    let doc = DOCUMENT.read().unwrap();
    let options = SearchOptions {
        query: query.to_string(),
        ..Default::default()
    };
    let result = doc.content.find_previous(&options, doc.content.get_selection_active());
    match result {
        Some(r) => serde_json::to_string(&r).unwrap_or_else(|_| "{}".to_string()),
        None => "{}".to_string(),
    }
}

// 查找文本，返回所有匹配的位置（字节偏移量）- 旧版兼容函数
#[deprecated(since = "0.2.0", note = "Use find_with_options instead")]
pub fn find_text_legacy(query: String) -> Vec<usize> {
    let doc = DOCUMENT.read().unwrap();
    let text = doc.content.get_text();
    let mut positions = Vec::new();
    
    if query.is_empty() {
        return positions;
    }
    
    let mut start = 0;
    while let Some(pos) = text[start..].find(&query) {
        let absolute_pos = start + pos;
        positions.push(absolute_pos);
        start = absolute_pos + query.len();
        if start >= text.len() {
            break;
        }
    }
    
    positions
}

// 查找并替换第一个匹配项 - 旧版兼容函数
#[deprecated(since = "0.2.0", note = "Use replace_text instead")]
pub fn replace_first(query: String, replacement: String) -> String {
    let mut doc = DOCUMENT.write().unwrap();
    let text = doc.content.get_text();
    
    if let Some(pos) = text.find(&query) {
        doc.content.delete(pos, query.len());
        doc.content.insert(pos, replacement);
        doc.update_metadata();
    }
    
    doc.content.get_text()
}

// 查找并替换所有匹配项 - 旧版兼容函数
#[deprecated(since = "0.2.0", note = "Use replace_text with all=true instead")]
pub fn replace_all_legacy(query: String, replacement: String) -> String {
    let doc_read = DOCUMENT.read().unwrap();
    let text = doc_read.content.get_text();
    drop(doc_read);
    
    if query.is_empty() || !text.contains(&query) {
        return text;
    }
    
    // 由于每次替换都会改变文本，我们重新构建文本
    let mut result = String::new();
    let mut last_end = 0;
    
    while let Some(pos) = text[last_end..].find(&query) {
        let absolute_pos = last_end + pos;
        result.push_str(&text[last_end..absolute_pos]);
        result.push_str(&replacement);
        last_end = absolute_pos + query.len();
    }
    result.push_str(&text[last_end..]);
    
    // 替换整个文档内容
    let mut doc_write = DOCUMENT.write().unwrap();
    *doc_write = Document::new(result);
    doc_write.update_metadata();
    doc_write.content.get_text()
}

// ==================== Document Save/Load APIs ====================

/// Serializable document structure for saving/loading
#[derive(Serialize, Deserialize, Debug)]
struct SerializableDocument {
    title: String,
    author: String,
    created_at: u64,
    modified_at: u64,
    content: String,
}

// 将文档保存为 JSON 字符串
pub fn save_document_to_json() -> String {
    let doc = DOCUMENT.read().unwrap();
    let serializable = SerializableDocument {
        title: doc.metadata.title.clone(),
        author: doc.metadata.author.clone(),
        created_at: doc.metadata.created_at,
        modified_at: doc.metadata.modified_at,
        content: doc.content.get_text(),
    };
    
    match serde_json::to_string(&serializable) {
        Ok(json) => json,
        Err(e) => format!("Error: {}", e),
    }
}

// 从 JSON 字符串加载文档
pub fn load_document_from_json(json: String) -> String {
    match serde_json::from_str::<SerializableDocument>(&json) {
        Ok(serializable) => {
            let mut doc = DOCUMENT.write().unwrap();
            *doc = Document {
                content: PieceTree::new(serializable.content),
                metadata: DocumentMetadata {
                    title: serializable.title,
                    author: serializable.author,
                    created_at: serializable.created_at,
                    modified_at: serializable.modified_at,
                    word_count: 0,
                    char_count: 0,
                },
            };
            doc.update_metadata();
            doc.content.get_text()
        }
        Err(e) => format!("Error: {}", e),
    }
}

// 获取文档的纯文本内容（用于保存为 .txt）
pub fn get_document_as_text() -> String {
    let doc = DOCUMENT.read().unwrap();
    doc.content.get_text()
}

// 从纯文本加载文档
pub fn load_document_from_text(text: String) -> String {
    let mut doc = DOCUMENT.write().unwrap();
    *doc = Document::new(text);
    doc.update_metadata();
    doc.content.get_text()
}

pub fn save_to_file(path: String) -> String {
    let json = save_document_to_json();
    if json.starts_with("Error:") {
        return json;
    }
    
    match fs::write(&path, json) {
        Ok(_) => format!("Successfully saved to {}", path),
        Err(e) => format!("Error saving file: {}", e),
    }
}

// 从指定路径加载文档 (JSON 格式)
pub fn load_from_file(path: String) -> String {
    match fs::read_to_string(&path) {
        Ok(json) => load_document_from_json(json),
        Err(e) => format!("Error reading file: {}", e),
    }
}

// 导出为纯文本文件
pub fn export_to_txt(path: String) -> String {
    let text = get_document_as_text();
    match fs::write(&path, text) {
        Ok(_) => format!("Successfully exported to {}", path),
        Err(e) => format!("Error exporting file: {}", e),
    }
}

// ==================== Text Attributes APIs ====================

/// Gets text attributes at the specified offset
pub fn get_text_attributes_at(offset: usize) -> String {
    let doc = DOCUMENT.read().unwrap();
    let offset = offset.min(doc.content.total_char_count);
    
    // Find the piece at the given offset
    let mut accumulated_chars = 0usize;
    for piece in &doc.content.pieces {
        let piece_start = accumulated_chars;
        let piece_end = accumulated_chars + piece.piece_char_length;
        
        if offset >= piece_start && offset < piece_end {
            if let Some(attrs) = &piece.attributes {
                return format!(
                    "{},{},{},{},{},{},{}",
                    attrs.bold.map_or("None", |b| if b { "true" } else { "false" }),
                    attrs.italic.map_or("None", |b| if b { "true" } else { "false" }),
                    attrs.underline.map_or("None", |b| if b { "true" } else { "false" }),
                    attrs.font_size.map(|s| s.to_string()).unwrap_or_else(|| "None".to_string()),
                    attrs.font_family.clone().unwrap_or_else(|| "None".to_string()),
                    attrs.foreground.clone().unwrap_or_else(|| "None".to_string()),
                    attrs.background.clone().unwrap_or_else(|| "None".to_string())
                );
            }
            return "None,None,None,None,None,None,None".to_string();
        }
        
        accumulated_chars = piece_end;
    }
    
    "None,None,None,None,None,None,None".to_string()
}

/// Applies text attributes to the specified range
pub fn apply_text_attributes(start: usize, end: usize, attributes_json: String) -> String {
    // Parse the attributes JSON
    let attributes: Result<TextAttributes, _> = serde_json::from_str(&attributes_json);
    if attributes.is_err() {
        return format!("Error: Invalid attributes JSON");
    }
    let attrs = attributes.unwrap();
    
    let mut doc = DOCUMENT.write().unwrap();
    let start = start.min(doc.content.total_char_count);
    let end = end.min(doc.content.total_char_count);
    
    if start >= end {
        return String::new();
    }
    
    // Clone the content to iterate
    let pieces: Vec<_> = doc.content.pieces.clone();
    let mut new_pieces = Vec::new();
    let mut accumulated_chars = 0usize;
    
    for piece in pieces {
        let piece_start = accumulated_chars;
        let piece_end = accumulated_chars + piece.piece_char_length;
        
        if piece_end <= start {
            // Piece is entirely before the range
            new_pieces.push(piece);
        } else if piece_start >= end {
            // Piece is entirely after the range
            new_pieces.push(piece);
        } else {
            // Piece overlaps with the range - may need to split
            let range_start = start.max(piece_start);
            let range_end = end.min(piece_end);
            
            // Left part (before range)
            if range_start > piece_start {
                let left_piece = Piece::new_with_attrs(
                    piece.start,
                    range_start - piece_start,
                    piece.buffer_id,
                    range_start - piece_start,
                    piece.attributes.clone(),
                );
                new_pieces.push(left_piece);
            }
            
            // Middle part (with new attributes)
            let middle_piece = Piece::new_with_attrs(
                piece.start + (range_start - piece_start),
                range_end - range_start,
                piece.buffer_id,
                range_end - range_start,
                Some(attrs.clone()),
            );
            new_pieces.push(middle_piece);
            
            // Right part (after range)
            if range_end < piece_end {
                let right_start = piece.start + (range_end - piece_start);
                let right_length = piece_end - range_end;
                let right_piece = Piece::new_with_attrs(
                    right_start,
                    right_length,
                    piece.buffer_id,
                    right_length,
                    piece.attributes.clone(),
                );
                new_pieces.push(right_piece);
            }
        }
        
        accumulated_chars = piece_end;
    }
    
    doc.content.pieces = new_pieces;
    doc.update_metadata();
    doc.content.get_text()
}

/// Removes text attributes from the specified range
pub fn remove_text_attributes(start: usize, end: usize) -> String {
    let mut doc = DOCUMENT.write().unwrap();
    let start = start.min(doc.content.total_char_count);
    let end = end.min(doc.content.total_char_count);
    
    if start >= end {
        return String::new();
    }
    
    // Clone the content to iterate
    let pieces: Vec<_> = doc.content.pieces.clone();
    let mut new_pieces = Vec::new();
    let mut accumulated_chars = 0usize;
    
    for piece in pieces {
        let piece_start = accumulated_chars;
        let piece_end = accumulated_chars + piece.piece_char_length;
        
        if piece_end <= start {
            // Piece is entirely before the range
            new_pieces.push(piece);
        } else if piece_start >= end {
            // Piece is entirely after the range
            new_pieces.push(piece);
        } else {
            // Piece overlaps with the range - may need to split
            let range_start = start.max(piece_start);
            let range_end = end.min(piece_end);
            
            // Left part (before range)
            if range_start > piece_start {
                let left_piece = Piece::new_with_attrs(
                    piece.start,
                    range_start - piece_start,
                    piece.buffer_id,
                    range_start - piece_start,
                    piece.attributes.clone(),
                );
                new_pieces.push(left_piece);
            }
            
            // Middle part (without attributes)
            let middle_piece = Piece::new_with_attrs(
                piece.start + (range_start - piece_start),
                range_end - range_start,
                piece.buffer_id,
                range_end - range_start,
                None,
            );
            new_pieces.push(middle_piece);
            
            // Right part (after range)
            if range_end < piece_end {
                let right_start = piece.start + (range_end - piece_start);
                let right_length = piece_end - range_end;
                let right_piece = Piece::new_with_attrs(
                    right_start,
                    right_length,
                    piece.buffer_id,
                    right_length,
                    piece.attributes.clone(),
                );
                new_pieces.push(right_piece);
            }
        }
        
        accumulated_chars = piece_end;
    }
    
    doc.content.pieces = new_pieces;
    doc.update_metadata();
    doc.content.get_text()
}

/// Gets all text with their attributes as JSON
pub fn get_text_with_attributes() -> String {
    let doc = DOCUMENT.read().unwrap();
    let mut result = Vec::new();

    for piece in &doc.content.pieces {
        let buffer_idx = PieceTree::buffer_idx(&piece.buffer_id);
        if let Some(buffer) = doc.content.buffers.get(buffer_idx) {
            let piece_text = if piece.start + piece.length <= buffer.len() {
                buffer[piece.start..piece.start + piece.length].to_string()
            } else {
                String::new()
            };
            
            let attrs_json = if let Some(attrs) = &piece.attributes {
                serde_json::to_string(attrs).unwrap_or_else(|_| "null".to_string())
            } else {
                "null".to_string()
            };
            
            result.push(format!("{{\"text\": \"{}\", \"attrs\": {}}}",
                piece_text.replace('"', "\\\"").replace('\n', "\\n"),
                attrs_json));
        }
    }

    format!("[{}]", result.join(", "))
}

// ==================== Line Breaking APIs ====================

use crate::line_layout::LineLayout;

/// Layouts text and returns JSON layout information
pub fn layout_text(text: &str, width: f32) -> String {
    let mut layout = LineLayout::new();
    layout.layout_to_json(text, width)
}

/// Calculates the width of text in abstract units
pub fn calculate_text_width(text: &str) -> f32 {
    let mut layout = LineLayout::new();
    layout.breaker_mut().calculate_text_width(text)
}

/// Gets the number of lines needed for text at given width
pub fn get_line_count_for_width(text: &str, width: f32) -> usize {
    let mut layout = LineLayout::new();
    layout.breaker_mut().break_lines(text, Some(width)).len()
}

/// Gets the height needed for text at given width
pub fn get_text_height(text: &str, width: f32, line_height: f32, font_size: f32) -> f32 {
    use crate::line_layout::measure;
    measure::get_text_height(text, width, line_height, font_size)
}

/// Layouts the current document state and returns JSON layout information
pub fn layout_current_document(width: f32) -> String {
    let doc = DOCUMENT.read().unwrap();
    let text = doc.content.get_text();
    let mut layout = LineLayout::new();
    layout.layout_to_json(&text, width)
}

// ==================== OOXML Document APIs ====================

use crate::ooxml::{parse_ooxml, ParsedDocument};

/// Load and parse an OOXML (.docx) document from file path
/// Returns JSON string containing extracted text, styles, and metadata
pub fn load_ooxml_document(file_path: &str) -> String {
    match std::fs::read(file_path) {
        Ok(file_data) => {
            match parse_ooxml(&file_data) {
                Ok(document) => {
                    serde_json::to_string(&document).unwrap_or_else(|e| format!("JSON error: {}", e))
                }
                Err(e) => format!("OOXML error: {}", e),
            }
        }
        Err(e) => format!("File error: {}", e),
    }
}

/// Load and parse an OOXML (.docx) document from raw bytes
/// Returns JSON string containing extracted text, styles, and metadata
pub fn load_ooxml_from_bytes(file_data: &[u8]) -> String {
    match parse_ooxml(file_data) {
        Ok(document) => {
            serde_json::to_string(&document).unwrap_or_else(|e| format!("JSON error: {}", e))
        }
        Err(e) => format!("OOXML error: {}", e),
    }
}

/// Export a document to OOXML (.docx) format
/// Takes a JSON string representing the document and returns ZIP bytes
pub fn export_to_ooxml(document_json: &str) -> Vec<u8> {
    let document: Result<ParsedDocument, _> = serde_json::from_str(document_json);
    
    match document {
        Ok(doc) => create_minimal_docx(&doc.text),
        Err(e) => format!("Error: {}", e).into_bytes(),
    }
}

/// Create a minimal .docx file with the given text content
fn create_minimal_docx(text: &str) -> Vec<u8> {
    use std::io::Write;
    use zip::ZipWriter;
    use std::io::Cursor;
    
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#;
        
        zip.start_file("[Content_Types].xml", zip::write::FileOptions::default()).unwrap();
        zip.write_all(content_types.as_bytes()).unwrap();
        
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#;
        
        zip.start_file("_rels/.rels", zip::write::FileOptions::default()).unwrap();
        zip.write_all(rels.as_bytes()).unwrap();
        
        let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;
        
        zip.start_file("word/_rels/document.xml.rels", zip::write::FileOptions::default()).unwrap();
        zip.write_all(doc_rels.as_bytes()).unwrap();
        
        // Simple XML escaping for text content
        let escape_xml = |s: &str| -> String {
            s.replace('&', "&amp;")
             .replace('<', "&lt;")
             .replace('>', "&gt;")
             .replace('"', "&quot;")
             .replace("'", "&apos;")
        };
        
        let paragraphs: Vec<String> = text.lines().map(|line| {
            format!(r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:t>{}</w:t></w:r></w:p>"#, escape_xml(line))
        }).collect();
        
        let document = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    {}
  </w:body>
</w:document>"#, paragraphs.join("\n    "));
        
        zip.start_file("word/document.xml", zip::write::FileOptions::default()).unwrap();
        zip.write_all(document.as_bytes()).unwrap();
        
        let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:styleId="Normal" w:type="paragraph">
    <w:name w:val="Normal"/>
    <w:rPr>
      <w:sz w:val="24"/>
      <w:szCs w:val="24"/>
    </w:rPr>
  </w:style>
  <w:style w:styleId="Heading1" w:type="paragraph">
    <w:name w:val="Heading 1"/>
    <w:basedOn w:val="Normal"/>
    <w:rPr>
      <w:b w:val="1"/>
      <w:sz w:val="36"/>
      <w:szCs w:val="36"/>
    </w:rPr>
  </w:style>
</w:styles>"#;
        
        zip.start_file("word/styles.xml", zip::write::FileOptions::default()).unwrap();
        zip.write_all(styles.as_bytes()).unwrap();
        
        let core_props = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>Exported Document</dc:title>
  <dc:creator>Velum</dc:creator>
  <dcterms:created xsi:type="dcterms:W3CDTF">2024-01-01T00:00:00Z</dcterms:created>
  <dcterms:modified xsi:type="dcterms:W3CDTF">2024-01-01T00:00:00Z</dcterms:modified>
</cp:coreProperties>"#;
        
        zip.start_file("docProps/core.xml", zip::write::FileOptions::default()).unwrap();
        zip.write_all(core_props.as_bytes()).unwrap();
        
        let app_props = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>Velum</Application>
  <AppVersion>1.0</AppVersion>
</Properties>"#;
        
        zip.start_file("docProps/app.xml", zip::write::FileOptions::default()).unwrap();
        zip.write_all(app_props.as_bytes()).unwrap();
        
        zip.finish().unwrap();
    }
    
    buffer.into_inner()
}

/// Get just the text content from a .docx file
pub fn extract_ooxml_text(file_path: &str) -> String {
    match std::fs::read(file_path) {
        Ok(file_data) => {
            match parse_ooxml(&file_data) {
                Ok(document) => document.text,
                Err(e) => format!("Error: {}", e),
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}

/// Get document statistics from a .docx file
pub fn get_ooxml_stats(file_path: &str) -> String {
    match std::fs::read(file_path) {
        Ok(file_data) => {
            match parse_ooxml(&file_data) {
                Ok(document) => {
                    let stats = serde_json::json!({
                        "paragraph_count": document.paragraph_count,
                        "char_count": document.char_count,
                        "word_count": document.word_count,
                        "style_count": document.styles.len(),
                        "title": document.title,
                        "author": document.author,
                    });
                    stats.to_string()
                }
                Err(e) => format!("Error: {}", e),
            }
        }
        Err(e) => format!("Error: {}", e),
    }
}


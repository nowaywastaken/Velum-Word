use crate::piece_tree::PieceTree;
use once_cell::sync::Lazy;
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

// ==================== Find and Replace APIs ====================

// 查找文本，返回所有匹配的位置（字节偏移量）
pub fn find_text(query: String) -> Vec<usize> {
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

// 查找并替换第一个匹配项
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

// 查找并替换所有匹配项
pub fn replace_all(query: String, replacement: String) -> String {
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

use serde::{Deserialize, Serialize};

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

// ==================== File System IO APIs ====================

use std::fs;

// 保存文档到指定路径 (JSON 格式)
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

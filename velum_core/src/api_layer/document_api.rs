//! # Document API
//!
//! Provides document manipulation operations for the FFI interface.

use crate::piece_tree::PieceTree;
use crate::TextAttributes;
use std::sync::{Arc, RwLock};
use crate::find::{SearchOptions, SearchResult};

/// Cursor position information
#[derive(Debug, Clone)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// Selection range
#[derive(Debug, Clone)]
pub struct SelectionRange {
    pub start: usize,
    pub end: usize,
}

/// Search result
#[derive(Debug, Clone)]
pub struct SearchResultApi {
    pub position: usize,
    pub length: usize,
}

/// Document statistics
#[derive(Debug, Clone, Default)]
pub struct DocumentStats {
    pub char_count: usize,
    pub word_count: usize,
    pub line_count: usize,
}

/// Document API trait
pub trait DocumentApi {
    /// Get the full document text
    fn get_text(&self) -> String;

    /// Get text in a range
    fn get_text_range(&self, start: usize, end: usize) -> Option<String>;

    /// Get document length in characters
    fn len(&self) -> usize;

    /// Get document statistics
    fn stats(&self) -> DocumentStats;

    /// Insert text at position
    fn insert(&self, position: usize, text: &str) -> Result<(), String>;

    /// Delete text in range
    fn delete(&self, start: usize, end: usize) -> Result<(), String>;

    /// Get cursor position for offset
    fn cursor_position(&self, offset: usize) -> Option<CursorPosition>;

    /// Get offset for cursor position
    fn offset_from_position(&self, line: usize, column: usize) -> Option<usize>;

    /// Find next occurrence
    fn find_next(&self, pattern: &str, options: &SearchOptions) -> Option<SearchResultApi>;

    /// Find previous occurrence
    fn find_previous(&self, pattern: &str, options: &SearchOptions) -> Option<SearchResultApi>;

    /// Get all occurrences
    fn find_all(&self, pattern: &str, options: &SearchOptions) -> Vec<SearchResultApi>;
}

/// Implementation using PieceTree
pub struct PieceTreeDocumentApi {
    document: Arc<RwLock<PieceTree>>,
}

impl PieceTreeDocumentApi {
    pub fn new(document: Arc<RwLock<PieceTree>>) -> Self {
        Self { document }
    }
}

impl DocumentApi for PieceTreeDocumentApi {
    fn get_text(&self) -> String {
        self.document.read().unwrap().get_text()
    }

    fn get_text_range(&self, start: usize, end: usize) -> Option<String> {
        let text = self.document.read().unwrap().get_text();
        if start <= end && end <= text.len() {
            Some(text[start..end].to_string())
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.document.read().unwrap().total_char_count
    }

    fn stats(&self) -> DocumentStats {
        let text = self.document.read().unwrap().get_text();
        DocumentStats {
            char_count: text.chars().count(),
            word_count: text.split_whitespace().count(),
            line_count: text.lines().count(),
        }
    }

    fn insert(&self, position: usize, text: &str) -> Result<(), String> {
        let mut doc = self.document.write().unwrap();
        doc.insert(position, text.to_string());
        Ok(())
    }

    fn delete(&self, start: usize, end: usize) -> Result<(), String> {
        let mut doc = self.document.write().unwrap();
        doc.delete(start, end - start);
        Ok(())
    }

    fn cursor_position(&self, offset: usize) -> Option<CursorPosition> {
        let text = self.document.read().unwrap().get_text();
        if offset > text.len() {
            return None;
        }

        let mut line = 1;
        let mut column = 1;
        let mut current_offset = 0;

        for ch in text.chars() {
            if current_offset == offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            current_offset += ch.len_utf8();
        }

        Some(CursorPosition {
            line,
            column,
            byte_offset: offset,
        })
    }

    fn offset_from_position(&self, line: usize, mut column: usize) -> Option<usize> {
        let text = self.document.read().unwrap().get_text();
        let mut current_line = 1;
        let mut offset = 0;

        for ch in text.chars() {
            if current_line == line {
                if column == 1 {
                    return Some(offset);
                }
                column -= 1;
            }

            if ch == '\n' {
                current_line += 1;
            }
            offset += ch.len_utf8();
        }

        None
    }

    fn find_next(&self, pattern: &str, options: &SearchOptions) -> Option<SearchResultApi> {
        let doc = self.document.read().unwrap();
        let cursor = doc.get_selection_active();
        let results = doc.find_all(options);
        let results_vec = results.results;

        // Find first result after cursor
        for r in &results_vec {
            if r.start >= cursor {
                return Some(SearchResultApi {
                    position: r.start,
                    length: r.end - r.start,
                });
            }
        }

        // Wrap around - return first result if any
        if let Some(r) = results_vec.first() {
            return Some(SearchResultApi {
                position: r.start,
                length: r.end - r.start,
            });
        }

        None
    }

    fn find_previous(&self, pattern: &str, options: &SearchOptions) -> Option<SearchResultApi> {
        let doc = self.document.read().unwrap();
        let cursor = doc.get_selection_active();
        let results = doc.find_all(options);
        // Find last result before cursor
        let mut last: Option<SearchResultApi> = None;
        for r in results.results.iter() {
            if r.start < cursor {
                last = Some(SearchResultApi {
                    position: r.start,
                    length: r.end - r.start,
                });
            }
        }
        last
    }

    fn find_all(&self, pattern: &str, options: &SearchOptions) -> Vec<SearchResultApi> {
        let doc = self.document.read().unwrap();
        let results = doc.find_all(options);
        results.results
            .into_iter()
            .map(|r| SearchResultApi {
                position: r.start,
                length: r.end - r.start,
            })
            .collect()
    }
}

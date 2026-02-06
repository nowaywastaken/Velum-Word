use serde::{Serialize, Deserialize};
use crate::find::{SearchOptions, SearchResult, SearchResultSet, search, find_all_in_text};
use std::fmt;
use log::{trace, debug};

/// Represents which buffer a piece comes from
/// -1 means original buffer (index 0), other values are buffer indices
const MAX_UNDO_DEPTH: usize = 100;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(pub isize);

impl BufferId {
    pub const ORIGINAL: BufferId = BufferId(-1);

    pub fn is_original(&self) -> bool {
        self.0 == -1
    }

    pub fn to_index(&self) -> usize {
        if self.0 < 0 {
            0
        } else {
            self.0 as usize
        }
    }
}

impl Default for BufferId {
    fn default() -> Self {
        BufferId::ORIGINAL
    }
}

impl fmt::Display for BufferId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_original() {
            write!(f, "Original")
        } else {
            write!(f, "Buffer({})", self.0)
        }
    }
}

/// Text attributes for rich text formatting
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TextAttributes {
    pub bold: Option<bool>,           // 加粗
    pub italic: Option<bool>,         // 斜体
    pub underline: Option<bool>,      // 下划线
    pub font_size: Option<u16>,       // 字体大小
    pub font_family: Option<String>,  // 字体名称
    pub foreground: Option<String>,   // 前景色（十六进制如 "#FF0000"）
    pub background: Option<String>,   // 背景色
}

impl TextAttributes {
    /// Creates new text attributes with all fields set to None
    pub fn new() -> Self {
        TextAttributes::default()
    }
}

/// Represents a piece of text from a buffer
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Piece {
    /// Starting position in the buffer (byte offset)
    pub start: usize,
    /// Length of this piece in bytes
    pub length: usize,
    /// Identifier for which buffer this piece comes from (buffer index)
    pub buffer_id: BufferId,
    /// Character length for UTF-16/Unicode handling
    pub piece_char_length: usize,
    /// Text attributes for rich text formatting
    pub attributes: Option<TextAttributes>,
}

impl Piece {
    /// Creates a new piece without attributes
    pub fn new(start: usize, length: usize, buffer_id: BufferId, piece_char_length: usize) -> Self {
        Piece {
            start,
            length,
            buffer_id,
            piece_char_length,
            attributes: None,
        }
    }

    /// Creates a new piece with attributes
    pub fn new_with_attrs(
        start: usize,
        length: usize,
        buffer_id: BufferId,
        piece_char_length: usize,
        attributes: Option<TextAttributes>,
    ) -> Self {
        Piece {
            start,
            length,
            buffer_id,
            piece_char_length,
            attributes,
        }
    }

    /// Returns the end position in the buffer
    pub fn end(&self) -> usize {
        self.start + self.length
    }
}

/// Represents a text selection with anchor and active positions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// The anchor position (where selection started, stays fixed during shift+arrow)
    pub anchor: usize,
    /// The active position (current cursor/selection end, moves during navigation)
    pub active: usize,
}

impl Selection {
    /// Creates a new selection with the given anchor and active positions
    pub fn new(anchor: usize, active: usize) -> Self {
        Selection { anchor, active }
    }

    /// Returns true if the selection is empty (anchor == active)
    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }

    /// Returns true if the selection is collapsed (same as is_empty)
    pub fn collapsed(&self) -> bool {
        self.is_empty()
    }

    /// Returns the start position (minimum of anchor and active)
    pub fn start(&self) -> usize {
        self.anchor.min(self.active)
    }

    /// Returns the end position (maximum of anchor and active)
    pub fn end(&self) -> usize {
        self.anchor.max(self.active)
    }

    /// Returns the selection length
    pub fn length(&self) -> usize {
        self.end().saturating_sub(self.start())
    }
}

impl From<(usize, usize)> for Selection {
    fn from(value: (usize, usize)) -> Self {
        Selection::new(value.0, value.1)
    }
}

impl Default for Selection {
    fn default() -> Self {
        Selection { anchor: 0, active: 0 }
    }
}

#[derive(Debug, Clone)]
pub enum Change {
    Insert {
        offset: usize,
        length: usize,
    },
    Delete {
        offset: usize,
        text: String,
    },
}

/// Main Piece Tree data structure
pub struct PieceTree {
    /// All pieces in the document
    pub pieces: Vec<Piece>,
    /// Map of buffer IDs to their content
    pub buffers: Vec<String>,
    /// Total character count
    pub total_char_count: usize,
    /// Total byte length
    pub total_length: usize,
    /// Next buffer index to assign (0 is original, starts from 1 for adds)
    next_buffer_index: isize,
    /// Undo stack
    undo_stack: Vec<Change>,
    /// Redo stack
    redo_stack: Vec<Change>,
    /// Whether we are currently undoing or redoing
    is_undoing_redoing: bool,
    /// Current text selection
    pub selection: Selection,
    /// Saved selection for undo/redo
    saved_selection: Option<Selection>,
}

impl PieceTree {
    /// Creates a new PieceTree with initial content
    pub fn new(content: String) -> Self {
        if content.is_empty() {
            return PieceTree::empty();
        }
        let length = content.len();
        let char_length = content.chars().count();

        // Initial buffer
        let buffers = vec![content];

        // Single piece covering the whole buffer
        let piece = Piece::new(0, length, BufferId::ORIGINAL, char_length);

        PieceTree {
            pieces: vec![piece],
            buffers,
            total_char_count: char_length,
            total_length: length,
            next_buffer_index: 1,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_undoing_redoing: false,
            selection: Selection::default(),
            saved_selection: None,
        }
    }

    /// Creates an empty PieceTree
    pub fn empty() -> Self {
        PieceTree {
            pieces: Vec::new(),
            buffers: vec![String::new()],
            total_char_count: 0,
            total_length: 0,
            next_buffer_index: 1,  // First insert should use BufferId(1), referencing buffers[1]
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_undoing_redoing: false,
            selection: Selection::default(),
            saved_selection: None,
        }
    }

    /// Creates a new PieceTree from pre-loaded data (e.g. from OOXML)
    pub fn from_loaded_data(pieces: Vec<Piece>, buffers: Vec<String>) -> Self {
        let total_char_count = pieces.iter().map(|p| p.piece_char_length).sum();
        let total_length = pieces.iter().map(|p| p.length).sum();

        let next_buffer_index = if buffers.len() > 1 {
            buffers.len() as isize
        } else {
            1
        };

        PieceTree {
            pieces,
            buffers,
            total_char_count,
            total_length,
            next_buffer_index,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_undoing_redoing: false,
            selection: Selection::default(),
            saved_selection: None,
        }
    }

    /// Gets the next buffer ID and increments the counter
    /// Buffer IDs: -1 = original buffer, 0, 1, 2... = added buffers
    /// This maps directly to buffer array indices (0 = original, 1 = first added, etc.)
    fn next_buffer_id(&mut self) -> BufferId {
        let id = BufferId(self.next_buffer_index);
        self.next_buffer_index += 1;
        id
    }

    /// Gets buffer index from BufferId
    #[inline]
    pub fn buffer_idx(buffer_id: &BufferId) -> usize {
        buffer_id.to_index()
    }

    // ==================== Selection Management ====================

    /// Sets the selection with anchor and active positions
    pub fn set_selection(&mut self, anchor: usize, active: usize) {
        let max_pos = self.total_char_count.max(self.total_length);
        self.selection.anchor = anchor.min(max_pos);
        self.selection.active = active.min(max_pos);
    }

    /// Moves the selection to the specified position (collapses to cursor)
    pub fn move_selection_to(&mut self, offset: usize) {
        let max_pos = self.total_char_count.max(self.total_length);
        let offset = offset.min(max_pos);
        self.selection.anchor = offset;
        self.selection.active = offset;
    }

    /// Clears the selection by collapsing to the end of the document
    pub fn clear_selection(&mut self) {
        let max_pos = self.total_char_count.max(self.total_length);
        self.selection.anchor = max_pos;
        self.selection.active = max_pos;
    }

    /// Gets the selection anchor position
    pub fn get_selection_anchor(&self) -> usize {
        self.selection.anchor
    }

    /// Gets the selection active position
    pub fn get_selection_active(&self) -> usize {
        self.selection.active
    }

    /// Returns true if there is a selection (not collapsed)
    pub fn has_selection(&self) -> bool {
        !self.selection.is_empty()
    }

    /// Gets the selected text range (start, end)
    pub fn get_selection_range(&self) -> (usize, usize) {
        (self.selection.start(), self.selection.end())
    }

    /// Gets the selected text content
    pub fn get_selection_text(&self) -> String {
        if self.selection.is_empty() {
            return String::new();
        }
        let (start, end) = self.get_selection_range();
        self.get_text_range(start, end - start)
    }

    // ==================== Insertion ====================

    /// Inserts text at the specified character offset (without attributes)
    /// Returns true if successful
    pub fn insert(&mut self, offset: usize, text: String) -> bool {
        self.insert_with_attrs(offset, text, None)
    }

    /// Inserts text at the specified character offset with optional attributes
    /// Returns true if successful
    pub fn insert_with_attrs(&mut self, char_offset: usize, text: String, attributes: Option<TextAttributes>) -> bool {
        if text.is_empty() {
            return true;
        }

        let char_count = text.chars().count();
        let byte_count = text.len();

        // Clamp offset to document length
        let max_offset = self.total_char_count;
        let char_offset = std::cmp::min(char_offset, max_offset);

        // Record change for undo
        if !self.is_undoing_redoing {
            // Save current selection for undo
            self.saved_selection = Some(self.selection);
            self.undo_stack.push(Change::Insert {
                offset: char_offset,
                length: byte_count,
            });
            if self.undo_stack.len() > MAX_UNDO_DEPTH {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }

        trace!("insert: char_offset={}, text='{}' ({} bytes, {} chars)",
                  char_offset, text, byte_count, char_count);

        // Add the new text to buffers
        let new_buffer_id = self.next_buffer_id();
        self.buffers.push(text.clone());

        if self.pieces.is_empty() {
            // Empty document - create first piece
            let piece = Piece::new_with_attrs(0, byte_count, new_buffer_id, char_count, attributes);
            self.pieces.push(piece);
            self.total_char_count += char_count;
            self.total_length += byte_count;
            // Move selection after inserted text
            if !self.is_undoing_redoing {
                self.move_selection_to(char_offset + char_count);
            }
            return true;
        }

        // Find position to insert using character-based API
        let (piece_idx, byte_offset_in_piece) = match self.find_piece_and_byte_offset_from_char(char_offset) {
            Some(result) => result,
            None => {
                // Insert at end
                let piece = Piece::new_with_attrs(0, byte_count, new_buffer_id, char_count, attributes);
                self.pieces.push(piece);
                self.total_char_count += char_count;
                self.total_length += byte_count;
                // Move selection after inserted text
                if !self.is_undoing_redoing {
                    self.move_selection_to(char_offset + char_count);
                }
                return true;
            }
        };

        trace!("piece_idx={}, byte_offset_in_piece={}", piece_idx, byte_offset_in_piece);

        let piece = &mut self.pieces[piece_idx];

        if byte_offset_in_piece == 0 {
            // Insert at the beginning of this piece
            let new_piece = Piece::new_with_attrs(0, byte_count, new_buffer_id, char_count, attributes);
            self.pieces.insert(piece_idx, new_piece);
            trace!("insert at beginning");
        } else if byte_offset_in_piece == piece.length {
            // Insert at the end of this piece
            let new_piece = Piece::new_with_attrs(0, byte_count, new_buffer_id, char_count, attributes);
            self.pieces.insert(piece_idx + 1, new_piece);
            trace!("insert at end");
        } else {
            // Split the piece and insert in the middle
            // byte_offset_in_piece is the character offset within the piece where we split

            // Capture original values before updating
            let original_piece_start = piece.start;
            let original_piece_length = piece.length;

            // Calculate left and right pieces using char_indices for correct UTF-8 handling
            let piece_buffer_idx = Self::buffer_idx(&piece.buffer_id);
            let piece_buffer = &self.buffers[piece_buffer_idx];
            let original_piece_text = &piece_buffer[original_piece_start..original_piece_start + original_piece_length];

            // Get all characters in the piece
            let chars: Vec<char> = original_piece_text.chars().collect();
            let total_chars_in_piece = chars.len();

            // Clamp byte_offset_in_piece to valid range
            let split_char_idx = byte_offset_in_piece.min(total_chars_in_piece);

            // left_piece: first split_char_idx characters
            let left_chars: String = chars[..split_char_idx].iter().collect();
            let left_byte_count = left_chars.len();
            let left_char_count = left_chars.chars().count();

            // right_piece: remaining characters
            let right_chars: String = chars[split_char_idx..].iter().collect();
            let right_piece_byte_length = right_chars.len();
            let right_piece_char_length = right_chars.chars().count();

            trace!("left='{}' ({} bytes, {} chars)", left_chars, left_byte_count, left_char_count);
            trace!("right='{}' ({} bytes, {} chars)", right_chars, right_piece_byte_length, right_piece_char_length);

            // Update left piece
            piece.length = left_byte_count;
            piece.piece_char_length = left_char_count;

            // Create right piece
            let right_piece = Piece::new_with_attrs(
                original_piece_start + left_byte_count,
                right_piece_byte_length,
                piece.buffer_id,
                right_piece_char_length,
                piece.attributes.clone(),
            );

            // Create the new piece to insert
            let new_piece = Piece::new_with_attrs(0, byte_count, new_buffer_id, char_count, attributes);

            // Insert: left piece (unchanged idx) + new piece (idx+1) + right piece (idx+2)
            self.pieces.insert(piece_idx + 1, new_piece);
            self.pieces.insert(piece_idx + 2, right_piece);

            trace!("insert middle");
        }

        self.total_char_count += char_count;
        self.total_length += byte_count;

        // Move selection after inserted text
        if !self.is_undoing_redoing {
            self.move_selection_to(char_offset + char_count);
        }

        true
    }

    /// Finds the piece and byte offset for a given character position (character-based API)
    /// Returns (piece_index, byte_offset_within_piece)
    /// Returns None if char_offset is beyond the document
    fn find_piece_and_byte_offset_from_char(&self, char_offset: usize) -> Option<(usize, usize)> {
        if char_offset == 0 {
            if !self.pieces.is_empty() {
                return Some((0, 0));
            }
            return None;
        }

        let mut accumulated_chars = 0usize;

        for (idx, piece) in self.pieces.iter().enumerate() {
            let piece_end_chars = accumulated_chars + piece.piece_char_length;

            if char_offset < accumulated_chars {
                accumulated_chars = piece_end_chars;
                continue;
            }

            if char_offset < piece_end_chars {
                // The character is within this piece
                let char_offset_in_piece = char_offset - accumulated_chars;

                // Find the byte offset for this character position
                let piece_buffer_idx = Self::buffer_idx(&piece.buffer_id);
                if let Some(buffer) = self.buffers.get(piece_buffer_idx) {
                    let piece_text = &buffer[piece.start..piece.start + piece.length];

                    // Find the byte offset of the char_offset_in_piece-th character
                    let byte_offset: usize = if char_offset_in_piece == 0 {
                        0
                    } else {
                        piece_text.char_indices()
                            .nth(char_offset_in_piece - 1)
                            .map(|(byte_idx, c)| byte_idx + c.len_utf8())
                            .unwrap_or(piece.length)
                    };

                    return Some((idx, byte_offset));
                }
            }

            if char_offset == piece_end_chars {
                // At the boundary - return end of this piece
                return Some((idx, piece.length));
            }

            accumulated_chars = piece_end_chars;
        }

        // Handle inserting at the exact end of the document
        if accumulated_chars == char_offset && !self.pieces.is_empty() {
            let last_idx = self.pieces.len() - 1;
            let last_piece = &self.pieces[last_idx];
            return Some((last_idx, last_piece.length));
        }

        None
    }

    // ==================== Deletion ====================

    /// Deletes text from the specified byte position with the given byte length
    /// Returns true if successful
    pub fn delete(&mut self, offset: usize, length: usize) -> bool {
        if length == 0 || self.pieces.is_empty() {
            return false;
        }

        let end_offset = offset.saturating_add(length);
        if end_offset > self.total_length {
            return false;
        }

        // Record change for undo
        if !self.is_undoing_redoing {
            // Save current selection for undo
            self.saved_selection = Some(self.selection);
            let deleted_text = self.get_text_range(offset, length);
            self.undo_stack.push(Change::Delete {
                offset,
                text: deleted_text,
            });
            if self.undo_stack.len() > MAX_UNDO_DEPTH {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }

        let mut deleted_chars = 0;
        let mut deleted_bytes = 0;
        let mut new_pieces = Vec::new();

        let mut current_offset = 0usize;

        for piece in &self.pieces {
            let piece_start = current_offset;
            let piece_end = current_offset + piece.length;

            if piece_end <= offset {
                // Piece is entirely before delete range
                new_pieces.push(piece.clone());
                current_offset = piece_end;
                continue;
            }

            if piece_start >= end_offset {
                // Piece is entirely after delete range
                new_pieces.push(piece.clone());
                continue;
            }

            // This piece overlaps with the delete range
            let delete_start_in_piece = if offset > piece_start { offset - piece_start } else { 0 };
            let delete_end_in_piece = if end_offset < piece_end { end_offset - piece_start } else { piece.length };

            deleted_bytes += delete_end_in_piece - delete_start_in_piece;
            deleted_chars += delete_end_in_piece - delete_start_in_piece;

            if delete_start_in_piece > 0 {
                // Keep left part
                let left_piece = Piece::new_with_attrs(
                    piece.start,
                    delete_start_in_piece,
                    piece.buffer_id,
                    delete_start_in_piece,
                    piece.attributes.clone(),
                );
                new_pieces.push(left_piece);
            }

            if delete_end_in_piece < piece.length {
                // Keep right part - calculate correct start position
                let right_start = piece.start + delete_end_in_piece;
                let right_length = piece.length - delete_end_in_piece;
                let right_piece = Piece::new_with_attrs(
                    right_start,
                    right_length,
                    piece.buffer_id,
                    right_length,
                    piece.attributes.clone(),
                );
                new_pieces.push(right_piece);
            }

            current_offset = piece_end;
        }

        self.pieces = new_pieces;
        self.total_char_count = self.total_char_count.saturating_sub(deleted_chars);
        self.total_length = self.total_length.saturating_sub(deleted_bytes);

        // Adjust selection after delete
        if !self.is_undoing_redoing {
            let delete_start = offset;
            let delete_end = end_offset;

            // If selection is entirely after deleted range, shift it left
            if self.selection.start() >= delete_end {
                let shift = delete_end - delete_start;
                self.selection.anchor = self.selection.anchor.saturating_sub(shift);
                self.selection.active = self.selection.active.saturating_sub(shift);
            } else if self.selection.end() > delete_start && self.selection.start() < delete_end {
                // Selection overlaps with deleted range - collapse to delete start
                self.move_selection_to(delete_start);
            } else if self.selection.end() > delete_start {
                // Selection end is within deleted range
                let shift = self.selection.end().saturating_sub(delete_start);
                self.selection.anchor = self.selection.anchor.saturating_sub(shift.min(self.selection.anchor));
                self.selection.active = self.selection.active.saturating_sub(shift.min(self.selection.active));
            }
            // If selection is entirely before deleted range, no adjustment needed
        }

        true
    }

    // ==================== Text Retrieval ====================

    /// Gets the full text content
    pub fn get_text(&self) -> String {
        self.get_text_range(0, self.total_length)
    }

    /// Gets text content from byte position with byte length
    pub fn get_text_range(&self, offset: usize, length: usize) -> String {
        if length == 0 || self.pieces.is_empty() {
            return String::new();
        }

        let mut result = String::with_capacity(length);
        let mut current_offset = 0usize;
        let end_offset = offset + length;

        for piece in &self.pieces {
            let piece_start = current_offset;
            let piece_end = current_offset + piece.length;

            if piece_end <= offset {
                current_offset = piece_end;
                continue;
            }

            if piece_start >= end_offset {
                break;
            }

            let start_in_piece = if offset > piece_start { offset - piece_start } else { 0 };
            let end_in_piece = if end_offset < piece_end { end_offset - piece_start } else { piece.length };

            let buffer_idx = Self::buffer_idx(&piece.buffer_id);
            if let Some(buffer) = self.buffers.get(buffer_idx) {
                let start_byte = piece.start + start_in_piece;
                let end_byte = piece.start + end_in_piece;
                if start_byte < buffer.len() && end_byte <= buffer.len() {
                    result.push_str(&buffer[start_byte..end_byte]);
                }
            }

            current_offset = piece_end;
        }

        result
    }

    // ==================== Undo/Redo ====================

    /// Undoes the last change
    pub fn undo(&mut self) -> bool {
        if let Some(change) = self.undo_stack.pop() {
            self.is_undoing_redoing = true;
            let redo_change = match change {
                Change::Insert { offset, length } => {
                    let deleted_text = self.get_text_range(offset, length);
                    self.delete(offset, length);
                    Change::Delete {
                        offset,
                        text: deleted_text,
                    }
                }
                Change::Delete { offset, text } => {
                    let length = text.len();
                    self.insert(offset, text);
                    Change::Insert { offset, length }
                }
            };
            self.redo_stack.push(redo_change);
            self.is_undoing_redoing = false;
            // Restore selection
            if let Some(saved_sel) = self.saved_selection {
                self.selection = saved_sel;
            }
            return true;
        }
        false
    }

    /// Redoes the last undone change
    pub fn redo(&mut self) -> bool {
        if let Some(change) = self.redo_stack.pop() {
            self.is_undoing_redoing = true;
            let undo_change = match change {
                Change::Insert { offset, length } => {
                    let deleted_text = self.get_text_range(offset, length);
                    self.delete(offset, length);
                    Change::Delete {
                        offset,
                        text: deleted_text,
                    }
                }
                Change::Delete { offset, text } => {
                    let length = text.len();
                    self.insert(offset, text);
                    Change::Insert { offset, length }
                }
            };
            self.undo_stack.push(undo_change);
            self.is_undoing_redoing = false;
            // Restore selection
            if let Some(saved_sel) = self.saved_selection {
                self.selection = saved_sel;
            }
            return true;
        }
        false
    }

    /// Returns true if there are undoable changes available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if there are redoable changes available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    // ==================== Navigation ====================

    /// Moves to the specified character position and returns (line, column)
    pub fn move_to(&self, char_offset: usize) -> (usize, usize) {
        if self.pieces.is_empty() {
            return (1, 1);
        }

        let mut line = 1usize;
        let mut column = 1usize;
        let mut char_count = 0usize;

        for piece in &self.pieces {
            let buffer_idx = Self::buffer_idx(&piece.buffer_id);
            if let Some(buffer) = self.buffers.get(buffer_idx) {
                let piece_text = if piece.start + piece.length <= buffer.len() {
                    &buffer[piece.start..piece.start + piece.length]
                } else {
                    ""
                };

                for c in piece_text.chars() {
                    if char_count >= char_offset {
                        return (line, column);
                    }

                    if c == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    char_count += 1;
                }
            }
        }

        // If we get here, char_offset is beyond the text
        let full_text = self.get_text();
        if full_text.is_empty() {
            return (1, 1);
        }

        line = full_text.chars().filter(|&c| c == '\n').count() + 1;

        if let Some(last_newline_pos) = full_text.rfind('\n') {
            column = full_text[last_newline_pos + 1..].chars().count() + 1;
        } else {
            column = full_text.chars().count() + 1;
        }

        (line, column)
    }

    /// Gets the content of a specific line (1-indexed)
    pub fn get_line(&self, line_number: usize) -> Option<String> {
        if line_number == 0 || self.pieces.is_empty() {
            return None;
        }

        let mut current_line = 1usize;
        let mut current_line_content = String::new();

        for piece in &self.pieces {
            let buffer_idx = Self::buffer_idx(&piece.buffer_id);
            if let Some(buffer) = self.buffers.get(buffer_idx) {
                let piece_text = if piece.start + piece.length <= buffer.len() {
                    &buffer[piece.start..piece.start + piece.length]
                } else {
                    ""
                };

                for c in piece_text.chars() {
                    if c == '\n' {
                        if current_line == line_number {
                            return Some(current_line_content);
                        }
                        current_line += 1;
                        current_line_content.clear();
                    } else {
                        if current_line == line_number {
                            current_line_content.push(c);
                        }
                    }
                }
            }
        }

        if current_line == line_number && !current_line_content.is_empty() {
            return Some(current_line_content);
        }

        None
    }

    /// Gets the line count
    pub fn get_line_count(&self) -> usize {
        if self.pieces.is_empty() {
            return 0;
        }

        let full_text = self.get_text();
        if full_text.is_empty() {
            return 1;
        }

        full_text.chars().filter(|&c| c == '\n').count() + 1
    }

    /// Gets the character offset for the start of a specific line (1-indexed)
    /// Returns 0 if the line number is invalid
    pub fn get_offset_at_line(&self, line_number: usize) -> usize {
        if line_number == 0 || line_number == 1 {
            return 0;
        }

        if self.pieces.is_empty() {
            return 0;
        }

        let mut current_line = 1usize;
        let mut char_count = 0usize;

        for piece in &self.pieces {
            let buffer_idx = Self::buffer_idx(&piece.buffer_id);
            if let Some(buffer) = self.buffers.get(buffer_idx) {
                let piece_text = if piece.start + piece.length <= buffer.len() {
                    &buffer[piece.start..piece.start + piece.length]
                } else {
                    ""
                };

                for c in piece_text.chars() {
                    if current_line == line_number {
                        return char_count;
                    }

                    if c == '\n' {
                        current_line += 1;
                    }
                    char_count += 1;
                }
            }
        }

        // If line_number is beyond the document, return the total length
        char_count
    }

    /// Gets total character count
    pub fn char_count(&self) -> usize {
        self.total_char_count
    }

    /// Gets total byte length
    pub fn len(&self) -> usize {
        self.total_length
    }

    /// Checks if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    /// Gets the piece at the given index
    pub fn get_piece(&self, index: usize) -> Option<&Piece> {
        self.pieces.get(index)
    }

    /// Gets the number of pieces
    pub fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    /// Gets all pieces (for debugging)
    pub fn get_all_pieces(&self) -> &Vec<Piece> {
        &self.pieces
    }

    // ==================== Find & Replace ====================

    /// Finds all matches in the document
    pub fn find_all(&self, options: &SearchOptions) -> SearchResultSet {
        let text = self.get_text();
        find_all_in_text(&text, options)
    }

    /// Finds the next match starting from the given position
    pub fn find_next(&self, options: &SearchOptions, from: usize) -> Option<SearchResult> {
        let text = self.get_text();
        let mut search_options = options.clone();
        search_options.search_backward = false;
        search(&text, &search_options, from)
    }

    /// Finds the previous match before the given position
    pub fn find_previous(&self, options: &SearchOptions, from: usize) -> Option<SearchResult> {
        let text = self.get_text();
        let mut search_options = options.clone();
        search_options.search_backward = true;
        search(&text, &search_options, from)
    }

    /// Replaces the first match after the given position
    /// Returns true if a replacement was made
    pub fn replace_one(&mut self, options: &SearchOptions) -> bool {
        if options.query.is_empty() {
            return false;
        }

        let from = self.selection.active;
        if let Some(result) = self.find_next(options, from) {
            let matched_text = result.matched_text.clone();
            let matched_len = matched_text.len();

            // Delete the matched text
            self.delete(result.start, matched_len);

            // Insert the replacement
            self.insert(result.start, options.replace.clone());

            true
        } else {
            false
        }
    }

    /// Replaces all matches in the document
    /// Returns the number of replacements made
    pub fn replace_all(&mut self, options: &SearchOptions) -> usize {
        if options.query.is_empty() {
            return 0;
        }

        let text = self.get_text();
        let results = self.find_all(options);

        if results.results.is_empty() {
            return 0;
        }

        // Work backwards to preserve positions
        let mut replacements = 0;
        for result in results.results.iter().rev() {
            let matched_len = result.matched_text.len();

            // Delete the matched text
            self.delete(result.start, matched_len);

            // Insert the replacement
            self.insert(result.start, options.replace.clone());

            replacements += 1;
        }

        replacements
    }

    /// Searches for text with options, returns JSON result for FFI
    pub fn find_text_json(&self, query: &str, options_json: &str) -> String {
        // Parse options from JSON
        let options: Result<SearchOptions, _> = serde_json::from_str(options_json);
        let options = options.unwrap_or_else(|_| SearchOptions {
            query: query.to_string(),
            replace: String::new(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            wrap_around: true,
            search_backward: false,
        });

        let results = self.find_all(&options);
        serde_json::to_string(&results).unwrap_or_else(|_| "{}".to_string())
    }

    /// Gets the count of matches for a query
    pub fn get_match_count(&self, query: &str) -> i32 {
        let options = SearchOptions {
            query: query.to_string(),
            ..Default::default()
        };
        self.find_all(&options).total_count as i32
    }

    /// Replaces text and returns the result as JSON
    pub fn replace_text_json(&mut self, find: &str, replace: &str, all: bool) -> i32 {
        let options = SearchOptions {
            query: find.to_string(),
            replace: replace.to_string(),
            ..Default::default()
        };

        if all {
            self.replace_all(&options) as i32
        } else {
            if self.replace_one(&options) { 1 } else { 0 }
        }
    }

    /// Debug: prints tree structure
    pub fn debug_print(&self) {
        println!("PieceTree with {} pieces, {} chars, {} bytes",
                 self.pieces.len(), self.total_char_count, self.total_length);
        for (i, piece) in self.pieces.iter().enumerate() {
            let buffer_idx = Self::buffer_idx(&piece.buffer_id);
            let text = self.buffers.get(buffer_idx)
                .and_then(|b| {
                    let start = piece.start;
                    let len = piece.length.min(b.len().saturating_sub(start));
                    b.get(start..start + len)
                })
                .unwrap_or("");
            println!("  [{}] buf={}, start={}, len={}, char_len={}, text=\"{}\"",
                     i, piece.buffer_id, piece.start, piece.length, piece.piece_char_length, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_tree_basic() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.insert(6, "Beautiful ".to_string());
        assert_eq!(pt.get_text(), "Hello Beautiful World");
    }

    #[test]
    fn test_piece_tree_complex() {
        let mut pt = PieceTree::new("".to_string());
        pt.insert(0, "Rust".to_string());
        pt.insert(0, "I love ".to_string());
        pt.insert(11, "!".to_string());
        assert_eq!(pt.get_text(), "I love Rust!");
    }

    #[test]
    fn test_piece_tree_insert_middle() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.insert(5, " Beautiful".to_string());
        assert_eq!(pt.get_text(), "Hello Beautiful World");
    }

    #[test]
    fn test_piece_tree_delete() {
        let mut pt = PieceTree::new("Hello Beautiful World".to_string());
        pt.delete(5, 10); // Delete "Beautiful "
        assert_eq!(pt.get_text(), "Hello World");
    }

    #[test]
    fn test_piece_tree_delete_partial() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.delete(0, 6); // Delete "Hello "
        assert_eq!(pt.get_text(), "World");
    }

    #[test]
    fn test_piece_tree_delete_partial_end() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.delete(5, 6); // Delete " World"
        assert_eq!(pt.get_text(), "Hello");
    }

    #[test]
    fn test_piece_tree_get_range() {
        let mut pt = PieceTree::new("Hello Beautiful World".to_string());
        let range = pt.get_text_range(6, 9);
        assert_eq!(range, "Beautiful");
    }

    #[test]
    fn test_piece_tree_move_to() {
        let mut pt = PieceTree::new("Hello\nWorld".to_string());
        let (line, col) = pt.move_to(6);
        assert_eq!(line, 2);
        assert_eq!(col, 1);

        let (line, col) = pt.move_to(0);
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_piece_tree_line_count() {
        let pt = PieceTree::new("Line 1\nLine 2\nLine 3".to_string());
        assert_eq!(pt.get_line_count(), 3);
    }

    #[test]
    fn test_piece_tree_get_line() {
        let pt = PieceTree::new("Line 1\nLine 2\nLine 3".to_string());
        assert_eq!(pt.get_line(2).unwrap(), "Line 2");
        assert_eq!(pt.get_line(1).unwrap(), "Line 1");
        assert_eq!(pt.get_line(3).unwrap(), "Line 3");
    }

    #[test]
    fn test_piece_tree_empty() {
        let pt = PieceTree::new(String::new());
        assert!(pt.is_empty());
        assert_eq!(pt.get_text(), "");
    }

    #[test]
    fn test_piece_tree_multiple_buffers() {
        let mut pt = PieceTree::new("First".to_string());
        pt.insert(5, " Second".to_string());
        pt.insert(12, " Third".to_string());
        assert_eq!(pt.get_text(), "First Second Third");
    }

    #[test]
    fn test_piece_tree_utf8() {
        let mut pt = PieceTree::new("Hello 世界".to_string());
        // "Hello " = 6 bytes, insert at byte 6 (after space, before 世)
        pt.insert(6, " 你好".to_string());
        // Result: "Hello " + " 你好" + "世界" = "Hello  你好世界"
        assert_eq!(pt.get_text(), "Hello  你好世界");
    }

    #[test]
    fn test_piece_tree_insert_utf8_middle() {
        let mut pt = PieceTree::new("Hello世界".to_string());
        pt.insert(5, " 你好".to_string());
        assert_eq!(pt.get_text(), "Hello 你好世界");
    }

    #[test]
    fn test_piece_tree_delete_utf8() {
        let mut pt = PieceTree::new("Hello 世界 你好".to_string());
        let pos = "Hello ".len(); // 6 bytes
        let len = "世界 ".len(); // 7 bytes (2*3 + 1)
        pt.delete(pos, len);
        // After delete: "Hello " + "你好" = "Hello 你好"
        assert_eq!(pt.get_text(), "Hello 你好");
    }

    #[test]
    fn test_piece_tree_consecutive_inserts() {
        let mut pt = PieceTree::new("ABC".to_string());
        pt.insert(0, "X".to_string());
        pt.insert(1, "Y".to_string());
        pt.insert(2, "Z".to_string());
        assert_eq!(pt.get_text(), "XYZABC");
    }

    #[test]
    fn test_piece_tree_delete_all() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.delete(0, 11);
        assert!(pt.is_empty());
        assert_eq!(pt.get_text(), "");
    }

    #[test]
    fn test_piece_tree_piece_count() {
        let mut pt = PieceTree::new("Hello World".to_string());
        assert_eq!(pt.piece_count(), 1);
        pt.insert(5, " Beautiful".to_string());
        // Inserting in middle splits the original piece: left + new + right = 3 pieces
        assert_eq!(pt.piece_count(), 3);
    }

    #[test]
    fn test_piece_tree_newline_handling() {
        let mut pt = PieceTree::new("".to_string());
        pt.insert(0, "Line 1\n".to_string());
        pt.insert(7, "Line 2\n".to_string());
        pt.insert(14, "Line 3".to_string());

        assert_eq!(pt.get_text(), "Line 1\nLine 2\nLine 3");
        assert_eq!(pt.get_line_count(), 3);
        assert_eq!(pt.get_line(1).unwrap(), "Line 1");
        assert_eq!(pt.get_line(2).unwrap(), "Line 2");
        assert_eq!(pt.get_line(3).unwrap(), "Line 3");
    }

    #[test]
    fn test_piece_tree_move_to_end() {
        let mut pt = PieceTree::new("Hello\nWorld".to_string());
        let (line, col) = pt.move_to(11);
        assert_eq!(line, 2);
        assert_eq!(col, 6);
    }

    #[test]
    fn test_piece_tree_delete_middle_piece() {
        let mut pt = PieceTree::new("AAA BBB CCC".to_string());
        pt.delete(4, 4); // Delete "BBB "
        assert_eq!(pt.get_text(), "AAA CCC");
    }

    // ==================== Selection Tests ====================

    #[test]
    fn test_selection_default() {
        let sel = Selection::default();
        assert_eq!(sel.anchor, 0);
        assert_eq!(sel.active, 0);
        assert!(sel.is_empty());
        assert!(sel.collapsed());
    }

    #[test]
    fn test_selection_new() {
        let sel = Selection::new(5, 10);
        assert_eq!(sel.anchor, 5);
        assert_eq!(sel.active, 10);
        assert!(!sel.is_empty());
        assert_eq!(sel.start(), 5);
        assert_eq!(sel.end(), 10);
        assert_eq!(sel.length(), 5);
    }

    #[test]
    fn test_selection_from_tuple() {
        let sel: Selection = (3, 7).into();
        assert_eq!(sel.anchor, 3);
        assert_eq!(sel.active, 7);
    }

    #[test]
    fn test_selection_reversed() {
        let sel = Selection::new(10, 5);
        assert_eq!(sel.start(), 5);
        assert_eq!(sel.end(), 10);
        assert_eq!(sel.length(), 5);
    }

    #[test]
    fn test_selection_methods() {
        let sel = Selection::new(0, 0);
        assert!(sel.is_empty());
        assert!(sel.collapsed());

        let sel = Selection::new(5, 5);
        assert!(sel.is_empty());
        assert!(sel.collapsed());

        let sel = Selection::new(0, 5);
        assert!(!sel.is_empty());
        assert!(!sel.collapsed());
    }

    #[test]
    fn test_piece_tree_selection() {
        let mut pt = PieceTree::new("Hello World".to_string());

        // Default selection should be at end
        assert_eq!(pt.get_selection_anchor(), 0);
        assert_eq!(pt.get_selection_active(), 0);

        // Set selection
        pt.set_selection(0, 5);
        assert_eq!(pt.get_selection_anchor(), 0);
        assert_eq!(pt.get_selection_active(), 5);
        assert!(pt.has_selection());

        // Move selection
        pt.move_selection_to(10);
        assert_eq!(pt.get_selection_anchor(), 10);
        assert_eq!(pt.get_selection_active(), 10);
        assert!(!pt.has_selection());
    }

    #[test]
    fn test_piece_tree_selection_text() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.set_selection(0, 5);
        assert_eq!(pt.get_selection_text(), "Hello");

        pt.set_selection(6, 11);
        assert_eq!(pt.get_selection_text(), "World");

        pt.set_selection(5, 5); // collapsed
        assert_eq!(pt.get_selection_text(), "");
    }

    #[test]
    fn test_piece_tree_selection_after_insert() {
        let mut pt = PieceTree::new("Hello World".to_string());

        // Set selection in the middle
        pt.set_selection(5, 5); // cursor after "Hello"

        // Insert text - selection should move after inserted text
        pt.insert(5, " Beautiful".to_string());
        assert_eq!(pt.get_text(), "Hello Beautiful World");
        assert_eq!(pt.get_selection_anchor(), 15); // 5 + 10 (len of " Beautiful")
        assert_eq!(pt.get_selection_active(), 15);
    }

    #[test]
    fn test_piece_tree_selection_after_delete() {
        let mut pt = PieceTree::new("Hello Beautiful World".to_string());

        // Set selection after "Hello"
        pt.set_selection(5, 5);

        // Delete " Beautiful" (10 chars)
        pt.delete(5, 10);
        assert_eq!(pt.get_text(), "Hello World");

        // Selection should be at delete position
        assert_eq!(pt.get_selection_anchor(), 5);
        assert_eq!(pt.get_selection_active(), 5);
    }

    #[test]
    fn test_piece_tree_selection_adjust_after_delete() {
        let mut pt = PieceTree::new("Hello World".to_string());

        // Set selection after deleted range
        pt.set_selection(10, 10); // at end

        // Delete "Hello " (6 chars)
        pt.delete(0, 6);
        assert_eq!(pt.get_text(), "World");

        // Selection should shift left by 6
        assert_eq!(pt.get_selection_anchor(), 4);
        assert_eq!(pt.get_selection_active(), 4);
    }

    #[test]
    fn test_piece_tree_get_selection_range() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.set_selection(0, 5);
        assert_eq!(pt.get_selection_range(), (0, 5));

        pt.set_selection(5, 0); // reversed
        assert_eq!(pt.get_selection_range(), (0, 5));
    }

    #[test]
    fn test_piece_tree_clear_selection() {
        let mut pt = PieceTree::new("Hello World".to_string());
        pt.set_selection(0, 5);
        assert!(pt.has_selection());

        pt.clear_selection();
        assert!(!pt.has_selection());
        assert_eq!(pt.get_selection_anchor(), 11); // end of text
        assert_eq!(pt.get_selection_active(), 11);
    }
}

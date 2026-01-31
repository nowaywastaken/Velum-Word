use std::fmt;

/// Represents which buffer a piece comes from
/// -1 means original buffer (index 0), other values are buffer indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Represents a piece of text from a buffer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    /// Starting position in the buffer (byte offset)
    pub start: usize,
    /// Length of this piece in bytes
    pub length: usize,
    /// Identifier for which buffer this piece comes from (buffer index)
    pub buffer_id: BufferId,
    /// Character length for UTF-16/Unicode handling
    pub piece_char_length: usize,
}

impl Piece {
    /// Creates a new piece
    pub fn new(start: usize, length: usize, buffer_id: BufferId, piece_char_length: usize) -> Self {
        Piece {
            start,
            length,
            buffer_id,
            piece_char_length,
        }
    }

    /// Returns the end position in the buffer
    pub fn end(&self) -> usize {
        self.start + self.length
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
}

impl PieceTree {
    /// Creates a new PieceTree with initial content
    pub fn new(content: String) -> Self {
        if content.is_empty() {
            return PieceTree::empty();
        }
        
        let char_count = content.chars().count();
        let length = content.len();
        
        let mut buffers = Vec::new();
        buffers.push(String::new()); // Original buffer at index 0
        buffers.push(content.clone()); // Add buffer at index 1
        
        let piece = Piece::new(0, length, BufferId(1), char_count);
        
        let mut pieces = Vec::new();
        pieces.push(piece);
        
        PieceTree {
            pieces,
            buffers,
            total_char_count: char_count,
            total_length: length,
            next_buffer_index: 1, // Next add will be at index 2
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_undoing_redoing: false,
        }
    }

    /// Creates an empty PieceTree
    pub fn empty() -> Self {
        PieceTree {
            pieces: Vec::new(),
            buffers: vec![String::new()],
            total_char_count: 0,
            total_length: 0,
            next_buffer_index: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_undoing_redoing: false,
        }
    }

    /// Gets the next buffer ID and increments the counter
    fn next_buffer_id(&mut self) -> BufferId {
        self.next_buffer_index += 1;
        BufferId(self.next_buffer_index)
    }

    /// Gets buffer index from BufferId
    #[inline]
    fn buffer_idx(buffer_id: &BufferId) -> usize {
        buffer_id.to_index()
    }

    // ==================== Insertion ====================

    /// Inserts text at the specified byte offset
    /// Returns true if successful
    pub fn insert(&mut self, offset: usize, text: String) -> bool {
        if text.is_empty() {
            return true;
        }

        let char_count = text.chars().count();
        let byte_count = text.len();
        
        // Clamp offset
        let offset = std::cmp::min(offset, self.total_length);

        // Record change for undo
        if !self.is_undoing_redoing {
            self.undo_stack.push(Change::Insert {
                offset,
                length: byte_count,
            });
            self.redo_stack.clear();
        }

        // Debug
        eprintln!("DEBUG insert: offset={}, text='{}' ({} bytes, {} chars)", 
                  offset, text, byte_count, char_count);

        // Add the new text to buffers
        let new_buffer_id = self.next_buffer_id();
        self.buffers.push(text.clone());

        if self.pieces.is_empty() {
            // Empty document - create first piece
            let piece = Piece::new(0, byte_count, new_buffer_id, char_count);
            self.pieces.push(piece);
            self.total_char_count += char_count;
            self.total_length += byte_count;
            return true;
        }

        // Find position to insert (in characters, then convert to bytes)
        let (piece_idx, char_offset) = match self.find_piece_and_char_offset(offset) {
            Some(result) => result,
            None => {
                // Insert at end
                let piece = Piece::new(0, byte_count, new_buffer_id, char_count);
                self.pieces.push(piece);
                self.total_char_count += char_count;
                self.total_length += byte_count;
                return true;
            }
        };

        eprintln!("DEBUG: piece_idx={}, char_offset={}", piece_idx, char_offset);

        let piece = &mut self.pieces[piece_idx];
        
        if char_offset == 0 {
            // Insert at the beginning of this piece
            let new_piece = Piece::new(0, byte_count, new_buffer_id, char_count);
            self.pieces.insert(piece_idx, new_piece);
            eprintln!("DEBUG: insert at beginning, pieces.len()={}", self.pieces.len());
        } else if char_offset == piece.piece_char_length {
            // Insert at the end of this piece
            let new_piece = Piece::new(0, byte_count, new_buffer_id, char_count);
            self.pieces.insert(piece_idx + 1, new_piece);
            eprintln!("DEBUG: insert at end, pieces.len()={}", self.pieces.len());
        } else {
            // Split the piece and insert in the middle
            // Get buffer for the piece being split
            let piece_buffer_idx = Self::buffer_idx(&piece.buffer_id);
            let piece_buffer = &self.buffers[piece_buffer_idx];
            
            // Find the byte offset that corresponds to the character boundary
            let left_text: String = piece_buffer[piece.start..piece.start + piece.length]
                .chars()
                .take(char_offset)
                .collect();
            let left_byte_count = left_text.len();

            eprintln!("DEBUG: left_text='{}' ({} bytes)", left_text, left_byte_count);

            // Capture original values before updating
            let original_piece_length = piece.length;
            let original_piece_char_length = piece.piece_char_length;
            
            // Find the byte offset of the character at char_offset in the original piece
            let piece_buffer_idx = Self::buffer_idx(&piece.buffer_id);
            let piece_buffer = &self.buffers[piece_buffer_idx];
            let original_piece_text = &piece_buffer[piece.start..piece.start + original_piece_length];
            
            // Get the byte offset right AFTER the character at char_offset - 1
            // This is where the right piece should start
            let right_piece_byte_offset: usize = if char_offset == 0 {
                0
            } else {
                original_piece_text.char_indices()
                    .nth(char_offset - 1)
                    .map(|(byte_idx, c)| byte_idx + c.len_utf8())
                    .unwrap_or(original_piece_length)
            };

            eprintln!("DEBUG: right_piece_byte_offset={}", right_piece_byte_offset);

            // Update left piece
            piece.length = left_byte_count;
            piece.piece_char_length = char_offset;

            // Create right piece with correct values
            let right_piece = Piece::new(
                piece.start + right_piece_byte_offset,
                original_piece_length - right_piece_byte_offset,
                piece.buffer_id,
                original_piece_char_length - char_offset,
            );

            // Insert new piece and right piece
            let new_piece = Piece::new(0, byte_count, new_buffer_id, char_count);
            
            if right_piece.buffer_id == new_piece.buffer_id && 
               right_piece.start == new_piece.start + new_piece.length {
                // Merge right_piece into new_piece
                let mut merged_piece = new_piece;
                merged_piece.length += right_piece.length;
                merged_piece.piece_char_length += right_piece.piece_char_length;
                
                // Just insert merged_piece at piece_idx + 1
                self.pieces.insert(piece_idx + 1, merged_piece);
            } else {
                // Can't merge with right_piece, insert both
                self.pieces.insert(piece_idx + 1, new_piece);
                self.pieces.insert(piece_idx + 2, right_piece);
            }
            
            eprintln!("DEBUG: insert middle, pieces.len()={}", self.pieces.len());
        }

        self.total_char_count += char_count;
        self.total_length += byte_count;
        true
    }

    /// Finds the piece and character offset for a given byte position
    fn find_piece_and_char_offset(&self, byte_offset: usize) -> Option<(usize, usize)> {
        if byte_offset > self.total_length {
            return None;
        }

        if byte_offset == 0 {
            if !self.pieces.is_empty() {
                return Some((0, 0));
            }
            return None;
        }

        let mut accumulated_bytes = 0usize;
        let mut accumulated_chars = 0usize;

        for (idx, piece) in self.pieces.iter().enumerate() {
            let piece_start_bytes = accumulated_bytes;
            let piece_end_bytes = accumulated_bytes + piece.length;

            if byte_offset < piece_start_bytes {
                continue;
            }

            if byte_offset < piece_end_bytes {
                // Calculate character offset within this piece (piece-local, not cumulative)
                let piece_buffer_idx = Self::buffer_idx(&piece.buffer_id);
                if let Some(buffer) = self.buffers.get(piece_buffer_idx) {
                    let piece_text = &buffer[piece.start..piece.start + piece.length];
                    
                    // Find the byte offset of the character that contains byte_offset
                    // or the character right before it if byte_offset is in the middle of a char
                    let byte_offset_in_piece = byte_offset - piece_start_bytes;
                    let char_byte_offsets: Vec<usize> = piece_text.char_indices()
                        .map(|(byte_idx, _)| byte_idx)
                        .collect();
                    
                    // Find the valid byte offset (character boundary)
                    let valid_byte_offset = if char_byte_offsets.is_empty() {
                        0
                    } else if byte_offset_in_piece >= *char_byte_offsets.last().unwrap() + 1 {
                        // After the last character, use the end
                        piece_text.len()
                    } else {
                        // Find the character whose byte range contains byte_offset_in_piece
                        // The character starts at or before byte_offset
                        // Use >= to select the character STARTING AT byte_offset when exactly matching
                        let mut valid = *char_byte_offsets.last().unwrap();
                        for &char_start in &char_byte_offsets {
                            if char_start >= byte_offset_in_piece {
                                valid = char_start;
                                break;
                            }
                            valid = char_start;
                        }
                        valid
                    };
                    
                    // Count characters before the valid byte offset
                    let char_offset = char_byte_offsets.iter()
                        .take_while(|&&byte_idx| byte_idx < valid_byte_offset)
                        .count();
                    
                    return Some((idx, char_offset));
                }
            }

            if byte_offset == piece_end_bytes {
                // If there's a next piece, insert at the start of it
                if idx + 1 < self.pieces.len() {
                    return Some((idx + 1, 0));
                }
                // Otherwise, insert at the end of the last piece
                return Some((idx, piece.piece_char_length));
            }

            accumulated_bytes = piece_end_bytes;
            accumulated_chars += piece.piece_char_length;
        }

        // Position at the very end
        if accumulated_bytes == byte_offset && !self.pieces.is_empty() {
            let last_idx = self.pieces.len() - 1;
            return Some((last_idx, accumulated_chars + self.pieces[last_idx].piece_char_length));
        }

        None
    }

    /// Helper to get the original length of a piece from its buffer
    fn piece_length_original(&self) -> usize {
        self.buffers[Self::buffer_idx(&self.pieces[0].buffer_id)].len()
    }

    /// Helper to get the original character length of a piece
    fn piece_char_length_original(&self) -> usize {
        self.buffers[Self::buffer_idx(&self.pieces[0].buffer_id)].chars().count()
    }

    /// Finds the piece and offset for a given byte position
    fn find_piece_and_offset(&self, offset: usize) -> Option<(usize, usize)> {
        if offset > self.total_length {
            return None;
        }

        if offset == 0 {
            // Special case: return the first piece at offset 0
            if !self.pieces.is_empty() {
                return Some((0, 0));
            }
            return None;
        }

        let mut accumulated = 0usize;

        for (idx, piece) in self.pieces.iter().enumerate() {
            let piece_start = accumulated;
            let piece_end = accumulated + piece.length;

            if offset < piece_start {
                continue;
            }

            if offset < piece_end {
                return Some((idx, offset - piece_start));
            }

            if offset == piece_end {
                // Return position after this piece
                return Some((idx, piece.length));
            }

            accumulated = piece_end;
        }

        // Position at the very end
        if accumulated == offset && !self.pieces.is_empty() {
            let last_idx = self.pieces.len() - 1;
            return Some((last_idx, self.pieces[last_idx].length));
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
            let deleted_text = self.get_text_range(offset, length);
            self.undo_stack.push(Change::Delete {
                offset,
                text: deleted_text,
            });
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
                let left_piece = Piece::new(
                    piece.start,
                    delete_start_in_piece,
                    piece.buffer_id,
                    delete_start_in_piece,
                );
                new_pieces.push(left_piece);
            }

            if delete_end_in_piece < piece.length {
                // Keep right part - calculate correct start position
                let right_start = piece.start + delete_end_in_piece;
                let right_length = piece.length - delete_end_in_piece;
                let right_piece = Piece::new(
                    right_start,
                    right_length,
                    piece.buffer_id,
                    right_length,
                );
                new_pieces.push(right_piece);
            }

            current_offset = piece_end;
        }

        self.pieces = new_pieces;
        self.total_char_count = self.total_char_count.saturating_sub(deleted_chars);
        self.total_length = self.total_length.saturating_sub(deleted_bytes);

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
            let buffer = &self.buffers[buffer_idx];
            result.push_str(&buffer[piece.start + start_in_piece..piece.start + end_in_piece]);

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
            return true;
        }
        false
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
}

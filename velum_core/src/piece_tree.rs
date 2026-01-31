#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Original,
    Add,
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub source: Source,
    pub start: usize,
    pub length: usize,
}

pub struct PieceTree {
    original_buffer: String,
    added_buffer: String,
    pieces: Vec<Piece>,
}

impl PieceTree {
    pub fn new(content: String) -> Self {
        let length = content.len();
        Self {
            original_buffer: content,
            added_buffer: String::new(),
            pieces: vec![Piece {
                source: Source::Original,
                start: 0,
                length,
            }],
        }
    }

    pub fn insert(&mut self, offset: usize, text: String) {
        let add_start = self.added_buffer.len();
        let add_len = text.len();
        self.added_buffer.push_str(&text);

        let new_piece = Piece {
            source: Source::Add,
            start: add_start,
            length: add_len,
        };

        if offset == 0 {
            self.pieces.insert(0, new_piece);
            return;
        }

        let mut current_offset = 0;
        for i in 0..self.pieces.len() {
            let piece_len = self.pieces[i].length;
            if current_offset + piece_len >= offset {
                let split_at = offset - current_offset;
                if split_at == 0 {
                    self.pieces.insert(i, new_piece);
                    return;
                } else if split_at == piece_len {
                    self.pieces.insert(i + 1, new_piece);
                    return;
                } else {
                    let left = Piece {
                        source: self.pieces[i].source,
                        start: self.pieces[i].start,
                        length: split_at,
                    };
                    let right = Piece {
                        source: self.pieces[i].source,
                        start: self.pieces[i].start + split_at,
                        length: piece_len - split_at,
                    };
                    self.pieces[i] = left;
                    self.pieces.insert(i + 1, new_piece);
                    self.pieces.insert(i + 2, right);
                    return;
                }
            }
            current_offset += piece_len;
        }

        // If offset is at the very end
        self.pieces.push(new_piece);
    }

    pub fn get_text(&self) -> String {
        let mut result = String::new();
        for piece in &self.pieces {
            let buffer = match piece.source {
                Source::Original => &self.original_buffer,
                Source::Add => &self.added_buffer,
            };
            result.push_str(&buffer[piece.start..piece.start + piece.length]);
        }
        result
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
}

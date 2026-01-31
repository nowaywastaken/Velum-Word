//! # Line Breaking Module
//!
//! Implements the Knuth-Plass line breaking algorithm for optimal text layout.
//! This module provides efficient line breaking with hyphenation support.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use unicode_segmentation::UnicodeSegmentation;

/// Represents the type of line break
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakType {
    /// Explicit newline in the source text
    HardBreak,
    /// Automatic line wrap
    SoftBreak,
    /// Line ends with a hyphen
    Hyphenated,
}

/// Represents a single line after breaking
#[derive(Debug, Clone)]
pub struct Line {
    /// Start byte offset in the original text
    pub start: usize,
    /// End byte offset in the original text
    pub end: usize,
    /// Width of the line in abstract units
    pub width: f32,
    /// Type of break that ended this line
    pub break_type: BreakType,
}

impl Line {
    /// Creates a new line with the given parameters
    #[inline]
    pub fn new(start: usize, end: usize, width: f32, break_type: BreakType) -> Self {
        Line {
            start,
            end,
            width,
            break_type,
        }
    }

    /// Returns the length of the line in characters
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if the line is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Character width information for text measurement
#[derive(Debug, Clone)]
pub struct CharWidth {
    /// Character
    pub ch: char,
    /// Width in abstract units
    pub width: f32,
}

/// Simple font metrics for ASCII text (space=32 to tilde=127, 96 characters)
const ASCII_WIDTHS: [f32; 96] = [
    // Space to ? (32-63)
    0.278, 0.278, 0.365, 0.365, 0.556, 0.556, 0.333, 0.333,
    0.365, 0.365, 0.278, 0.278, 0.365, 0.278, 0.278, 0.365,
    0.278, 0.278, 0.365, 0.278, 0.278, 0.365, 0.278, 0.278,
    0.365, 0.278, 0.278, 0.365, 0.278, 0.278, 0.365, 0.278,
    // @ to O (64-79)
    0.556, 0.556, 0.556, 0.556, 0.556, 0.556, 0.556, 0.556,
    0.556, 0.556, 0.556, 0.556, 0.556, 0.556, 0.556, 0.556,
    // P to _ (80-95)
    0.556, 0.556, 0.556, 0.556, 0.556, 0.556, 0.556, 0.556,
    0.556, 0.556, 0.278, 0.278, 0.365, 0.365, 0.365, 0.365,
    // ` to o (96-111)
    0.278, 0.5, 0.556, 0.556, 0.556, 0.556, 0.278, 0.333,
    0.556, 0.556, 0.333, 0.333, 0.365, 0.333, 0.556, 0.556,
    // p to ~ (112-127)
    0.278, 0.278, 0.5, 0.278, 0.833, 0.5, 0.5, 0.365,
    0.278, 0.5, 0.333, 0.444, 0.333, 0.333, 0.5, 0.278,
];

/// Common CJK character width (typically full-width)
const CJK_WIDTH: f32 = 1.0;

/// Minimum ratio for considering a line "too loose"
const TOO_LOOSE_RATIO: f32 = 0.7;

/// Maximum ratio for considering a line "too tight"
const TOO_TIGHT_RATIO: f32 = 1.5;

/// Maximum number of characters to look ahead for break points
const MAX_LOOKAHEAD: usize = 20;

/// Penalties for various line break situations
const PENALTY_HYPHEN: i32 = 50;
const PENALTY_HARD: i32 = 100;
const PENALTY_FLAGGED: i32 = 20;

/// Demerits multipliers
const DEMERITS_FLAGGED: f32 = 100.0;
const DEMERITS_DOUBLE: f32 = 50.0;
const DEMERITS_HYPHEN: f32 = 30.0;

/// Line breaker configuration
#[derive(Debug, Clone)]
pub struct LineBreakerConfig {
    /// Maximum line width
    pub max_width: f32,
    /// Hyphenation enabled
    pub hyphenation_enabled: bool,
    /// Tab width in abstract units
    pub tab_width: f32,
    /// Word spacing adjustment
    pub word_spacing: f32,
}

impl Default for LineBreakerConfig {
    fn default() -> Self {
        LineBreakerConfig {
            max_width: 500.0,
            hyphenation_enabled: true,
            tab_width: 40.0,
            word_spacing: 4.0,
        }
    }
}

/// Represents a potential break point in the text
#[derive(Debug, Clone)]
struct BreakPoint {
    /// Position in the text (byte offset)
    position: usize,
    /// Character offset from line start
    char_offset: usize,
    /// Width from line start to this point
    width: f32,
    /// Type of break
    break_type: BreakType,
    /// Whether this is a hyphenated break
    is_hyphenated: bool,
    /// Penalty for this break
    penalty: i32,
    /// Whether this break is flagged (problematic)
    flagged: bool,
}

/// Box for use in binary heap (needs to be Clone for BinaryHeap)
#[derive(Debug, Clone)]
struct BreakBox {
    break_point: BreakPoint,
    demerits: f32,
    line_number: usize,
}

impl Eq for BreakBox {}

impl PartialEq for BreakBox {
    fn eq(&self, other: &Self) -> bool {
        self.demerits == other.demerits
    }
}

impl Ord for BreakBox {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap behavior
        other.demerits.partial_cmp(&self.demerits).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for BreakBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.cmp(self))
    }
}

/// Main line breaker struct implementing the Knuth-Plass algorithm
#[derive(Debug, Clone)]
pub struct LineBreaker {
    pub config: LineBreakerConfig,
    // Cache for text widths
    width_cache: HashMap<String, f32>,
    // Cache for character widths
    char_widths: HashMap<char, f32>,
}

impl Default for LineBreaker {
    fn default() -> Self {
        LineBreaker::new()
    }
}

impl LineBreaker {
    /// Creates a new line breaker with default configuration
    #[inline]
    pub fn new() -> Self {
        LineBreaker {
            config: LineBreakerConfig::default(),
            width_cache: HashMap::new(),
            char_widths: HashMap::new(),
        }
    }

    /// Creates a new line breaker with the given configuration
    #[inline]
    pub fn with_config(config: LineBreakerConfig) -> Self {
        LineBreaker {
            config,
            width_cache: HashMap::new(),
            char_widths: HashMap::new(),
        }
    }

    /// Creates a line breaker for a specific width
    #[inline]
    pub fn with_width(max_width: f32) -> Self {
        let mut config = LineBreakerConfig::default();
        config.max_width = max_width;
        LineBreaker::with_config(config)
    }

    /// Updates the maximum width
    #[inline]
    pub fn set_max_width(&mut self, max_width: f32) {
        self.config.max_width = max_width;
    }

    /// Enables or disables hyphenation
    #[inline]
    pub fn set_hyphenation(&mut self, enabled: bool) {
        self.config.hyphenation_enabled = enabled;
    }

    /// Gets the width of a single character
    #[inline]
    fn char_width(&mut self, ch: char) -> f32 {
        if let Some(width) = self.char_widths.get(&ch) {
            return *width;
        }

        let width = match ch {
            '\t' => self.config.tab_width,
            ' ' => 0.25, // Standard space width
            c if c.is_ascii() && (c as u8) >= 32 && (c as u8) < 128 => {
                ASCII_WIDTHS[(c as u8 - 32) as usize]
            }
            // CJK and other wide characters
            c if c as u32 >= 0x1100 => CJK_WIDTH, // Hangul, CJK, etc.
            c if c as u32 >= 0x3000 => CJK_WIDTH, // CJK symbols and punctuation
            // Default to average character width
            _ => 0.5,
        };

        self.char_widths.insert(ch, width);
        width
    }

    /// Calculates the width of a substring
    fn text_width(&mut self, text: &str) -> f32 {
        // Check cache first
        if let Some(&width) = self.width_cache.get(text) {
            return width;
        }

        let mut width = 0.0f32;
        for ch in text.chars() {
            width += self.char_width(ch);
        }

        self.width_cache.insert(text.to_string(), width);
        width
    }

    /// Clears the width cache
    #[inline]
    pub fn clear_cache(&mut self) {
        self.width_cache.clear();
        self.char_widths.clear();
    }

    /// Checks if a character is a valid break point (after the character)
    #[inline]
    fn is_break_after(&self, ch: char) -> bool {
        matches!(
            ch,
            ' ' | '\t' | '-' | '–' | '—' | ';' | ':' | ',' | '.' | '!' | '?' | ')' | ']' | '}'
        )
    }

    /// Checks if a character can be hyphenated
    #[inline]
    fn can_hyphenate(&self, ch: char) -> bool {
        ch.is_alphabetic() && !ch.is_ascii() || ch.is_ascii_alphabetic()
    }

    /// Checks if a character is CJK
    #[inline]
    fn is_cjk(&self, ch: char) -> bool {
        let code = ch as u32;
        // CJK Unified Ideographs
        (code >= 0x4E00 && code <= 0x9FFF)
        || // CJK Unified Ideographs Extension A
        (code >= 0x3400 && code <= 0x4DBF)
        || // CJK Symbols and Punctuation
        (code >= 0x3000 && code <= 0x303F)
        || // Hiragana
        (code >= 0x3040 && code >= 0x309F)
        || // Katakana
        (code >= 0x30A0 && code <= 0x30FF)
    }

    /// Gets break points for a line
    fn get_break_points(&mut self, text: &str) -> Vec<BreakPoint> {
        let mut break_points: Vec<BreakPoint> = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        // Add start break point
        break_points.push(BreakPoint {
            position: 0,
            char_offset: 0,
            width: 0.0,
            break_type: BreakType::SoftBreak,
            is_hyphenated: false,
            penalty: 0,
            flagged: false,
        });

        let mut current_width = 0.0f32;
        let mut in_word = false;
        let mut word_start = 0usize;

        for (i, ch) in chars.iter().enumerate() {
            let char_width = self.char_width(*ch);
            current_width += char_width;

            // Handle CJK characters - each can be a break point
            if self.is_cjk(*ch) {
                // CJK: break after each character
                break_points.push(BreakPoint {
                    position: text.floor_char_to_byte(i + 1),
                    char_offset: i + 1,
                    width: current_width,
                    break_type: BreakType::SoftBreak,
                    is_hyphenated: false,
                    penalty: 0,
                    flagged: false,
                });
                in_word = false;
                continue;
            }

            // Handle ASCII/whitespace
            if *ch == ' ' || *ch == '\t' {
                in_word = false;
                continue;
            }

            // Start of a word
            if !in_word {
                word_start = i;
                in_word = true;
            }

            // Check if we can break after this character
            if self.is_break_after(*ch) {
                // Calculate penalty based on character
                let penalty = match *ch {
                    '-' | '–' | '—' => PENALTY_HYPHEN,
                    '!' | '?' => PENALTY_HARD,
                    _ => 0,
                };

                break_points.push(BreakPoint {
                    position: text.floor_char_to_byte(i + 1),
                    char_offset: i + 1,
                    width: current_width,
                    break_type: BreakType::SoftBreak,
                    is_hyphenated: false,
                    penalty,
                    flagged: false,
                });
            }
        }

        // Add end break point
        break_points.push(BreakPoint {
            position: text.len(),
            char_offset: len,
            width: current_width,
            break_type: BreakType::HardBreak,
            is_hyphenated: false,
            penalty: 0,
            flagged: false,
        });

        break_points
    }

    /// Simple syllable-based hyphenation (basic implementation)
    fn get_hyphenation_points(&self, text: &str) -> Vec<usize> {
        if !self.config.hyphenation_enabled {
            return Vec::new();
        }

        let mut points = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        // Very basic hyphenation: break before last 3+ characters
        // In a full implementation, this would use a hyphenation dictionary
        if len > 4 {
            // Conservative: only hyphenate long words
            points.push(len - 2);
        }

        points
    }

    /// Calculates demerits for a line based on its ratio to max width
    fn calculate_demerits(&self, line_width: f32, line_number: usize, ratio: f32) -> f32 {
        let demerits = if ratio > 1.0 {
            // Line is too long - square of excess ratio
            (ratio - 1.0).powi(2) * 100.0
        } else if ratio < TOO_LOOSE_RATIO {
            // Line is very loose - exponential penalty
            (TOO_LOOSE_RATIO / ratio).powi(2) * 10.0
        } else if ratio > TOO_TIGHT_RATIO {
            // Line is very tight
            (ratio / TOO_TIGHT_RATIO).powi(2) * 50.0
        } else {
            // Acceptable ratio
            0.0
        };

        demerits
    }

    /// Main breaking algorithm - finds optimal breaks using Knuth-Plass
    fn find_breaks(&mut self, text: &str) -> Vec<BreakPoint> {
        let break_points = self.get_break_points(text);
        if break_points.len() < 2 {
            return break_points;
        }

        let max_width = self.config.max_width;
        let mut candidates: BinaryHeap<BreakBox> = BinaryHeap::new();
        let mut active_breaks: Vec<(usize, BreakPoint, f32)> = Vec::new(); // (line_number, break_point, total_demerits)
        let mut chosen_breaks: HashMap<usize, (usize, BreakPoint)> = HashMap::new(); // position -> (prev_position, break_point)

        // Initialize with first break point
        if let Some(first) = break_points.first() {
            active_breaks.push((0, first.clone(), 0.0));
        }

        let mut best_break: Option<BreakPoint> = None;
        let mut best_demerits = f32::MAX;

        // Process break points
        for (i, current) in break_points.iter().enumerate() {
            if current.position == 0 {
                continue;
            }

            // Try to find breaks ending at this position
            let mut new_candidates: Vec<(usize, BreakPoint, f32)> = Vec::new();

            for (line_num, prev_break, total_demerits) in &active_breaks {
                // Calculate line width
                let line_width = current.width - prev_break.width;

                // Skip if line is too long
                if line_width > max_width * 2.0 {
                    continue;
                }

                // Calculate ratio and demerits
                let ratio = if line_width > 0.001 {
                    max_width / line_width
                } else {
                    f32::MAX
                };

                let line_demerits = self.calculate_demerits(line_width, *line_num + 1, ratio);
                let mut total = total_demerits + line_demerits;

                // Add penalty for flagged breaks
                if current.flagged {
                    total += DEMERITS_FLAGGED;
                }

                // Add hyphenation penalty
                if current.is_hyphenated {
                    total += DEMERITS_HYPHEN;
                }

                // Add penalty based on character
                total += current.penalty as f32;

                // Add demerits for double penalties (too many hyphenated lines)
                if *line_num > 0 {
                    // Check previous line's hyphenation
                    if let Some((_, prev_prev)) = chosen_breaks.get(&prev_break.position) {
                        if prev_prev.is_hyphenated && current.is_hyphenated {
                            total += DEMERITS_DOUBLE;
                        }
                    }
                }

                // Limit look-ahead
                if current.char_offset - prev_break.char_offset > MAX_LOOKAHEAD {
                    continue;
                }

                // Check if this is a valid end (hard break or acceptable soft break)
                if current.break_type == BreakType::HardBreak {
                    if total < best_demerits {
                        best_demerits = total;
                        best_break = Some(current.clone());
                    }
                } else if line_width <= max_width {
                    // Valid break point
                    new_candidates.push((*line_num + 1, current.clone(), total));
                    chosen_breaks.insert(current.position, (prev_break.position, current.clone()));
                }
            }

            // Add new candidates and prune
            for candidate in new_candidates {
                active_breaks.push(candidate);
            }

            // Prune: keep only best candidates for each line number
            active_breaks.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));
            if active_breaks.len() > 10 {
                active_breaks.truncate(10);
            }
        }

        // Reconstruct best breaks
        let mut result = Vec::new();
        if let Some(end_break) = best_break {
            let mut pos = end_break.position;
            while pos > 0 {
                if let Some((prev_pos, break_point)) = chosen_breaks.get(&pos) {
                    result.insert(0, break_point.clone());
                    pos = *prev_pos;
                } else {
                    break;
                }
            }
        }

        result
    }

    /// Breaks text into lines with optimal breaks
    pub fn break_lines(&mut self, text: &str, max_width: Option<f32>) -> Vec<Line> {
        if text.is_empty() {
            return Vec::new();
        }

        if let Some(width) = max_width {
            self.config.max_width = width;
        }

        // Split by explicit newlines first
        let paragraphs: Vec<&str> = text.split('\n').collect();
        let mut lines = Vec::new();

        for paragraph in paragraphs {
            if paragraph.is_empty() {
                // Empty line - add a hard break
                lines.push(Line::new(0, 0, 0.0, BreakType::HardBreak));
                continue;
            }

            let breaks = self.find_breaks(paragraph);

            // Convert break points to lines
            let mut prev_end = 0usize;
            for (i, bp) in breaks.iter().enumerate() {
                let start = if i == 0 { 0 } else { prev_end };
                let end = bp.position;

                if end > start {
                    let line_text = &paragraph[start..end];
                    let width = self.text_width(line_text);
                    lines.push(Line::new(start, end, width, bp.break_type));
                }

                prev_end = end;
            }
        }

        lines
    }

    /// Calculates the width of text
    pub fn calculate_text_width(&mut self, text: &str) -> f32 {
        self.text_width(text)
    }
}

/// Extension trait for byte offset calculation
trait ByteOffsetExt {
    fn floor_char_to_byte(&self, char_idx: usize) -> usize;
}

impl ByteOffsetExt for str {
    fn floor_char_to_byte(&self, char_idx: usize) -> usize {
        if char_idx == 0 {
            return 0;
        }

        let mut char_count = 0usize;
        for (byte_idx, _) in self.char_indices() {
            if char_count >= char_idx {
                return byte_idx;
            }
            char_count += 1;
        }
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_line_breaking() {
        let mut breaker = LineBreaker::with_width(100.0);
        let text = "This is a test of line breaking.";
        let lines = breaker.break_lines(text, None);

        assert!(!lines.is_empty());
        for line in &lines {
            assert!(line.width <= 100.0 + 1.0, "Line width {} exceeds max", line.width);
        }
    }

    #[test]
    fn test_empty_text() {
        let mut breaker = LineBreaker::new();
        let lines = breaker.break_lines("", None);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_cjk_line_breaking() {
        let mut breaker = LineBreaker::with_width(50.0);
        let text = "这是一个测试文本，用于测试中文分行";
        let lines = breaker.break_lines(text, None);

        assert!(!lines.is_empty());
        for line in &lines {
            assert!(line.width <= 50.0 + 1.0, "Line width {} exceeds max", line.width);
        }
    }

    #[test]
    fn test_char_width_calculation() {
        let mut breaker = LineBreaker::new();

        // ASCII characters
        assert!(breaker.char_width('a') > 0.0);
        assert!(breaker.char_width(' ') > 0.0);

        // Tab
        assert!(breaker.char_width('\t') > 0.0);

        // CJK
        assert!(breaker.char_width('中') > 0.0);
    }

    #[test]
    fn test_text_width() {
        let mut breaker = LineBreaker::new();
        let width = breaker.calculate_text_width("hello");
        assert!(width > 0.0);

        // Should be cached
        let width2 = breaker.calculate_text_width("hello");
        assert_eq!(width, width2);
    }

    #[test]
    fn test_cache_clearing() {
        let mut breaker = LineBreaker::new();
        breaker.calculate_text_width("test");
        breaker.clear_cache();
        // After clearing, should still work but cache is empty
        let width = breaker.calculate_text_width("test");
        assert!(width > 0.0);
    }

    #[test]
    fn test_line_structure() {
        let mut breaker = LineBreaker::with_width(100.0);
        let text = "Hello world";
        let lines = breaker.break_lines(text, None);

        if !lines.is_empty() {
            for line in &lines {
                assert!(line.start <= line.end);
                assert!(line.width >= 0.0);
            }
        }
    }

    #[test]
    fn test_break_types() {
        let mut breaker = LineBreaker::with_width(100.0);
        let text = "Line one\nLine two\nLine three";
        let lines = breaker.break_lines(text, None);

        // Should have lines from all paragraphs
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_long_text() {
        let mut breaker = LineBreaker::with_width(80.0);
        let text = "This is a longer piece of text that should be broken into multiple lines because it exceeds the maximum width of eighty characters by quite a significant margin.";
        let lines = breaker.break_lines(text, None);

        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.width <= 80.0 + 1.0, "Line width {} exceeds max", line.width);
        }
    }
}

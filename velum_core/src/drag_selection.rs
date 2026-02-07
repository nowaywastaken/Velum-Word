//! # Drag Selection Module
//!
//! Provides drag-to-select functionality with multiple selection modes
//! including character, word, line, column, block, and document selection.
//! Supports both mouse and touch input with auto-scrolling during drag.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Document position representing a location in the document
/// Contains both character offset and visual position for mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentPosition {
    /// Character offset in the document (0-based)
    pub char_offset: usize,
    /// Visual line number (0-based)
    pub line: usize,
    /// Visual column/character position on the line (0-based)
    pub column: usize,
}

impl Default for DocumentPosition {
    fn default() -> Self {
        DocumentPosition {
            char_offset: 0,
            line: 0,
            column: 0,
        }
    }
}

impl DocumentPosition {
    /// Creates a new document position
    pub fn new(char_offset: usize, line: usize, column: usize) -> Self {
        DocumentPosition {
            char_offset,
            line,
            column,
        }
    }

    /// Creates a position at the start of document
    pub fn start() -> Self {
        DocumentPosition::default()
    }

    /// Creates a position at the end of document
    pub fn end() -> Self {
        DocumentPosition::default()
    }

    /// Returns true if this position is at the start
    pub fn is_start(&self) -> bool {
        self.char_offset == 0
    }

    /// Returns true if this position is at the end
    pub fn is_end(&self, total_chars: usize) -> bool {
        self.char_offset >= total_chars.saturating_sub(1)
    }
}

impl fmt::Display for DocumentPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(offset={}, line={}, col={})", self.char_offset, self.line, self.column)
    }
}

/// Selection mode for different drag behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// Normal character-by-character selection (mobile/trackpad)
    Character,
    /// Word selection (double-click)
    Word,
    /// Line selection (triple-click or drag margin)
    Line,
    /// Column selection (Alt + drag)
    Column,
    /// Block selection (rectangular area)
    Block,
    /// Select entire document
    Document,
}

impl Default for SelectionMode {
    fn default() -> Self {
        SelectionMode::Character
    }
}

/// Direction for auto-scrolling during drag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollDirection {
    /// No scrolling needed
    None,
    /// Scroll up
    Up,
    /// Scroll down
    Down,
    /// Scroll left
    Left,
    /// Scroll right
    Right,
    /// Scroll diagonally up-left
    UpLeft,
    /// Scroll diagonally up-right
    UpRight,
    /// Scroll diagonally down-left
    DownLeft,
    /// Scroll diagonally down-right
    DownRight,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::None
    }
}

/// Target type being dragged
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragTarget {
    /// Dragging selected text
    Selection,
    /// Dragging an image
    Image,
    /// Dragging a table
    Table,
}

impl Default for DragTarget {
    fn default() -> Self {
        DragTarget::Selection
    }
}

/// Drag state for tracking selection changes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DragPhase {
    /// Current phase of the drag operation
    pub phase: DragPhaseType,
    /// Timestamp when drag started (in milliseconds)
    pub start_time_ms: u64,
    /// Current timestamp (in milliseconds)
    pub current_time_ms: u64,
    /// Drag distance in pixels from start
    pub distance_x: f32,
    /// Drag distance in pixels from start
    pub distance_y: f32,
    /// Whether this is an extend selection (Shift key)
    pub is_extend: bool,
    /// Whether column/rectangular selection is active
    pub is_column: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DragPhaseType {
    /// No drag operation
    Idle,
    /// Drag operation has started
    Started,
    /// Currently dragging
    Dragging,
    /// Drag operation ended
    Ended,
    /// Drag was cancelled
    Cancelled,
}

impl Default for DragPhase {
    fn default() -> Self {
        DragPhase {
            phase: DragPhaseType::Idle,
            start_time_ms: 0,
            current_time_ms: 0,
            distance_x: 0.0,
            distance_y: 0.0,
            is_extend: false,
            is_column: false,
        }
    }
}

/// Main drag selection state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DragSelectionState {
    /// Whether a drag operation is currently active
    pub is_dragging: bool,
    /// Position where drag started
    pub start_position: DocumentPosition,
    /// Current cursor/mouse position
    pub current_position: DocumentPosition,
    /// Anchor position (fixed end of selection)
    pub anchor_position: DocumentPosition,
    /// Current selection mode
    pub selection_mode: SelectionMode,
    /// Current scroll direction
    pub scroll_direction: ScrollDirection,
    /// Current drag phase
    pub drag_phase: DragPhase,
    /// Whether selection is being extended (Shift key)
    pub extend_selection: bool,
    /// Whether this is column/block selection
    pub is_column_selection: bool,
    /// Selection start in document coordinates
    pub selection_start: usize,
    /// Selection end in document coordinates
    pub selection_end: usize,
    /// Multiple ranges for non-contiguous selection
    pub selection_ranges: Vec<(usize, usize)>,
    /// Current drag target
    pub drag_target: DragTarget,
    /// Visual feedback indicator
    pub show_visual_feedback: bool,
}

impl Default for DragSelectionState {
    fn default() -> Self {
        DragSelectionState {
            is_dragging: false,
            start_position: DocumentPosition::default(),
            current_position: DocumentPosition::default(),
            anchor_position: DocumentPosition::default(),
            selection_mode: SelectionMode::default(),
            scroll_direction: ScrollDirection::default(),
            drag_phase: DragPhase::default(),
            extend_selection: false,
            is_column_selection: false,
            selection_start: 0,
            selection_end: 0,
            selection_ranges: Vec::new(),
            drag_target: DragTarget::Selection,
            show_visual_feedback: false,
        }
    }
}

impl DragSelectionState {
    /// Creates a new drag selection state
    pub fn new() -> Self {
        DragSelectionState::default()
    }

    /// Resets the drag state to initial conditions
    pub fn reset(&mut self) {
        self.is_dragging = false;
        self.start_position = DocumentPosition::default();
        self.current_position = DocumentPosition::default();
        self.anchor_position = DocumentPosition::default();
        self.selection_mode = SelectionMode::default();
        self.scroll_direction = ScrollDirection::default();
        self.drag_phase = DragPhase::default();
        self.extend_selection = false;
        self.is_column_selection = false;
        self.selection_start = 0;
        self.selection_end = 0;
        self.selection_ranges.clear();
        self.show_visual_feedback = false;
    }

    /// Starts a new drag operation from the given position
    pub fn start_drag(
        &mut self,
        position: DocumentPosition,
        mode: SelectionMode,
        is_extend: bool,
        is_column: bool,
        target: DragTarget,
    ) {
        self.is_dragging = true;
        self.start_position = position;
        self.current_position = position;
        self.anchor_position = position;
        self.selection_mode = mode;
        self.extend_selection = is_extend;
        self.is_column_selection = is_column;
        self.drag_target = target;
        self.selection_start = position.char_offset;
        self.selection_end = position.char_offset;
        self.show_visual_feedback = true;

        self.drag_phase = DragPhase {
            phase: DragPhaseType::Started,
            start_time_ms: current_time_ms(),
            current_time_ms: current_time_ms(),
            distance_x: 0.0,
            distance_y: 0.0,
            is_extend,
            is_column,
        };
    }

    /// Updates the current position during drag
    pub fn update_position(&mut self, position: DocumentPosition) {
        if !self.is_dragging {
            return;
        }

        self.current_position = position;
        self.update_selection_range();
        self.update_drag_phase();
    }

    /// Updates drag distance based on current position
    pub fn update_drag_distance(&mut self, start_screen_x: f32, start_screen_y: f32, current_x: f32, current_y: f32) {
        if !self.is_dragging {
            return;
        }

        self.drag_phase.distance_x = current_x - start_screen_x;
        self.drag_phase.distance_y = current_y - start_screen_y;
        self.drag_phase.current_time_ms = current_time_ms();
    }

    /// Updates the selection range based on current positions
    fn update_selection_range(&mut self) {
        if self.is_column_selection {
            // For column selection, we track ranges per line
            self.selection_start = self.anchor_position.char_offset;
            self.selection_end = self.current_position.char_offset;
        } else {
            // Normal selection: anchor is fixed, active moves
            let anchor = self.anchor_position.char_offset;
            let active = self.current_position.char_offset;

            if anchor <= active {
                self.selection_start = anchor;
                self.selection_end = active;
            } else {
                self.selection_start = active;
                self.selection_end = anchor;
            }
        }
    }

    /// Updates the drag phase and scroll direction
    fn update_drag_phase(&mut self) {
        self.drag_phase.phase = DragPhaseType::Dragging;
        self.drag_phase.current_time_ms = current_time_ms();

        // Calculate scroll direction based on position relative to viewport
        self.scroll_direction = self.calculate_scroll_direction();
    }

    /// Ends the drag operation
    pub fn end_drag(&mut self) {
        if !self.is_dragging {
            return;
        }

        self.is_dragging = false;
        self.drag_phase.phase = DragPhaseType::Ended;
        self.drag_phase.current_time_ms = current_time_ms();
        self.scroll_direction = ScrollDirection::None;
    }

    /// Cancels the drag operation
    pub fn cancel_drag(&mut self) {
        self.is_dragging = false;
        self.drag_phase.phase = DragPhaseType::Cancelled;
        self.drag_phase.current_time_ms = current_time_ms();
        self.scroll_direction = ScrollDirection::None;
    }

    /// Sets the selection mode
    pub fn set_mode(&mut self, mode: SelectionMode) {
        self.selection_mode = mode;
    }

    /// Toggles column selection mode
    pub fn toggle_column_mode(&mut self) {
        self.is_column_selection = !self.is_column_selection;
        self.drag_phase.is_column = self.is_column_selection;
    }

    /// Sets whether selection is being extended
    pub fn set_extend(&mut self, extend: bool) {
        self.extend_selection = extend;
        self.drag_phase.is_extend = extend;
    }

    /// Adds a selection range (for non-contiguous selection)
    pub fn add_selection_range(&mut self, start: usize, end: usize) {
        self.selection_ranges.push((start, end));
    }

    /// Clears all selection ranges
    pub fn clear_selection_ranges(&mut self) {
        self.selection_ranges.clear();
    }

    /// Gets the current selection as a single range
    pub fn get_selection(&self) -> (usize, usize) {
        (self.selection_start, self.selection_end)
    }

    /// Gets all selection ranges
    pub fn get_all_selections(&self) -> &[(usize, usize)] {
        &self.selection_ranges
    }

    /// Returns true if there is an active selection
    pub fn has_selection(&self) -> bool {
        if !self.selection_ranges.is_empty() {
            return true;
        }
        self.selection_end > self.selection_start
    }

    /// Returns the selection length
    pub fn selection_length(&self) -> usize {
        self.selection_end.saturating_sub(self.selection_start)
    }

    /// Calculates the scroll direction based on position
    fn calculate_scroll_direction(&mut self) -> ScrollDirection {
        // Position-based scroll direction would be calculated here
        // based on viewport boundaries
        ScrollDirection::None
    }

    /// Determines the appropriate selection mode for a drag operation
    pub fn determine_selection_mode(
        &self,
        is_on_margin: bool,
        is_alt_pressed: bool,
        click_count: u32,
    ) -> SelectionMode {
        if is_alt_pressed {
            SelectionMode::Column
        } else if is_on_margin {
            SelectionMode::Line
        } else {
            match click_count {
                2 => SelectionMode::Word,
                3 => SelectionMode::Line,
                _ => SelectionMode::Character,
            }
        }
    }

    /// Gets the current drag phase
    pub fn phase(&self) -> DragPhaseType {
        self.drag_phase.phase
    }

    /// Returns true if drag is active
    pub fn is_active(&self) -> bool {
        self.is_dragging && self.drag_phase.phase == DragPhaseType::Dragging
    }

    /// Returns true if drag has started but not yet moved
    pub fn is_just_started(&self) -> bool {
        self.is_dragging && self.drag_phase.phase == DragPhaseType::Started
    }

    /// Gets the drag velocity (pixels per ms)
    pub fn drag_velocity(&self) -> (f32, f32) {
        let elapsed = self.drag_phase.current_time_ms.saturating_sub(self.drag_phase.start_time_ms);
        if elapsed == 0 {
            (0.0, 0.0)
        } else {
            (
                self.drag_phase.distance_x / elapsed as f32,
                self.drag_phase.distance_y / elapsed as f32,
            )
        }
    }
}

/// Gets current timestamp in milliseconds
#[inline]
fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Configuration for drag selection behavior
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DragSelectionConfig {
    /// Minimum drag distance in pixels to start selection
    pub drag_threshold: f32,
    /// Scroll speed multiplier
    pub scroll_speed: f32,
    /// Auto-scroll delay in ms
    pub auto_scroll_delay_ms: u64,
    /// Word boundaries for word selection
    pub word_separators: &'static str,
    /// Enable column selection with Alt key
    pub alt_column_selection: bool,
    /// Enable line selection on margin drag
    pub margin_line_selection: bool,
    /// Enable triple-click for line selection
    pub triple_click_line_selection: bool,
    /// Visual feedback style
    pub feedback_style: VisualFeedbackStyle,
}

impl Default for DragSelectionConfig {
    fn default() -> Self {
        DragSelectionConfig {
            drag_threshold: 5.0,
            scroll_speed: 1.0,
            auto_scroll_delay_ms: 100,
            word_separators: " \t\n\r.,!?;:\"'()[]{}<>/\\|@#$%^&*-_+=`~",
            alt_column_selection: true,
            margin_line_selection: true,
            triple_click_line_selection: true,
            feedback_style: VisualFeedbackStyle::Highlight,
        }
    }
}

/// Visual feedback style for selections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualFeedbackStyle {
    /// Simple highlight background
    Highlight,
    /// Dashed underline
    Underline,
    /// Cursor line highlight
    CursorLine,
    /// Bracket matching style
    Brackets,
    /// No visual feedback
    None,
}

/// Handles word boundary detection for word selection
pub mod word_boundary {

    /// Finds the word boundary at the given offset in text
    pub fn find_word_start(text: &str, offset: usize) -> usize {
        if text.is_empty() || offset == 0 {
            return 0;
        }

        let chars: Vec<char> = text.chars().collect();
        let offset = offset.min(chars.len());

        // Look for word start (non-separator after separator, or start of text)
        let separators = " \t\n\r.,!?;:\"'()[]{}<>/\\|@#$%^&*-_+=`~";

        // Find the start of the current/previous word
        let mut i = offset.saturating_sub(1);
        while i > 0 {
            if !separators.contains(chars[i]) && separators.contains(chars[i - 1]) {
                return i;
            }
            i = i.saturating_sub(1);
        }

        // Either at start or first char is non-separator
        if chars.get(0).map(|c| !separators.contains(*c)).unwrap_or(false) {
            0
        } else {
            // Find first non-separator
            chars.iter()
                .position(|c| !separators.contains(*c))
                .unwrap_or(0)
        }
    }

    /// Finds the word boundary at the given offset in text
    pub fn find_word_end(text: &str, offset: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        if chars.is_empty() {
            return 0;
        }

        let separators = " \t\n\r.,!?;:\"'()[]{}<>/\\|@#$%^&*-_+=`~";

        // Start from offset and find end of word
        let mut i = offset.min(chars.len().saturating_sub(1));

        // Skip separators first
        while i < chars.len() && separators.contains(chars[i]) {
            i += 1;
        }

        // Find end of word
        while i < chars.len() && !separators.contains(chars[i]) {
            i += 1;
        }

        i
    }

    /// Gets the word at the given offset
    pub fn get_word_at(text: &str, offset: usize) -> Option<(usize, usize)> {
        if text.is_empty() {
            return None;
        }

        let start = find_word_start(text, offset);
        let end = find_word_end(text, offset);

        if start < end {
            Some((start, end))
        } else {
            None
        }
    }
}

/// Handles line boundary detection for line selection
pub mod line_boundary {

    /// Gets the line range for a given line number
    pub fn get_line_range(text: &str, line_number: usize) -> Option<(usize, usize)> {
        if text.is_empty() {
            return None;
        }

        let mut line_start = 0usize;
        let mut current_line = 0usize;

        for (byte_idx, ch) in text.char_indices() {
            if ch == '\n' {
                if current_line == line_number {
                    return Some((line_start, byte_idx));
                }
                current_line += 1;
                line_start = byte_idx + 1;
            }
        }

        // Handle last line (no trailing newline)
        if current_line == line_number {
            Some((line_start, text.len()))
        } else {
            None
        }
    }

    /// Gets the line number for a given character offset
    pub fn get_line_number(text: &str, offset: usize) -> usize {
        if text.is_empty() {
            return 0;
        }

        text.char_indices()
            .take(offset)
            .filter(|(_, ch)| *ch == '\n')
            .count()
    }

    /// Gets the start offset of a line
    pub fn get_line_start(text: &str, line_number: usize) -> Option<usize> {
        get_line_range(text, line_number).map(|(start, _)| start)
    }

    /// Gets the end offset of a line
    pub fn get_line_end(text: &str, line_number: usize) -> Option<usize> {
        get_line_range(text, line_number).map(|(_, end)| end)
    }
}

/// Main drag selection handler
#[derive(Debug, Clone)]
pub struct DragSelectionHandler {
    state: DragSelectionState,
    config: DragSelectionConfig,
}

impl Default for DragSelectionHandler {
    fn default() -> Self {
        DragSelectionHandler::new()
    }
}

impl DragSelectionHandler {
    /// Creates a new drag selection handler
    pub fn new() -> Self {
        DragSelectionHandler {
            state: DragSelectionState::new(),
            config: DragSelectionConfig::default(),
        }
    }

    /// Creates a handler with custom configuration
    pub fn with_config(config: DragSelectionConfig) -> Self {
        DragSelectionHandler {
            state: DragSelectionState::new(),
            config,
        }
    }

    /// Gets the current state
    pub fn state(&self) -> &DragSelectionState {
        &self.state
    }

    /// Gets a mutable reference to the state
    pub fn state_mut(&mut self) -> &mut DragSelectionState {
        &mut self.state
    }

    /// Gets the configuration
    pub fn config(&self) -> &DragSelectionConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn set_config(&mut self, config: DragSelectionConfig) {
        self.config = config;
    }

    /// Handles mouse/touch down event
    pub fn on_pointer_down(
        &mut self,
        position: DocumentPosition,
        mode: SelectionMode,
        is_extend: bool,
        target: DragTarget,
    ) {
        self.state.start_drag(position, mode, is_extend, false, target);
    }

    /// Handles pointer move event
    pub fn on_pointer_move(&mut self, position: DocumentPosition) {
        self.state.update_position(position);
    }

    /// Handles pointer up event
    pub fn on_pointer_up(&mut self) {
        self.state.end_drag();
    }

    /// Handles pointer cancel event
    pub fn on_pointer_cancel(&mut self) {
        self.state.cancel_drag();
    }

    /// Handles Alt key press for column selection toggle
    pub fn on_alt_key_pressed(&mut self) {
        if self.config.alt_column_selection {
            self.state.toggle_column_mode();
        }
    }

    /// Handles mode change
    pub fn on_mode_change(&mut self, mode: SelectionMode) {
        self.state.set_mode(mode);
    }

    /// Gets the current selection as ranges
    pub fn get_selection_ranges(&self) -> Vec<(usize, usize)> {
        let mut ranges = self.state.selection_ranges.clone();

        if self.state.selection_start < self.state.selection_end {
            ranges.insert(0, (self.state.selection_start, self.state.selection_end));
        }

        ranges
    }

    /// Gets the primary selection
    pub fn get_primary_selection(&self) -> Option<(usize, usize)> {
        if self.state.selection_start < self.state.selection_end {
            Some((self.state.selection_start, self.state.selection_end))
        } else {
            None
        }
    }

    /// Expands selection to word boundaries
    pub fn expand_to_word(&self, text: &str, offset: usize) -> (usize, usize) {
        let start = word_boundary::find_word_start(text, offset);
        let end = word_boundary::find_word_end(text, offset);
        (start, end)
    }

    /// Expands selection to line boundaries
    pub fn expand_to_line(&self, text: &str, line: usize) -> (usize, usize) {
        line_boundary::get_line_range(text, line).unwrap_or((0, text.len()))
    }

    /// Expands selection to document boundaries
    pub fn expand_to_document(&self, _text: &str) -> (usize, usize) {
        (0, usize::MAX)
    }

    /// Determines if auto-scroll is needed
    pub fn needs_auto_scroll(&self) -> bool {
        self.state.scroll_direction != ScrollDirection::None
    }

    /// Gets the auto-scroll delta
    pub fn get_scroll_delta(&self) -> (i32, i32) {
        match self.state.scroll_direction {
            ScrollDirection::None => (0, 0),
            ScrollDirection::Up => (0, -1),
            ScrollDirection::Down => (0, 1),
            ScrollDirection::Left => (-1, 0),
            ScrollDirection::Right => (1, 0),
            ScrollDirection::UpLeft => (-1, -1),
            ScrollDirection::UpRight => (1, -1),
            ScrollDirection::DownLeft => (-1, 1),
            ScrollDirection::DownRight => (1, 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_selection_state_default() {
        let state = DragSelectionState::default();

        assert!(!state.is_dragging);
        assert_eq!(state.selection_start, 0);
        assert_eq!(state.selection_end, 0);
        assert!(!state.has_selection());
    }

    #[test]
    fn test_drag_selection_start_drag() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);

        assert!(state.is_dragging);
        assert_eq!(state.start_position, pos);
        assert_eq!(state.anchor_position, pos);
        assert_eq!(state.selection_mode, SelectionMode::Character);
        assert_eq!(state.selection_start, 100);
        assert_eq!(state.selection_end, 100);
    }

    #[test]
    fn test_drag_selection_update() {
        let mut state = DragSelectionState::new();
        let start_pos = DocumentPosition::new(100, 5, 10);
        let end_pos = DocumentPosition::new(200, 10, 20);

        state.start_drag(start_pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.update_position(end_pos);

        assert_eq!(state.current_position, end_pos);
        assert_eq!(state.selection_start, 100);
        assert_eq!(state.selection_end, 200);
        assert!(state.has_selection());
    }

    #[test]
    fn test_drag_selection_reverse() {
        let mut state = DragSelectionState::new();
        let start_pos = DocumentPosition::new(200, 10, 20);
        let end_pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(start_pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.update_position(end_pos);

        // Should handle reverse selection correctly
        assert_eq!(state.selection_start, 100);
        assert_eq!(state.selection_end, 200);
    }

    #[test]
    fn test_drag_selection_column_mode() {
        let mut state = DragSelectionState::new();
        let start_pos = DocumentPosition::new(100, 5, 10);
        let end_pos = DocumentPosition::new(200, 10, 20);

        state.start_drag(start_pos, SelectionMode::Column, false, true, DragTarget::Selection);
        state.update_position(end_pos);

        assert!(state.is_column_selection);
        assert!(state.has_selection());
    }

    #[test]
    fn test_drag_selection_reset() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.update_position(DocumentPosition::new(200, 10, 20));

        state.reset();

        assert!(!state.is_dragging);
        assert_eq!(state.selection_start, 0);
        assert_eq!(state.selection_end, 0);
    }

    #[test]
    fn test_drag_selection_end() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.end_drag();

        assert!(!state.is_dragging);
        assert_eq!(state.drag_phase.phase, DragPhaseType::Ended);
    }

    #[test]
    fn test_drag_selection_cancel() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.cancel_drag();

        assert!(!state.is_dragging);
        assert_eq!(state.drag_phase.phase, DragPhaseType::Cancelled);
    }

    #[test]
    fn test_drag_selection_length() {
        let mut state = DragSelectionState::new();
        let start_pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(start_pos, SelectionMode::Character, false, false, DragTarget::Selection);
        assert_eq!(state.selection_length(), 0);

        state.update_position(DocumentPosition::new(150, 5, 10));
        assert_eq!(state.selection_length(), 50);
    }

    #[test]
    fn test_drag_selection_ranges() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.add_selection_range(10, 20);
        state.add_selection_range(30, 40);

        assert_eq!(state.selection_ranges.len(), 2);
        assert_eq!(state.selection_ranges[0], (10, 20));
        assert_eq!(state.selection_ranges[1], (30, 40));
    }

    #[test]
    fn test_document_position() {
        let pos = DocumentPosition::new(100, 5, 10);

        assert_eq!(pos.char_offset, 100);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
        assert!(!pos.is_start());
    }

    #[test]
    fn test_document_position_is_start() {
        let pos = DocumentPosition::new(0, 0, 0);
        assert!(pos.is_start());

        let pos = DocumentPosition::new(10, 0, 0);
        assert!(!pos.is_start());
    }

    #[test]
    fn test_selection_mode_default() {
        assert_eq!(SelectionMode::default(), SelectionMode::Character);
    }

    #[test]
    fn test_selection_mode_variants() {
        let modes = [
            SelectionMode::Character,
            SelectionMode::Word,
            SelectionMode::Line,
            SelectionMode::Column,
            SelectionMode::Block,
            SelectionMode::Document,
        ];

        assert_eq!(modes.len(), 6);
    }

    #[test]
    fn test_scroll_direction_variants() {
        let directions = [
            ScrollDirection::None,
            ScrollDirection::Up,
            ScrollDirection::Down,
            ScrollDirection::Left,
            ScrollDirection::Right,
            ScrollDirection::UpLeft,
            ScrollDirection::UpRight,
            ScrollDirection::DownLeft,
            ScrollDirection::DownRight,
        ];

        assert_eq!(directions.len(), 9);
    }

    #[test]
    fn test_drag_target_variants() {
        let targets = [
            DragTarget::Selection,
            DragTarget::Image,
            DragTarget::Table,
        ];

        assert_eq!(targets.len(), 3);
    }

    #[test]
    fn test_drag_handler() {
        let mut handler = DragSelectionHandler::new();
        let pos = DocumentPosition::new(100, 5, 10);

        handler.on_pointer_down(pos, SelectionMode::Character, false, DragTarget::Selection);
        assert!(handler.state().is_dragging);

        handler.on_pointer_up();
        assert!(!handler.state().is_dragging);
    }

    #[test]
    fn test_drag_handler_config() {
        let config = DragSelectionConfig::default();
        let handler = DragSelectionHandler::with_config(config);

        assert_eq!(handler.config().drag_threshold, 5.0);
        assert_eq!(handler.config().scroll_speed, 1.0);
    }

    #[test]
    fn test_drag_handler_get_selection() {
        let mut handler = DragSelectionHandler::new();
        let start_pos = DocumentPosition::new(100, 5, 10);
        let end_pos = DocumentPosition::new(200, 10, 20);

        handler.on_pointer_down(start_pos, SelectionMode::Character, false, DragTarget::Selection);
        handler.on_pointer_move(end_pos);

        let selection = handler.get_primary_selection();
        assert_eq!(selection, Some((100, 200)));
    }

    #[test]
    fn test_drag_phase_default() {
        let phase = DragPhase::default();
        assert_eq!(phase.phase, DragPhaseType::Idle);
        assert_eq!(phase.distance_x, 0.0);
        assert_eq!(phase.distance_y, 0.0);
    }

    #[test]
    fn test_drag_phase_dragging() {
        let mut phase = DragPhase::default();
        phase.phase = DragPhaseType::Dragging;
        phase.distance_x = 10.0;
        phase.distance_y = 20.0;

        assert_eq!(phase.phase, DragPhaseType::Dragging);
    }

    // Word boundary tests

    #[test]
    fn test_word_boundary_find_start() {
        let text = "Hello world, test!";

        assert_eq!(word_boundary::find_word_start(text, 0), 0);
        assert_eq!(word_boundary::find_word_start(text, 5), 0);
        assert_eq!(word_boundary::find_word_start(text, 6), 6);
        assert_eq!(word_boundary::find_word_start(text, 11), 11);
        assert_eq!(word_boundary::find_word_start(text, 13), 13);
    }

    #[test]
    fn test_word_boundary_find_end() {
        let text = "Hello world, test!";

        assert_eq!(word_boundary::find_word_end(text, 0), 5);
        assert_eq!(word_boundary::find_word_end(text, 5), 5);
        assert_eq!(word_boundary::find_word_end(text, 6), 11);
        assert_eq!(word_boundary::find_word_end(text, 12), 17);
    }

    #[test]
    fn test_word_boundary_get_word() {
        let text = "Hello world, test!";

        let word = word_boundary::get_word_at(text, 3);
        assert_eq!(word, Some((0, 5)));

        let word = word_boundary::get_word_at(text, 7);
        assert_eq!(word, Some((6, 11)));

        let word = word_boundary::get_word_at(text, 14);
        assert_eq!(word, Some((13, 17)));
    }

    #[test]
    fn test_word_boundary_empty_text() {
        let text = "";

        assert_eq!(word_boundary::find_word_start(text, 0), 0);
        assert_eq!(word_boundary::find_word_end(text, 0), 0);
        assert_eq!(word_boundary::get_word_at(text, 0), None);
    }

    #[test]
    fn test_word_boundary_whitespace_only() {
        let text = "   \t\n  ";

        assert_eq!(word_boundary::get_word_at(text, 0), None);
    }

    // Line boundary tests

    #[test]
    fn test_line_boundary_get_range() {
        let text = "Line 1\nLine 2\nLine 3";

        let range = line_boundary::get_line_range(text, 0);
        assert_eq!(range, Some((0, 6)));

        let range = line_boundary::get_line_range(text, 1);
        assert_eq!(range, Some((7, 13)));

        let range = line_boundary::get_line_range(text, 2);
        assert_eq!(range, Some((14, 19)));
    }

    #[test]
    fn test_line_boundary_get_line_number() {
        let text = "Line 1\nLine 2\nLine 3";

        assert_eq!(line_boundary::get_line_number(text, 0), 0);
        assert_eq!(line_boundary::get_line_number(text, 6), 0);
        assert_eq!(line_boundary::get_line_number(text, 7), 1);
        assert_eq!(line_boundary::get_line_number(text, 13), 1);
        assert_eq!(line_boundary::get_line_number(text, 14), 2);
    }

    #[test]
    fn test_line_boundary_single_line() {
        let text = "Single line";

        let range = line_boundary::get_line_range(text, 0);
        assert_eq!(range, Some((0, 11)));
    }

    #[test]
    fn test_line_boundary_empty_text() {
        let text = "";

        let range = line_boundary::get_line_range(text, 0);
        assert_eq!(range, None);

        assert_eq!(line_boundary::get_line_number(text, 0), 0);
    }

    #[test]
    fn test_line_boundary_get_line_start() {
        let text = "Line 1\nLine 2\nLine 3";

        assert_eq!(line_boundary::get_line_start(text, 0), Some(0));
        assert_eq!(line_boundary::get_line_start(text, 1), Some(7));
        assert_eq!(line_boundary::get_line_start(text, 2), Some(14));
    }

    #[test]
    fn test_line_boundary_get_line_end() {
        let text = "Line 1\nLine 2\nLine 3";

        assert_eq!(line_boundary::get_line_end(text, 0), Some(6));
        assert_eq!(line_boundary::get_line_end(text, 1), Some(13));
        assert_eq!(line_boundary::get_line_end(text, 2), Some(19));
    }

    #[test]
    fn test_drag_handler_expand_to_word() {
        let handler = DragSelectionHandler::new();
        let text = "Hello world";
        let offset = 7; // At 'w' in "world"

        let range = handler.expand_to_word(text, offset);
        assert_eq!(range, (6, 11));
    }

    #[test]
    fn test_drag_handler_expand_to_line() {
        let handler = DragSelectionHandler::new();
        let text = "Line 1\nLine 2\nLine 3";

        let range = handler.expand_to_line(text, 1);
        assert_eq!(range, (7, 13));
    }

    #[test]
    fn test_drag_handler_needs_auto_scroll() {
        let mut handler = DragSelectionHandler::new();
        assert!(!handler.needs_auto_scroll());

        // Set scroll direction
        handler.state_mut().scroll_direction = ScrollDirection::Down;
        assert!(handler.needs_auto_scroll());
    }

    #[test]
    fn test_drag_handler_get_scroll_delta() {
        let mut handler = DragSelectionHandler::new();

        handler.state_mut().scroll_direction = ScrollDirection::Up;
        assert_eq!(handler.get_scroll_delta(), (0, -1));

        handler.state_mut().scroll_direction = ScrollDirection::Down;
        assert_eq!(handler.get_scroll_delta(), (0, 1));

        handler.state_mut().scroll_direction = ScrollDirection::Left;
        assert_eq!(handler.get_scroll_delta(), (-1, 0));

        handler.state_mut().scroll_direction = ScrollDirection::Right;
        assert_eq!(handler.get_scroll_delta(), (1, 0));
    }

    #[test]
    fn test_drag_selection_extend_mode() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, true, false, DragTarget::Selection);

        assert!(state.extend_selection);
        assert!(state.drag_phase.is_extend);
    }

    #[test]
    fn test_drag_selection_clear_ranges() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.add_selection_range(10, 20);
        state.add_selection_range(30, 40);

        assert_eq!(state.selection_ranges.len(), 2);

        state.clear_selection_ranges();
        assert!(state.selection_ranges.is_empty());
    }

    #[test]
    fn test_drag_selection_target() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Image);

        assert_eq!(state.drag_target, DragTarget::Image);
    }

    #[test]
    fn test_drag_selection_visual_feedback() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);

        assert!(state.show_visual_feedback);
    }

    #[test]
    fn test_drag_selection_determine_mode() {
        let state = DragSelectionState::new();

        // Normal click
        let mode = state.determine_selection_mode(false, false, 1);
        assert_eq!(mode, SelectionMode::Character);

        // Double click
        let mode = state.determine_selection_mode(false, false, 2);
        assert_eq!(mode, SelectionMode::Word);

        // Triple click
        let mode = state.determine_selection_mode(false, false, 3);
        assert_eq!(mode, SelectionMode::Line);

        // Alt + click (column)
        let mode = state.determine_selection_mode(false, true, 1);
        assert_eq!(mode, SelectionMode::Column);

        // Margin click (line)
        let mode = state.determine_selection_mode(true, false, 1);
        assert_eq!(mode, SelectionMode::Line);
    }

    #[test]
    fn test_drag_selection_is_active() {
        let mut state = DragSelectionState::new();

        assert!(!state.is_active());

        let pos = DocumentPosition::new(100, 5, 10);
        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);

        assert!(state.is_just_started());

        state.update_position(DocumentPosition::new(200, 10, 20));

        assert!(state.is_active());
    }

    #[test]
    fn test_drag_selection_velocity() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);
        state.update_drag_distance(0.0, 0.0, 100.0, 200.0);

        let (vx, vy) = state.drag_velocity();

        // Velocity should be non-zero after movement
        assert!(vx > 0.0 || vy > 0.0);
    }

    #[test]
    fn test_drag_selection_velocity_no_movement() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);

        let (vx, vy) = state.drag_velocity();

        // Zero velocity when no movement
        assert_eq!(vx, 0.0);
        assert_eq!(vy, 0.0);
    }

    #[test]
    fn test_drag_selection_all_selection_modes() {
        let modes = [
            SelectionMode::Character,
            SelectionMode::Word,
            SelectionMode::Line,
            SelectionMode::Column,
            SelectionMode::Block,
            SelectionMode::Document,
        ];

        for mode in modes {
            let mut state = DragSelectionState::new();
            let pos = DocumentPosition::new(100, 5, 10);

            state.start_drag(pos, mode, false, false, DragTarget::Selection);

            assert_eq!(state.selection_mode, mode);
        }
    }

    #[test]
    fn test_drag_selection_toggle_column() {
        let mut state = DragSelectionState::new();
        let pos = DocumentPosition::new(100, 5, 10);

        state.start_drag(pos, SelectionMode::Character, false, false, DragTarget::Selection);

        assert!(!state.is_column_selection);

        state.toggle_column_mode();
        assert!(state.is_column_selection);

        state.toggle_column_mode();
        assert!(!state.is_column_selection);
    }

    #[test]
    fn test_drag_handler_on_alt_key() {
        let mut handler = DragSelectionHandler::new();
        let pos = DocumentPosition::new(100, 5, 10);

        handler.on_pointer_down(pos, SelectionMode::Character, false, DragTarget::Selection);

        assert!(!handler.state().is_column_selection);

        handler.on_alt_key_pressed();
        assert!(handler.state().is_column_selection);
    }

    #[test]
    fn test_drag_handler_on_mode_change() {
        let mut handler = DragSelectionHandler::new();
        let pos = DocumentPosition::new(100, 5, 10);

        handler.on_pointer_down(pos, SelectionMode::Character, false, DragTarget::Selection);
        assert_eq!(handler.state().selection_mode, SelectionMode::Character);

        handler.on_mode_change(SelectionMode::Word);
        assert_eq!(handler.state().selection_mode, SelectionMode::Word);
    }

    #[test]
    fn test_drag_handler_on_pointer_cancel() {
        let mut handler = DragSelectionHandler::new();
        let pos = DocumentPosition::new(100, 5, 10);

        handler.on_pointer_down(pos, SelectionMode::Character, false, DragTarget::Selection);
        handler.on_pointer_cancel();

        assert!(!handler.state().is_dragging);
        assert_eq!(handler.state().phase(), DragPhaseType::Cancelled);
    }

    #[test]
    fn test_drag_handler_get_selection_ranges() {
        let mut handler = DragSelectionHandler::new();
        let start_pos = DocumentPosition::new(100, 5, 10);
        let end_pos = DocumentPosition::new(200, 10, 20);

        handler.on_pointer_down(start_pos, SelectionMode::Character, false, DragTarget::Selection);
        handler.on_pointer_move(end_pos);

        let ranges = handler.get_selection_ranges();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], (100, 200));
    }

    #[test]
    fn test_drag_selection_config_default() {
        let config = DragSelectionConfig::default();

        assert_eq!(config.drag_threshold, 5.0);
        assert_eq!(config.scroll_speed, 1.0);
        assert_eq!(config.auto_scroll_delay_ms, 100);
        assert!(config.alt_column_selection);
        assert!(config.margin_line_selection);
        assert!(config.triple_click_line_selection);
        assert_eq!(config.feedback_style, VisualFeedbackStyle::Highlight);
    }

    #[test]
    fn test_visual_feedback_style_variants() {
        let styles = [
            VisualFeedbackStyle::Highlight,
            VisualFeedbackStyle::Underline,
            VisualFeedbackStyle::CursorLine,
            VisualFeedbackStyle::Brackets,
            VisualFeedbackStyle::None,
        ];

        assert_eq!(styles.len(), 5);
    }

    #[test]
    fn test_drag_selection_set_extend() {
        let mut state = DragSelectionState::new();

        state.set_extend(true);
        assert!(state.extend_selection);

        state.set_extend(false);
        assert!(!state.extend_selection);
    }

    #[test]
    fn test_drag_selection_has_selection_empty() {
        let state = DragSelectionState::default();

        assert!(!state.has_selection());
    }

    #[test]
    fn test_drag_selection_has_selection_with_ranges() {
        let mut state = DragSelectionState::new();
        state.add_selection_range(10, 20);

        assert!(state.has_selection());
    }
}

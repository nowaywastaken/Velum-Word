// Selection Rendering Module
// Provides selection and caret rendering functionality for Velum

/// Selection rendering configuration
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionRenderConfig {
    /// Highlight color for normal selection
    pub highlight_color: String,
    /// Highlight color for column selection
    pub column_color: String,
    /// Text color when selected
    pub text_color: Option<String>,
    /// Corner radius for selection rectangles
    pub corner_radius: f32,
    /// Opacity of selection highlight (0.0-1.0)
    pub opacity: f32,
    /// Whether to draw selection border
    pub show_border: bool,
    /// Border color
    pub border_color: String,
    /// Border width
    pub border_width: f32,
    /// Whether to use rounded corners
    pub rounded_corners: bool,
    /// Whether to invert colors (dark mode)
    pub invert_colors: bool,
}

impl Default for SelectionRenderConfig {
    fn default() -> Self {
        SelectionRenderConfig {
            highlight_color: "#3399FF".to_string(),
            column_color: "#9966FF".to_string(),
            text_color: Some("#FFFFFF".to_string()),
            corner_radius: 2.0,
            opacity: 0.3,
            show_border: false,
            border_color: "#FFFFFF".to_string(),
            border_width: 1.0,
            rounded_corners: true,
            invert_colors: false,
        }
    }
}

/// Represents a selection rectangle for rendering
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionRect {
    /// X position in document coordinates
    pub x: f32,
    /// Y position in document coordinates
    pub y: f32,
    /// Width of the rectangle
    pub width: f32,
    /// Height of the rectangle
    pub height: f32,
    /// Line index this rectangle belongs to
    pub line_index: usize,
    /// Character start position on the line
    pub char_start: usize,
    /// Character end position on the line
    pub char_end: usize,
    /// Whether this is part of a column selection
    pub is_column: bool,
}

impl SelectionRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32, line_index: usize, char_start: usize, char_end: usize) -> Self {
        SelectionRect {
            x,
            y,
            width,
            height,
            line_index,
            char_start,
            char_end,
            is_column: false,
        }
    }
}

/// Selection render data containing all rectangles to draw
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionRenderData {
    pub rectangles: Vec<SelectionRect>,
    pub bounding_box: Option<SelectionBoundingBox>,
    pub range_count: usize,
    pub has_selection: bool,
    pub color: String,
}

/// Bounding box for all selections
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionBoundingBox {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub width: f32,
    pub height: f32,
}

impl SelectionBoundingBox {
    pub fn from_rectangles(rectangles: &[SelectionRect]) -> Option<Self> {
        if rectangles.is_empty() {
            return None;
        }
        let mut min_x = rectangles[0].x;
        let mut min_y = rectangles[0].y;
        let mut max_x = rectangles[0].x + rectangles[0].width;
        let mut max_y = rectangles[0].y + rectangles[0].height;
        for rect in rectangles.iter().skip(1) {
            min_x = min_x.min(rect.x);
            min_y = min_y.min(rect.y);
            max_x = max_x.max(rect.x + rect.width);
            max_y = max_y.max(rect.y + rect.height);
        }
        Some(SelectionBoundingBox {
            min_x,
            min_y,
            max_x,
            max_y,
            width: max_x - min_x,
            height: max_y - min_y,
        })
    }
}

/// Selection renderer for converting selection state to render data
#[derive(Debug, Clone)]
pub struct SelectionRenderer {
    config: SelectionRenderConfig,
    char_widths: Vec<f32>,
    avg_char_width: f32,
}

impl SelectionRenderer {
    pub fn new() -> Self {
        SelectionRenderer {
            config: SelectionRenderConfig::default(),
            char_widths: Vec::new(),
            avg_char_width: 10.0,
        }
    }

    pub fn with_config(config: SelectionRenderConfig) -> Self {
        SelectionRenderer {
            config,
            char_widths: Vec::new(),
            avg_char_width: 10.0,
        }
    }

    pub fn set_character_widths(&mut self, widths: Vec<f32>) {
        self.char_widths = widths.clone();
        if !widths.is_empty() {
            self.avg_char_width = widths.iter().sum::<f32>() / widths.len() as f32;
        }
    }

    pub fn set_avg_char_width(&mut self, width: f32) {
        self.avg_char_width = width.max(1.0);
    }

    pub fn get_effective_color(&self, is_dark_mode: bool) -> String {
        if is_dark_mode && !self.config.invert_colors {
            if self.config.highlight_color.starts_with("#") {
                let hex = &self.config.highlight_color[1..];
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    let new_r = (r as u16 + 100).min(255) as u8;
                    let new_g = (g as u16 + 100).min(255) as u8;
                    let new_b = (b as u16 + 100).min(255) as u8;
                    return format!("#{:02X}{:02X}{:02X}", new_r, new_g, new_b);
                }
            }
        }
        self.config.highlight_color.clone()
    }
}

/// Caret (cursor) rendering information
#[derive(Debug, Clone, PartialEq)]
pub struct CaretRenderInfo {
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub is_overstrike: bool,
    pub color: String,
    pub is_visible: bool,
    pub is_focused: bool,
}

impl Default for CaretRenderInfo {
    fn default() -> Self {
        CaretRenderInfo {
            x: 0.0,
            y: 0.0,
            height: 20.0,
            is_overstrike: false,
            color: "#000000".to_string(),
            is_visible: true,
            is_focused: false,
        }
    }
}

/// Selection highlight style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionHighlightStyle {
    Standard,
    Themed,
    Custom,
    CaretOnly,
}

impl Default for SelectionHighlightStyle {
    fn default() -> Self {
        SelectionHighlightStyle::Standard
    }
}

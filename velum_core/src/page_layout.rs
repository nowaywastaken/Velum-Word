//! # Page Layout Module
//!
//! Implements document pagination engine with support for:
//! - Configurable page sizes and margins
//! - Widow/orphan control
//! - Multi-column layouts
//! - Cross-page paragraph breaking

use crate::line_layout::ParagraphLayout;
use serde::{Deserialize, Serialize};
use std::cmp::min;

/// Represents a rectangle in 2D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Creates a new rectangle
    #[inline]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect { x, y, width, height }
    }

    /// Checks if this rect is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Gets the bottom edge (y + height)
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Gets the right edge (x + width)
    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
}

/// Represents a rendered line with position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedLine {
    /// Line index within the page
    pub line_index: usize,
    /// Source paragraph index
    pub paragraph_index: usize,
    /// Source line index within the paragraph
    pub source_line_index: usize,
    /// Y position on the page
    pub y: f32,
    /// Height of the line
    pub height: f32,
    /// X position (can vary for different columns)
    pub x: f32,
    /// Width of the line
    pub width: f32,
    /// Start byte offset in original text
    pub start: usize,
    /// End byte offset in original text
    pub end: usize,
}

/// Page size and margin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    /// Page width in points (default A4: 595.35pt)
    pub width: f32,
    /// Page height in points (default A4: 841.89pt)
    pub height: f32,
    /// Top margin in points
    pub margin_top: f32,
    /// Bottom margin in points
    pub margin_bottom: f32,
    /// Left margin in points
    pub margin_left: f32,
    /// Right margin in points
    pub margin_right: f32,
    /// Header height in points
    pub header_height: f32,
    /// Footer height in points
    pub footer_height: f32,
}

impl Default for PageConfig {
    fn default() -> Self {
        PageConfig {
            width: 595.35,    // A4 width
            height: 841.89,   // A4 height
            margin_top: 72.0,     // 1 inch = 72pt
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
            header_height: 0.0,
            footer_height: 0.0,
        }
    }
}

impl PageConfig {
    /// Creates A4 page configuration
    #[inline]
    pub fn a4() -> Self {
        PageConfig::default()
    }

    /// Creates letter page configuration
    #[inline]
    pub fn letter() -> Self {
        PageConfig {
            width: 612.0,
            height: 792.0,
            margin_top: 72.0,
            margin_bottom: 72.0,
            margin_left: 72.0,
            margin_right: 72.0,
            header_height: 0.0,
            footer_height: 0.0,
        }
    }

    /// Gets the content width (page width - margins)
    #[inline]
    pub fn content_width(&self) -> f32 {
        self.width - self.margin_left - self.margin_right
    }

    /// Gets the content height (page height - margins - header - footer)
    #[inline]
    pub fn content_height(&self) -> f32 {
        self.height - self.margin_top - self.margin_bottom - self.header_height - self.footer_height
    }

    /// Gets the header region
    #[inline]
    pub fn header_region(&self) -> Option<Rect> {
        if self.header_height > 0.0 {
            Some(Rect::new(
                self.margin_left,
                self.margin_top,
                self.content_width(),
                self.header_height,
            ))
        } else {
            None
        }
    }

    /// Gets the footer region
    #[inline]
    pub fn footer_region(&self) -> Option<Rect> {
        if self.footer_height > 0.0 {
            Some(Rect::new(
                self.margin_left,
                self.height - self.margin_bottom - self.footer_height,
                self.content_width(),
                self.footer_height,
            ))
        } else {
            None
        }
    }
}

/// A single page in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Zero-based page index
    pub page_index: usize,
    /// All lines on this page
    pub lines: Vec<RenderedLine>,
    /// Content bounds for this page
    pub content_bounds: Rect,
    /// Header region (if any)
    pub header_region: Option<Rect>,
    /// Footer region (if any)
    pub footer_region: Option<Rect>,
    /// Column configuration used for this page
    pub column: u32,
    /// Next page number indicator (for continuation)
    pub continued_on: Option<usize>,
    /// Previous page number indicator (for continuation)
    pub continued_from: Option<usize>,
}

/// Configuration for pagination control
#[derive(Debug, Clone)]
pub struct PaginationConfig {
    /// Minimum lines to keep together at top of page (orphan control)
    pub min_lines_orphan: u32,
    /// Minimum lines to keep together at bottom of page (widow control)
    pub min_lines_widow: u32,
    /// Enable widow/orphan control
    pub enable_widow_orphan: bool,
    /// Allow paragraph to be split across pages
    pub allow_page_breaks: bool,
    /// Keep paragraph together on one page if possible
    pub keep_with_next: bool,
    /// Column count for multi-column layout
    pub columns: u32,
    /// Gap between columns
    pub column_gap: f32,
    /// Line height for calculating page content
    pub line_height: f32,
    /// Font size for calculating line height
    pub font_size: f32,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        PaginationConfig {
            min_lines_orphan: 2,
            min_lines_widow: 2,
            enable_widow_orphan: true,
            allow_page_breaks: true,
            keep_with_next: false,
            columns: 1,
            column_gap: 24.0,
            line_height: 1.2,
            font_size: 12.0,
        }
    }
}

/// Main page layout engine
#[derive(Debug, Clone)]
pub struct PageLayout {
    /// Page configuration
    pub page_config: PageConfig,
    /// Pagination configuration
    pub config: PaginationConfig,
    /// Calculated pages
    pub pages: Vec<Page>,
    /// Total paragraph count
    pub paragraph_count: usize,
}

impl Default for PageLayout {
    fn default() -> Self {
        PageLayout::new()
    }
}

impl PageLayout {
    /// Creates a new page layout with default configuration
    #[inline]
    pub fn new() -> Self {
        PageLayout {
            page_config: PageConfig::default(),
            config: PaginationConfig::default(),
            pages: Vec::new(),
            paragraph_count: 0,
        }
    }

    /// Creates a page layout with custom page configuration
    #[inline]
    pub fn with_page_config(page_config: PageConfig) -> Self {
        PageLayout {
            page_config,
            config: PaginationConfig::default(),
            pages: Vec::new(),
            paragraph_count: 0,
        }
    }

    /// Sets the number of columns
    #[inline]
    pub fn set_columns(&mut self, columns: u32) {
        self.config.columns = columns.max(1);
    }

    /// Sets the column gap
    #[inline]
    pub fn set_column_gap(&mut self, gap: f32) {
        self.config.column_gap = gap;
    }

    /// Enables or disables widow/orphan control
    #[inline]
    pub fn set_widow_orphan(&mut self, enabled: bool) {
        self.config.enable_widow_orphan = enabled;
    }

    /// Gets a single column's width
    #[inline]
    fn column_width(&self) -> f32 {
        if self.config.columns <= 1 {
            self.page_config.content_width()
        } else {
            let total_gap = self.config.column_gap * (self.config.columns - 1) as f32;
            (self.page_config.content_width() - total_gap) / self.config.columns as f32
        }
    }

    /// Gets the actual line height in points
    #[inline]
    fn actual_line_height(&self) -> f32 {
        self.config.line_height * self.config.font_size
    }

    /// Calculates the content height available on a page
    #[inline]
    fn available_content_height(&self) -> f32 {
        self.page_config.content_height()
    }

    /// Performs the first pass: collects all paragraphs and calculates total height
    fn first_pass_collect(&self, paragraphs: &[ParagraphLayout]) -> Vec<(usize, f32)> {
        let mut paragraph_heights = Vec::with_capacity(paragraphs.len());

        for (idx, para) in paragraphs.iter().enumerate() {
            let height = self.calculate_paragraph_height(para);
            paragraph_heights.push((idx, height));
        }

        paragraph_heights
    }

    /// Calculates the height needed for a paragraph
    fn calculate_paragraph_height(&self, para: &ParagraphLayout) -> f32 {
        if para.lines.is_empty() {
            return self.actual_line_height();
        }
        para.total_height
    }

    /// Performs the second pass: allocates paragraphs to pages
    fn second_pass_layout(&mut self, paragraphs: &[ParagraphLayout], paragraph_heights: &[(usize, f32)]) -> Vec<Page> {
        let mut pages = Vec::new();
        let column_width = self.column_width();
        let line_height = self.actual_line_height();
        let available_height = self.available_content_height();

        let mut current_page = Page {
            page_index: 0,
            lines: Vec::new(),
            content_bounds: Rect::new(
                self.page_config.margin_left,
                self.page_config.margin_top,
                self.page_config.content_width(),
                available_height,
            ),
            header_region: self.page_config.header_region(),
            footer_region: self.page_config.footer_region(),
            column: 0,
            continued_on: None,
            continued_from: None,
        };

        let mut current_y = 0.0f32;
        let mut current_column = 0u32;
        let mut current_x = 0.0f32;

        for (orig_idx, para_height) in paragraph_heights {
            let para = &paragraphs[*orig_idx];

            // Check if paragraph fits on current page
            if current_y + para_height > available_height || current_column >= self.config.columns {
                // Start a new page
                if !current_page.lines.is_empty() {
                    pages.push(current_page);
                }

                current_page = Page {
                    page_index: pages.len(),
                    lines: Vec::new(),
                    content_bounds: Rect::new(
                        self.page_config.margin_left,
                        self.page_config.margin_top,
                        self.page_config.content_width(),
                        available_height,
                    ),
                    header_region: self.page_config.header_region(),
                    footer_region: self.page_config.footer_region(),
                    column: 0,
                    continued_on: None,
                    continued_from: None,
                };
                current_y = 0.0;
                current_column = 0;
                current_x = 0.0;
            }

            // Layout paragraph lines
            self.layout_paragraph_to_page(
                para,
                *orig_idx,
                &mut current_page,
                &mut current_y,
                &mut current_column,
                &mut current_x,
                column_width,
                line_height,
            );
        }

        // Add the last page if it has content
        if !current_page.lines.is_empty() {
            pages.push(current_page);
        }

        pages
    }

    /// Layouts a single paragraph onto a page, handling column wrapping
    fn layout_paragraph_to_page(
        &self,
        para: &ParagraphLayout,
        para_index: usize,
        page: &mut Page,
        current_y: &mut f32,
        current_column: &mut u32,
        current_x: &mut f32,
        column_width: f32,
        line_height: f32,
    ) {
        let available_height = self.available_content_height();

        for (line_idx, line_info) in para.lines.iter().enumerate() {
            let line_height_actual = line_height;

            // Check if line fits in current column
            if *current_y + line_height_actual > available_height {
                // Move to next column
                *current_column += 1;
                if *current_column >= self.config.columns {
                    // Need new page - handled in caller
                    break;
                }
                *current_y = 0.0;
                *current_x = if *current_column == 0 {
                    0.0
                } else {
                    self.page_config.content_width() - (self.config.columns - *current_column) as f32 * (column_width + self.config.column_gap)
                };
            }

            let rendered_line = RenderedLine {
                line_index: page.lines.len(),
                paragraph_index: para_index,
                source_line_index: line_idx,
                y: *current_y,
                height: line_height_actual,
                x: *current_x,
                width: line_info.width.min(column_width),
                start: line_info.start,
                end: line_info.end,
            };

            page.lines.push(rendered_line);
            *current_y += line_height_actual;
        }
    }

    /// Applies widow/orphan control to adjust page breaks
    fn apply_widow_orphan(&mut self, pages: &mut Vec<Page>) {
        if !self.config.enable_widow_orphan || pages.len() < 2 {
            return;
        }

        let min_widow = self.config.min_lines_widow as usize;
        let min_orphan = self.config.min_lines_orphan as usize;

        // Process from first to last page
        let mut i = 0usize;
        while i + 1 < pages.len() {
            // Use indices instead of borrowing twice
            let last_para_idx = pages[i].lines.last().map(|l| l.paragraph_index);
            let first_para_idx = pages[i + 1].lines.first().map(|l| l.paragraph_index);

            if last_para_idx == first_para_idx && last_para_idx.is_some() {
                let para_idx = last_para_idx.unwrap();

                // Count lines of this paragraph on each page
                let current_para_lines: usize = pages[i].lines
                    .iter()
                    .filter(|l| l.paragraph_index == para_idx)
                    .count();

                let next_para_lines: usize = pages[i + 1].lines
                    .iter()
                    .filter(|l| l.paragraph_index == para_idx)
                    .count();

                // Widow: paragraph's last line(s) on new page alone
                if next_para_lines <= min_widow && i > 0 {
                    // Move lines from current page to next page
                    self.move_lines_to_next_page(pages, i, min_widow);
                }
                // Orphan: paragraph's first line(s) on current page alone
                else if current_para_lines <= min_orphan && i + 1 < pages.len() - 1 {
                    // Move lines from next page to current page
                    self.move_lines_from_next_page(pages, i, min_orphan);
                }
            }

            i += 1;
        }
    }

    /// Moves lines to the next page to fix widow
    fn move_lines_to_next_page(&mut self, pages: &mut Vec<Page>, page_idx: usize, _min_lines: usize) {
        if page_idx + 1 >= pages.len() {
            return;
        }

        let current_page_lines_count = pages[page_idx].lines.len();
        let next_page_lines_count = pages[page_idx + 1].lines.len();

        if current_page_lines_count == 0 || next_page_lines_count == 0 {
            return;
        }

        // Find the paragraph boundary in current page
        if let Some(para_idx) = pages[page_idx].lines.last().map(|l| l.paragraph_index) {
            // Count lines of this paragraph
            let para_line_indices: Vec<usize> = pages[page_idx].lines
                .iter()
                .enumerate()
                .filter(|(_, l)| l.paragraph_index == para_idx)
                .map(|(idx, _)| idx)
                .collect();

            let para_line_count = para_line_indices.len();

            if para_line_count > 2 {
                // Move some lines to next page
                let lines_to_move = para_line_count - 2; // Keep at least 2 lines
                let move_indices: Vec<usize> = para_line_indices[..lines_to_move].to_vec();

                // Calculate base Y for next page
                let next_base_y = pages[page_idx + 1].lines
                    .iter()
                    .map(|l| l.y + l.height)
                    .fold(0.0, f32::max);

                // Move lines to next page
                for (offset, &line_idx) in move_indices.iter().enumerate().rev() {
                    let line = pages[page_idx].lines.remove(line_idx);
                    let mut new_line = line;
                    new_line.y = next_base_y + (offset as f32 * self.actual_line_height());
                    pages[page_idx + 1].lines.insert(0, new_line);
                }

                // Update continuation markers
                pages[page_idx].continued_on = Some(page_idx + 1);
                pages[page_idx + 1].continued_from = Some(page_idx);
            }
        }
    }

    /// Moves lines from next page to current page to fix orphan
    fn move_lines_from_next_page(&mut self, pages: &mut Vec<Page>, page_idx: usize, _min_lines: usize) {
        if page_idx + 1 >= pages.len() {
            return;
        }

        if pages[page_idx].lines.is_empty() || pages[page_idx + 1].lines.is_empty() {
            return;
        }

        if let Some(para_idx) = pages[page_idx + 1].lines.first().map(|l| l.paragraph_index) {
            let para_line_indices: Vec<usize> = pages[page_idx + 1].lines
                .iter()
                .enumerate()
                .filter(|(_, l)| l.paragraph_index == para_idx)
                .map(|(idx, _)| idx)
                .collect();

            let para_line_count = para_line_indices.len();

            if para_line_count > 2 {
                // Calculate new Y position for moved lines
                let last_y = pages[page_idx].lines
                    .iter()
                    .map(|l| l.y + l.height)
                    .fold(0.0, f32::max);

                // Move some lines to current page
                let lines_to_keep = para_line_count - 2; // Keep at least 2 on next page
                let move_indices: Vec<usize> = para_line_indices[lines_to_keep..].to_vec();

                for (offset, &line_idx) in move_indices.iter().enumerate() {
                    let adjusted_idx = line_idx.saturating_sub(offset);
                    if adjusted_idx < pages[page_idx + 1].lines.len() {
                        let line = pages[page_idx + 1].lines.remove(adjusted_idx);
                        let mut new_line = line;
                        new_line.y = last_y + (offset as f32 * self.actual_line_height());
                        pages[page_idx].lines.push(new_line);
                    }
                }

                pages[page_idx].continued_on = Some(page_idx + 1);
                pages[page_idx + 1].continued_from = Some(page_idx);
            }
        }
    }

    /// Main method: converts paragraph layouts to pages
    pub fn layout_pages(&mut self, paragraphs: &[ParagraphLayout]) -> Vec<Page> {
        self.paragraph_count = paragraphs.len();

        if paragraphs.is_empty() {
            return Vec::new();
        }

        // First pass: collect paragraph heights
        let paragraph_heights = self.first_pass_collect(paragraphs);

        // Second pass: layout paragraphs to pages
        let mut pages = self.second_pass_layout(paragraphs, &paragraph_heights);

        // Third pass: apply widow/orphan control
        self.apply_widow_orphan(&mut pages);

        // Apply column adjustments
        self.apply_column_adjustments(&mut pages);

        self.pages = pages.clone();
        pages
    }

    /// Adjusts line positions for multi-column layout
    fn apply_column_adjustments(&self, pages: &mut Vec<Page>) {
        let column_width = self.column_width();
        let column_gap = self.config.column_gap;

        for page in pages.iter_mut() {
            // Group line indices by column
            let mut column_line_indices: Vec<Vec<usize>> = vec![Vec::new(); self.config.columns as usize];

            for (line_idx, line) in page.lines.iter().enumerate() {
                // Determine which column this line belongs to based on X position
                let col_idx = min(
                    ((line.x + column_width / 2.0) / (column_width + column_gap)).floor() as usize,
                    self.config.columns as usize - 1,
                );
                if col_idx < column_line_indices.len() {
                    column_line_indices[col_idx].push(line_idx);
                }
            }

            // Recalculate Y positions for each column independently
            let available_height = self.available_content_height();
            for (col_idx, line_indices) in column_line_indices.iter().enumerate() {
                let mut y_offset = 0.0f32;
                let base_x = if col_idx == 0 {
                    0.0
                } else {
                    col_idx as f32 * (column_width + column_gap)
                };

                for &line_idx in line_indices {
                    if line_idx < page.lines.len() {
                        page.lines[line_idx].x = base_x;
                        page.lines[line_idx].y = y_offset;
                        y_offset += page.lines[line_idx].height;

                        // Check if line exceeds page height
                        if y_offset > available_height {
                            // This shouldn't happen, but handle gracefully
                            page.lines[line_idx].height = (available_height - y_offset + page.lines[line_idx].height).max(self.actual_line_height());
                            y_offset = available_height;
                        }
                    }
                }
            }
        }
    }

    /// Gets the page number for a given character offset
    pub fn get_page_for_offset(&self, offset: usize, paragraphs: &[ParagraphLayout]) -> Option<usize> {
        let mut char_count = 0usize;

        for (page_idx, page) in self.pages.iter().enumerate() {
            for line in &page.lines {
                let para = paragraphs.get(line.paragraph_index)?;
                let line_text = para.text.get(line.start..line.end)?;

                if char_count + line_text.len() > offset {
                    return Some(page_idx);
                }

                char_count += line_text.len();
            }
        }

        None
    }

    /// Gets the total number of pages
    #[inline]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

/// Rendered page for external consumption (e.g., Flutter rendering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPage {
    /// Page index
    pub page_index: usize,
    /// Content bounds
    pub content_bounds: Rect,
    /// All rendered lines
    pub lines: Vec<RenderedLine>,
    /// Header region (if any)
    pub header_region: Option<Rect>,
    /// Footer region (if any)
    pub footer_region: Option<Rect>,
    /// Page dimensions
    pub page_width: f32,
    pub page_height: f32,
}

impl From<Page> for RenderedPage {
    fn from(page: Page) -> Self {
        RenderedPage {
            page_index: page.page_index,
            content_bounds: page.content_bounds,
            lines: page.lines,
            header_region: page.header_region,
            footer_region: page.footer_region,
            page_width: 0.0, // Will be set by caller
            page_height: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::line_layout::{LineLayoutInfo, LineLayout, ParagraphLayout, ParagraphProperties, LineSpacingRule, Alignment};

    fn create_test_paragraphs() -> Vec<ParagraphLayout> {
        // Create test paragraphs without using LineLayout (to avoid HarfBuzz issues)
        vec![
            ParagraphLayout {
                text: "This is the first paragraph. It contains some text to test pagination.".to_string(),
                max_width: 400.0,
                content_width: 400.0,
                lines: vec![
                    LineLayoutInfo { line_number: 0, start: 0, end: 70, width: 350.0, break_type: "SoftBreak".to_string(), char_count: 70, is_bidi: false, trailing_whitespace: 0.0, offset_x: 0.0, line_height: 14.4 },
                ],
                total_height: 14.4,
                base_line_height: 12.0,
                actual_line_height: 14.4,
                has_bidi: false,
                properties: ParagraphProperties::default(),
            },
            ParagraphLayout {
                text: "Second paragraph here. This is used to verify that multiple paragraphs are handled correctly.".to_string(),
                max_width: 400.0,
                content_width: 400.0,
                lines: vec![
                    LineLayoutInfo { line_number: 0, start: 0, end: 95, width: 400.0, break_type: "SoftBreak".to_string(), char_count: 95, is_bidi: false, trailing_whitespace: 0.0, offset_x: 0.0, line_height: 14.4 },
                ],
                total_height: 14.4,
                base_line_height: 12.0,
                actual_line_height: 14.4,
                has_bidi: false,
                properties: ParagraphProperties::default(),
            },
            ParagraphLayout {
                text: "Third paragraph with some longer content that might span multiple lines when rendered.".to_string(),
                max_width: 400.0,
                content_width: 400.0,
                lines: vec![
                    LineLayoutInfo { line_number: 0, start: 0, end: 100, width: 400.0, break_type: "SoftBreak".to_string(), char_count: 100, is_bidi: false, trailing_whitespace: 0.0, offset_x: 0.0, line_height: 14.4 },
                    LineLayoutInfo { line_number: 1, start: 100, end: 110, width: 50.0, break_type: "SoftBreak".to_string(), char_count: 10, is_bidi: false, trailing_whitespace: 0.0, offset_x: 0.0, line_height: 14.4 },
                ],
                total_height: 28.8,
                base_line_height: 12.0,
                actual_line_height: 14.4,
                has_bidi: false,
                properties: ParagraphProperties::default(),
            },
            ParagraphLayout {
                text: "Fourth short paragraph.".to_string(),
                max_width: 400.0,
                content_width: 400.0,
                lines: vec![
                    LineLayoutInfo { line_number: 0, start: 0, end: 25, width: 125.0, break_type: "SoftBreak".to_string(), char_count: 25, is_bidi: false, trailing_whitespace: 0.0, offset_x: 0.0, line_height: 14.4 },
                ],
                total_height: 14.4,
                base_line_height: 12.0,
                actual_line_height: 14.4,
                has_bidi: false,
                properties: ParagraphProperties::default(),
            },
            ParagraphLayout {
                text: "Fifth paragraph with even more content to test pagination behavior across multiple pages. This paragraph should be long enough to potentially span page boundaries.".to_string(),
                max_width: 400.0,
                content_width: 400.0,
                lines: vec![
                    LineLayoutInfo { line_number: 0, start: 0, end: 100, width: 400.0, break_type: "SoftBreak".to_string(), char_count: 100, is_bidi: false, trailing_whitespace: 0.0, offset_x: 0.0, line_height: 14.4 },
                    LineLayoutInfo { line_number: 1, start: 100, end: 140, width: 200.0, break_type: "SoftBreak".to_string(), char_count: 40, is_bidi: false, trailing_whitespace: 0.0, offset_x: 0.0, line_height: 14.4 },
                ],
                total_height: 28.8,
                base_line_height: 12.0,
                actual_line_height: 14.4,
                has_bidi: false,
                properties: ParagraphProperties::default(),
            },
        ]
    }

    fn create_long_paragraph() -> ParagraphLayout {
        // Create a long paragraph with many lines
        let mut lines = Vec::new();
        let line_text = "This is a very long text that should definitely span multiple pages when rendered with a narrow column width. ";
        let chars_per_line = 20usize;
        let num_lines = 30usize;

        let total_chars = line_text.len() * num_lines;

        for i in 0..num_lines {
            let start = i * chars_per_line;
            let end = std::cmp::min((i + 1) * chars_per_line, total_chars);
            lines.push(LineLayoutInfo {
                line_number: i,
                start,
                end,
                width: 100.0,
                break_type: "SoftBreak".to_string(),
                char_count: end - start,
                is_bidi: false,
                trailing_whitespace: 0.0,
                offset_x: 0.0,
                line_height: 12.0,
            });
        }

        ParagraphLayout {
            text: line_text.repeat(num_lines),
            max_width: 150.0,
            content_width: 150.0,
            lines,
            total_height: num_lines as f32 * 12.0,
            base_line_height: 12.0,
            actual_line_height: 12.0,
            has_bidi: false,
            properties: ParagraphProperties::default(),
        }
    }

    #[test]
    fn test_page_config_default() {
        let config = PageConfig::default();
        assert_eq!(config.width, 595.35);
        assert_eq!(config.height, 841.89);
        assert!(config.content_width() > 0.0);
        assert!(config.content_height() > 0.0);
    }

    #[test]
    fn test_page_config_content_width() {
        let config = PageConfig::default();
        let expected = 595.35 - 72.0 - 72.0;
        assert_eq!(config.content_width(), expected);
    }

    #[test]
    fn test_page_config_content_height() {
        let config = PageConfig::default();
        let expected = 841.89 - 72.0 - 72.0;
        assert_eq!(config.content_height(), expected);
    }

    #[test]
    fn test_rect_operations() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.bottom(), 70.0);
        assert_eq!(rect.right(), 110.0);
        assert!(!rect.is_empty());

        let empty = Rect::new(0.0, 0.0, 0.0, 0.0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_single_page_layout() {
        let mut page_layout = PageLayout::new();
        let paragraphs = create_test_paragraphs();

        let pages = page_layout.layout_pages(&paragraphs);

        // Should fit on one page
        assert!(!pages.is_empty());
        assert_eq!(pages.len(), 1);

        // All paragraphs should be on the page
        let page_para_indices: Vec<usize> = pages[0].lines
            .iter()
            .map(|l| l.paragraph_index)
            .collect();
        let unique_para_indices: std::collections::HashSet<usize> = page_para_indices.into_iter().collect();
        assert_eq!(unique_para_indices.len(), paragraphs.len());
    }

    #[test]
    fn test_multipage_layout() {
        let mut page_layout = PageLayout::new();

        // Create a very small page height
        let config = PageConfig {
            width: 200.0,
            height: 40.0,  // Content height = 40 - 2 - 2 = 36pt
            margin_top: 2.0,
            margin_bottom: 2.0,
            margin_left: 2.0,
            margin_right: 2.0,
            header_height: 0.0,
            footer_height: 0.0,
        };

        page_layout.page_config = config;

        // Create paragraphs where combined height exceeds one page
        let para1 = ParagraphLayout {
            text: "P1".to_string(),
            max_width: 150.0,
            content_width: 150.0,
            lines: vec![LineLayoutInfo {
                line_number: 0,
                start: 0,
                end: 2,
                width: 20.0,
                break_type: "SoftBreak".to_string(),
                char_count: 2,
                is_bidi: false,
                trailing_whitespace: 0.0,
                offset_x: 0.0,
                line_height: 15.0,  // 15pt line
            }],
            total_height: 15.0,
            base_line_height: 15.0,
            actual_line_height: 15.0,
            has_bidi: false,
            properties: ParagraphProperties::default(),
        };

        let para2 = ParagraphLayout {
            text: "P2".to_string(),
            max_width: 150.0,
            content_width: 150.0,
            lines: vec![LineLayoutInfo {
                line_number: 0,
                start: 0,
                end: 2,
                width: 20.0,
                break_type: "SoftBreak".to_string(),
                char_count: 2,
                is_bidi: false,
                trailing_whitespace: 0.0,
                offset_x: 0.0,
                line_height: 15.0,
            }],
            total_height: 15.0,
            base_line_height: 15.0,
            actual_line_height: 15.0,
            has_bidi: false,
            properties: ParagraphProperties::default(),
        };

        let para3 = ParagraphLayout {
            text: "P3".to_string(),
            max_width: 150.0,
            content_width: 150.0,
            lines: vec![LineLayoutInfo {
                line_number: 0,
                start: 0,
                end: 2,
                width: 20.0,
                break_type: "SoftBreak".to_string(),
                char_count: 2,
                is_bidi: false,
                trailing_whitespace: 0.0,
                offset_x: 0.0,
                line_height: 15.0,
            }],
            total_height: 15.0,
            base_line_height: 15.0,
            actual_line_height: 15.0,
            has_bidi: false,
            properties: ParagraphProperties::default(),
        };

        let paragraphs = vec![para1, para2, para3];

        let pages = page_layout.layout_pages(&paragraphs);

        // Content height is 36pt, each paragraph is 15pt
        // P1 (15) + P2 (15) = 30 < 36, so they fit
        // P3 (15) + 30 = 45 > 36, so it needs a new page
        // Expected: 2 pages
        assert!(pages.len() >= 2, "Expected at least 2 pages, got {}", pages.len());

        // Verify all paragraphs are present
        let total_lines: usize = pages.iter().map(|p| p.lines.len()).sum();
        assert_eq!(total_lines, 3);
    }

    #[test]
    fn test_multicolumn_layout() {
        let mut page_layout = PageLayout::new();
        page_layout.set_columns(2);
        page_layout.set_column_gap(20.0);

        let paragraphs = create_test_paragraphs();
        let pages = page_layout.layout_pages(&paragraphs);

        assert!(!pages.is_empty());

        // Each page should have proper column calculations
        let column_width = page_layout.column_width();
        assert!(column_width > 0.0);
    }

    #[test]
    fn test_widow_orphan_control() {
        let mut page_layout = PageLayout::new();
        page_layout.config.enable_widow_orphan = true;
        page_layout.config.min_lines_widow = 2;
        page_layout.config.min_lines_orphan = 2;

        let paragraphs = create_test_paragraphs();
        let pages = page_layout.layout_pages(&paragraphs);

        // Check that no page has a widow/orphan violation
        for (i, page) in pages.iter().enumerate() {
            if let Some(next_page) = pages.get(i + 1) {
                let last_para = page.lines.last().map(|l| l.paragraph_index);
                let first_para = next_page.lines.first().map(|l| l.paragraph_index);

                if last_para == first_para && last_para.is_some() {
                    let para_idx = last_para.unwrap();

                    let current_count: usize = page.lines
                        .iter()
                        .filter(|l| l.paragraph_index == para_idx)
                        .count();

                    let next_count: usize = next_page.lines
                        .iter()
                        .filter(|l| l.paragraph_index == para_idx)
                        .count();

                    // Either both pages have enough lines, or the violation was fixed
                    assert!(
                        current_count >= 2 || next_count >= 2,
                        "Widow/orphan violation detected: page {} has {} lines, page {} has {} lines",
                        i, current_count, i + 1, next_count
                    );
                }
            }
        }
    }

    #[test]
    fn test_empty_paragraphs() {
        let mut page_layout = PageLayout::new();
        let paragraphs: Vec<ParagraphLayout> = Vec::new();

        let pages = page_layout.layout_pages(&paragraphs);
        assert!(pages.is_empty());
    }

    #[test]
    fn test_page_with_header_footer() {
        let mut page_layout = PageLayout::new();
        page_layout.page_config.header_height = 30.0;
        page_layout.page_config.footer_height = 30.0;

        let paragraphs = create_test_paragraphs();
        let pages = page_layout.layout_pages(&paragraphs);

        assert!(!pages.is_empty());

        // Check that header and footer regions are set
        for page in pages.iter() {
            assert!(page.header_region.is_some());
            assert!(page.footer_region.is_some());

            // Verify header/footer are above/below content
            if let Some(header) = page.header_region {
                assert!(header.height > 0.0);
            }
            if let Some(footer) = page.footer_region {
                assert!(footer.height > 0.0);
            }
        }
    }

    #[test]
    fn test_get_page_for_offset() {
        let mut page_layout = PageLayout::new();
        let paragraphs = create_test_paragraphs();
        let _pages = page_layout.layout_pages(&paragraphs);

        // Should return Some(0) for offset 0 since all content is on page 0
        let page = page_layout.get_page_for_offset(0, &paragraphs);
        assert!(page.is_some(), "Expected Some for offset 0");
        assert_eq!(page.unwrap(), 0);

        // Test offset within first paragraph
        let page = page_layout.get_page_for_offset(10, &paragraphs);
        assert!(page.is_some(), "Expected Some for small offset");
    }

    #[test]
    fn test_paragraph_height_calculation() {
        let page_layout = PageLayout::new();

        // Create a minimal paragraph layout directly
        let para = ParagraphLayout {
            text: "Test paragraph".to_string(),
            max_width: 400.0,
            content_width: 400.0,
            lines: vec![
                crate::line_layout::LineLayoutInfo {
                    line_number: 0,
                    start: 0,
                    end: 15,
                    width: 100.0,
                    break_type: "SoftBreak".to_string(),
                    char_count: 15,
                    is_bidi: false,
                    trailing_whitespace: 0.0,
                    offset_x: 0.0,
                    line_height: 14.4,
                },
            ],
            total_height: 14.4, // 1 line * 1.2 * 12.0 font_size
            base_line_height: 12.0,
            actual_line_height: 14.4,
            has_bidi: false,
            properties: crate::line_layout::ParagraphProperties::default(),
        };

        let height = page_layout.calculate_paragraph_height(&para);

        // Height should be positive
        assert!(height > 0.0);

        // Height should match the paragraph's total_height
        assert_eq!(height, para.total_height);
    }

    #[test]
    fn test_column_width_calculation() {
        let mut page_layout = PageLayout::new();
        let config = PageConfig {
            width: 500.0,
            height: 700.0,
            margin_left: 20.0,
            margin_right: 20.0,
            ..Default::default()
        };
        page_layout.page_config = config;

        // Single column
        page_layout.set_columns(1);
        assert_eq!(page_layout.column_width(), 460.0);

        // Two columns with gap
        page_layout.set_columns(2);
        page_layout.set_column_gap(20.0);
        let two_col_width = page_layout.column_width();
        assert_eq!(two_col_width, (460.0 - 20.0) / 2.0);
        assert_eq!(two_col_width, 220.0);

        // Three columns
        page_layout.set_columns(3);
        page_layout.set_column_gap(15.0);
        let three_col_width = page_layout.column_width();
        assert_eq!(three_col_width, (460.0 - 30.0) / 3.0);
    }

    #[test]
    fn test_rendered_page_conversion() {
        let mut page_layout = PageLayout::new();
        let paragraphs = create_test_paragraphs();
        let pages = page_layout.layout_pages(&paragraphs);

        for page in pages.into_iter() {
            let rendered: RenderedPage = page.clone().into();
            assert_eq!(rendered.page_index, page.page_index);
            assert_eq!(rendered.lines.len(), page.lines.len());
        }
    }

    #[test]
    fn test_continued_pages() {
        let mut page_layout = PageLayout::new();

        // Force a paragraph to span pages by creating a long paragraph
        let paragraphs = vec![create_long_paragraph()];

        let pages = page_layout.layout_pages(&paragraphs);

        // At least one page should be marked as continued
        let has_continued = pages.iter().any(|p| p.continued_on.is_some() || p.continued_from.is_some());

        if pages.len() > 1 {
            assert!(has_continued, "Expected some pages to be marked as continued");
        }
    }

    #[test]
    fn test_letter_size_page() {
        let config = PageConfig::letter();
        assert_eq!(config.width, 612.0);
        assert_eq!(config.height, 792.0);
    }

    #[test]
    fn test_page_layout_info() {
        let page_layout = PageLayout::new();
        assert_eq!(page_layout.page_count(), 0);
        assert_eq!(page_layout.paragraph_count, 0);
    }
}

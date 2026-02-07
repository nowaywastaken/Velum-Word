//! # Layout API
//!
//! Provides layout and pagination information for the FFI interface.

use crate::page_layout::{PageConfig, PageLayout, RenderedPage, RenderedLine};
use crate::line_layout::{DocumentLayout, LineLayout, ParagraphLayout};

/// Page information for FFI
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub index: usize,
    pub width: f32,
    pub height: f32,
    pub line_count: usize,
}

/// Line information for FFI
#[derive(Debug, Clone)]
pub struct LineInfo {
    pub page_index: usize,
    pub line_index: usize,
    pub y: f32,
    pub height: f32,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// Viewport configuration
#[derive(Debug, Clone, Copy)]
pub struct ViewportInfo {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

/// Layout API trait
pub trait LayoutApi {
    /// Get total page count
    fn page_count(&self) -> usize;

    /// Get page dimensions
    fn page_info(&self, page_index: usize) -> Option<PageInfo>;

    /// Get page for character offset
    fn page_for_offset(&self, offset: usize) -> Option<usize>;

    /// Get line info for offset
    fn line_info(&self, offset: usize) -> Option<LineInfo>;

    /// Get all visible lines in viewport
    fn visible_lines(&self, viewport: ViewportInfo) -> Vec<LineInfo>;

    /// Check if offset is visible
    fn is_visible(&self, offset: usize, viewport: ViewportInfo) -> bool;
}

/// Implementation using PageLayout
#[derive(Debug)]
pub struct PageLayoutApi {
    page_layout: PageLayout,
    page_config: PageConfig,
}

impl PageLayoutApi {
    pub fn new(page_layout: PageLayout, page_config: PageConfig) -> Self {
        Self { page_layout, page_config }
    }
}

impl LayoutApi for PageLayoutApi {
    fn page_count(&self) -> usize {
        self.page_layout.pages.len()
    }

    fn page_info(&self, page_index: usize) -> Option<PageInfo> {
        self.page_layout.pages.get(page_index).map(|page| PageInfo {
            index: page.page_index,
            width: self.page_config.width,
            height: self.page_config.height,
            line_count: page.lines.len(),
        })
    }

    fn page_for_offset(&self, offset: usize) -> Option<usize> {
        // Simple implementation: find the page that contains the offset
        let mut char_count = 0usize;
        for (page_idx, page) in self.page_layout.pages.iter().enumerate() {
            let page_char_count: usize = page.lines.iter().map(|l| l.end - l.start).sum();
            if char_count + page_char_count > offset {
                return Some(page_idx);
            }
            char_count += page_char_count;
        }
        None
    }

    fn line_info(&self, offset: usize) -> Option<LineInfo> {
        // Find the page and line containing this offset
        for (page_idx, page) in self.page_layout.pages.iter().enumerate() {
            for (line_idx, line) in page.lines.iter().enumerate() {
                if line.start <= offset && offset < line.end {
                    return Some(LineInfo {
                        page_index: page_idx,
                        line_index: line_idx,
                        y: line.y,
                        height: line.height,
                        start_offset: line.start,
                        end_offset: line.end,
                    });
                }
            }
        }
        None
    }

    fn visible_lines(&self, viewport: ViewportInfo) -> Vec<LineInfo> {
        let mut visible = Vec::new();
        let viewport_height = viewport.height / viewport.scale;

        for (page_idx, page) in self.page_layout.pages.iter().enumerate() {
            let page_top = page_idx as f32 * (self.page_config.height + 20.0); // Add gap
            let page_bottom = page_top + self.page_config.height;

            // Check if page intersects viewport
            if page_bottom < 0.0 || page_top > viewport_height {
                continue;
            }

            for (line_idx, line) in page.lines.iter().enumerate() {
                let line_top = page_top + line.y;
                let line_bottom = line_top + line.height;

                if line_bottom >= 0.0 && line_top <= viewport_height {
                    visible.push(LineInfo {
                        page_index: page_idx,
                        line_index: line_idx,
                        y: line.y,
                        height: line.height,
                        start_offset: line.start,
                        end_offset: line.end,
                    });
                }
            }
        }

        visible
    }

    fn is_visible(&self, offset: usize, viewport: ViewportInfo) -> bool {
        !self.visible_lines(viewport).is_empty()
    }
}

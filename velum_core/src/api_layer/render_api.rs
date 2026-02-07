//! # Render API
//!
//! Provides rendering operations for the FFI interface.

use crate::page_layout::{RenderedPage, Rect};
use crate::image::RenderedImage;

/// Rectangle for rendering
#[derive(Debug, Clone, Copy)]
pub struct RenderRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Dirty rect for incremental updates
#[derive(Debug, Clone)]
pub struct DirtyRect {
    pub rect: RenderRect,
    pub from_offset: usize,
    pub to_offset: usize,
}

/// Render command types
#[derive(Debug, Clone)]
pub enum RenderCommand {
    DrawText {
        text: String,
        x: f32,
        y: f32,
        font_size: f32,
        color: String,
        bold: bool,
        italic: bool,
    },
    DrawRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: String,
    },
    DrawImage {
        image_id: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    DrawSelection {
        rects: Vec<RenderRect>,
        color: String,
    },
    DrawCursor {
        x: f32,
        y: f32,
        height: f32,
        color: String,
    },
}

/// Page render data
#[derive(Debug, Clone)]
pub struct PageRenderData {
    pub page_index: usize,
    pub width: f32,
    pub height: f32,
    pub commands: Vec<RenderCommand>,
    pub images: Vec<RenderedImage>,
}

/// Render API trait
pub trait RenderApi {
    /// Get dirty regions for incremental update
    fn dirty_regions(&self, from_offset: usize, to_offset: usize) -> Vec<DirtyRect>;

    /// Get render commands for a page
    fn render_page_commands(&self, page_index: usize) -> Option<Vec<RenderCommand>>;

    /// Get all images on a page
    fn page_images(&self, page_index: usize) -> Vec<RenderedImage>;

    /// Export page as image bytes (PNG)
    fn export_page_as_png(&self, page_index: usize, scale: f32) -> Option<Vec<u8>>;
}

/// Simple render API implementation
#[derive(Debug, Clone)]
pub struct SimpleRenderApi {
    pages: Vec<RenderedPage>,
}

impl SimpleRenderApi {
    pub fn new(pages: Vec<RenderedPage>) -> Self {
        Self { pages }
    }
}

impl RenderApi for SimpleRenderApi {
    fn dirty_regions(&self, from_offset: usize, to_offset: usize) -> Vec<DirtyRect> {
        let mut dirty = Vec::new();

        for page in &self.pages {
            // Check if page has content in the affected range
            let has_content = page.lines.iter().any(|line| {
                (line.start >= from_offset && line.start < to_offset) ||
                (line.end > from_offset && line.end <= to_offset) ||
                (line.start <= from_offset && line.end >= to_offset)
            });

            if has_content {
                dirty.push(DirtyRect {
                    rect: RenderRect {
                        x: page.content_bounds.x,
                        y: page.content_bounds.y,
                        width: page.content_bounds.width,
                        height: page.content_bounds.height,
                    },
                    from_offset,
                    to_offset,
                });
            }
        }

        dirty
    }

    fn render_page_commands(&self, page_index: usize) -> Option<Vec<RenderCommand>> {
        let page = self.pages.get(page_index)?;
        let mut commands = Vec::new();

        // Add text rendering commands
        for line in &page.lines {
            commands.push(RenderCommand::DrawText {
                text: format!("[Text {}..{}]", line.start, line.end),
                x: line.x,
                y: line.y,
                font_size: 12.0,
                color: "#000000".to_string(),
                bold: false,
                italic: false,
            });
        }

        // Add selection rects if any (placeholder)
        // In real implementation, this would come from selection state

        Some(commands)
    }

    fn page_images(&self, page_index: usize) -> Vec<RenderedImage> {
        // In a full implementation, this would track images per page
        Vec::new()
    }

    fn export_page_as_png(&self, _page_index: usize, _scale: f32) -> Option<Vec<u8>> {
        // In a full implementation, this would render to PNG
        // For now, return None
        None
    }
}

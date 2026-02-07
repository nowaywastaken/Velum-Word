//! Image module for Velum word processing core.
//! Provides image loading, caching, scaling, anchoring, and text wrapping functionality.
//!
//! # Features
//! - PNG, JPEG, GIF, BMP, WebP image format support
//! - OOXML relationship-based image loading
//! - Image caching for performance
//! - Flexible scaling modes (None, Exact, Percentage, FitToContainer)
//! - Multiple anchoring types (Inline, Floating)
//! - Text wrapping support (Square, Tight, Through, TopBottom, Behind, InFront)
//!
//! # Example
//!
//! ```rust
//! use velum_core::image::{ImageCache, ScaleMode, WrapType};
//!
//! // Create image cache
//! let mut cache = ImageCache::new();
//!
//! // Load image from OOXML package
//! let image = cache.load_from_ooxml(b"...", "image/png", "word/media/image1.png")?;
//! ```
//!
//! # OOXML Image Relationships
//!
//! Images in OOXML documents are referenced through relationships in document.xml.rels:
//! ```xml
//! <Relationship Id="rId5"
//!     Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
//!     Target="media/image1.png"/>
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use log::debug;
use once_cell::sync::Lazy;

use crate::ooxml::{ContentType, Relationship, RelationshipType, DocumentImage, PackagePart};

/// EMU (English Metric Unit) conversion constants
/// 1 inch = 914400 EMUs
/// 1 point = 12700 EMUs
pub const EMU_PER_INCH: f32 = 914400.0;
pub const EMU_PER_POINT: f32 = 12700.0;
pub const EMU_PER_PIXEL: f32 = 9525.0;  // Assuming 96 DPI

// ============================================================================
// Scale Mode
// ============================================================================

/// Specifies how an image should be scaled relative to its original size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScaleMode {
    /// No scaling - use the original dimensions
    None,
    /// Exact dimensions specified in EMUs or pixels
    Exact(WidthHeight),
    /// Percentage of original size (0.0 to 10.0, where 1.0 = 100%)
    Percentage(f32),
    /// Scale to fit within container while maintaining aspect ratio
    FitToContainer,
    /// Scale to fill container while maintaining aspect ratio
    FillContainer,
}

/// Width and height specification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct WidthHeight {
    /// Width value
    pub width: f32,
    /// Height value
    pub height: f32,
}

impl Default for WidthHeight {
    fn default() -> Self {
        Self { width: 0.0, height: 0.0 }
    }
}

impl WidthHeight {
    /// Create a new width-height pair
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Calculate aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0.0 {
            1.0
        } else {
            self.width / self.height
        }
    }
}

// ============================================================================
// Image Anchor Type
// ============================================================================

/// Defines how an image is anchored to the document content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageAnchorType {
    /// Inline anchor - flows with text like a character
    Inline,
    /// Floating anchor - positioned absolutely on the page
    Floating,
    /// Anchor to a specific page
    Page,
    /// Anchor to a paragraph
    Paragraph,
    /// Anchor to a character position
    Character,
}

// ============================================================================
// Anchor Position
// ============================================================================

/// Specifies where an anchored element should be positioned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorPosition {
    /// Anchor to a specific page (1-based page number)
    Page(usize),
    /// Anchor to a paragraph by its ID
    Paragraph(String),
    /// Anchor to a character by its position
    Character(usize),
    /// Anchor to margin (top, bottom, left, right)
    Margin(String),
}

// ============================================================================
// Wrap Type
// ============================================================================

/// Defines how text should wrap around a floating image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapType {
    /// Square wrap - text flows around the bounding box
    Square,
    /// Tight wrap - text follows the actual shape of the image
    Tight,
    /// Through wrap - text flows over the entire image
    Through,
    /// Top and bottom only - text is above and below the image
    TopBottom,
    /// Behind text - image appears behind the text
    Behind,
    /// In front of text - image appears on top of the text
    InFront,
}

impl WrapType {
    /// Check if this wrap type places the image behind text
    pub fn is_behind_text(&self) -> bool {
        matches!(self, WrapType::Behind)
    }

    /// Check if this wrap type places the image in front of text
    pub fn is_in_front_of_text(&self) -> bool {
        matches!(self, WrapType::InFront)
    }

    /// Check if this wrap type requires wrap polygon calculation
    pub fn requires_wrap_polygon(&self) -> bool {
        matches!(self, WrapType::Tight | WrapType::Through)
    }
}

// ============================================================================
// Wrap Distance
// ============================================================================

/// Defines the distance between an image and surrounding text.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WrapDistance {
    /// Distance from top edge to text
    pub top: f32,
    /// Distance from bottom edge to text
    pub bottom: f32,
    /// Distance from left edge to text
    pub left: f32,
    /// Distance from right edge to text
    pub right: f32,
}

impl Default for WrapDistance {
    fn default() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }
}

impl WrapDistance {
    /// Create new wrap distance with all sides equal
    pub fn uniform(distance: f32) -> Self {
        Self {
            top: distance,
            bottom: distance,
            left: distance,
            right: distance,
        }
    }

    /// Create new wrap distance with vertical and horizontal values
    pub fn vertical_horizontal(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    /// Get total horizontal distance (left + right)
    pub fn horizontal_total(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical distance (top + bottom)
    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }
}

// ============================================================================
// Scale
// ============================================================================

/// Represents the scale applied to an image.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    /// Horizontal scale factor (1.0 = original size)
    pub x: f32,
    /// Vertical scale factor (1.0 = original size)
    pub y: f32,
}

impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

impl Scale {
    /// Create a uniform scale (same for both axes)
    pub fn uniform(scale: f32) -> Self {
        Self { x: scale, y: scale }
    }

    /// Create a new scale with different x and y factors
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Calculate scale from original and desired dimensions
    pub fn from_dimensions(original: Size, desired: Size) -> Self {
        let x = if original.width > 0.0 { desired.width / original.width } else { 1.0 };
        let y = if original.height > 0.0 { desired.height / original.height } else { 1.0 };
        Self { x, y }
    }

    /// Apply scale to a size
    pub fn apply(&self, size: Size) -> Size {
        Size::new(size.width * self.x, size.height * self.y)
    }
}

// ============================================================================
// Point and Size
// ============================================================================

/// 2D point with floating-point coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl Default for Point {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Translate point by another point
    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }
}

/// 2D size with floating-point dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    /// Width dimension
    pub width: f32,
    /// Height dimension
    pub height: f32,
}

impl Default for Size {
    fn default() -> Self {
        Self { width: 0.0, height: 0.0 }
    }
}

impl Size {
    /// Create a new size
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a size from EMU dimensions
    pub fn from_emu(width_emu: u32, height_emu: u32) -> Self {
        Self::new(
            width_emu as f32 / EMU_PER_PIXEL,
            height_emu as f32 / EMU_PER_PIXEL,
        )
    }

    /// Convert to EMU dimensions
    pub fn to_emu(&self) -> (u32, u32) {
        (
            (self.width * EMU_PER_PIXEL) as u32,
            (self.height * EMU_PER_PIXEL) as u32,
        )
    }

    /// Check if size is valid (positive dimensions)
    pub fn is_valid(&self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }

    /// Calculate aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0.0 {
            1.0
        } else {
            self.width / self.height
        }
    }

    /// Scale to fit within a container while maintaining aspect ratio
    pub fn scale_to_fit(&self, max_width: f32, max_height: f32) -> Self {
        let ratio = self.aspect_ratio();
        let container_ratio = if max_height > 0.0 { max_width / max_height } else { 1.0 };

        if ratio > container_ratio {
            // Width is the limiting factor
            Self::new(max_width, max_width / ratio)
        } else {
            // Height is the limiting factor
            Self::new(max_height * ratio, max_height)
        }
    }

    /// Scale to fill a container while maintaining aspect ratio
    pub fn scale_to_fill(&self, max_width: f32, max_height: f32) -> Self {
        let ratio = self.aspect_ratio();
        let container_ratio = if max_height > 0.0 { max_width / max_height } else { 1.0 };

        if ratio < container_ratio {
            // Width is the limiting factor
            Self::new(max_width, max_width / ratio)
        } else {
            // Height is the limiting factor
            Self::new(max_height * ratio, max_height)
        }
    }
}

// ============================================================================
// Rendered Image
// ============================================================================

/// Contains all information needed to render an image in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedImage {
    /// Unique identifier for this image
    pub image_id: String,
    /// Position where the image should be drawn
    pub position: Point,
    /// Size at which the image should be rendered
    pub size: Size,
    /// Original source dimensions
    pub source_size: Size,
    /// Scale factors applied to the image
    pub scale: Scale,
    /// How the image is anchored in the document
    pub anchor_type: ImageAnchorType,
    /// How text should wrap around this image
    pub wrap_type: Option<WrapType>,
    /// Distance between image and wrapped text
    pub wrap_distance: Option<WrapDistance>,
    /// Z-order for layering (higher = on top)
    pub z_order: i32,
    /// Whether the image is visible
    pub visible: bool,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
    /// Opacity (0.0 to 1.0)
    pub opacity: f32,
}

impl Default for RenderedImage {
    fn default() -> Self {
        Self {
            image_id: String::new(),
            position: Point::default(),
            size: Size::default(),
            source_size: Size::default(),
            scale: Scale::default(),
            anchor_type: ImageAnchorType::Inline,
            wrap_type: None,
            wrap_distance: None,
            z_order: 0,
            visible: true,
            alt_text: None,
            opacity: 1.0,
        }
    }
}

impl RenderedImage {
    /// Create a new rendered image
    pub fn new(
        image_id: String,
        position: Point,
        size: Size,
        anchor_type: ImageAnchorType,
    ) -> Self {
        Self {
            image_id,
            position,
            size,
            source_size: size,
            scale: Scale::uniform(1.0),
            anchor_type,
            wrap_type: None,
            wrap_distance: None,
            z_order: 0,
            visible: true,
            alt_text: None,
            opacity: 1.0,
        }
    }

    /// Calculate the bounding rectangle for this image
    pub fn bounding_rect(&self) -> Rect {
        Rect::from_point_size(self.position, self.size)
    }
}

// ============================================================================
// Rectangle
// ============================================================================

/// Axis-aligned rectangle for bounds calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// Left edge X coordinate
    pub x: f32,
    /// Top edge Y coordinate
    pub y: f32,
    /// Rectangle width
    pub width: f32,
    /// Rectangle height
    pub height: f32,
}

impl Default for Rect {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }
    }
}

impl Rect {
    /// Create a rectangle from point and size
    pub fn from_point_size(point: Point, size: Size) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }

    /// Create a rectangle from x, y, width, height
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Get the left edge
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Get the right edge
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get the top edge
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Get the bottom edge
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    /// Check if a point is inside this rectangle
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.left() && point.x <= self.right()
            && point.y >= self.top() && point.y <= self.bottom()
    }

    /// Get the expanded rectangle with wrap distance applied
    pub fn expanded(&self, distance: WrapDistance) -> Rect {
        Rect::new(
            self.x - distance.left,
            self.y - distance.top,
            self.width + distance.horizontal_total(),
            self.height + distance.vertical_total(),
        )
    }
}

// ============================================================================
// Image Data
// ============================================================================

/// Raw image data with format information.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Raw image bytes
    pub data: Vec<u8>,
    /// Detected image format
    pub format: ImageFormat,
    /// Image dimensions in pixels
    pub dimensions: Size,
    /// Whether this is an animated image
    pub is_animated: bool,
    /// Number of animation frames (if animated)
    pub frame_count: usize,
    /// Color depth in bits per pixel
    pub bit_depth: u16,
    /// Color type (grayscale, RGB, RGBA, palette, etc.)
    pub color_type: ColorType,
}

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageFormat {
    /// Portable Network Graphics
    Png,
    /// JPEG (Joint Photographic Experts Group)
    Jpeg,
    /// Graphics Interchange Format
    Gif,
    /// Bitmap image
    Bmp,
    /// WebP image
    WebP,
    /// Scalable Vector Graphics
    Svg,
    /// TIFF image
    Tiff,
    /// Unknown format
    Unknown,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "PNG"),
            ImageFormat::Jpeg => write!(f, "JPEG"),
            ImageFormat::Gif => write!(f, "GIF"),
            ImageFormat::Bmp => write!(f, "BMP"),
            ImageFormat::WebP => write!(f, "WebP"),
            ImageFormat::Svg => write!(f, "SVG"),
            ImageFormat::Tiff => write!(f, "TIFF"),
            ImageFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ImageFormat {
    /// Detect format from magic bytes at the start of the data
    pub fn from_magic_bytes(data: &[u8]) -> Self {
        if data.len() < 12 {
            return ImageFormat::Unknown;
        }

        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return ImageFormat::Png;
        }

        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return ImageFormat::Jpeg;
        }

        // GIF87a or GIF89a: 47 49 46 38 37 61 or 47 49 46 38 39 61
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return ImageFormat::Gif;
        }

        // BMP: 42 4D
        if data.starts_with(&[0x42, 0x4D]) {
            return ImageFormat::Bmp;
        }

        // WebP: 52 49 46 46 ... (RIFF) followed by 57 45 42 50 (WEBP)
        if data.len() >= 12
            && data[0..4] == [0x52, 0x49, 0x46, 0x46]
            && &data[8..12] == b"WEBP"
        {
            return ImageFormat::WebP;
        }

        // SVG: 3C 73 76 67 (<?xml or <svg)
        if data.starts_with(b"<?xml") || (data.starts_with(b"<svg") && data.len() > 4) {
            return ImageFormat::Svg;
        }

        ImageFormat::Unknown
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Bmp => "image/bmp",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Svg => "image/svg+xml",
            ImageFormat::Tiff => "image/tiff",
            ImageFormat::Unknown => "application/octet-stream",
        }
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::WebP => "webp",
            ImageFormat::Svg => "svg",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Unknown => "bin",
        }
    }
}

/// Color type enumeration for image data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorType {
    /// Grayscale (1, 2, 4, or 8 bit)
    Grayscale,
    /// Grayscale with alpha
    GrayscaleAlpha,
    /// RGB (true color)
    Rgb,
    /// RGB with alpha (RGBA)
    Rgba,
    /// Palette-indexed
    Palette,
    /// Unknown color type
    Unknown,
}

// ============================================================================
// Image Cache
// ============================================================================

/// Thread-safe image cache for storing loaded images.
#[derive(Debug)]
pub struct ImageCache {
    /// Cache entries indexed by image path
    cache: HashMap<String, Arc<ImageData>>,
    /// Maximum cache size in bytes
    max_size_bytes: usize,
    /// Current cache size in bytes
    current_size: usize,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageCache {
    /// Create a new image cache with default settings (100MB limit)
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size_bytes: 100 * 1024 * 1024, // 100 MB
            current_size: 0,
        }
    }

    /// Create a new image cache with custom size limit
    pub fn with_max_size(max_size_bytes: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size_bytes,
            current_size: 0,
        }
    }

    /// Load an image from raw bytes and cache it
    pub fn load(&mut self, path: String, data: Vec<u8>) -> Result<Arc<ImageData>, ImageError> {
        let format = ImageFormat::from_magic_bytes(&data);

        if format == ImageFormat::Unknown {
            return Err(ImageError::UnknownFormat);
        }

        // Parse image dimensions based on format
        let dimensions = self::decode_dimensions(&data, format)?;

        let image_data = Arc::new(ImageData {
            data,
            format,
            dimensions,
            is_animated: false,
            frame_count: 1,
            bit_depth: 32,
            color_type: ColorType::Rgba,
        });

        // Calculate entry size
        let entry_size = image_data.data.len();

        // Check if we need to evict entries
        while self.current_size + entry_size > self.max_size_bytes && !self.cache.is_empty() {
            // Simple eviction: remove the first entry (FIFO)
            let keys: Vec<_> = self.cache.keys().cloned().collect();
            if let Some(key) = keys.first() {
                if let Some(entry) = self.cache.remove(key) {
                    self.current_size -= entry.data.len();
                }
            } else {
                break;
            }
        }

        // Insert the new image
        self.cache.insert(path.clone(), Arc::clone(&image_data));
        self.current_size += entry_size;

        debug!("Loaded image: {}, format: {}, dimensions: {}x{}",
            path, format, dimensions.width as u32, dimensions.height as u32);

        Ok(image_data)
    }

    /// Load an image from OOXML package data
    pub fn load_from_ooxml(
        &mut self,
        data: &[u8],
        content_type: ContentType,
        path: String,
    ) -> Result<Arc<ImageData>, ImageError> {
        let format = match content_type {
            ContentType::ImagePng => ImageFormat::Png,
            ContentType::ImageJpeg => ImageFormat::Jpeg,
            ContentType::ImageGif => ImageFormat::Gif,
            ContentType::ImageBmp => ImageFormat::Bmp,
            ContentType::ImageWebP => ImageFormat::WebP,
            ContentType::ImageTiff => ImageFormat::Tiff,
            ContentType::ImageSvg => ImageFormat::Svg,
            ContentType::Thumbnail => ImageFormat::from_magic_bytes(data),
            _ => return Err(ImageError::UnsupportedFormat),
        };

        let dimensions = self::decode_dimensions(data, format)?;

        let image_data = Arc::new(ImageData {
            data: data.to_vec(),
            format,
            dimensions,
            is_animated: false,
            frame_count: 1,
            bit_depth: 32,
            color_type: ColorType::Rgba,
        });

        // Calculate entry size
        let entry_size = image_data.data.len();

        // Check if we need to evict entries
        while self.current_size + entry_size > self.max_size_bytes && !self.cache.is_empty() {
            let keys: Vec<_> = self.cache.keys().cloned().collect();
            if let Some(key) = keys.first() {
                if let Some(entry) = self.cache.remove(key) {
                    self.current_size -= entry.data.len();
                }
            } else {
                break;
            }
        }

        self.cache.insert(path.clone(), Arc::clone(&image_data));
        self.current_size += entry_size;

        debug!("Loaded image from OOXML: {}, format: {}, dimensions: {}x{}",
            path, format, dimensions.width as u32, dimensions.height as u32);

        Ok(image_data)
    }

    /// Get an image from the cache
    pub fn get(&self, path: &str) -> Option<Arc<ImageData>> {
        self.cache.get(path).cloned()
    }

    /// Check if an image is in the cache
    pub fn contains(&self, path: &str) -> bool {
        self.cache.contains_key(path)
    }

    /// Remove an image from the cache
    pub fn remove(&mut self, path: &str) -> Option<Arc<ImageData>> {
        if let Some(entry) = self.cache.remove(path) {
            self.current_size -= entry.data.len();
            Some(entry)
        } else {
            None
        }
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_size = 0;
    }

    /// Get the current number of cached images
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get the current cache size in bytes
    pub fn size_bytes(&self) -> usize {
        self.current_size
    }
}

// ============================================================================
// Image Errors
// ============================================================================

/// Errors that can occur during image loading and processing.
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Unknown or unsupported image format")]
    UnknownFormat,
    #[error("Unsupported image format")]
    UnsupportedFormat,
    #[error("Failed to decode image: {0}")]
    DecodeError(String),
    #[error("Image dimensions exceed maximum allowed size")]
    DimensionsExceeded,
    #[error("Invalid image dimensions (zero or negative)")]
    InvalidDimensions,
    #[error("Image not found in cache")]
    NotFound,
    #[error("Failed to load image from OOXML: {0}")]
    OoxmlLoadError(String),
}

// ============================================================================
// Image Scaling
// ============================================================================

/// Calculate the final size for an image based on scale mode and container.
#[doc(hidden)]
pub fn calculate_image_size(
    source_size: Size,
    scale_mode: ScaleMode,
    container_size: Option<Size>,
) -> Size {
    match scale_mode {
        ScaleMode::None => source_size,
        ScaleMode::Exact(dim) => Size::new(dim.width, dim.height),
        ScaleMode::Percentage(pct) => {
            Size::new(source_size.width * pct, source_size.height * pct)
        }
        ScaleMode::FitToContainer => {
            if let Some(container) = container_size {
                source_size.scale_to_fit(container.width, container.height)
            } else {
                source_size
            }
        }
        ScaleMode::FillContainer => {
            if let Some(container) = container_size {
                source_size.scale_to_fill(container.width, container.height)
            } else {
                source_size
            }
        }
    }
}

/// Calculate the rendering scale from source to desired dimensions
#[doc(hidden)]
pub fn calculate_scale(source: Size, desired: Size) -> Scale {
    Scale::from_dimensions(source, desired)
}

// ============================================================================
// Text Wrap Polygon
// ============================================================================

/// Represents a polygon for text wrapping around complex shapes.
#[derive(Debug, Clone)]
pub struct WrapPolygon {
    /// List of points defining the polygon
    pub points: Vec<Point>,
    /// Whether the polygon is valid (at least 3 points)
    pub is_valid: bool,
}

impl WrapPolygon {
    /// Create a simple rectangular wrap polygon
    pub fn from_rect(rect: Rect, distance: WrapDistance) -> Self {
        let expanded = rect.expanded(distance);
        let points = vec![
            Point::new(expanded.left(), expanded.top()),
            Point::new(expanded.right(), expanded.top()),
            Point::new(expanded.right(), expanded.bottom()),
            Point::new(expanded.left(), expanded.bottom()),
            Point::new(expanded.left(), expanded.top()),
        ];
        let is_valid = !points.is_empty();
        Self { points, is_valid }
    }

    /// Create an elliptical wrap polygon
    pub fn from_ellipse(center: Point, radii: Size, distance: WrapDistance) -> Self {
        let rx = (radii.width + distance.horizontal_total()) / 2.0;
        let ry = (radii.height + distance.vertical_total()) / 2.0;

        let mut points = Vec::with_capacity(64);
        for i in 0..64 {
            let angle = (i as f32 / 64.0) * std::f32::consts::TAU;
            points.push(Point::new(
                center.x + rx * angle.cos(),
                center.y + ry * angle.sin(),
            ));
        }

        let is_valid = !points.is_empty();
        Self { points, is_valid }
    }
}

/// Calculate the wrap region that text should avoid
#[doc(hidden)]
pub fn calculate_wrap_region(image: &RenderedImage) -> WrapPolygon {
    let bounding_rect = image.bounding_rect();

    match image.wrap_type {
        Some(WrapType::Square) | None => {
            let distance = image.wrap_distance.unwrap_or_default();
            WrapPolygon::from_rect(bounding_rect, distance)
        }
        Some(WrapType::Tight) => {
            // Tight wrap would require actual image alpha mask
            // For now, fall back to square
            let distance = image.wrap_distance.unwrap_or_default();
            WrapPolygon::from_rect(bounding_rect, distance)
        }
        Some(WrapType::Through) => {
            // Through wrap means no wrap region
            WrapPolygon { points: Vec::new(), is_valid: false }
        }
        Some(WrapType::TopBottom) => {
            // Top and bottom wrap: exclude the horizontal strip
            let distance = image.wrap_distance.unwrap_or_default();
            WrapPolygon::from_rect(bounding_rect, distance)
        }
        Some(WrapType::Behind | WrapType::InFront) => {
            // Behind/in front: no wrap region needed
            WrapPolygon { points: Vec::new(), is_valid: false }
        }
    }
}

// ============================================================================
// Image Dimension Decoding
// ============================================================================

/// Decode image dimensions from raw bytes (format-dependent).
fn decode_dimensions(data: &[u8], format: ImageFormat) -> Result<Size, ImageError> {
    match format {
        ImageFormat::Png => decode_png_dimensions(data),
        ImageFormat::Jpeg => decode_jpeg_dimensions(data),
        ImageFormat::Gif => decode_gif_dimensions(data),
        ImageFormat::Bmp => decode_bmp_dimensions(data),
        ImageFormat::WebP => decode_webp_dimensions(data),
        ImageFormat::Svg => decode_svg_dimensions(data),
        ImageFormat::Tiff => decode_tiff_dimensions(data),
        ImageFormat::Unknown => Err(ImageError::UnknownFormat),
    }
}

/// Decode PNG dimensions from IHDR chunk.
fn decode_png_dimensions(data: &[u8]) -> Result<Size, ImageError> {
    // PNG signature: 8 bytes, IHDR starts at byte 8
    if data.len() < 24 {
        return Err(ImageError::InvalidDimensions);
    }

    // IHDR chunk length (4 bytes big-endian) + type (4 bytes) + width (4) + height (4)
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);

    if width == 0 || height == 0 {
        return Err(ImageError::InvalidDimensions);
    }

    if width > 1_000_000 || height > 1_000_000 {
        return Err(ImageError::DimensionsExceeded);
    }

    Ok(Size::new(width as f32, height as f32))
}

/// Decode JPEG dimensions from SOF markers.
fn decode_jpeg_dimensions(data: &[u8]) -> Result<Size, ImageError> {
    if data.len() < 2 {
        return Err(ImageError::InvalidDimensions);
    }

    let mut i = 0;
    while i < data.len().saturating_sub(1) {
        if data[i] == 0xFF {
            let marker = data[i + 1];

            // SOF0-SOF3 markers contain dimensions
            if matches!(marker, 0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE | 0xCF) {
                if i + 9 < data.len() {
                    // Skip length (2 bytes) and precision (1 byte)
                    let height = u16::from_be_bytes([data[i + 5], data[i + 6]]);
                    let width = u16::from_be_bytes([data[i + 7], data[i + 8]]);

                    if width == 0 || height == 0 {
                        return Err(ImageError::InvalidDimensions);
                    }

                    return Ok(Size::new(width as f32, height as f32));
                }
            }
        }
        i += 1;
    }

    Err(ImageError::DecodeError("No SOF marker found".to_string()))
}

/// Decode GIF dimensions from Logical Screen Descriptor.
fn decode_gif_dimensions(data: &[u8]) -> Result<Size, ImageError> {
    if data.len() < 10 {
        return Err(ImageError::InvalidDimensions);
    }

    let width = u16::from_le_bytes([data[6], data[7]]);
    let height = u16::from_le_bytes([data[8], data[9]]);

    if width == 0 || height == 0 {
        return Err(ImageError::InvalidDimensions);
    }

    Ok(Size::new(width as f32, height as f32))
}

/// Decode BMP dimensions from BITMAPINFOHEADER.
fn decode_bmp_dimensions(data: &[u8]) -> Result<Size, ImageError> {
    if data.len() < 54 {
        return Err(ImageError::InvalidDimensions);
    }

    // BMP stores height as absolute value, can be negative for top-down
    let height = i32::from_le_bytes([data[14], data[15], data[16], data[17]]);
    let width = i32::from_le_bytes([data[18], data[19], data[20], data[21]]);

    if width <= 0 || height == 0 {
        return Err(ImageError::InvalidDimensions);
    }

    Ok(Size::new(width as f32, height.abs() as f32))
}

/// Decode WebP dimensions from RIFF header.
fn decode_webp_dimensions(data: &[u8]) -> Result<Size, ImageError> {
    if data.len() < 12 {
        return Err(ImageError::InvalidDimensions);
    }

    // WebP Extended Format has VP8X at byte 12
    // VP8X: 4 bytes signature + 4 bytes flags + 4 bytes canvas width/height
    if data.len() >= 20 && &data[12..16] == b"VP8X" {
        // Canvas dimensions are at offset 16, 24-bit LE
        let width_minus_one = u32::from_le_bytes([data[20], data[21], data[22], 0]);
        let height_minus_one = u32::from_le_bytes([data[23], data[24], data[25], 0]);

        let width = width_minus_one + 1;
        let height = height_minus_one + 1;

        return Ok(Size::new(width as f32, height as f32));
    }

    // Simple WebP (VP8) has dimensions at byte 26-33
    if data.len() >= 34 {
        let width = u16::from_le_bytes([data[26], data[27]]);
        let height = u16::from_le_bytes([data[28], data[29]]);

        if width == 0 || height == 0 {
            return Err(ImageError::InvalidDimensions);
        }

        return Ok(Size::new(width as f32, height as f32));
    }

    Err(ImageError::InvalidDimensions)
}

/// Decode SVG dimensions from viewBox or width/height attributes.
fn decode_svg_dimensions(data: &[u8]) -> Result<Size, ImageError> {
    let svg_str = String::from_utf8_lossy(data);

    // Try to find viewBox first
    if let Some(viewbox_match) = regex::Regex::new(r#"viewBox\s*=\s*["']([^"']+)["']"#)
        .ok()
        .and_then(|re| re.captures(&svg_str))
    {
        if let Some(viewbox) = viewbox_match.get(1) {
            let parts: Vec<&str> = viewbox.as_str().split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(w), Ok(h)) = (parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                    if w > 0.0 && h > 0.0 {
                        return Ok(Size::new(w, h));
                    }
                }
            }
        }
    }

    // Try width and height attributes
    let width_re = regex::Regex::new(r#"width\s*=\s*["'](\d+\.?\d*)(px|pt|em|ex|%)?["']"#).ok();
    let height_re = regex::Regex::new(r#"height\s*=\s*["'](\d+\.?\d*)(px|pt|em|ex|%)?["']"#).ok();

    let mut width = None;
    let mut height = None;

    if let Some(re) = width_re {
        if let Some(captures) = re.captures(&svg_str) {
            if let Some(m) = captures.get(1) {
                width = m.as_str().parse::<f32>().ok();
            }
        }
    }

    if let Some(re) = height_re {
        if let Some(captures) = re.captures(&svg_str) {
            if let Some(m) = captures.get(1) {
                height = m.as_str().parse::<f32>().ok();
            }
        }
    }

    match (width, height) {
        (Some(w), Some(h)) if w > 0.0 && h > 0.0 => Ok(Size::new(w, h)),
        (Some(w), _) if w > 0.0 => Ok(Size::new(w, w)), // Assume square if only width
        (_, Some(h)) if h > 0.0 => Ok(Size::new(h, h)), // Assume square if only height
        _ => Ok(Size::new(100.0, 100.0)), // Default SVG size
    }
}

/// Decode TIFF dimensions from IFD.
fn decode_tiff_dimensions(data: &[u8]) -> Result<Size, ImageError> {
    if data.len() < 8 {
        return Err(ImageError::InvalidDimensions);
    }

    // Check byte order
    let is_le = data[0] == b'I' && data[1] == b'b';
    let is_be = data[0] == b'M' && data[1] == 0x00;

    if !is_le && !is_be {
        return Err(ImageError::InvalidDimensions);
    }

    let offset = if is_le {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]])
    };

    // Simplified TIFF parsing - just read width/height from first IFD
    // This is a basic implementation; a full parser would handle more cases
    if data.len() > offset as usize + 12 {
        let idx = offset as usize;
        let num_entries = if is_le {
            u16::from_le_bytes([data[idx], data[idx + 1]])
        } else {
            u16::from_be_bytes([data[idx], data[idx + 1]])
        };

        // Look for ImageWidth (254) and ImageHeight (257) tags
        for i in 0..num_entries.min(10) {
            let entry_idx = idx + 2 + (i as usize * 12);
            if entry_idx + 12 > data.len() {
                break;
            }

            let tag = if is_le {
                u16::from_le_bytes([data[entry_idx], data[entry_idx + 1]])
            } else {
                u16::from_be_bytes([data[entry_idx], data[entry_idx + 1]])
            };

            if tag == 256 { // ImageWidth
                let value = if is_le {
                    u32::from_le_bytes([data[entry_idx + 8], data[entry_idx + 9],
                                       data[entry_idx + 10], data[entry_idx + 11]])
                } else {
                    u32::from_be_bytes([data[entry_idx + 8], data[entry_idx + 9],
                                       data[entry_idx + 10], data[entry_idx + 11]])
                };

                if value > 0 && value < 1_000_000 {
                    let height_offset = idx + 2 + 12; // Next entry
                    if height_offset + 12 <= data.len() {
                        let height_tag = if is_le {
                            u16::from_le_bytes([data[height_offset], data[height_offset + 1]])
                        } else {
                            u16::from_be_bytes([data[height_offset], data[height_offset + 1]])
                        };

                        if height_tag == 257 { // ImageHeight
                            let height = if is_le {
                                u32::from_le_bytes([data[height_offset + 8], data[height_offset + 9],
                                                   data[height_offset + 10], data[height_offset + 11]])
                            } else {
                                u32::from_be_bytes([data[height_offset + 8], data[height_offset + 9],
                                                   data[height_offset + 10], data[height_offset + 11]])
                            };

                            if height > 0 && height < 1_000_000 {
                                return Ok(Size::new(value as f32, height as f32));
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: try basic dimensions at fixed offsets
    if data.len() >= 18 {
        let width = if is_le {
            u16::from_le_bytes([data[12], data[13]])
        } else {
            u16::from_be_bytes([data[12], data[13]])
        };
        let height = if is_le {
            u16::from_le_bytes([data[14], data[15]])
        } else {
            u16::from_be_bytes([data[14], data[15]])
        };

        if width > 0 && height > 0 {
            return Ok(Size::new(width as f32, height as f32));
        }
    }

    Err(ImageError::DecodeError("Failed to parse TIFF dimensions".to_string()))
}

// ============================================================================
// Image Loading from OOXML
// ============================================================================

/// Load images from OOXML package relationships.
pub fn load_images_from_relationships(
    relationships: &[Relationship],
    parts: &HashMap<String, PackagePart>,
) -> HashMap<String, DocumentImage> {
    let mut images = HashMap::new();

    for rel in relationships {
        if rel.relationship_type == RelationshipType::Image {
            // Extract image ID from relationship
            let image_id = rel.id.clone();
            let image_path = rel.target.clone();

            // Determine content type from path extension
            let _content_type = match image_path.rsplit('.').next() {
                Some("png") => ContentType::ImagePng,
                Some("jpg") | Some("jpeg") => ContentType::ImageJpeg,
                Some("gif") => ContentType::ImageGif,
                Some("bmp") => ContentType::ImageBmp,
                Some("webp") => ContentType::ImageWebP,
                Some("svg") => ContentType::ImageSvg,
                Some("tiff") | Some("tif") => ContentType::ImageTiff,
                _ => ContentType::Unknown(image_path.clone()),
            };

            // Get image data if present in package
            let image_data = if !image_path.starts_with("http") {
                parts.get(&image_path).map(|part| part.data.clone())
            } else {
                None // Linked image, not embedded
            };

            let doc_image = DocumentImage {
                id: image_id.clone(),
                path: image_path.clone(),
                original_width: None,
                original_height: None,
                desired_width: None,
                desired_height: None,
                scale_x: None,
                scale_y: None,
                title: None,
                alt_description: None,
                is_linked: image_data.is_none(),
            };

            images.insert(image_id.clone(), doc_image);

            debug!("Found image reference: {} ({}), linked: {}",
                image_id, image_path, image_data.is_none());
        }
    }

    images
}

// ============================================================================
// Default Image Cache (Global)
// ============================================================================

/// Global default image cache.
pub static DEFAULT_IMAGE_CACHE: Lazy<ImageCache> = Lazy::new(ImageCache::new);

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_mode_exact() {
        let mode = ScaleMode::Exact(WidthHeight::new(100.0, 200.0));
        assert_eq!(mode, ScaleMode::Exact(WidthHeight::new(100.0, 200.0)));
    }

    #[test]
    fn test_scale_mode_percentage() {
        let mode = ScaleMode::Percentage(0.5);
        if let ScaleMode::Percentage(pct) = mode {
            assert!((pct - 0.5).abs() < 0.001);
        } else {
            panic!("Expected Percentage variant");
        }
    }

    #[test]
    fn test_width_height_aspect_ratio() {
        let wh = WidthHeight::new(100.0, 50.0);
        assert!((wh.aspect_ratio() - 2.0).abs() < 0.001);

        let zero_h = WidthHeight::new(100.0, 0.0);
        assert!((zero_h.aspect_ratio() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_size_scale_to_fit() {
        let size = Size::new(100.0, 50.0);

        // Fit to wider container
        let fitted = size.scale_to_fit(200.0, 100.0);
        assert!((fitted.width - 200.0).abs() < 0.001);
        assert!((fitted.height - 100.0).abs() < 0.001);

        // Fit to taller container
        let fitted = size.scale_to_fit(100.0, 200.0);
        assert!((fitted.width - 100.0).abs() < 0.001);
        assert!((fitted.height - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_size_scale_to_fill() {
        let size = Size::new(50.0, 100.0);

        // Fill wider container
        let filled = size.scale_to_fill(200.0, 100.0);
        assert!((filled.width - 100.0).abs() < 0.001);
        assert!((filled.height - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_size_invalid() {
        let size = Size::new(0.0, 100.0);
        assert!(!size.is_valid());

        let size = Size::new(-10.0, 100.0);
        assert!(!size.is_valid());

        let size = Size::new(100.0, 100.0);
        assert!(size.is_valid());
    }

    #[test]
    fn test_wrap_distance() {
        let distance = WrapDistance::uniform(10.0);
        assert_eq!(distance.top, 10.0);
        assert_eq!(distance.bottom, 10.0);
        assert_eq!(distance.left, 10.0);
        assert_eq!(distance.right, 10.0);
        assert_eq!(distance.horizontal_total(), 20.0);
        assert_eq!(distance.vertical_total(), 20.0);
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let rect2 = Rect::new(50.0, 50.0, 100.0, 100.0);

        assert!(rect1.intersects(&rect2));
        assert!(rect2.intersects(&rect1));

        let rect3 = Rect::new(200.0, 200.0, 100.0, 100.0);
        assert!(!rect1.intersects(&rect3));
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(rect.contains(Point::new(50.0, 50.0)));
        assert!(!rect.contains(Point::new(100.0, 100.0)));
        assert!(!rect.contains(Point::new(-1.0, 50.0)));
    }

    #[test]
    fn test_image_format_magic_bytes() {
        // PNG
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(ImageFormat::from_magic_bytes(&png_data), ImageFormat::Png);

        // JPEG
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(ImageFormat::from_magic_bytes(&jpeg_data), ImageFormat::Jpeg);

        // GIF87a
        let gif_data = b"GIF87a";
        assert_eq!(ImageFormat::from_magic_bytes(gif_data), ImageFormat::Gif);

        // BMP
        let bmp_data = vec![0x42, 0x4D, 0x00, 0x00];
        assert_eq!(ImageFormat::from_magic_bytes(&bmp_data), ImageFormat::Bmp);

        // Unknown
        let unknown_data = vec![0x00, 0x00, 0x00, 0x00];
        assert_eq!(ImageFormat::from_magic_bytes(&unknown_data), ImageFormat::Unknown);
    }

    #[test]
    fn test_image_format_mime_type() {
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        assert_eq!(ImageFormat::Gif.mime_type(), "image/gif");
        assert_eq!(ImageFormat::Bmp.mime_type(), "image/bmp");
        assert_eq!(ImageFormat::WebP.mime_type(), "image/webp");
        assert_eq!(ImageFormat::Svg.mime_type(), "image/svg+xml");
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Gif.extension(), "gif");
        assert_eq!(ImageFormat::Bmp.extension(), "bmp");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
        assert_eq!(ImageFormat::Svg.extension(), "svg");
    }

    #[test]
    fn test_image_cache() {
        let mut cache = ImageCache::with_max_size(1024);

        // Load an image
        let data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D];
        let result = cache.load("test.png".to_string(), data);
        assert!(result.is_ok());

        // Check cache
        assert!(cache.contains("test.png"));
        assert_eq!(cache.len(), 1);
        assert!(cache.get("test.png").is_some());

        // Remove
        cache.remove("test.png");
        assert!(!cache.contains("test.png"));
        assert_eq!(cache.len(), 0);

        // Clear
        let _ = cache.load("test2.png".to_string(), vec![0x89, 0x50, 0x4E]);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_calculate_image_size() {
        let source = Size::new(100.0, 200.0);

        // None mode
        let result = calculate_image_size(source, ScaleMode::None, None);
        assert_eq!(result.width, 100.0);
        assert_eq!(result.height, 200.0);

        // Exact mode
        let result = calculate_image_size(source, ScaleMode::Exact(WidthHeight::new(50.0, 100.0)), None);
        assert_eq!(result.width, 50.0);
        assert_eq!(result.height, 100.0);

        // Percentage mode
        let result = calculate_image_size(source, ScaleMode::Percentage(0.5), None);
        assert!((result.width - 50.0).abs() < 0.001);
        assert!((result.height - 100.0).abs() < 0.001);

        // FitToContainer mode
        let result = calculate_image_size(source, ScaleMode::FitToContainer, Some(Size::new(25.0, 25.0)));
        assert!((result.width - 12.5).abs() < 0.001);
        assert!((result.height - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_scale_calculate() {
        let source = Size::new(100.0, 200.0);
        let desired = Size::new(50.0, 100.0);

        let scale = calculate_scale(source, desired);
        assert!((scale.x - 0.5).abs() < 0.001);
        assert!((scale.y - 0.5).abs() < 0.001);

        let scaled = scale.apply(source);
        assert!((scaled.width - 50.0).abs() < 0.001);
        assert!((scaled.height - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_wrap_polygon() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let distance = WrapDistance::uniform(10.0);

        let polygon = WrapPolygon::from_rect(rect, distance);
        assert!(polygon.is_valid);
        assert_eq!(polygon.points.len(), 5);

        // First and last points should match
        assert_eq!(polygon.points[0], polygon.points[4]);
    }

    #[test]
    fn test_wrap_type_properties() {
        assert!(WrapType::Behind.is_behind_text());
        assert!(!WrapType::InFront.is_behind_text());
        assert!(WrapType::InFront.is_in_front_of_text());
        assert!(!WrapType::Square.is_in_front_of_text());
        assert!(!WrapType::Tight.requires_wrap_polygon());
        assert!(WrapType::Through.requires_wrap_polygon());
    }

    #[test]
    fn test_relationship_type_detection() {
        let rel_type = RelationshipType::from_string(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
        );
        assert_eq!(rel_type, RelationshipType::Image);
        assert!(rel_type.is_image());

        let other_type = RelationshipType::from_string(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles"
        );
        assert_eq!(other_type, RelationshipType::Styles);
        assert!(!other_type.is_image());
    }

    #[test]
    fn test_content_type_image_detection() {
        let png = ContentType::from_string("image/png");
        assert!(png.is_image());
        assert_eq!(png, ContentType::ImagePng);

        let jpeg = ContentType::from_string("image/jpeg");
        assert!(jpeg.is_image());
        assert_eq!(jpeg, ContentType::ImageJpeg);

        let svg = ContentType::from_string("image/svg+xml");
        assert!(svg.is_image());
        assert_eq!(svg, ContentType::ImageSvg);

        let doc = ContentType::from_string("application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml");
        assert!(!doc.is_image());
    }

    #[test]
    fn test_rendered_image_bounding_rect() {
        let image = RenderedImage {
            image_id: "test".to_string(),
            position: Point::new(10.0, 20.0),
            size: Size::new(100.0, 50.0),
            ..RenderedImage::default()
        };

        let rect = image.bounding_rect();
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_point_translate() {
        let point = Point::new(10.0, 20.0);
        let translated = point.translate(5.0, 10.0);
        assert_eq!(translated.x, 15.0);
        assert_eq!(translated.y, 30.0);
    }

    #[test]
    fn test_scale_from_dimensions() {
        let original = Size::new(200.0, 100.0);
        let desired = Size::new(100.0, 100.0);

        let scale = Scale::from_dimensions(original, desired);
        assert!((scale.x - 0.5).abs() < 0.001);
        assert!((scale.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_decode_png_dimensions() {
        // Create minimal PNG with 100x200 dimensions
        let mut png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        ];
        // IHDR chunk
        png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]); // length
        png_data.extend_from_slice(b"IHDR");
        png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x64]); // width: 100
        png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0xC8]); // height: 200
        png_data.extend_from_slice(&[0x08, 0x02, 0x00, 0x00, 0x00]); // rest of IHDR
        // CRC (placeholder)
        png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let dimensions = decode_png_dimensions(&png_data).unwrap();
        assert_eq!(dimensions.width, 100.0);
        assert_eq!(dimensions.height, 200.0);
    }

    #[test]
    fn test_decode_gif_dimensions() {
        // GIF89a header with 320x240 dimensions
        let gif_data = b"GIF89a\x40\x01\xF0\x00\x00\x00";

        let dimensions = decode_gif_dimensions(gif_data).unwrap();
        assert_eq!(dimensions.width, 320.0);
        assert_eq!(dimensions.height, 240.0);
    }

    #[test]
    fn test_calculate_wrap_region() {
        let image = RenderedImage {
            image_id: "test".to_string(),
            position: Point::new(0.0, 0.0),
            size: Size::new(100.0, 100.0),
            wrap_type: Some(WrapType::Square),
            wrap_distance: Some(WrapDistance::uniform(5.0)),
            ..RenderedImage::default()
        };

        let region = calculate_wrap_region(&image);
        assert!(region.is_valid);
    }

    #[test]
    fn test_size_to_emu_conversion() {
        let size = Size::new(100.0, 200.0);
        let (w, h) = size.to_emu();
        // Check conversion is reasonable (exact values depend on DPI assumption)
        assert!(w > 0);
        assert!(h > 0);
    }
}

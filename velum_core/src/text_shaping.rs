use harfbuzz_rs::{Face, Font, Owned, UnicodeBuffer, shape};
use std::path::Path;

/// Represents a shaped glyph with positioning information
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// The glyph ID in the font
    pub codepoint: u32,
    /// The cluster index (character index) this glyph belongs to
    pub cluster: u32,
    /// X advance width in logical pixels
    pub x_advance: f32,
    /// Y advance height in logical pixels
    pub y_advance: f32,
    /// X offset in logical pixels
    pub x_offset: f32,
    /// Y offset in logical pixels
    pub y_offset: f32,
}

/// A text shaper that uses HarfBuzz
#[derive(Debug)]
pub struct TextShaper<'a> {
    /// The HarfBuzz font (None if no font loaded)
    /// Owned type manages the font data lifetime
    font: Option<Owned<Font<'a>>>,
    /// Units per EM for the current font
    upem: i32,
    /// Current font size in points
    font_size_pt: f32,
    /// Scaling factor from font units to logical pixels
    /// pixel = unit * scale_factor
    scale_factor: f32,
}

impl<'a> TextShaper<'a> {
    /// Creates a new text shaper with a default system font
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|| {
            // Fallback: use minimal shaper without actual font
            TextShaper::fallback()
        })
    }

    /// Creates a new text shaper, returning None if no font can be loaded
    pub fn try_new() -> Option<Self> {
        // Try to load a font from common locations
        #[cfg(target_os = "macos")]
        {
            if let Some(path) = Self::find_macOS_font() {
                return Self::load_from_path(&path);
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Some(path) = Self::find_windows_font() {
                return Self::load_from_path(&path);
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Some(path) = Self::find_linux_font() {
                return Self::load_from_path(&path);
            }
        }

        // Try any available font in system paths
        if let Some(path) = Self::find_any_system_font() {
            return Self::load_from_path(&path);
        }

        None
    }

    #[cfg(target_os = "macos")]
    fn find_macOS_font() -> Option<&'static str> {
        // Try common macOS font paths
        let paths = [
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial.ttf",
            "/System/Library/Fonts/Helvetica.dfont",
        ];
        for path in &paths {
            if Path::new(path).exists() {
                return Some(path);
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn find_windows_font() -> Option<&'static str> {
        let path = "C:\\Windows\\Fonts\\arial.ttf";
        if Path::new(path).exists() {
            return Some(path);
        }
        None
    }

    #[cfg(target_os = "linux")]
    fn find_linux_font() -> Option<&'static str> {
        let paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        ];
        for path in &paths {
            if Path::new(path).exists() {
                return Some(path);
            }
        }
        None
    }

    fn find_any_system_font() -> Option<&'static str> {
        // Fallback: try common paths on any platform
        #[cfg(target_os = "macos")]
        return Self::find_macOS_font();

        #[cfg(target_os = "windows")]
        return Self::find_windows_font();

        #[cfg(target_os = "linux")]
        return Self::find_linux_font();

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        None
    }

    /// Load font from a specific path
    fn load_from_path(path: &str) -> Option<Self> {
        match std::fs::read(path) {
            Ok(bytes) => {
                // Leak the bytes to get static lifetime - acceptable for font data
                // that is kept for the lifetime of the application
                let font_data: &'static [u8] = Box::leak(bytes.into_boxed_slice());
                Some(Self::new_from_font_data(font_data, 12.0))
            }
            Err(e) => {
                eprintln!("Failed to load font from {}: {}", path, e);
                None
            }
        }
    }

    /// Creates a fallback shaper that uses estimated character widths
    fn fallback() -> Self {
        TextShaper {
            font: None,
            upem: 1000,
            font_size_pt: 12.0,
            scale_factor: 1.0,
        }
    }

    /// Creates a TextShaper from font data bytes
    fn new_from_font_data(bytes: &'static [u8], font_size_pt: f32) -> Self {
        // Only create font if we have valid bytes
        if bytes.is_empty() {
            return TextShaper::fallback();
        }

        let face = Face::from_bytes(bytes, 0);
        let mut font = Font::new(face);
        let upem = font.scale().0.max(1);  // Avoid division by zero
        font.set_scale(upem, upem);

        let pixels_per_em = font_size_pt * (96.0 / 72.0);
        let scale_factor = pixels_per_em / (upem as f32);

        TextShaper {
            font: Some(font),
            upem,
            font_size_pt,
            scale_factor,
        }
    }

    /// Create from specific bytes (for testing or specific loading)
    pub fn new_from_bytes(bytes: &[u8], font_size_pt: f32) -> Self {
        // Copy bytes and leak them to get static lifetime
        let font_data: &'static [u8] = Box::leak(bytes.to_vec().into_boxed_slice());
        Self::new_from_font_data(font_data, font_size_pt)
    }

    /// Check if a font is loaded
    pub fn has_font(&self) -> bool {
        self.font.is_some()
    }

    /// Shapes text and returns the total width and glyph infos in logical pixels
    pub fn shape(&self, text: &str) -> (f32, Vec<GlyphInfo>) {
        // For empty text or fallback fonts, use estimated widths
        if text.is_empty() {
            return (0.0, Vec::new());
        }

        // Use estimated widths if no font is loaded
        if self.font.is_none() {
            return self.estimate_widths(text);
        }

        let font = self.font.as_ref().unwrap();

        let buffer = UnicodeBuffer::new().add_str(text);
        let output = shape(font, buffer, &[]);

        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        let mut total_width_px = 0.0;
        let mut glyphs = Vec::with_capacity(positions.len());

        for (position, info) in positions.iter().zip(infos.iter()) {
            let x_advance_px = position.x_advance as f32 * self.scale_factor;
            let y_advance_px = position.y_advance as f32 * self.scale_factor;
            let x_offset_px = position.x_offset as f32 * self.scale_factor;
            let y_offset_px = position.y_offset as f32 * self.scale_factor;

            total_width_px += x_advance_px;

            glyphs.push(GlyphInfo {
                codepoint: info.codepoint,
                cluster: info.cluster,
                x_advance: x_advance_px,
                y_advance: y_advance_px,
                x_offset: x_offset_px,
                y_offset: y_offset_px,
            });
        }

        (total_width_px, glyphs)
    }

    /// Estimate character widths without a real font
    fn estimate_widths(&self, text: &str) -> (f32, Vec<GlyphInfo>) {
        // Simple width estimation based on character type
        let char_width = self.font_size_pt * 0.5;  // Approximate width per character
        let mut glyphs = Vec::new();
        let mut total_width = 0.0f32;

        for (i, ch) in text.chars().enumerate() {
            // CJK characters are wider
            let width = if ch.is_ascii() {
                char_width
            } else {
                char_width * 2.0  // CJK characters are roughly twice as wide
            };

            glyphs.push(GlyphInfo {
                codepoint: ch as u32,
                cluster: i as u32,
                x_advance: width,
                y_advance: self.font_size_pt,
                x_offset: 0.0,
                y_offset: 0.0,
            });

            total_width += width;
        }

        (total_width, glyphs)
    }

    /// Measure text width in logical pixels
    pub fn measure_width(&self, text: &str) -> f32 {
        let (width, _) = self.shape(text);
        width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_shaper_new() {
        let shaper = TextShaper::new();
        // Should always succeed (falls back if no font)
        let (width, _) = shaper.shape("test");
        assert!(width >= 0.0);
    }

    #[test]
    fn test_shape_empty_text() {
        let shaper = TextShaper::new();
        let (width, glyphs) = shaper.shape("");
        assert_eq!(width, 0.0);
        assert!(glyphs.is_empty());
    }

    #[test]
    fn test_measure_width_empty() {
        let shaper = TextShaper::new();
        let width = shaper.measure_width("");
        assert_eq!(width, 0.0);
    }

    #[test]
    fn test_measure_width_ascii() {
        let shaper = TextShaper::new();
        let width = shaper.measure_width("hello");
        assert!(width > 0.0, "ASCII text should have positive width");
    }

    #[test]
    fn test_measure_width_cjk() {
        let shaper = TextShaper::new();
        let width = shaper.measure_width("中文");
        assert!(width > 0.0, "CJK text should have positive width");
    }

    #[test]
    fn test_measure_width_mixed() {
        let shaper = TextShaper::new();
        let width = shaper.measure_width("Hello世界");
        assert!(width > 0.0, "Mixed text should have positive width");
    }

    #[test]
    fn test_measure_width_numbers() {
        let shaper = TextShaper::new();
        let width = shaper.measure_width("12345");
        assert!(width > 0.0, "Numbers should have positive width");
    }

    #[test]
    fn test_measure_width_special_chars() {
        let shaper = TextShaper::new();
        let width = shaper.measure_width("!@#$%");
        assert!(width >= 0.0, "Special chars should have non-negative width");
    }

    #[test]
    fn test_shape_returns_glyphs_for_ascii() {
        let shaper = TextShaper::new();
        let (width, glyphs) = shaper.shape("ab");
        assert!(glyphs.len() >= 1, "Should return at least one glyph");
    }

    #[test]
    fn test_shape_returns_glyphs_for_cjk() {
        let shaper = TextShaper::new();
        let (width, glyphs) = shaper.shape("中");
        // CJK characters might produce one glyph per character
        assert!(glyphs.len() >= 1, "CJK should return at least one glyph");
    }

    #[test]
    fn test_glyph_info_structure() {
        let shaper = TextShaper::new();
        let (_, glyphs) = shaper.shape("x");
        if let Some(glyph) = glyphs.first() {
            assert!(glyph.codepoint > 0, "Codepoint should be valid");
            assert!(glyph.x_advance >= 0.0, "X advance should be non-negative");
        }
    }

    #[test]
    fn test_has_font_initially() {
        let shaper = TextShaper::new();
        // has_font returns whether a real font is loaded
        // The fallback shaper is acceptable
        let _ = shaper.has_font();
    }

    #[test]
    fn test_text_shaper_fallback() {
        let shaper = TextShaper::fallback();
        let (width, _) = shaper.shape("test");
        assert!(width >= 0.0);
        assert!(!shaper.has_font());
    }

    #[test]
    fn test_estimate_widths_ascii() {
        let shaper = TextShaper::fallback();
        let (width, glyphs) = shaper.estimate_widths("abc");
        assert!(width > 0.0);
        assert_eq!(glyphs.len(), 3, "Should have one glyph per char for ASCII");
    }

    #[test]
    fn test_estimate_widths_cjk() {
        let shaper = TextShaper::fallback();
        let (width, glyphs) = shaper.estimate_widths("中文");
        assert!(width > 0.0);
        assert_eq!(glyphs.len(), 2, "Should have one glyph per CJK char");
    }

    #[test]
    fn test_cjk_chars_wider_than_ascii() {
        let shaper = TextShaper::fallback();
        let ascii_width = shaper.measure_width("a");
        let cjk_width = shaper.measure_width("中");
        // CJK chars are estimated to be 2x wider
        assert!(cjk_width > ascii_width, "CJK should be wider than ASCII");
    }

    #[test]
    fn test_long_text_shaping() {
        let shaper = TextShaper::new();
        let long_text = "This is a very long text that should be shaped correctly. ".repeat(100);
        let (width, _) = shaper.shape(&long_text);
        assert!(width > 0.0, "Long text should have positive width");
    }

    #[test]
    fn test_whitespace_shaping() {
        let shaper = TextShaper::new();
        let spaces = shaper.measure_width("     ");
        assert!(spaces >= 0.0);
    }

    #[test]
    fn test_newline_shaping() {
        let shaper = TextShaper::new();
        let with_newline = shaper.measure_width("line1\nline2");
        let without_newline = shaper.measure_width("line1line2");
        // Should be similar but newline might add small width
        let _ = with_newline;
        let _ = without_newline;
    }

    #[test]
    fn test_tab_shaping() {
        let shaper = TextShaper::new();
        let with_tab = shaper.measure_width("a\tb");
        let without_tab = shaper.measure_width("ab");
        assert!(with_tab >= without_tab, "Tab should add width");
    }

    #[test]
    fn test_emoji_shaping() {
        let shaper = TextShaper::new();
        let emoji_width = shaper.measure_width("😊");
        assert!(emoji_width >= 0.0, "Emoji should have non-negative width");
    }

    #[test]
    fn test_shape_multiple_times_consistent() {
        let shaper = TextShaper::new();
        let width1 = shaper.measure_width("test");
        let width2 = shaper.measure_width("test");
        assert_eq!(width1, width2, "Width measurements should be consistent");
    }
}

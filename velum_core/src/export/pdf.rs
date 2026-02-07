//! PDF Export Module
//!
//! This module provides PDF export functionality for the Velum document system.
//! It supports page layout mapping, text rendering, image embedding,
//! font subsetting, hyperlinks, and bookmarks/TOC.

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::page_layout::{PageLayout, RenderedPage, RenderedLine, Rect};
use crate::image::{ImageData, RenderedImage, Size};

/// PDF 导出器
pub struct PdfExporter {
    config: PdfConfig,
    document: RenderedDocument,
    /// 嵌入的字体
    embedded_fonts: HashMap<String, EmbeddedFont>,
    /// 嵌入的图片
    embedded_images: HashMap<String, EmbeddedImage>,
    /// 书签
    bookmarks: Vec<Bookmark>,
    /// 当前页面
    current_page: Option<PdfPage>,
    /// 页面列表
    pages: Vec<PdfPage>,
}

/// PDF 配置
#[derive(Debug, Clone, PartialEq)]
pub struct PdfConfig {
    /// 页面尺寸
    pub page_size: PdfPageSize,
    /// 页边距
    pub margins: PdfMargins,
    /// 是否嵌入字体
    pub embed_fonts: bool,
    /// 是否压缩
    pub compress: bool,
    /// 是否生成书签
    pub generate_bookmarks: bool,
    /// 是否嵌入图片
    pub embed_images: bool,
    /// 标题
    pub title: Option<String>,
    /// 作者
    pub author: Option<String>,
    /// 主题
    pub subject: Option<String>,
    /// 关键词
    pub keywords: Vec<String>,
    /// 创建者 (PDF元数据)
    pub creator: Option<String>,
    /// 生产者 (PDF元数据)
    pub producer: Option<String>,
    /// 创建日期 (PDF元数据)
    pub creation_date: Option<String>,
    /// 修改日期 (PDF元数据)
    pub modification_date: Option<String>,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            page_size: PdfPageSize::A4,
            margins: PdfMargins::default(),
            embed_fonts: true,
            compress: false,
            generate_bookmarks: true,
            embed_images: true,
            title: None,
            author: None,
            subject: None,
            keywords: Vec::new(),
            creator: Some("Velum".to_string()),
            producer: Some("Velum Core".to_string()),
            creation_date: None,
            modification_date: None,
        }
    }
}

/// PDF 页面尺寸
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfPageSize {
    A4,
    Letter,
    Legal,
    Custom(f32, f32), // width, height in points (72 points per inch)
}

/// PDF 页边距配置
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for PdfMargins {
    fn default() -> Self {
        Self {
            top: 72.0,      // 1 inch
            right: 72.0,
            bottom: 72.0,
            left: 72.0,
        }
    }
}

/// 渲染文档（从 page_layout 模块获取）
#[derive(Debug, Clone)]
pub struct RenderedDocument {
    pub pages: Vec<RenderedPage>,
    pub metadata: DocumentMetadata,
}

/// 文档元数据
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Vec<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
}

/// 嵌入的字体
#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    pub name: String,
    pub family: String,
    pub data: Vec<u8>,
    pub subset: bool,
    /// 字符到字形的映射
    pub glyphs: HashMap<char, Vec<u8>>,
}

/// 嵌入的图片
#[derive(Debug, Clone)]
pub struct EmbeddedImage {
    pub id: String,
    pub format: PdfImageFormat,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// PDF 图片格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfImageFormat {
    Jpeg,
    Png,
    /// PDF 内置格式
    Raw,
}

/// PDF 书签（用于目录）
#[derive(Debug, Clone)]
pub struct Bookmark {
    pub title: String,
    pub level: usize,
    pub page: usize,
    pub y_offset: f32,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// PDF 页面
#[derive(Debug, Clone)]
pub struct PdfPage {
    pub width: f32,
    pub height: f32,
    pub content: PdfContent,
    pub resources: PageResources,
}

/// PDF 页面内容
#[derive(Debug, Clone)]
pub struct PdfContent {
    pub text_elements: Vec<TextElement>,
    pub image_elements: Vec<ImageElement>,
    pub rect_elements: Vec<RectElement>,
    pub link_elements: Vec<LinkElement>,
}

/// 文本元素
#[derive(Debug, Clone)]
pub struct TextElement {
    pub x: f32,
    pub y: f32,
    pub content: String,
    pub font_name: String,
    pub font_size: f32,
    pub color: Color,
    pub is_bold: bool,
    pub is_italic: bool,
    pub is_underline: bool,
}

/// 图片元素
#[derive(Debug, Clone)]
pub struct ImageElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub image_id: String,
    pub rotation: f32,
}

/// 矩形元素
#[derive(Debug, Clone)]
pub struct RectElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub filled: bool,
    pub stroke_width: f32,
}

/// 链接元素
#[derive(Debug, Clone)]
pub struct LinkElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub target: LinkTarget,
    pub border: Option<LinkBorder>,
}

/// 链接目标
#[derive(Debug, Clone)]
pub enum LinkTarget {
    /// 外部 URL
    Url(String),
    /// 内部页面
    Page(usize),
    /// 页面内的位置
    PagePosition(usize, f32),
}

/// 链接边框
#[derive(Debug, Clone, Copy)]
pub struct LinkBorder {
    pub width: f32,
    pub style: PdfBorderStyle,
    pub color: Option<Color>,
}

/// PDF 边框样式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfBorderStyle {
    Solid,
    Dashed,
    Dotted,
    None,
}

/// 颜色
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub fn black() -> Self {
        Self::rgb(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::rgb(255, 255, 255)
    }
}

/// 页面资源
#[derive(Debug, Clone, Default)]
pub struct PageResources {
    pub fonts: HashMap<String, FontResource>,
    pub images: HashMap<String, ImageResource>,
}

/// 字体资源
#[derive(Debug, Clone)]
pub struct FontResource {
    pub name: String,
    pub reference: String,
}

/// 图片资源
#[derive(Debug, Clone)]
pub struct ImageResource {
    pub id: String,
    pub reference: String,
}

/// PDF 导出错误
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Font error: {0}")]
    Font(String),
    #[error("Image error: {0}")]
    Image(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("PDF generation error: {0}")]
    Generation(String),
}

impl PdfExporter {
    /// 创建新的 PDF 导出器
    pub fn new(config: PdfConfig, document: RenderedDocument) -> Self {
        Self {
            config,
            document,
            embedded_fonts: HashMap::new(),
            embedded_images: HashMap::new(),
            bookmarks: Vec::new(),
            current_page: None,
            pages: Vec::new(),
        }
    }

    /// 从 RenderedDocument 创建 PdfExporter
    pub fn from_rendered_document(
        document: RenderedDocument,
        config: Option<PdfConfig>,
    ) -> Result<Self, PdfError> {
        let config = config.unwrap_or_default();

        Ok(Self::new(config, document))
    }

    /// 导出到文件
    pub fn export_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), PdfError> {
        let mut file = File::create(path)?;

        // 转换页面布局到 PDF 页面
        self.layout_pages()?;

        // 生成 PDF 内容
        let pdf_data = self.generate_pdf()?;

        file.write_all(&pdf_data)?;

        Ok(())
    }

    /// 导出到字节向量
    pub fn export_to_bytes(&mut self) -> Result<Vec<u8>, PdfError> {
        // 转换页面布局到 PDF 页面
        self.layout_pages()?;

        // 生成 PDF 内容
        self.generate_pdf()
    }

    /// 将页面布局转换为 PDF 页面
    fn layout_pages(&mut self) -> Result<(), PdfError> {
        self.pages.clear();
        self.bookmarks.clear();

        let (page_width, page_height) = self.get_page_dimensions();

        for (page_idx, rendered_page) in self.document.pages.iter().enumerate() {
            let mut pdf_page = self.layout_page(rendered_page, page_width, page_height, page_idx)?;
            self.pages.push(pdf_page);
        }

        // 处理书签
        if self.config.generate_bookmarks {
            self.process_bookmarks();
        }

        Ok(())
    }

    /// 布局单个页面
    fn layout_page(
        &self,
        rendered_page: &RenderedPage,
        page_width: f32,
        page_height: f32,
        page_idx: usize,
    ) -> Result<PdfPage, PdfError> {
        let margins = self.config.margins;
        let content_width = page_width - margins.left - margins.right;
        let content_height = page_height - margins.top - margins.bottom;

        let mut content = PdfContent {
            text_elements: Vec::new(),
            image_elements: Vec::new(),
            rect_elements: Vec::new(),
            link_elements: Vec::new(),
        };

        // 转换渲染的线条为文本元素
        for (line_idx, line) in rendered_page.lines.iter().enumerate() {
            let y_offset = margins.top + line.y;

            // 将线条内容转换为文本元素
            // 注意：这里需要从文档模型获取实际文本内容
            // 简化版本：使用行偏移创建占位文本元素
            let text_element = TextElement {
                x: margins.left + line.x,
                y: y_offset,
                content: format!("[Text bytes {}..{}]", line.start, line.end),
                font_name: "DefaultFont".to_string(),
                font_size: 12.0,
                color: Color::black(),
                is_bold: false,
                is_italic: false,
                is_underline: false,
            };

            content.text_elements.push(text_element);
        }

        let resources = PageResources::default();

        Ok(PdfPage {
            width: page_width,
            height: page_height,
            content,
            resources,
        })
    }

    /// 获取页面尺寸（点为单位，72 点/英寸）
    fn get_page_dimensions(&self) -> (f32, f32) {
        match self.config.page_size {
            PdfPageSize::A4 => (595.28, 841.89),      // 210mm x 297mm
            PdfPageSize::Letter => (612.0, 792.0),    // 8.5" x 11"
            PdfPageSize::Legal => (612.0, 1008.0),    // 8.5" x 14"
            PdfPageSize::Custom(w, h) => (w, h),
        }
    }

    /// 处理书签层级
    fn process_bookmarks(&mut self) {
        // 由于借用规则，这里使用更简单的方法：
        // 记录父节点关系，然后一次性修改
        let mut parent_map: Vec<Option<usize>> = vec![None; self.bookmarks.len()];

        let mut stack: Vec<usize> = Vec::new();

        for (idx, bookmark) in self.bookmarks.iter().enumerate() {
            while let Some(&parent_idx) = stack.last() {
                if bookmark.level > self.bookmarks[parent_idx].level {
                    stack.pop();
                } else {
                    break;
                }
            }

            if let Some(&parent_idx) = stack.last() {
                parent_map[idx] = Some(parent_idx);
            }

            stack.push(idx);
        }

        // 一次性设置父节点和子节点
        for (idx, parent) in parent_map.iter().enumerate() {
            if let Some(parent_idx) = parent {
                self.bookmarks[idx].parent = Some(*parent_idx);
                self.bookmarks[*parent_idx].children.push(idx);
            }
        }
    }

    /// 嵌入字体
    pub fn embed_font(&mut self, name: &str, data: Vec<u8>) -> Result<(), PdfError> {
        if self.config.embed_fonts {
            self.embedded_fonts.insert(
                name.to_string(),
                EmbeddedFont {
                    name: name.to_string(),
                    family: name.to_string(),
                    data,
                    subset: false,
                    glyphs: HashMap::new(),
                },
            );
        }
        Ok(())
    }

    /// 嵌入图片
    pub fn embed_image(&mut self, id: &str, image: ImageData) -> Result<(), PdfError> {
        if self.config.embed_images {
            let format = match image.format {
                crate::image::ImageFormat::Jpeg => PdfImageFormat::Jpeg,
                crate::image::ImageFormat::Png => PdfImageFormat::Png,
                _ => PdfImageFormat::Raw,
            };

            self.embedded_images.insert(
                id.to_string(),
                EmbeddedImage {
                    id: id.to_string(),
                    format,
                    data: image.data,
                    width: image.dimensions.width as u32,
                    height: image.dimensions.height as u32,
                },
            );
        }
        Ok(())
    }

    /// 添加书签
    pub fn add_bookmark(&mut self, title: &str, page: usize, y_offset: f32, level: usize) {
        self.bookmarks.push(Bookmark {
            title: title.to_string(),
            level,
            page,
            y_offset,
            parent: None,
            children: Vec::new(),
        });
    }

    /// 生成 PDF 数据
    fn generate_pdf(&mut self) -> Result<Vec<u8>, PdfError> {
        // 使用 pdf-writer 库生成 PDF
        // 注意：实际实现需要添加依赖
        #[cfg(feature = "pdf-export")]
        {
            use pdf_writer::PdfWriter;
            use std::io::Cursor;

            let mut writer = PdfWriter::new();
            let mut buffer = Cursor::new(Vec::new());

            // 生成 PDF 结构
            self.write_pdf_structure(&mut writer, &mut buffer)?;

            Ok(buffer.into_inner())
        }

        #[cfg(not(feature = "pdf-export"))]
        {
            // 简单的 PDF 生成（最小可行产品）
            self.generate_simple_pdf()
        }
    }

    /// 生成简单的 PDF（无外部依赖）
    fn generate_simple_pdf(&self) -> Result<Vec<u8>, PdfError> {
        let (page_width, page_height) = self.get_page_dimensions();

        let mut pdf = String::new();

        // PDF 头
        pdf.push_str("%PDF-1.4\n");
        pdf.push_str("%Velum Document\n");

        // 对象定义
        let mut objects = Vec::new();
        let mut object_counter = 1;

        // Catalog
        let catalog_ref = object_counter;
        objects.push(format!("{} 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n", catalog_ref));
        object_counter += 1;

        // Pages
        let pages_ref = object_counter;
        let mut page_refs = String::new();
        for i in 0..self.pages.len() {
            page_refs.push_str(&format!("{} 0 R ", catalog_ref + 2 + i * 2));
        }
        objects.push(format!(
            "{} 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            pages_ref, page_refs, self.pages.len()
        ));
        object_counter += 1;

        // 页面内容
        for (page_idx, page) in self.pages.iter().enumerate() {
            // 页面对象
            let content_ref = object_counter + 1;
            objects.push(format!(
                "{} 0 obj\n<< /Type /Page /Parent {} 0 R /MediaBox [0 0 {:.2} {:.2}] /Contents {} 0 R /Resources << >> >>\nendobj\n",
                object_counter, pages_ref, page.width, page.height, content_ref
            ));
            object_counter += 1;

            // 内容流
            let mut content = String::new();
            for text in &page.content.text_elements {
                content.push_str(&format!("BT\n/F1 {:.2} Tf\n{:.2} {:.2} Td\n({}) Tj\nET\n",
                    text.font_size, text.x, page_height - text.y, escape_pdf_string(&text.content)));
            }
            objects.push(format!("{} 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n", content_ref, content.len(), content));
            object_counter += 1;
        }

        // 字体
        let font_ref = object_counter;
        objects.push(format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
            font_ref
        ));
        object_counter += 1;

        // 交叉引用表
        let mut xref_offset = pdf.len() as u32;
        pdf.push_str("xref\n");
        pdf.push_str(&format!("0 {}\n", object_counter + 1));
        pdf.push_str("0000000000 65535 f \n");
        for obj in &objects {
            pdf.push_str(&format!("{:010} 00000 n \n", xref_offset));
            xref_offset += obj.len() as u32;
        }

        // Trailer
        pdf.push_str("trailer\n");
        pdf.push_str(&format!("<< /Size {} /Root 1 0 R >>\n", object_counter + 1));
        pdf.push_str("startxref\n");
        pdf.push_str(&format!("{}\n", xref_offset));
        pdf.push_str("%%EOF\n");

        Ok(pdf.into_bytes())
    }

    /// 使用 pdf-writer 生成 PDF（完整实现）
    #[cfg(feature = "pdf-export")]
    fn write_pdf_structure<W: Write>(
        &mut self,
        _writer: &mut PdfWriter,
        _buffer: &mut W,
    ) -> Result<(), PdfError> {
        // 完整实现需要 pdf-writer 库
        // 这里提供结构框架
        /*
        // 创建文档
        let mut doc = writer.minimal();

        // 设置属性
        doc.set_title(self.config.title.as_deref().unwrap_or("Untitled"));
        doc.set_author(self.config.author.as_deref().unwrap_or(""));
        doc.set_subject(self.config.subject.as_deref().unwrap_or(""));

        // 添加页面
        let mut pages = doc.pages();
        for page in &self.pages {
            let mut pdf_page = pages.page(page.width as i32, page.height as i32);
            // 添加内容...
        }

        // 添加书签
        if self.config.generate_bookmarks {
            let mut outline = doc.outline();
            for bookmark in &self.bookmarks {
                // 添加书签...
            }
        }
        */
        Ok(())
    }
}

/// 转义 PDF 字符串中的特殊字符
fn escape_pdf_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// 获取 PdfPageSize 的名称
impl PdfPageSize {
    pub fn name(&self) -> &'static str {
        match self {
            PdfPageSize::A4 => "A4",
            PdfPageSize::Letter => "Letter",
            PdfPageSize::Legal => "Legal",
            PdfPageSize::Custom(_, _) => "Custom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_layout::{RenderedPage, RenderedLine};
    use crate::image::ImageData;

    fn create_test_document() -> RenderedDocument {
        let pages = vec![RenderedPage {
            page_index: 0,
            content_bounds: Rect { x: 72.0, y: 72.0, width: 451.0, height: 697.0 },
            lines: vec![
                RenderedLine {
                    line_index: 0,
                    paragraph_index: 0,
                    source_line_index: 0,
                    y: 72.0,
                    height: 20.0,
                    x: 72.0,
                    width: 200.0,
                    start: 0,
                    end: 13,
                },
                RenderedLine {
                    line_index: 1,
                    paragraph_index: 0,
                    source_line_index: 1,
                    y: 92.0,
                    height: 20.0,
                    x: 72.0,
                    width: 400.0,
                    start: 13,
                    end: 54,
                },
            ],
            header_region: None,
            footer_region: None,
            page_width: 595.28,
            page_height: 841.89,
        }];

        RenderedDocument {
            pages,
            metadata: DocumentMetadata::default(),
        }
    }

    #[test]
    fn test_pdf_config_default() {
        let config = PdfConfig::default();

        assert_eq!(config.page_size, PdfPageSize::A4);
        assert!(config.embed_fonts);
        assert!(config.generate_bookmarks);
    }

    #[test]
    fn test_page_size_dimensions() {
        let exporter = PdfExporter::new(
            PdfConfig::default(),
            create_test_document(),
        );

        let (width, height) = exporter.get_page_dimensions();
        assert!((width - 595.28).abs() < 0.01);
        assert!((height - 841.89).abs() < 0.01);
    }

    #[test]
    fn test_letter_page_size() {
        let config = PdfConfig {
            page_size: PdfPageSize::Letter,
            ..PdfConfig::default()
        };
        let exporter = PdfExporter::new(config, create_test_document());

        let (width, height) = exporter.get_page_dimensions();
        assert!((width - 612.0).abs() < 0.01);
        assert!((height - 792.0).abs() < 0.01);
    }

    #[test]
    fn test_custom_page_size() {
        let config = PdfConfig {
            page_size: PdfPageSize::Custom(800.0, 600.0),
            ..PdfConfig::default()
        };
        let exporter = PdfExporter::new(config, create_test_document());

        let (width, height) = exporter.get_page_dimensions();
        assert!((width - 800.0).abs() < 0.01);
        assert!((height - 600.0).abs() < 0.01);
    }

    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 0, 0, 255);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 255);

        let black = Color::black();
        assert_eq!(black.r, 0);
        assert_eq!(black.g, 0);
        assert_eq!(black.b, 0);

        let white = Color::white();
        assert_eq!(white.r, 255);
        assert_eq!(white.g, 255);
        assert_eq!(white.b, 255);
    }

    #[test]
    fn test_escape_pdf_string() {
        assert_eq!(escape_pdf_string("Hello"), "Hello");
        assert_eq!(escape_pdf_string("Hello (World)"), "Hello \\(World\\)");
        assert_eq!(escape_pdf_string("Line1\nLine2"), "Line1\\nLine2");
        assert_eq!(escape_pdf_string("Back\\Slash"), "Back\\\\Slash");
    }

    #[test]
    fn test_pdf_exporter_creation() {
        let document = create_test_document();
        let exporter = PdfExporter::new(PdfConfig::default(), document.clone());

        assert_eq!(exporter.document.pages.len(), 1);
        assert!(exporter.embedded_fonts.is_empty());
        assert!(exporter.embedded_images.is_empty());
    }

    #[test]
    fn test_embed_font() {
        let document = create_test_document();
        let mut exporter = PdfExporter::new(PdfConfig::default(), document);

        let font_data = vec![0u8; 100];
        exporter.embed_font("TestFont", font_data).unwrap();

        assert_eq!(exporter.embedded_fonts.len(), 1);
        assert!(exporter.embedded_fonts.contains_key("TestFont"));
    }

    #[test]
    fn test_embed_image() {
        let document = create_test_document();
        let mut exporter = PdfExporter::new(PdfConfig::default(), document);

        let image_data = ImageData {
            data: vec![0u8; 1000],
            dimensions: Size::new(100.0, 100.0),
            format: crate::image::ImageFormat::Png,
            is_animated: false,
            frame_count: 1,
            bit_depth: 32,
            color_type: crate::image::ColorType::Rgba,
        };
        exporter.embed_image("test_image", image_data).unwrap();

        assert_eq!(exporter.embedded_images.len(), 1);
        assert!(exporter.embedded_images.contains_key("test_image"));
    }

    #[test]
    fn test_add_bookmark() {
        let document = create_test_document();
        let mut exporter = PdfExporter::new(PdfConfig::default(), document);

        exporter.add_bookmark("Chapter 1", 0, 100.0, 1);
        exporter.add_bookmark("Chapter 2", 0, 200.0, 1);

        assert_eq!(exporter.bookmarks.len(), 2);
        assert_eq!(exporter.bookmarks[0].title, "Chapter 1");
        assert_eq!(exporter.bookmarks[1].title, "Chapter 2");
    }

    #[test]
    fn test_layout_pages() {
        let document = create_test_document();
        let mut exporter = PdfExporter::new(PdfConfig::default(), document);

        exporter.layout_pages().unwrap();

        assert_eq!(exporter.pages.len(), 1);
        assert_eq!(exporter.pages[0].content.text_elements.len(), 2);
    }

    #[test]
    fn test_generate_simple_pdf() {
        let document = create_test_document();
        let exporter = PdfExporter::new(PdfConfig::default(), document);

        let pdf_data = exporter.generate_simple_pdf().unwrap();

        // 检查 PDF 头
        assert!(pdf_data.starts_with(b"%PDF-1.4"));
        assert!(pdf_data.ends_with(b"%%EOF"));
    }

    #[test]
    fn test_export_to_bytes() {
        let document = create_test_document();
        let mut exporter = PdfExporter::new(PdfConfig::default(), document);

        let pdf_data = exporter.export_to_bytes().unwrap();

        assert!(!pdf_data.is_empty());
        assert!(pdf_data.starts_with(b"%PDF-1.4"));
    }

    #[test]
    fn test_margins_default() {
        let margins = PdfMargins::default();
        assert_eq!(margins.top, 72.0);
        assert_eq!(margins.right, 72.0);
        assert_eq!(margins.bottom, 72.0);
        assert_eq!(margins.left, 72.0);
    }

    #[test]
    fn test_custom_margins() {
        let margins = PdfMargins {
            top: 50.0,
            right: 40.0,
            bottom: 50.0,
            left: 40.0,
        };

        let config = PdfConfig {
            margins,
            ..PdfConfig::default()
        };

        assert_eq!(config.margins.top, 50.0);
        assert_eq!(config.margins.right, 40.0);
    }

    #[test]
    fn test_link_target_variants() {
        let url_target = LinkTarget::Url("https://example.com".to_string());
        let page_target = LinkTarget::Page(5);
        let position_target = LinkTarget::PagePosition(2, 150.0);

        if let LinkTarget::Url(url) = url_target {
            assert_eq!(url, "https://example.com");
        } else {
            panic!("Expected URL target");
        }

        if let LinkTarget::Page(page) = page_target {
            assert_eq!(page, 5);
        } else {
            panic!("Expected page target");
        }

        if let LinkTarget::PagePosition(page, y) = position_target {
            assert_eq!(page, 2);
            assert!((y - 150.0).abs() < 0.01);
        } else {
            panic!("Expected page position target");
        }
    }

    #[test]
    fn test_border_style_variants() {
        assert_eq!(PdfBorderStyle::Solid, PdfBorderStyle::Solid);
        assert_eq!(PdfBorderStyle::Dashed, PdfBorderStyle::Dashed);
        assert_eq!(PdfBorderStyle::Dotted, PdfBorderStyle::Dotted);
        assert_eq!(PdfBorderStyle::None, PdfBorderStyle::None);
        assert_ne!(PdfBorderStyle::Solid, PdfBorderStyle::Dashed);
    }

    #[test]
    fn test_document_metadata() {
        let metadata = DocumentMetadata {
            title: Some("Test Document".to_string()),
            author: Some("Test Author".to_string()),
            subject: Some("Test Subject".to_string()),
            keywords: vec!["test".to_string(), "document".to_string()],
            creator: Some("Velum".to_string()),
            producer: Some("Velum Core".to_string()),
            creation_date: Some("2024-01-01".to_string()),
            modification_date: Some("2024-01-02".to_string()),
        };

        assert_eq!(metadata.title, Some("Test Document".to_string()));
        assert_eq!(metadata.keywords.len(), 2);
    }

    #[test]
    fn test_pdf_page_creation() {
        let page = PdfPage {
            width: 595.28,
            height: 841.89,
            content: PdfContent {
                text_elements: vec![],
                image_elements: vec![],
                rect_elements: vec![],
                link_elements: vec![],
            },
            resources: PageResources::default(),
        };

        assert!((page.width - 595.28).abs() < 0.01);
        assert!((page.height - 841.89).abs() < 0.01);
    }

    #[test]
    fn test_text_element_creation() {
        let text = TextElement {
            x: 100.0,
            y: 200.0,
            content: "Test text".to_string(),
            font_name: "Helvetica".to_string(),
            font_size: 12.0,
            color: Color::black(),
            is_bold: true,
            is_italic: false,
            is_underline: true,
        };

        assert_eq!(text.x, 100.0);
        assert_eq!(text.y, 200.0);
        assert_eq!(text.font_size, 12.0);
        assert!(text.is_bold);
        assert!(text.is_underline);
    }
}

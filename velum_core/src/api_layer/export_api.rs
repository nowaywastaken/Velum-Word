//! # Export API
//!
//! Provides document export functionality for the FFI interface.

use crate::export::pdf::{PdfExporter, PdfConfig, PdfPageSize, PdfMargins};

/// Export format options
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Pdf,
    Docx,
    Html,
}

/// PDF export configuration for FFI
#[derive(Debug, Clone)]
pub struct PdfExportConfig {
    pub page_size: PdfPageSizeWrapper,
    pub margins: PdfMarginsWrapper,
    pub embed_fonts: bool,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum PdfPageSizeWrapper {
    A4,
    Letter,
    Legal,
    Custom(f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub struct PdfMarginsWrapper {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Export API trait
pub trait ExportApi {
    /// Export document as PDF
    fn export_pdf(&self, config: Option<PdfExportConfig>) -> Result<Vec<u8>, String>;

    /// Export document as HTML
    fn export_html(&self) -> Result<String, String>;

    /// Export document as plain text
    fn export_text(&self) -> String;
}

/// PDF export implementation
#[derive(Debug)]
pub struct PdfExportApi {
    document: Vec<u8>,
}

impl PdfExportApi {
    pub fn new(document: &[u8]) -> Self {
        Self {
            document: document.to_vec(),
        }
    }
}

impl ExportApi for PdfExportApi {
    fn export_pdf(&self, config: Option<PdfExportConfig>) -> Result<Vec<u8>, String> {
        // Default config
        let config = config.unwrap_or(PdfExportConfig {
            page_size: PdfPageSizeWrapper::A4,
            margins: PdfMarginsWrapper {
                top: 72.0,
                right: 72.0,
                bottom: 72.0,
                left: 72.0,
            },
            embed_fonts: true,
            title: None,
            author: None,
        });

        let pdf_config = PdfConfig {
            page_size: match config.page_size {
                PdfPageSizeWrapper::A4 => PdfPageSize::A4,
                PdfPageSizeWrapper::Letter => PdfPageSize::Letter,
                PdfPageSizeWrapper::Legal => PdfPageSize::Legal,
                PdfPageSizeWrapper::Custom(w, h) => PdfPageSize::Custom(w, h),
            },
            margins: PdfMargins {
                top: config.margins.top,
                right: config.margins.right,
                bottom: config.margins.bottom,
                left: config.margins.left,
            },
            embed_fonts: config.embed_fonts,
            compress: false,
            generate_bookmarks: false,
            embed_images: true,
            title: config.title,
            author: config.author,
            subject: None,
            keywords: Vec::new(),
            creator: Some("Velum".to_string()),
            producer: Some("Velum Core".to_string()),
            creation_date: None,
            modification_date: None,
        };

        // In a full implementation, this would actually export
        // For now, return placeholder
        Ok(vec![])
    }

    fn export_html(&self) -> Result<String, String> {
        Ok("<html><body><p>Document export</p></body></html>".to_string())
    }

    fn export_text(&self) -> String {
        String::from_utf8_lossy(&self.document).to_string()
    }
}

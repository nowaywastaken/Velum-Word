//! # Velum API Facade
//!
//! The main entry point for the Velum FFI interface.
//! Provides unified access to all Velum functionality.

use std::sync::{Arc, RwLock};
use crate::api_layer::document_api::{DocumentApi, PieceTreeDocumentApi, DocumentStats};
use crate::api_layer::layout_api::{LayoutApi, PageLayoutApi, ViewportInfo};
use crate::api_layer::render_api::{RenderApi, SimpleRenderApi};
use crate::api_layer::export_api::{ExportApi, PdfExportApi, ExportFormat};
use crate::piece_tree::PieceTree;
use crate::page_layout::{PageConfig, PageLayout};
use crate::export::pdf::PdfConfig;
use crate::find::SearchOptions;

/// The main Velum API struct - provides unified access to all functionality
pub struct VelumApi {
    document_api: Arc<dyn DocumentApi + Send + Sync>,
    layout_api: Arc<dyn LayoutApi + Send + Sync>,
    render_api: Arc<dyn RenderApi + Send + Sync>,
    export_api: Arc<dyn ExportApi + Send + Sync>,
}

impl VelumApi {
    /// Create a new VelumApi with default empty document
    pub fn new() -> Self {
        let document = Arc::new(RwLock::new(PieceTree::empty()));
        let page_config = PageConfig::default();
        let page_layout = PageLayout::new();

        Self::from_document(document, page_layout, page_config)
    }

    /// Create from existing document
    pub fn from_document(
        document: Arc<RwLock<PieceTree>>,
        page_layout: PageLayout,
        page_config: PageConfig,
    ) -> Self {
        let document_api = Arc::new(PieceTreeDocumentApi::new(document));
        let layout_api = Arc::new(PageLayoutApi::new(page_layout, page_config));
        let render_api = Arc::new(SimpleRenderApi::new(Vec::new()));
        let export_api = Arc::new(PdfExportApi::new(&[]));

        Self {
            document_api,
            layout_api,
            render_api,
            export_api,
        }
    }

    /// Get document API reference
    pub fn document(&self) -> &dyn DocumentApi {
        &*self.document_api
    }

    /// Get layout API reference
    pub fn layout(&self) -> &dyn LayoutApi {
        &*self.layout_api
    }

    /// Get render API reference
    pub fn render(&self) -> &dyn RenderApi {
        &*self.render_api
    }

    /// Get export API reference
    pub fn export(&self) -> &dyn ExportApi {
        &*self.export_api
    }
}

impl Default for VelumApi {
    fn default() -> Self {
        Self::new()
    }
}

// Flutter Rust Bridge generated code would go here
// The following types are used by the FFI bindings

/// Search options for find operations
#[derive(Debug, Clone, Default)]
pub struct FfiSearchOptions {
    pub query: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub wrap_around: bool,
    pub search_backward: bool,
}

impl From<FfiSearchOptions> for SearchOptions {
    fn from(opts: FfiSearchOptions) -> Self {
        SearchOptions {
            query: opts.query,
            replace: String::new(),
            case_sensitive: opts.case_sensitive,
            whole_word: opts.whole_word,
            regex: opts.regex,
            wrap_around: opts.wrap_around,
            search_backward: opts.search_backward,
        }
    }
}

/// Document statistics for FFI
#[derive(Debug, Clone, Default)]
pub struct FfiDocumentStats {
    pub char_count: usize,
    pub word_count: usize,
    pub line_count: usize,
}

impl From<DocumentStats> for FfiDocumentStats {
    fn from(stats: DocumentStats) -> Self {
        FfiDocumentStats {
            char_count: stats.char_count,
            word_count: stats.word_count,
            line_count: stats.line_count,
        }
    }
}

/// Cursor position for FFI
#[derive(Debug, Clone)]
pub struct FfiCursorPosition {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

/// Selection for FFI
#[derive(Debug, Clone)]
pub struct FfiSelection {
    pub start: usize,
    pub end: usize,
}

/// Page info for FFI
#[derive(Debug, Clone)]
pub struct FfiPageInfo {
    pub index: usize,
    pub width: f32,
    pub height: f32,
    pub line_count: usize,
}

/// Line info for FFI
#[derive(Debug, Clone)]
pub struct FfiLineInfo {
    pub page_index: usize,
    pub line_index: usize,
    pub y: f32,
    pub height: f32,
    pub start_offset: usize,
    pub end_offset: usize,
}

/// Dirty rect for FFI
#[derive(Debug, Clone)]
pub struct FfiDirtyRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub from_offset: usize,
    pub to_offset: usize,
}

/// Viewport for FFI
#[derive(Debug, Clone, Copy)]
pub struct FfiViewport {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

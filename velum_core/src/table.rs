//! # Table Module
//!
//! Provides comprehensive table support for document rendering including:
//! - Core table structures (Table, TableRow, TableCell, TableColumn)
//! - Table layout algorithms with column width calculation
//! - Grid computation for handling cell spanning
//! - Table rendering with borders and background colors

use crate::line_layout::{Alignment as LineLayoutAlignment, ParagraphLayout};
use crate::page_layout::Rect;
use crate::Alignment;
use serde::{Deserialize, Serialize};

/// Table width specification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TableWidth {
    /// Fixed width in points
    Fixed(f32),
    /// Auto width (distribute available space)
    Auto,
    /// Percentage of container width
    Percent(f32),
}

impl Default for TableWidth {
    fn default() -> Self {
        TableWidth::Auto
    }
}

/// Border style for table borders
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BorderStyle {
    /// No border
    None,
    /// Single solid border
    Single,
    /// Double solid border
    Double,
    /// Dotted border
    Dotted,
    /// Dashed border
    Dashed,
    /// Hairline border
    Hairline,
}

impl Default for BorderStyle {
    fn default() -> Self {
        BorderStyle::Single
    }
}

/// Individual border definition
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Border {
    pub width: f32,
    pub style: BorderStyle,
    pub color: u32,  // RGB color value
}

impl Default for Border {
    fn default() -> Self {
        Border {
            width: 1.0,
            style: BorderStyle::Single,
            color: 0x000000,  // Black
        }
    }
}

/// Table borders (all sides)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableBorders {
    pub top: Border,
    pub bottom: Border,
    pub left: Border,
    pub right: Border,
    pub horizontal_inside: Border,
    pub vertical_inside: Border,
}

impl Default for TableBorders {
    fn default() -> Self {
        TableBorders {
            top: Border::default(),
            bottom: Border::default(),
            left: Border::default(),
            right: Border::default(),
            horizontal_inside: Border::default(),
            vertical_inside: Border::default(),
        }
    }
}

/// Cell properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellProperties {
    /// Background color (RGB)
    pub background_color: Option<u32>,
    /// Text alignment within cell
    pub vertical_alignment: VerticalAlignment,
    /// Padding inside cell
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    /// Cell border override
    pub border_top: Option<Border>,
    pub border_bottom: Option<Border>,
    pub border_left: Option<Border>,
    pub border_right: Option<Border>,
}

impl Default for CellProperties {
    fn default() -> Self {
        CellProperties {
            background_color: None,
            vertical_alignment: VerticalAlignment::Top,
            padding_top: 2.0,
            padding_bottom: 2.0,
            padding_left: 2.0,
            padding_right: 2.0,
            border_top: None,
            border_bottom: None,
            border_left: None,
            border_right: None,
        }
    }
}

/// Vertical alignment for cell content
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

impl Default for VerticalAlignment {
    fn default() -> Self {
        VerticalAlignment::Top
    }
}

/// Table cell structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Column index where this cell starts
    pub column_index: usize,
    /// Row index where this cell starts
    pub row_index: usize,
    /// Number of columns this cell spans
    pub col_span: u32,
    /// Number of rows this cell spans
    pub row_span: u32,
    /// Content container (paragraph layouts)
    pub content: Vec<ParagraphLayout>,
    /// Cell-specific properties
    pub properties: CellProperties,
}

impl TableCell {
    /// Creates a new table cell with default properties
    pub fn new(column_index: usize, row_index: usize) -> Self {
        TableCell {
            column_index,
            row_index,
            col_span: 1,
            row_span: 1,
            content: Vec::new(),
            properties: CellProperties::default(),
        }
    }

    /// Sets the column span
    pub fn with_col_span(mut self, span: u32) -> Self {
        self.col_span = span.max(1);
        self
    }

    /// Sets the row span
    pub fn with_row_span(mut self, span: u32) -> Self {
        self.row_span = span.max(1);
        self
    }

    /// Gets the total number of columns this cell occupies
    pub fn span_columns(&self) -> usize {
        self.col_span as usize
    }

    /// Gets the total number of rows this cell occupies
    pub fn span_rows(&self) -> usize {
        self.row_span as usize
    }
}

/// Table row structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// Cells in this row
    pub cells: Vec<TableCell>,
    /// Row height (0 means auto)
    pub height: f32,
    /// Row-specific properties
    pub properties: RowProperties,
}

impl TableRow {
    /// Creates a new empty table row
    pub fn new() -> Self {
        TableRow {
            cells: Vec::new(),
            height: 0.0,
            properties: RowProperties::default(),
        }
    }

    /// Adds a cell to the row
    pub fn add_cell(&mut self, cell: TableCell) {
        self.cells.push(cell);
    }

    /// Gets the number of visible cells (accounting for colspan)
    pub fn visible_cell_count(&self) -> usize {
        self.cells.iter().map(|c| c.col_span as usize).sum()
    }
}

impl Default for TableRow {
    fn default() -> Self {
        TableRow::new()
    }
}

/// Row-specific properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowProperties {
    /// Row height rule
    pub height_rule: HeightRule,
    /// Row height value
    pub height_value: f32,
    /// Background color for the row
    pub background_color: Option<u32>,
}

impl Default for RowProperties {
    fn default() -> Self {
        RowProperties {
            height_rule: HeightRule::Auto,
            height_value: 0.0,
            background_color: None,
        }
    }
}

/// Row height rule
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HeightRule {
    /// Auto height based on content
    Auto,
    /// At least specified height
    AtLeast,
    /// Exactly specified height
    Exact,
    /// Multiple of baseline
    Multiple,
}

impl Default for HeightRule {
    fn default() -> Self {
        HeightRule::Auto
    }
}

/// Table column definition
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TableColumn {
    /// Column index (0-based)
    pub index: usize,
    /// Calculated width
    pub width: f32,
    /// Preferred/requested width
    pub preferred_width: f32,
    /// Width type
    pub width_type: WidthType,
}

impl TableColumn {
    /// Creates a new column with specified width
    pub fn new(index: usize, width: f32) -> Self {
        TableColumn {
            index,
            width,
            preferred_width: width,
            width_type: WidthType::Fixed,
        }
    }

    /// Creates an auto-width column
    pub fn auto(index: usize) -> Self {
        TableColumn {
            index,
            width: 0.0,
            preferred_width: 0.0,
            width_type: WidthType::Auto,
        }
    }
}

/// Column width type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WidthType {
    /// Fixed width in points
    Fixed,
    /// Auto width (distribute remaining space)
    Auto,
    /// Percentage of table width
    Percent,
}

/// Table properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableProperties {
    /// Table width
    pub width: TableWidth,
    /// Table alignment within container
    pub alignment: Alignment,
    /// Table borders
    pub borders: TableBorders,
    /// Cell margin (default padding)
    pub cell_margin: f32,
    /// Table indents
    pub indent: f32,
    /// Look-up (spacing between cells)
    pub look_up: f32,  // spacing between cells
    /// Table layout type
    pub layout_type: TableLayoutType,
}

impl Default for TableProperties {
    fn default() -> Self {
        TableProperties {
            width: TableWidth::Auto,
            alignment: crate::Alignment::Left,
            borders: TableBorders::default(),
            cell_margin: 2.0,
            indent: 0.0,
            look_up: 0.0,
            layout_type: TableLayoutType::AutoFit,
        }
    }
}

/// Table layout type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TableLayoutType {
    /// Fixed column widths
    Fixed,
    /// Auto-fit columns to content
    AutoFit,
    /// Auto-fit to window/table width
    AutoWidth,
}

/// Main table structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table rows
    pub rows: Vec<TableRow>,
    /// Column definitions
    pub columns: Vec<TableColumn>,
    /// Table properties
    pub properties: TableProperties,
    /// Calculated table width (after layout)
    pub calculated_width: f32,
    /// Calculated table height (after layout)
    pub calculated_height: f32,
}

impl Table {
    /// Creates a new empty table
    pub fn new() -> Self {
        Table {
            rows: Vec::new(),
            columns: Vec::new(),
            properties: TableProperties::default(),
            calculated_width: 0.0,
            calculated_height: 0.0,
        }
    }

    /// Adds a row to the table
    pub fn add_row(&mut self, row: TableRow) {
        self.rows.push(row);
    }

    /// Gets the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Gets the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

impl Default for Table {
    fn default() -> Self {
        Table::new()
    }
}

/// Grid cell for layout calculation
#[derive(Debug, Clone)]
pub struct GridCell {
    /// The table cell reference
    pub cell: *const TableCell,
    /// Whether this grid position is covered by a spanning cell
    pub is_covered: bool,
    /// Reference to the spanning cell that covers this position
    pub covering_cell: Option<*const TableCell>,
}

/// Grid for table layout
#[derive(Debug, Clone)]
pub struct TableGrid {
    /// 2D grid of cells (row, column)
    grid: Vec<Vec<GridCell>>,
    /// Number of rows
    row_count: usize,
    /// Number of columns
    column_count: usize,
    /// Row heights after calculation
    row_heights: Vec<f32>,
    /// Column widths after calculation
    column_widths: Vec<f32>,
}

impl TableGrid {
    /// Creates a new table grid from a table
    pub fn new(table: &Table) -> Self {
        let column_count = table.columns.len();
        let row_count = table.rows.len();

        // Initialize empty grid
        let mut grid = vec![vec![
            GridCell {
                cell: std::ptr::null(),
                is_covered: false,
                covering_cell: None,
            };
            column_count
        ];
        row_count];

        // Place cells in grid
        for (row_idx, row) in table.rows.iter().enumerate() {
            let mut col_idx = 0usize;
            for cell in &row.cells {
                // Skip if column index is already past grid bounds
                if col_idx >= column_count {
                    break;
                }

                // Find next available column
                while col_idx < column_count && grid[row_idx][col_idx].is_covered {
                    col_idx += 1;
                }

                if col_idx < column_count {
                    // Place cell reference
                    grid[row_idx][col_idx] = GridCell {
                        cell: cell as *const TableCell,
                        is_covered: false,
                        covering_cell: None,
                    };

                    // Mark spanned cells
                    for r_offset in 0..cell.row_span {
                        for c_offset in 0..cell.col_span {
                            let r = row_idx + r_offset as usize;
                            let c = col_idx + c_offset as usize;
                            if r < row_count && c < column_count {
                                if r_offset > 0 || c_offset > 0 {
                                    grid[r][c] = GridCell {
                                        cell: std::ptr::null(),
                                        is_covered: true,
                                        covering_cell: Some(cell as *const TableCell),
                                    };
                                }
                            }
                        }
                    }
                }

                // Move to next column after this cell
                col_idx += cell.col_span as usize;
            }
        }

        TableGrid {
            grid,
            row_count,
            column_count,
            row_heights: vec![0.0; row_count],
            column_widths: vec![0.0; column_count],
        }
    }

    /// Gets the cell at a grid position
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&TableCell> {
        if row < self.row_count && col < self.column_count {
            let grid_cell = &self.grid[row][col];
            if !grid_cell.is_covered && !grid_cell.cell.is_null() {
                unsafe { grid_cell.cell.as_ref() }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets the covering cell for a position
    pub fn get_covering_cell(&self, row: usize, col: usize) -> Option<&TableCell> {
        if row < self.row_count && col < self.column_count {
            let grid_cell = &self.grid[row][col];
            if let Some(ptr) = grid_cell.covering_cell {
                unsafe { ptr.as_ref() }
            } else {
                self.get_cell(row, col)
            }
        } else {
            None
        }
    }

    /// Checks if a position is covered
    pub fn is_covered(&self, row: usize, col: usize) -> bool {
        if row < self.row_count && col < self.column_count {
            self.grid[row][col].is_covered
        } else {
            true
        }
    }

    /// Sets the row height
    pub fn set_row_height(&mut self, row: usize, height: f32) {
        if row < self.row_count {
            self.row_heights[row] = height;
        }
    }

    /// Sets the column width
    pub fn set_column_width(&mut self, col: usize, width: f32) {
        if col < self.column_count {
            self.column_widths[col] = width;
        }
    }

    /// Gets row heights
    pub fn row_heights(&self) -> &[f32] {
        &self.row_heights
    }

    /// Gets column widths
    pub fn column_widths(&self) -> &[f32] {
        &self.column_widths
    }
}

/// Calculated cell position for rendering
#[derive(Debug, Clone)]
pub struct RenderedCell {
    /// Source cell reference
    pub cell: TableCell,
    /// Position in table
    pub row: usize,
    pub column: usize,
    /// Bounds of the cell
    pub bounds: Rect,
    /// Spanned columns
    pub colspan: u32,
    /// Spanned rows
    pub rowspan: u32,
    /// Rendered content bounds (inside borders)
    pub content_bounds: Rect,
}

/// Rendered table for display
#[derive(Debug, Clone)]
pub struct RenderedTable {
    /// Original table reference
    pub table: Table,
    /// Calculated grid
    pub grid: TableGrid,
    /// Rendered cells
    pub cells: Vec<RenderedCell>,
    /// Total table bounds
    pub bounds: Rect,
    /// Table borders
    pub borders: TableBorders,
}

impl RenderedTable {
    /// Creates a new rendered table from a table
    pub fn new(table: &Table, available_width: f32) -> Self {
        let mut rendered = RenderedTable {
            table: table.clone(),
            grid: TableGrid::new(table),
            cells: Vec::new(),
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            borders: table.properties.borders.clone(),
        };

        // Calculate column widths
        rendered.calculate_column_widths(available_width);

        // Calculate row heights based on content
        rendered.calculate_row_heights();

        // Calculate total bounds
        rendered.calculate_bounds();

        // Build rendered cells
        rendered.build_rendered_cells();

        rendered
    }

    /// Calculates column widths based on table properties
    fn calculate_column_widths(&mut self, available_width: f32) {
        let col_count = self.grid.column_count;
        let mut widths: Vec<f32> = vec![0.0; col_count];
        let mut preferred: Vec<f32> = vec![0.0; col_count];
        let mut auto_count = 0;
        let mut fixed_sum = 0.0;
        let mut percent_sum = 0.0;

        // First pass: collect column widths
        for (i, col) in self.table.columns.iter().enumerate() {
            match col.width_type {
                WidthType::Fixed => {
                    widths[i] = col.preferred_width.max(0.0);
                    preferred[i] = widths[i];
                    fixed_sum += widths[i];
                }
                WidthType::Auto => {
                    preferred[i] = 0.0;
                    auto_count += 1;
                }
                WidthType::Percent => {
                    preferred[i] = col.preferred_width;
                    percent_sum += col.preferred_width;
                }
            }
        }

        // Apply table width settings
        let table_width = match self.table.properties.width {
            TableWidth::Fixed(w) => w,
            TableWidth::Auto => available_width,
            TableWidth::Percent(p) => available_width * (p / 100.0),
        };

        // Account for borders and cell margins
        let border_width = self.borders.left.width + self.borders.right.width;
        let cell_margin_total = self.table.properties.cell_margin * 2.0 * col_count as f32;
        let content_width = table_width - border_width - cell_margin_total;

        // Adjust for fixed and percent widths
        let mut remaining_width = content_width;

        // Set fixed widths first
        for i in 0..col_count {
            if self.table.columns[i].width_type == WidthType::Fixed {
                widths[i] = self.table.columns[i].preferred_width;
                remaining_width -= widths[i];
            }
        }

        // Set percent widths
        if percent_sum > 0.0 {
            for i in 0..col_count {
                if self.table.columns[i].width_type == WidthType::Percent {
                    let percent = self.table.columns[i].preferred_width / percent_sum;
                    widths[i] = (content_width - fixed_sum) * percent;
                    remaining_width -= widths[i];
                }
            }
        }

        // Distribute remaining width to auto columns
        if auto_count > 0 && remaining_width > 0.0 {
            let per_auto = remaining_width / auto_count as f32;
            for i in 0..col_count {
                if self.table.columns[i].width_type == WidthType::Auto {
                    widths[i] = per_auto;
                }
            }
        }

        // Ensure minimum widths (at least 1 point per column)
        for width in widths.iter_mut() {
            if *width < 1.0 {
                *width = 1.0;
            }
        }

        // Store calculated widths
        for (i, &width) in widths.iter().enumerate() {
            self.grid.set_column_width(i, width);
        }
    }

    /// Calculates row heights based on content
    fn calculate_row_heights(&mut self) {
        let default_row_height = 20.0;  // Default row height in points

        for (row_idx, row) in self.table.rows.iter().enumerate() {
            let mut max_height = default_row_height;

            // Check each cell's content to determine height
            for cell in &row.cells {
                // Calculate content height for this cell
                let cell_content_height: f32 = cell
                    .content
                    .iter()
                    .map(|p| p.total_height)
                    .sum();

                // Account for cell padding
                let cell_height = cell_content_height
                    + cell.properties.padding_top
                    + cell.properties.padding_bottom;

                // For cells that span multiple rows, the height applies to the first row
                // The row will need to accommodate the maximum of all its cells
                max_height = f32::max(max_height, cell_height);
            }

            // Apply row height rule
            match row.properties.height_rule {
                HeightRule::Auto => {
                    self.grid.set_row_height(row_idx, max_height);
                }
                HeightRule::AtLeast => {
                    self.grid.set_row_height(row_idx, row.properties.height_value.max(max_height));
                }
                HeightRule::Exact => {
                    self.grid.set_row_height(row_idx, row.properties.height_value);
                }
                HeightRule::Multiple => {
                    let base_height = 12.0;  // Base line height
                    let computed = base_height * row.properties.height_value;
                    self.grid.set_row_height(row_idx, computed);
                }
            }
        }
    }

    /// Calculates total table bounds
    fn calculate_bounds(&mut self) {
        let total_width: f32 = self.grid.column_widths().iter().sum();
        let total_height: f32 = self.grid.row_heights().iter().sum();

        // Account for borders
        let border_left = self.borders.left.width;
        let border_top = self.borders.top.width;

        self.bounds = Rect::new(
            0.0,
            0.0,
            total_width + border_left + self.borders.right.width,
            total_height + border_top + self.borders.bottom.width,
        );
    }

    /// Builds rendered cells with positions
    fn build_rendered_cells(&mut self) {
        let border_left = self.borders.left.width;
        let border_top = self.borders.top.width;
        let cell_margin = self.table.properties.cell_margin;

        for row_idx in 0..self.grid.row_count {
            let mut y = border_top;

            // Accumulate y position from previous rows
            for r in 0..row_idx {
                y += self.grid.row_heights[r];
            }

            let _row_height = self.grid.row_heights[row_idx];

            let mut x = border_left;
            for col_idx in 0..self.grid.column_count {
                let col_width = self.grid.column_widths[col_idx];

                // Skip covered cells (they're part of a spanning cell)
                if self.grid.is_covered(row_idx, col_idx) {
                    continue;
                }

                if let Some(cell) = self.grid.get_cell(row_idx, col_idx) {
                    // Calculate cell bounds
                    let cell_width: f32 = (col_idx..col_idx + cell.col_span as usize)
                        .map(|i| self.grid.column_widths[i])
                        .sum();

                    let cell_height: f32 = (row_idx..row_idx + cell.row_span as usize)
                        .map(|i| self.grid.row_heights[i])
                        .sum();

                    let bounds = Rect::new(x, y, cell_width, cell_height);

                    // Calculate content bounds (inside borders and padding)
                    let content_bounds = Rect::new(
                        x + cell_margin,
                        y + cell_margin + cell.properties.padding_top,
                        cell_width - cell_margin * 2.0 - cell.properties.padding_left
                            - cell.properties.padding_right,
                        cell_height - cell_margin * 2.0 - cell.properties.padding_top
                            - cell.properties.padding_bottom,
                    );

                    // Adjust content height for vertical alignment
                    let actual_content_height: f32 = cell.content.iter().map(|p| p.total_height).sum();
                    let adjusted_content_y = match cell.properties.vertical_alignment {
                        VerticalAlignment::Top => content_bounds.y,
                        VerticalAlignment::Center => {
                            content_bounds.y + (content_bounds.height - actual_content_height) / 2.0
                        }
                        VerticalAlignment::Bottom => {
                            content_bounds.y + content_bounds.height - actual_content_height
                        }
                    };

                    let mut adjusted_bounds = content_bounds;
                    adjusted_bounds.y = adjusted_content_y;

                    self.cells.push(RenderedCell {
                        cell: cell.clone(),
                        row: row_idx,
                        column: col_idx,
                        bounds,
                        colspan: cell.col_span,
                        rowspan: cell.row_span,
                        content_bounds: adjusted_bounds,
                    });
                }

                x += col_width;
            }
        }
    }

    /// Gets rendered cells for a specific row
    pub fn cells_in_row(&self, row: usize) -> Vec<&RenderedCell> {
        self.cells
            .iter()
            .filter(|c| c.row == row)
            .collect()
    }

    /// Gets rendered cells for a specific column
    pub fn cells_in_column(&self, col: usize) -> Vec<&RenderedCell> {
        self.cells
            .iter()
            .filter(|c| c.column == col)
            .collect()
    }
}

/// Table layout engine
#[derive(Debug, Clone)]
pub struct TableLayout {
    /// Default column width
    pub default_column_width: f32,
    /// Default row height
    pub default_row_height: f32,
    /// Cell padding
    pub cell_padding: f32,
    /// Border width
    pub border_width: f32,
}

impl Default for TableLayout {
    fn default() -> Self {
        TableLayout {
            default_column_width: 100.0,
            default_row_height: 20.0,
            cell_padding: 2.0,
            border_width: 1.0,
        }
    }
}

impl TableLayout {
    /// Creates a new table layout engine
    pub fn new() -> Self {
        TableLayout::default()
    }

    /// Layouts a table within the available width
    pub fn layout_table(&self, table: &mut Table, available_width: f32) -> RenderedTable {
        // Ensure columns are initialized
        if table.columns.is_empty() {
            self.initialize_columns(table, available_width);
        }

        RenderedTable::new(table, available_width)
    }

    /// Initializes columns based on table structure
    fn initialize_columns(&self, table: &mut Table, available_width: f32) {
        // Count columns from the first row
        let column_count = table
            .rows
            .first()
            .map(|r| r.visible_cell_count())
            .unwrap_or(1);

        // Create auto-width columns
        let column_width = available_width / column_count.max(1) as f32;

        for i in 0..column_count {
            table.columns.push(TableColumn::new(i, column_width));
        }
    }
}

/// Builder for creating tables programmatically
#[derive(Debug, Clone)]
pub struct TableBuilder {
    table: Table,
}

impl TableBuilder {
    /// Creates a new table builder
    pub fn new() -> Self {
        TableBuilder {
            table: Table::new(),
        }
    }

    /// Sets table properties
    pub fn with_properties(mut self, properties: TableProperties) -> Self {
        self.table.properties = properties;
        self
    }

    /// Adds a row with cells
    pub fn add_row<F>(mut self, row_height: f32, cell_count: usize, f: F) -> Self
    where
        F: FnOnce(&mut [TableCell]),
    {
        let mut cells = Vec::with_capacity(cell_count);
        for i in 0..cell_count {
            cells.push(TableCell::new(i, self.table.rows.len()));
        }

        // Create a mutable slice for the closure to modify
        let cells_slice = cells.as_mut_slice();
        f(cells_slice);

        let mut row = TableRow::new();
        row.height = row_height;
        for cell in cells {
            row.add_cell(cell);
        }
        self.table.add_row(row);
        self
    }

    /// Builds the table
    pub fn build(self) -> Table {
        self.table
    }
}

impl Default for TableBuilder {
    fn default() -> Self {
        TableBuilder::new()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::table::*;
    use crate::line_layout::{Alignment as LineLayoutAlignment, LineLayout, ParagraphLayout};
    use crate::page_layout::Rect;

    fn create_test_paragraph(text: &str) -> ParagraphLayout {
        let mut layout = LineLayout::new();
        layout.layout_paragraph(text, 100.0)
    }

    #[test]
    fn test_table_cell_creation() {
        let cell = TableCell::new(0, 0);
        assert_eq!(cell.column_index, 0);
        assert_eq!(cell.row_index, 0);
        assert_eq!(cell.col_span, 1);
        assert_eq!(cell.row_span, 1);
        assert!(cell.content.is_empty());
    }

    #[test]
    fn test_table_cell_with_spans() {
        let cell = TableCell::new(0, 0)
            .with_col_span(2)
            .with_row_span(3);

        assert_eq!(cell.col_span, 2);
        assert_eq!(cell.row_span, 3);
        assert_eq!(cell.span_columns(), 2);
        assert_eq!(cell.span_rows(), 3);
    }

    #[test]
    fn test_table_row_creation() {
        let mut row = TableRow::new();
        assert!(row.cells.is_empty());
        assert_eq!(row.height, 0.0);
        assert_eq!(row.visible_cell_count(), 0);

        row.add_cell(TableCell::new(0, 0));
        row.add_cell(TableCell::new(1, 0).with_col_span(2));
        assert_eq!(row.cells.len(), 2);
        assert_eq!(row.visible_cell_count(), 3); // 1 + 2 = 3
    }

    #[test]
    fn test_table_column_creation() {
        let col = TableColumn::new(0, 100.0);
        assert_eq!(col.index, 0);
        assert_eq!(col.width, 100.0);
        assert_eq!(col.preferred_width, 100.0);
        assert_eq!(col.width_type, WidthType::Fixed);

        let auto_col = TableColumn::auto(1);
        assert_eq!(auto_col.width_type, WidthType::Auto);
        assert_eq!(auto_col.width, 0.0);
    }

    #[test]
    fn test_table_creation() {
        let mut table = Table::new();
        assert_eq!(table.row_count(), 0);
        assert_eq!(table.column_count(), 0);

        table.add_row(TableRow::new());
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn test_table_grid_basic() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 100.0),
            TableColumn::new(1, 100.0),
            TableColumn::new(2, 100.0),
        ];

        let mut row1 = TableRow::new();
        row1.add_cell(TableCell::new(0, 0));
        row1.add_cell(TableCell::new(1, 0));
        row1.add_cell(TableCell::new(2, 0));
        table.add_row(row1);

        let mut row2 = TableRow::new();
        row2.add_cell(TableCell::new(0, 1));
        row2.add_cell(TableCell::new(1, 1));
        row2.add_cell(TableCell::new(2, 1));
        table.add_row(row2);

        let grid = TableGrid::new(&table);
        assert_eq!(grid.row_count, 2);
        assert_eq!(grid.column_count, 3);

        // Check all cells are accessible
        for row in 0..2 {
            for col in 0..3 {
                assert!(grid.get_cell(row, col).is_some());
                assert!(!grid.is_covered(row, col));
            }
        }
    }

    #[test]
    fn test_table_grid_with_colspan() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 50.0),
            TableColumn::new(1, 100.0),
        ];

        let mut row = TableRow::new();
        row.add_cell(TableCell::new(0, 0));                    // column 0
        row.add_cell(TableCell::new(1, 0).with_col_span(2));  // columns 1-2
        table.add_row(row);

        let grid = TableGrid::new(&table);

        // Cell at (0, 0) should be accessible
        assert!(grid.get_cell(0, 0).is_some());
        assert!(!grid.is_covered(0, 0));

        // Cell at (0, 1) should be covered
        assert!(grid.is_covered(0, 1));
        assert!(grid.get_cell(0, 1).is_none());
    }

    #[test]
    fn test_table_grid_with_rowspan() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 100.0),
            TableColumn::new(1, 100.0),
        ];

        let mut row1 = TableRow::new();
        row1.add_cell(TableCell::new(0, 0).with_row_span(2));  // spans rows 0-1
        row1.add_cell(TableCell::new(1, 0));
        table.add_row(row1);

        let mut row2 = TableRow::new();
        row2.add_cell(TableCell::new(1, 1));  // This should be covered at column 0
        table.add_row(row2);

        let grid = TableGrid::new(&table);

        // Row 0, col 0 should have the spanning cell
        assert!(grid.get_cell(0, 0).is_some());

        // Row 1, col 0 should be covered
        assert!(grid.is_covered(1, 0));

        // Row 1, col 1 should have the second cell
        assert!(grid.get_cell(1, 1).is_some());
    }

    #[test]
    fn test_rendered_table_column_widths() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 100.0),
            TableColumn::new(1, 150.0),
            TableColumn::new(2, 200.0),
        ];

        let row = TableRow::new();
        table.add_row(row);

        let rendered = RenderedTable::new(&table, 500.0);

        let widths = rendered.grid.column_widths();
        assert_eq!(widths.len(), 3);
        // Widths should be at least the minimum
        assert!(widths[0] >= 1.0);
        assert!(widths[1] >= 1.0);
        assert!(widths[2] >= 1.0);
    }

    #[test]
    fn test_rendered_table_auto_columns() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::auto(0),
            TableColumn::auto(1),
            TableColumn::auto(2),
        ];

        let row = TableRow::new();
        table.add_row(row);

        let rendered = RenderedTable::new(&table, 300.0);

        let widths = rendered.grid.column_widths();
        // All columns should get equal width
        assert_eq!(widths.len(), 3);
        let total: f32 = widths.iter().sum();
        assert!((total - 300.0).abs() < 1.0);
    }

    #[test]
    fn test_rendered_table_row_heights() {
        let mut table = Table::new();
        table.columns = vec![TableColumn::new(0, 100.0)];

        let mut row = TableRow::new();
        let mut cell = TableCell::new(0, 0);
        cell.content.push(create_test_paragraph("Line 1\nLine 2\nLine 3"));
        row.add_cell(cell);
        table.add_row(row);

        let rendered = RenderedTable::new(&table, 100.0);

        let heights = rendered.grid.row_heights();
        assert_eq!(heights.len(), 1);
        assert!(heights[0] > 0.0);
    }

    #[test]
    fn test_rendered_table_bounds() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 100.0),
            TableColumn::new(1, 100.0),
        ];

        let row = TableRow::new();
        table.add_row(row);

        let rendered = RenderedTable::new(&table, 200.0);

        assert!(rendered.bounds.width > 0.0);
        assert!(rendered.bounds.height > 0.0);
    }

    #[test]
    fn test_rendered_cells() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 100.0),
            TableColumn::new(1, 100.0),
        ];

        let mut row = TableRow::new();
        row.add_cell(TableCell::new(0, 0));
        row.add_cell(TableCell::new(1, 0));
        table.add_row(row);

        let rendered = RenderedTable::new(&table, 200.0);

        assert_eq!(rendered.cells.len(), 2);

        // First cell should be on the left
        assert_eq!(rendered.cells[0].column, 0);
        assert_eq!(rendered.cells[0].bounds.x, rendered.borders.left.width);

        // Second cell should be on the right
        assert_eq!(rendered.cells[1].column, 1);
        assert!(rendered.cells[1].bounds.x > rendered.cells[0].bounds.x);
    }

    #[test]
    fn test_rendered_cell_with_colspan() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 50.0),
            TableColumn::new(1, 50.0),
            TableColumn::new(2, 50.0),
        ];

        let mut row = TableRow::new();
        row.add_cell(TableCell::new(0, 0).with_col_span(2));
        row.add_cell(TableCell::new(2, 0));
        table.add_row(row);

        let rendered = RenderedTable::new(&table, 150.0);

        // Should have 2 rendered cells (1 spanning + 1 regular)
        assert_eq!(rendered.cells.len(), 2);

        let spanning = &rendered.cells[0];
        assert_eq!(spanning.colspan, 2);
        assert!(spanning.bounds.width >= 100.0); // Two columns width
    }

    #[test]
    fn test_cells_in_row() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 50.0),
            TableColumn::new(1, 50.0),
            TableColumn::new(2, 50.0),
        ];

        let mut row1 = TableRow::new();
        row1.add_cell(TableCell::new(0, 0));
        row1.add_cell(TableCell::new(1, 0));
        row1.add_cell(TableCell::new(2, 0));
        table.add_row(row1);

        let mut row2 = TableRow::new();
        row2.add_cell(TableCell::new(0, 1));
        row2.add_cell(TableCell::new(1, 1));
        table.add_row(row2);

        let rendered = RenderedTable::new(&table, 150.0);

        let row1_cells = rendered.cells_in_row(0);
        assert_eq!(row1_cells.len(), 3);

        let row2_cells = rendered.cells_in_row(1);
        assert_eq!(row2_cells.len(), 2);
    }

    #[test]
    fn test_table_layout_engine() {
        let layout = TableLayout::new();
        let mut table = Table::new();

        // Add row without columns
        table.add_row(TableRow::new());

        let rendered = layout.layout_table(&mut table, 300.0);

        assert!(rendered.grid.column_count > 0);
        assert_eq!(rendered.grid.row_count, 1);
    }

    #[test]
    fn test_table_builder() {
        let table = TableBuilder::new()
            .add_row(20.0, 3, |cells| {
                cells[0].content.push(create_test_paragraph("Cell 1"));
                cells[1].content.push(create_test_paragraph("Cell 2"));
                cells[2].content.push(create_test_paragraph("Cell 3"));
            })
            .add_row(25.0, 2, |cells| {
                cells[0].content.push(create_test_paragraph("Spanned"));
                cells[0].col_span = 2;  // Set colspan on existing cell
            })
            .build();

        assert_eq!(table.row_count(), 2);
        assert!(table.column_count() >= 3);
    }

    #[test]
    fn test_table_width_types() {
        let fixed = TableWidth::Fixed(500.0);
        assert_eq!(fixed, TableWidth::Fixed(500.0));

        let auto = TableWidth::Auto;
        assert_eq!(auto, TableWidth::Auto);

        let percent = TableWidth::Percent(75.0);
        assert_eq!(percent, TableWidth::Percent(75.0));
    }

    #[test]
    fn test_border_styles() {
        assert_eq!(BorderStyle::None, BorderStyle::None);
        assert_eq!(BorderStyle::Single, BorderStyle::Single);
        assert_eq!(BorderStyle::Double, BorderStyle::Double);
        assert_eq!(BorderStyle::Dotted, BorderStyle::Dotted);
        assert_eq!(BorderStyle::Dashed, BorderStyle::Dashed);
        assert_eq!(BorderStyle::Hairline, BorderStyle::Hairline);
    }

    #[test]
    fn test_cell_properties_defaults() {
        let props = CellProperties::default();
        assert_eq!(props.background_color, None);
        assert_eq!(props.vertical_alignment, VerticalAlignment::Top);
        assert!(props.padding_top >= 0.0);
        assert!(props.padding_bottom >= 0.0);
        assert!(props.padding_left >= 0.0);
        assert!(props.padding_right >= 0.0);
    }

    #[test]
    fn test_table_borders_defaults() {
        let borders = TableBorders::default();
        assert_eq!(borders.top.style, BorderStyle::Single);
        assert_eq!(borders.bottom.style, BorderStyle::Single);
        assert_eq!(borders.left.style, BorderStyle::Single);
        assert_eq!(borders.right.style, BorderStyle::Single);
    }

    #[test]
    fn test_height_rule() {
        assert_eq!(HeightRule::Auto, HeightRule::Auto);
        assert_eq!(HeightRule::AtLeast, HeightRule::AtLeast);
        assert_eq!(HeightRule::Exact, HeightRule::Exact);
        assert_eq!(HeightRule::Multiple, HeightRule::Multiple);
    }

    #[test]
    fn test_vertical_alignment() {
        assert_eq!(VerticalAlignment::Top, VerticalAlignment::Top);
        assert_eq!(VerticalAlignment::Center, VerticalAlignment::Center);
        assert_eq!(VerticalAlignment::Bottom, VerticalAlignment::Bottom);
    }

    #[test]
    fn test_width_type() {
        assert_eq!(WidthType::Fixed, WidthType::Fixed);
        assert_eq!(WidthType::Auto, WidthType::Auto);
        assert_eq!(WidthType::Percent, WidthType::Percent);
    }

    #[test]
    fn test_table_layout_type() {
        assert_eq!(TableLayoutType::Fixed, TableLayoutType::Fixed);
        assert_eq!(TableLayoutType::AutoFit, TableLayoutType::AutoFit);
        assert_eq!(TableLayoutType::AutoWidth, TableLayoutType::AutoWidth);
    }

    #[test]
    fn test_row_properties_defaults() {
        let props = RowProperties::default();
        assert_eq!(props.height_rule, HeightRule::Auto);
        assert_eq!(props.height_value, 0.0);
        assert_eq!(props.background_color, None);
    }

    #[test]
    fn test_rendered_cell_bounds() {
        let mut table = Table::new();
        table.columns = vec![TableColumn::new(0, 100.0)];

        let mut row = TableRow::new();
        let cell = TableCell::new(0, 0);
        row.add_cell(cell);
        table.add_row(row);

        let rendered = RenderedTable::new(&table, 100.0);

        if let Some(rendered_cell) = rendered.cells.first() {
            // Content bounds should be inside cell bounds
            assert!(rendered_cell.content_bounds.x >= rendered_cell.bounds.x);
            assert!(rendered_cell.content_bounds.y >= rendered_cell.bounds.y);
            assert!(rendered_cell.content_bounds.width <= rendered_cell.bounds.width);
            assert!(rendered_cell.content_bounds.height <= rendered_cell.bounds.height);
        }
    }

    #[test]
    fn test_complex_rowspan_colspan() {
        let mut table = Table::new();
        table.columns = vec![
            TableColumn::new(0, 50.0),
            TableColumn::new(1, 50.0),
            TableColumn::new(2, 50.0),
            TableColumn::new(3, 50.0),
        ];

        // Row 1: Cell spanning (0,0) to (1,1), Cell at (2,0), Cell at (3,0)
        let mut row1 = TableRow::new();
        row1.add_cell(TableCell::new(0, 0).with_col_span(2).with_row_span(2));
        row1.add_cell(TableCell::new(2, 0));
        row1.add_cell(TableCell::new(3, 0));
        table.add_row(row1);

        // Row 2: Cell at (2,1), Cell at (3,1)
        let mut row2 = TableRow::new();
        row2.add_cell(TableCell::new(2, 1));
        row2.add_cell(TableCell::new(3, 1));
        table.add_row(row2);

        let rendered = RenderedTable::new(&table, 200.0);

        // Should have 4 rendered cells (2 in row 1 + 2 in row 2)
        assert_eq!(rendered.cells.len(), 4);

        // Check grid positions
        for cell in &rendered.cells {
            assert!(cell.column < 4);
            assert!(cell.row < 2);
        }
    }
}

use crate::format::{BorderSpec, Color};
use crate::ids::NodeId;
use crate::nodes::{Block, Paragraph};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableFormat {
    pub width: Option<f32>,
    pub column_widths: Vec<f32>,
    pub border: Option<BorderSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellFormat {
    pub background: Option<Color>,
    pub border: Option<BorderSpec>,
    pub colspan: u32,
    pub rowspan: u32,
    pub vertical_align: VerticalAlign,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum VerticalAlign {
    #[default]
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub id: NodeId,
    pub format: CellFormat,
    pub blocks: Vec<Block>,
}

impl TableCell {
    pub fn new() -> Self {
        Self {
            id: NodeId::new(),
            format: CellFormat::default(),
            blocks: vec![Block::Paragraph(Paragraph::new())],
        }
    }
}

impl Default for TableCell {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: NodeId,
    pub height: Option<f32>,
    pub cells: Vec<TableCell>,
}

impl TableRow {
    pub fn with_cells(cells: Vec<TableCell>) -> Self {
        Self {
            id: NodeId::new(),
            height: None,
            cells,
        }
    }

    /// Grid column where `cell_index` begins (sum of prior colspans).
    pub fn grid_column_for_cell_index(&self, cell_index: usize) -> usize {
        self.cells
            .iter()
            .take(cell_index)
            .map(|c| c.format.colspan.max(1) as usize)
            .sum()
    }

    /// Physical cell index covering `grid_col` in this row.
    pub fn cell_index_at_grid_column(&self, grid_col: usize) -> Option<usize> {
        let mut consumed = 0usize;
        for (i, cell) in self.cells.iter().enumerate() {
            let span = cell.format.colspan.max(1) as usize;
            if grid_col < consumed + span {
                return Some(i);
            }
            consumed += span;
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: NodeId,
    pub format: TableFormat,
    pub rows: Vec<TableRow>,
    /// Imported OOXML table style id, if any.
    #[serde(default)]
    pub style_id: Option<crate::ids::StyleId>,
}

impl Table {
    pub fn grid_column_count(&self) -> usize {
        self.rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|c| c.format.colspan.max(1) as usize)
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(0)
    }

    pub fn new(rows: u32, cols: u32) -> Self {
        let column_width = 100.0;
        let mut table_rows = Vec::with_capacity(rows as usize);
        for _ in 0..rows {
            let cells: Vec<TableCell> = (0..cols).map(|_| TableCell::new()).collect();
            table_rows.push(TableRow::with_cells(cells));
        }
        Self {
            id: NodeId::new(),
            format: TableFormat {
                width: Some(column_width * cols as f32),
                column_widths: vec![column_width; cols as usize],
                border: Some(BorderSpec::default()),
            },
            rows: table_rows,
            style_id: None,
        }
    }
}

/// Plaintext from every paragraph block in a cell.
pub fn cell_visible_text(cell: &TableCell) -> String {
    cell.blocks
        .iter()
        .filter_map(|b| b.paragraph())
        .map(|p| p.visible_text())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Text in the cell covering `grid_col` on `row`, if any.
pub fn cell_text_at_grid(table: &Table, row: usize, grid_col: usize) -> Option<String> {
    let row_data = table.rows.get(row)?;
    let mut consumed = 0usize;
    for cell in &row_data.cells {
        let span = cell.format.colspan.max(1) as usize;
        if consumed <= grid_col && grid_col < consumed + span {
            let text = cell_visible_text(cell);
            return if text.is_empty() { None } else { Some(text) };
        }
        consumed += span;
    }
    None
}

/// Sum numeric values in `grid_col` for rows `[0, row)`.
pub fn sum_numeric_above(table: &Table, row: usize, grid_col: usize) -> f64 {
    if row == 0 {
        return 0.0;
    }
    (0..row)
        .filter_map(|ri| cell_text_at_grid(table, ri, grid_col))
        .filter_map(|text| text.trim().parse::<f64>().ok())
        .sum()
}

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

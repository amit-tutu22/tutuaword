use tw_model::NodeId;

pub type PageIndex = u32;

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub atlas_x: f32,
    pub atlas_y: f32,
    pub atlas_w: f32,
    pub atlas_h: f32,
    pub color: u32,
    pub font_id: u32,
}

#[derive(Debug, Clone)]
pub struct TextLine {
    pub y: f32,
    pub x: f32,
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_height: f32,
    pub glyphs: Vec<PositionedGlyph>,
    pub paragraph_id: NodeId,
    pub run_map: Vec<(f32, f32, NodeId, usize)>,
    pub list_marker: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImageLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub image_id: NodeId,
    pub asset_id: String,
    /// Encoded source bytes (PNG, JPEG, ...) shared with the document model so
    /// the renderer can hand them to the platform decoder. Empty for
    /// placeholders.
    pub encoded: std::sync::Arc<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct TableCellLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub cell_id: NodeId,
    pub background: Option<u32>,
    pub lines: Vec<TextLine>,
}

#[derive(Debug, Clone)]
pub struct TableLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub table_id: NodeId,
    pub cells: Vec<TableCellLayout>,
    pub grid_lines: Vec<f32>,
}

#[derive(Debug, Clone)]
pub enum LayoutBox {
    TextLine(TextLine),
    Image(ImageLayout),
    Table(TableLayout),
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
    },
}

#[derive(Debug, Clone)]
pub struct PageLayout {
    pub page_index: PageIndex,
    pub width: f32,
    pub height: f32,
    pub content_top: f32,
    pub content_left: f32,
    pub content_width: f32,
    pub content_height: f32,
    pub boxes: Vec<LayoutBox>,
}

#[derive(Debug, Clone)]
pub struct HitTestResult {
    pub page: PageIndex,
    pub run_id: NodeId,
    pub char_offset: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LineMap {
    pub lines: Vec<TextLine>,
}

impl LineMap {
    pub fn hit_test(&self, x: f32, y: f32) -> Option<HitTestResult> {
        for line in &self.lines {
            if y >= line.y - line.ascent && y <= line.y + line.descent {
                for &(x_start, x_end, run_id, char_offset) in &line.run_map {
                    if x >= x_start && x <= x_end {
                        return Some(HitTestResult {
                            page: 0,
                            run_id,
                            char_offset,
                        });
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocumentLayout {
    pub pages: Vec<PageLayout>,
}

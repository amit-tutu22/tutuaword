use tw_model::NodeId;

pub type PageIndex = u32;

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u32,
    /// Source character this glyph represents (for PDF / accessibility export).
    pub codepoint: char,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationKind {
    Underline,
    DoubleUnderline,
    Strikethrough,
    Highlight,
}

#[derive(Debug, Clone)]
pub struct TextDecoration {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: u32,
    pub kind: DecorationKind,
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
    /// X positions immediately after a space character, used for justification.
    pub justify_stops: Vec<f32>,
    pub decorations: Vec<TextDecoration>,
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
    /// ARGB color per grid line segment (parallel to `grid_lines` chunks of 4).
    pub grid_line_colors: Vec<u32>,
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
                    // Empty runs have zero width; give them a clickable caret target.
                    let end = if x_end <= x_start {
                        x_start + 4.0
                    } else {
                        x_end
                    };
                    if x >= x_start && x <= end {
                        return Some(HitTestResult {
                            page: 0,
                            run_id,
                            char_offset,
                        });
                    }
                }
                // Click on the blank part of the line → caret at end of line.
                if let Some((run_id, char_offset)) = line_end_offset(line) {
                    return Some(HitTestResult {
                        page: 0,
                        run_id,
                        char_offset,
                    });
                }
            }
        }
        // Below/above all lines: still land on the first available run so an
        // empty page remains editable without a precise click.
        self.lines.first().and_then(|line| {
            line.run_map.first().map(|&(_, _, run_id, char_offset)| HitTestResult {
                page: 0,
                run_id,
                char_offset,
            })
        })
    }

    /// Caret `(x, baseline_y, height)` for a document position within a run.
    pub fn caret_at(&self, run_id: NodeId, char_offset: usize) -> Option<(f32, f32, f32)> {
        let mut best: Option<(f32, f32, f32)> = None;

        for line in &self.lines {
            for (index, &(x_start, x_end, rid, seg_offset)) in line.run_map.iter().enumerate() {
                if rid != run_id {
                    continue;
                }

                let seg_chars = segment_char_count(line, index, x_start, x_end);
                if char_offset < seg_offset {
                    continue;
                }
                let local = char_offset - seg_offset;
                // Some characters (especially whitespace) may not rasterize into
                // positioned glyphs, which can make `seg_char_count` too small.
                // Don't skip; interpolate with a safe effective char count below.

                let x = caret_x_in_segment(line, x_start, x_end, local, seg_chars);
                let height = line.ascent + line.descent;
                best = Some((x, line.y, height));
            }
        }

        if best.is_some() {
            return best;
        }

        // Position is past the last laid-out segment — snap to the run's end.
        for line in self.lines.iter().rev() {
            for &(x_start, x_end, rid, _) in line.run_map.iter().rev() {
                if rid == run_id {
                    let x = if x_end > x_start { x_end } else { x_start };
                    return Some((x, line.y, line.ascent + line.descent));
                }
            }
        }

        self.lines
            .first()
            .map(|line| (line.x, line.y, line.ascent + line.descent))
    }

    /// Last editable position on this page's final line.
    pub fn tail_hit(&self, page: u32) -> Option<HitTestResult> {
        let line = self.lines.last()?;
        let (run_id, char_offset) = line_end_offset(line)?;
        Some(HitTestResult {
            page,
            run_id,
            char_offset,
        })
    }
}

fn line_end_offset(line: &TextLine) -> Option<(NodeId, usize)> {
    let last_index = line.run_map.len().checked_sub(1)?;
    let &(x_start, x_end, run_id, char_offset) = line.run_map.get(last_index)?;
    let seg_chars = segment_char_count(line, last_index, x_start, x_end);
    Some((run_id, char_offset + seg_chars))
}

fn segment_char_count(line: &TextLine, index: usize, x_start: f32, x_end: f32) -> usize {
    let glyph_count = line
        .glyphs
        .iter()
        .filter(|g| g.x >= x_start - 0.5 && g.x < x_end + 0.5)
        .count();
    if glyph_count > 0 {
        return glyph_count;
    }
    if index + 1 < line.run_map.len() {
        let next_offset = line.run_map[index + 1].3;
        let seg_offset = line.run_map[index].3;
        return next_offset.saturating_sub(seg_offset);
    }
    0
}

fn caret_x_in_segment(
    _line: &TextLine,
    x_start: f32,
    x_end: f32,
    local_offset: usize,
    seg_char_count: usize,
) -> f32 {
    if local_offset == 0 {
        return x_start;
    }
    if x_end <= x_start {
        return x_start;
    }
    let effective_chars = seg_char_count.max(local_offset + 1).max(1);
    let t = (local_offset as f32) / (effective_chars as f32);
    let t_clamped = t.clamp(0.0, 1.0);
    x_start + (x_end - x_start) * t_clamped
}

#[derive(Debug, Clone, Default)]
pub struct DocumentLayout {
    pub pages: Vec<PageLayout>,
}

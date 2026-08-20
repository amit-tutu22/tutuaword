use tw_model::NodeId;

pub type PageIndex = u32;

#[derive(Debug, Clone, PartialEq)]
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
    /// Read-only equation preview frame (F14.S2).
    MathFrame,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDecoration {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: u32,
    pub kind: DecorationKind,
}

#[derive(Debug, Clone, PartialEq)]
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
    /// Character count of each `run_map` segment, parallel to `run_map`.
    ///
    /// Used with `run_map` to recover source text for PDF / accessibility when
    /// ligatures or zero-bitmap glyphs (spaces) would otherwise lose characters.
    pub run_map_chars: Vec<usize>,
    pub list_marker: Option<String>,
    /// X positions immediately after a space character, used for justification.
    pub justify_stops: Vec<f32>,
    pub decorations: Vec<TextDecoration>,
    /// When true, the line is painted but excluded from caret hit-testing
    /// (e.g. SmartArt / Chart placeholder captions).
    pub decorative: bool,
}

#[derive(Debug, Clone, PartialEq)]
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
    pub rotation_deg: f32,
    pub opacity: f32,
    pub crop_left: f32,
    pub crop_top: f32,
    pub crop_right: f32,
    pub crop_bottom: f32,
    /// When set, this image is a diagram/chart raster preview and participates
    /// in read-only shape selection (F12.S2 / F13.S2 hardening).
    pub selection_shape_kind: Option<tw_model::ShapeKind>,
}

/// Read-only imported shape placeholder bounds (F11.S1).
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub shape_id: NodeId,
    pub shape_type: tw_model::ShapeKind,
    pub fill: Option<u32>,
    pub stroke: Option<u32>,
    pub stroke_width: f32,
    /// Sample / edited chart dataset for in-editor bar preview (F13.S3).
    pub chart_data: Option<tw_model::ChartData>,
    /// SmartArt layout style for placeholder previews.
    pub diagram_kind: tw_model::DiagramKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableCellLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub cell_id: NodeId,
    pub background: Option<u32>,
    pub lines: Vec<TextLine>,
    /// Nested tables laid out inside this cell (F09.S5).
    pub nested_tables: Vec<TableLayout>,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutBox {
    TextLine(TextLine),
    Image(ImageLayout),
    Shape(ShapeLayout),
    Table(TableLayout),
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LineMap {
    pub lines: Vec<TextLine>,
}

impl LineMap {
    pub fn hit_test(&self, x: f32, y: f32) -> Option<HitTestResult> {
        // Gather every line whose vertical band contains `y`. Table cells in the
        // same row share a baseline, so we must consider them together — otherwise
        // the leftmost cell's blank-line fallback steals clicks meant for neighbors.
        let y_matches: Vec<&TextLine> = self
            .lines
            .iter()
            .filter(|line| y >= line.y - line.ascent && y <= line.y + line.descent)
            .collect();

        // Prefer an exact run/glyph hit across all Y-matching lines first.
        for line in &y_matches {
            for (index, &(x_start, x_end, run_id, seg_offset)) in line.run_map.iter().enumerate() {
                // Empty runs have zero width; give them a clickable caret target.
                let end = if x_end <= x_start {
                    x_start + 4.0
                } else {
                    x_end
                };
                if x >= x_start && x <= end {
                    let char_offset =
                        hit_offset_in_segment(line, index, x, x_start, x_end, seg_offset);
                    return Some(HitTestResult {
                        page: 0,
                        run_id,
                        char_offset,
                    });
                }
            }
        }

        // Blank-area click: pick the line that owns this X (column for tables,
        // sole line for body paragraphs) and put the caret at its end.
        if let Some(line) = pick_line_for_x(&y_matches, x) {
            if let Some((run_id, char_offset)) = line_end_offset(line) {
                return Some(HitTestResult {
                    page: 0,
                    run_id,
                    char_offset,
                });
            }
        }

        // Probe landed between line bands — snap to the nearest line instead of
        // always jumping to the page's last line (which broke Up/Down mid-page).
        let probe_lines: Vec<&TextLine> = if y_matches.is_empty() {
            nearest_line_by_y(&self.lines, y).into_iter().collect()
        } else {
            y_matches
        };

        for line in &probe_lines {
            for (index, &(x_start, x_end, run_id, seg_offset)) in line.run_map.iter().enumerate() {
                let end = if x_end <= x_start {
                    x_start + 4.0
                } else {
                    x_end
                };
                if x >= x_start && x <= end {
                    let char_offset =
                        hit_offset_in_segment(line, index, x, x_start, x_end, seg_offset);
                    return Some(HitTestResult {
                        page: 0,
                        run_id,
                        char_offset,
                    });
                }
            }
            if let Some(line) = pick_line_for_x(&probe_lines, x) {
                if let Some((run_id, char_offset)) = line_end_offset(line) {
                    return Some(HitTestResult {
                        page: 0,
                        run_id,
                        char_offset,
                    });
                }
            }
        }

        // Outside every line band: land on the nearest edge so empty pages stay
        // editable. Above content → first line; below → last line.
        let edge = if let Some(first) = self.lines.first() {
            let first_top = first.y - first.ascent;
            if y < first_top {
                first
            } else {
                self.lines.last().unwrap_or(first)
            }
        } else {
            return None;
        };
        if y < edge.y - edge.ascent {
            edge.run_map.first().map(|&(_, _, run_id, char_offset)| HitTestResult {
                page: 0,
                run_id,
                char_offset,
            })
        } else {
            line_end_offset(edge).map(|(run_id, char_offset)| HitTestResult {
                page: 0,
                run_id,
                char_offset,
            })
        }
    }

    /// Caret `(x, baseline_y, height)` for a document position within a run.
    ///
    /// Returns `None` when [run_id] is not laid out on this page so callers can
    /// search other pages (critical after Enter creates a paragraph on page N+1).
    pub fn caret_at(&self, run_id: NodeId, char_offset: usize) -> Option<(f32, f32, f32)> {
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
                if local > seg_chars {
                    continue;
                }

                let x = caret_x_in_segment(line, x_start, x_end, local, seg_chars);
                let height = line.ascent + line.descent;
                return Some((x, line.y, height));
            }
        }

        // Position is past the last laid-out segment on this page — snap only when
        // it matches the run end here; otherwise the run may continue on page N+1.
        for line in self.lines.iter().rev() {
            for (index, &(x_start, x_end, rid, seg_offset)) in line.run_map.iter().enumerate().rev() {
                if rid != run_id {
                    continue;
                }
                let seg_chars = segment_char_count(line, index, x_start, x_end);
                let end_offset = seg_offset + seg_chars;
                if char_offset == end_offset {
                    let x = if x_end > x_start { x_end } else { x_start };
                    return Some((x, line.y, line.ascent + line.descent));
                }
                if char_offset > end_offset {
                    return None;
                }
            }
        }

        None
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
    let seg_chars = line
        .run_map_chars
        .get(last_index)
        .copied()
        .unwrap_or_else(|| segment_char_count(line, last_index, x_start, x_end));
    Some((run_id, char_offset + seg_chars))
}

/// Resolve the character offset under horizontal position `x` inside a segment.
fn hit_offset_in_segment(
    line: &TextLine,
    index: usize,
    x: f32,
    x_start: f32,
    x_end: f32,
    seg_offset: usize,
) -> usize {
    let seg_chars = segment_char_count(line, index, x_start, x_end);
    if x_end <= x_start || seg_chars == 0 {
        return seg_offset;
    }
    if x <= x_start {
        return seg_offset;
    }
    if x >= x_end {
        return seg_offset + seg_chars;
    }

    // Walk visible glyphs and pick the nearer edge (Word-style).
    let mut glyphs_in_seg: Vec<&PositionedGlyph> = line
        .glyphs
        .iter()
        .filter(|g| g.x >= x_start - 0.5 && g.x < x_end + 0.5)
        .collect();
    glyphs_in_seg.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

    if !glyphs_in_seg.is_empty() {
        let mut offset = seg_offset;
        for g in glyphs_in_seg {
            let mid = g.x + g.width * 0.5;
            if x < mid {
                return offset;
            }
            offset += 1;
        }
        return seg_offset + seg_chars;
    }

    // Whitespace-only segment: interpolate across the segment width.
    let t = (x - x_start) / (x_end - x_start);
    let local = (t * seg_chars as f32).round() as usize;
    seg_offset + local.min(seg_chars)
}

fn distance_to_line_band(y: f32, line: &TextLine) -> f32 {
    let top = line.y - line.ascent;
    let bottom = line.y + line.descent;
    if y < top {
        top - y
    } else if y > bottom {
        y - bottom
    } else {
        0.0
    }
}

fn nearest_line_by_y<'a>(lines: &'a [TextLine], y: f32) -> Option<&'a TextLine> {
    lines
        .iter()
        .min_by(|a, b| {
            distance_to_line_band(y, a)
                .partial_cmp(&distance_to_line_band(y, b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Among lines sharing a Y band, choose the one that owns horizontal position `x`.
///
/// Body paragraphs usually contribute a single Y match, so blank clicks anywhere
/// on that band still resolve. Table cells on one row contribute several matches;
/// prefer a line whose horizontal span contains `x`, else the rightmost line
/// whose left edge is at or left of `x`.
fn pick_line_for_x<'a>(lines: &[&'a TextLine], x: f32) -> Option<&'a TextLine> {
    if lines.is_empty() {
        return None;
    }
    if lines.len() == 1 {
        return Some(lines[0]);
    }
    let mut containing: Option<&TextLine> = None;
    for line in lines {
        let right = if line.width > 0.0 {
            line.x + line.width
        } else {
            line.x + 4.0
        };
        if x >= line.x - 0.5 && x <= right + 0.5 {
            containing = Some(*line);
        }
    }
    if let Some(line) = containing {
        return Some(line);
    }
    let mut best: Option<&TextLine> = None;
    for line in lines {
        if line.x <= x + 0.5 {
            best = Some(*line);
        }
    }
    best.or_else(|| {
        lines
            .iter()
            .min_by(|a, b| {
                a.x.partial_cmp(&b.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    })
}

fn segment_char_count(line: &TextLine, index: usize, x_start: f32, x_end: f32) -> usize {
    if let Some(&count) = line.run_map_chars.get(index) {
        if count > 0 {
            return count;
        }
    }
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
    if local_offset >= seg_char_count {
        return x_end;
    }
    let effective_chars = seg_char_count.max(1);
    let t = (local_offset as f32) / (effective_chars as f32);
    x_start + (x_end - x_start) * t.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Default)]
pub struct DocumentLayout {
    pub pages: Vec<PageLayout>,
}

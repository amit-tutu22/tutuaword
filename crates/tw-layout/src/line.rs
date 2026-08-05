use tw_model::{Alignment, LineSpacing, Paragraph};
use tw_shape::{AtlasKey, GlyphAtlas, TextShaper};
use unicode_linebreak::{linebreaks, BreakOpportunity};

const MARKER_GUTTER: f32 = 24.0;

/// Word's default tab grid (`w:defaultTabStop` of 720 twips), measured from the
/// paragraph's text origin. Explicit `w:tabs` stops are not modelled yet.
const DEFAULT_TAB_INTERVAL: f32 = 36.0;

/// Next tab stop strictly after `cursor_x`, so a tab always advances.
fn next_tab_stop(cursor_x: f32, origin_x: f32) -> f32 {
    let offset = cursor_x - origin_x;
    let stops = (offset / DEFAULT_TAB_INTERVAL).floor() + 1.0;
    origin_x + stops * DEFAULT_TAB_INTERVAL
}

/// Where a paragraph is placed and how wide it may run.
#[derive(Debug, Clone, Copy)]
pub struct ParagraphFrame {
    /// Left edge of the paragraph's text, including any indent.
    pub x: f32,
    pub y: f32,
    pub max_width: f32,
    /// Left edge of the containing column. Word measures the default tab grid
    /// from here, so an indent shifts the text but not the tab stops.
    pub tab_origin: f32,
}

impl ParagraphFrame {
    /// Frame whose tab grid starts at the text edge, for contexts without a
    /// separate column origin.
    pub fn new(x: f32, y: f32, max_width: f32) -> Self {
        Self {
            x,
            y,
            max_width,
            tab_origin: x,
        }
    }

    pub fn indented_in(column_x: f32, indent: f32, y: f32, column_width: f32) -> Self {
        Self {
            x: column_x + indent,
            y,
            max_width: column_width - indent,
            tab_origin: column_x,
        }
    }
}

pub fn layout_paragraph(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    para: &Paragraph,
    frame: ParagraphFrame,
    default_color: u32,
) -> (Vec<super::types::TextLine>, f32) {
    let ParagraphFrame {
        x,
        y,
        max_width,
        tab_origin,
    } = frame;
    let font_id = shaper.default_font();
    let mut lines = Vec::new();
    let mut current_y = y;
    let size = para
        .runs
        .iter()
        .filter_map(|r| r.format.font_size)
        .fold(0.0f32, f32::max)
        .max(12.0);
    let line_height = line_height_for(para, size);
    let ascent = size;
    let descent = size * 0.25;

    let text = para.full_text();
    if text.trim().is_empty() {
        lines.push(super::types::TextLine {
            y: current_y + ascent,
            x,
            width: 0.0,
            ascent,
            descent,
            line_height,
            glyphs: Vec::new(),
            paragraph_id: para.id,
            run_map: vec![(x, x, para.runs[0].id, 0)],
            list_marker: None,
        });
        return (lines, line_height);
    }

    // Greedy line breaking: accumulate break opportunities until the candidate
    // line no longer fits, then emit at the last opportunity that did fit.
    let mut line_start = 0usize;
    let mut last_fit: Option<usize> = None;

    for (byte_idx, opportunity) in linebreaks(&text) {
        if byte_idx <= line_start {
            continue;
        }

        let mandatory = matches!(opportunity, BreakOpportunity::Mandatory);
        if mandatory {
            emit_line(
                shaper,
                atlas,
                atlas_ctx(para, &text, line_start, byte_idx, x, current_y + ascent, tab_origin),
                font_id,
                default_color,
                &mut lines,
            );
            current_y += line_height;
            line_start = byte_idx;
            last_fit = None;
            continue;
        }

        let candidate = trim_trailing(&text, line_start, byte_idx);
        let width = measure_range(shaper, para, line_start, candidate, tab_origin - x, font_id);

        if width <= max_width {
            last_fit = Some(byte_idx);
            continue;
        }

        let break_at = match last_fit {
            Some(fit) if fit > line_start => fit,
            // A single unbreakable token wider than the column: let it overflow.
            _ => byte_idx,
        };
        emit_line(
            shaper,
            atlas,
            atlas_ctx(para, &text, line_start, break_at, x, current_y + ascent, tab_origin),
            font_id,
            default_color,
            &mut lines,
        );
        current_y += line_height;
        line_start = break_at;
        last_fit = if break_at == byte_idx {
            None
        } else {
            Some(byte_idx)
        };
    }

    if line_start < text.len() {
        emit_line(
            shaper,
            atlas,
            atlas_ctx(para, &text, line_start, text.len(), x, current_y + ascent, tab_origin),
            font_id,
            default_color,
            &mut lines,
        );
        current_y += line_height;
    }

    if lines.is_empty() {
        lines.push(super::types::TextLine {
            y: current_y + ascent,
            x,
            width: 0.0,
            ascent,
            descent,
            line_height,
            glyphs: Vec::new(),
            paragraph_id: para.id,
            run_map: vec![(x, x, para.runs[0].id, 0)],
            list_marker: None,
        });
        current_y += line_height;
    }

    apply_alignment(&mut lines, max_width, para.format.alignment.unwrap_or(Alignment::Left));

    (lines, current_y - y)
}

fn line_height_for(para: &Paragraph, size: f32) -> f32 {
    let single = size * 1.2;
    match para.format.line_spacing {
        Some(LineSpacing::Double) => single * 2.0,
        Some(LineSpacing::Multiple(m)) => single * m.max(0.1),
        Some(LineSpacing::Exactly(pt)) => pt.max(1.0),
        Some(LineSpacing::AtLeast(pt)) => single.max(pt),
        Some(LineSpacing::Single) | None => single,
    }
}

/// Byte range of `text` with trailing whitespace removed, used so a line broken
/// at a space is not measured or aligned including that space.
fn trim_trailing(text: &str, start: usize, end: usize) -> usize {
    start + text[start..end].trim_end().len()
}

struct LineContext<'a> {
    para: &'a Paragraph,
    text: &'a str,
    start: usize,
    end: usize,
    x: f32,
    baseline_y: f32,
    tab_origin: f32,
}

fn atlas_ctx<'a>(
    para: &'a Paragraph,
    text: &'a str,
    start: usize,
    end: usize,
    x: f32,
    baseline_y: f32,
    tab_origin: f32,
) -> LineContext<'a> {
    LineContext {
        para,
        text,
        start,
        end,
        x,
        baseline_y,
        tab_origin,
    }
}

fn emit_line(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    ctx: LineContext<'_>,
    font_id: Option<tw_shape::FontId>,
    default_color: u32,
    lines: &mut Vec<super::types::TextLine>,
) {
    let end = trim_trailing(ctx.text, ctx.start, ctx.end);
    let line_text = &ctx.text[ctx.start..end];
    let (line, _) = shape_line(
        shaper,
        atlas,
        ctx.para,
        line_text,
        ctx.start,
        ctx.x,
        ctx.baseline_y,
        ctx.tab_origin,
        font_id,
        default_color,
    );
    lines.push(line);
}

/// The face a run asks for: its family at its weight and slant, falling back to
/// the paragraph's font when the run does not name one.
fn font_for(
    shaper: &mut TextShaper,
    format: &tw_model::CharFormat,
    fallback: tw_shape::FontId,
) -> tw_shape::FontId {
    shaper
        .fonts_mut()
        .resolve_styled(
            format.font_family.as_deref(),
            format.bold.unwrap_or(false),
            format.italic.unwrap_or(false),
        )
        .unwrap_or(fallback)
}

/// Sums shaped advances for a candidate line without touching the glyph atlas.
fn measure_range(
    shaper: &mut TextShaper,
    para: &Paragraph,
    start_byte: usize,
    end_byte: usize,
    tab_origin_offset: f32,
    font_id: Option<tw_shape::FontId>,
) -> f32 {
    let Some(fid) = font_id.or_else(|| shaper.default_font()) else {
        return 0.0;
    };
    // Widths are relative to the line origin, so the tab grid resolves against
    // the column origin expressed in the same space.
    let mut width = 0.0;
    for (segment_text, run, _) in run_segments_for_range(para, start_byte, end_byte) {
        if segment_text.is_empty() {
            continue;
        }
        let run_font = font_for(shaper, &run.format, fid);
        for (piece_index, piece) in segment_text.split('\t').enumerate() {
            if piece_index > 0 {
                width = next_tab_stop(width, tab_origin_offset);
            }
            if piece.is_empty() {
                continue;
            }
            let shaped = shaper.shape(piece, &run.format, run_font);
            width += shaped.glyphs.iter().map(|g| g.x_advance).sum::<f32>();
        }
    }
    width
}

/// Shape and attach list marker glyphs to the first line of a paragraph block.
pub fn apply_list_markers(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    lines: &mut [super::types::TextLine],
    marker: &str,
    marker_x: f32,
    default_color: u32,
) {
    if lines.is_empty() || marker.is_empty() {
        return;
    }
    let marker_para = Paragraph::with_text(marker.to_string());
    let baseline = lines[0].y;
    let font_id = shaper.default_font();
    let (mut marker_line, _) = shape_line(
        shaper,
        atlas,
        &marker_para,
        marker,
        0,
        marker_x,
        baseline,
        marker_x,
        font_id,
        default_color,
    );
    lines[0].glyphs.append(&mut marker_line.glyphs);
    lines[0].list_marker = Some(marker.to_string());
    let _ = MARKER_GUTTER;
}

fn shape_line(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    para: &Paragraph,
    line_text: &str,
    line_start_byte: usize,
    x: f32,
    baseline_y: f32,
    tab_origin: f32,
    font_id: Option<tw_shape::FontId>,
    default_color: u32,
) -> (super::types::TextLine, f32) {
    let fid = font_id.or_else(|| shaper.default_font()).unwrap();
    let mut cursor_x = x;
    let mut glyphs = Vec::new();
    let mut run_map = Vec::new();
    let line_end_byte = line_start_byte + line_text.len();

    let segments = run_segments_for_range(para, line_start_byte, line_end_byte);
    let default_size = para.runs.first().and_then(|r| r.format.font_size).unwrap_or(12.0);
    let ascent = default_size;
    let descent = default_size * 0.25;
    let line_height = default_size * 1.35;

    for (segment_text, run, char_offset) in segments {
        if segment_text.is_empty() {
            continue;
        }
        let size = run.format.font_size.unwrap_or(default_size);
        let color = run
            .format
            .color
            .map(|c| c.to_argb())
            .unwrap_or(default_color);
        let seg_start_x = cursor_x;
        let run_font = font_for(shaper, &run.format, fid);

        // Tabs jump to the next stop rather than being shaped, which would
        // render them as `.notdef` boxes.
        for (piece_index, piece) in segment_text.split('\t').enumerate() {
            if piece_index > 0 {
                cursor_x = next_tab_stop(cursor_x, tab_origin);
            }
            if piece.is_empty() {
                continue;
            }

            let shaped = shaper.shape(piece, &run.format, run_font);
            for g in &shaped.glyphs {
                let key = AtlasKey::new(g.font_key(), g.glyph_id, size);
                let entry = match atlas.get(&key).cloned() {
                    Some(entry) => Some(entry),
                    None => {
                        // Rasterize from the glyph's own face, which is a
                        // fallback when the run font lacked the character.
                        let raster = shaper.rasterize_glyph(g.font, g.glyph_id, size);
                        if raster.width == 0 || raster.height == 0 {
                            None
                        } else {
                            Some(atlas.insert(key, &raster))
                        }
                    }
                };

                if let Some(entry) = entry {
                    // Color bitmaps carry their own palette; a white tint leaves
                    // them untouched under the renderer's Modulate blend.
                    let tint = if entry.is_color { 0xFFFF_FFFF } else { color };
                    glyphs.push(super::types::PositionedGlyph {
                        glyph_id: g.glyph_id,
                        x: cursor_x + g.x_offset + entry.bearing_x,
                        y: baseline_y + g.y_offset - entry.bearing_y,
                        width: entry.width as f32,
                        height: entry.height as f32,
                        atlas_x: entry.x as f32,
                        atlas_y: entry.y as f32,
                        atlas_w: entry.width as f32,
                        atlas_h: entry.height as f32,
                        color: tint,
                        font_id: g.font_key(),
                    });
                }
                cursor_x += g.x_advance;
            }
        }

        run_map.push((seg_start_x, cursor_x, run.id, char_offset));
    }

    let width = cursor_x - x;
    let line = super::types::TextLine {
        y: baseline_y,
        x,
        width,
        ascent,
        descent,
        line_height,
        glyphs,
        paragraph_id: para.id,
        run_map,
        list_marker: None,
    };
    (line, width)
}

fn run_segments_for_range(
    para: &Paragraph,
    start_byte: usize,
    end_byte: usize,
) -> Vec<(String, tw_model::Run, usize)> {
    let mut segments = Vec::new();
    let mut byte_cursor = 0usize;

    for run in &para.runs {
        let run_text = run.text();
        let run_start = byte_cursor;
        let run_end = byte_cursor + run_text.len();
        byte_cursor = run_end;

        if run_end <= start_byte || run_start >= end_byte {
            continue;
        }

        let local_start = start_byte.saturating_sub(run_start);
        let local_end = (end_byte - run_start).min(run_text.len());
        if local_start >= local_end {
            continue;
        }

        segments.push((
            run_text[local_start..local_end].to_string(),
            run.clone(),
            local_start,
        ));
    }

    if segments.is_empty() && !para.runs.is_empty() {
        let run = para.runs[0].clone();
        segments.push((String::new(), run, 0));
    }

    segments
}

fn apply_alignment(lines: &mut [super::types::TextLine], max_width: f32, alignment: Alignment) {
    for line in lines.iter_mut() {
        let offset = match alignment {
            Alignment::Left | Alignment::Justify => 0.0,
            Alignment::Center => (max_width - line.width) / 2.0,
            Alignment::Right => max_width - line.width,
        };
        line.x += offset;
        for g in &mut line.glyphs {
            g.x += offset;
        }
        for entry in &mut line.run_map {
            entry.0 += offset;
            entry.1 += offset;
        }
    }
}

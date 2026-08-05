use tw_model::{Alignment, LineSpacing, Paragraph, RevisionType};
use tw_shape::{AtlasKey, GlyphAtlas, TextShaper};
use unicode_linebreak::{linebreaks, BreakOpportunity};

const MARKER_GUTTER: f32 = 24.0;

/// Word's default tab grid (`w:defaultTabStop` of 720 twips), measured from the
/// paragraph's text origin. Explicit `w:tabs` stops are not modelled yet.
/// Word's default tab grid (`w:defaultTabStop` of 720 twips).
pub const DEFAULT_TAB_INTERVAL: f32 = 36.0;

/// Next tab stop strictly after `cursor_x`, so a tab always advances.
fn next_tab_stop(cursor_x: f32, origin_x: f32, tab_interval: f32) -> f32 {
    let interval = tab_interval.max(1.0);
    let offset = cursor_x - origin_x;
    let stops = (offset / interval).floor() + 1.0;
    origin_x + stops * interval
}

fn with_opacity(argb: u32, factor: f32) -> u32 {
    let a = (((argb >> 24) as f32) * factor.clamp(0.0, 1.0)) as u32;
    (a << 24) | (argb & 0x00FF_FFFF)
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
    /// Distance between default tab stops (`DocumentSettings.default_tab_stop`).
    pub tab_interval: f32,
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
            tab_interval: DEFAULT_TAB_INTERVAL,
        }
    }

    pub fn with_tab_interval(mut self, tab_interval: f32) -> Self {
        self.tab_interval = tab_interval.max(1.0);
        self
    }

    pub fn indented_in(column_x: f32, indent: f32, y: f32, column_width: f32) -> Self {
        Self {
            x: column_x + indent,
            y,
            max_width: column_width - indent,
            tab_origin: column_x,
            tab_interval: DEFAULT_TAB_INTERVAL,
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
        tab_interval,
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
    let (ascent, descent, _) = font_id
        .map(|fid| shaper.vertical_metrics(fid, size))
        .unwrap_or((size, size * 0.25, size * 0.1));

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
            justify_stops: Vec::new(),
            decorations: Vec::new(),
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
            let tail = &text[byte_idx..];
            // unicode-linebreak marks spaces as mandatory breaks. A trailing
            // whitespace-only tail (e.g. "A " while typing) must stay on the
            // current line so the gap and caret advance immediately.
            if tail.is_empty() || tail.chars().all(char::is_whitespace) {
                continue;
            }
            emit_line(
                shaper,
                atlas,
                atlas_ctx(
                    para,
                    &text,
                    line_start,
                    byte_idx,
                    x,
                    current_y + ascent,
                    tab_origin,
                    tab_interval,
                ),
                font_id,
                default_color,
                true,
                &mut lines,
            );
            current_y += line_height;
            line_start = byte_idx;
            last_fit = None;
            continue;
        }

        let candidate = trim_trailing(&text, line_start, byte_idx);
        let width = measure_range(
            shaper,
            para,
            line_start,
            candidate,
            tab_origin - x,
            tab_interval,
            font_id,
        );

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
            atlas_ctx(
                para,
                &text,
                line_start,
                break_at,
                x,
                current_y + ascent,
                tab_origin,
                tab_interval,
            ),
            font_id,
            default_color,
            true,
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
        // Keep trailing whitespace on the paragraph's last line so a typed space
        // is visible and the caret can advance before the next character.
        emit_line(
            shaper,
            atlas,
            atlas_ctx(
                para,
                &text,
                line_start,
                text.len(),
                x,
                current_y + ascent,
                tab_origin,
                tab_interval,
            ),
            font_id,
            default_color,
            false,
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
            justify_stops: Vec::new(),
            decorations: Vec::new(),
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
    tab_interval: f32,
}

fn atlas_ctx<'a>(
    para: &'a Paragraph,
    text: &'a str,
    start: usize,
    end: usize,
    x: f32,
    baseline_y: f32,
    tab_origin: f32,
    tab_interval: f32,
) -> LineContext<'a> {
    LineContext {
        para,
        text,
        start,
        end,
        x,
        baseline_y,
        tab_origin,
        tab_interval,
    }
}

fn emit_line(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    ctx: LineContext<'_>,
    font_id: Option<tw_shape::FontId>,
    default_color: u32,
    trim_end: bool,
    lines: &mut Vec<super::types::TextLine>,
) {
    let end = if trim_end && ctx.end < ctx.text.len() {
        trim_trailing(ctx.text, ctx.start, ctx.end)
    } else {
        ctx.end
    };
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
        ctx.tab_interval,
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
    tab_interval: f32,
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
                width = next_tab_stop(width, tab_origin_offset, tab_interval);
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
        DEFAULT_TAB_INTERVAL,
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
    tab_interval: f32,
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
    let mut line_ascent = 0.0f32;
    let mut line_descent = 0.0f32;
    let mut line_gap = 0.0f32;
    let mut justify_stops = Vec::new();
    let mut decorations = Vec::new();

    for (segment_text, run, char_offset) in segments {
        if segment_text.is_empty() {
            continue;
        }
        let size = run.format.font_size.unwrap_or(default_size);
        let run_font = font_for(shaper, &run.format, fid);
        let (run_ascent, run_descent, run_gap) = shaper.vertical_metrics(run_font, size);
        line_ascent = line_ascent.max(run_ascent);
        line_descent = line_descent.max(run_descent);
        line_gap = line_gap.max(run_gap);
        let mut color = run
            .format
            .color
            .map(|c| c.to_argb())
            .unwrap_or(default_color);
        let is_deleted = run
            .revision
            .as_ref()
            .is_some_and(|rev| rev.revision_type == RevisionType::Delete);
        if is_deleted {
            color = with_opacity(color, 0.45);
        }
        let seg_start_x = cursor_x;

        // Tabs jump to the next stop rather than being shaped, which would
        // render them as `.notdef` boxes.
        for (piece_index, piece) in segment_text.split('\t').enumerate() {
            if piece_index > 0 {
                cursor_x = next_tab_stop(cursor_x, tab_origin, tab_interval);
            }
            if piece.is_empty() {
                continue;
            }

            let piece_chars: Vec<char> = piece.chars().collect();
            let shaped = shaper.shape(piece, &run.format, run_font);
            for g in &shaped.glyphs {
                let codepoint = piece_chars
                    .get(g.cluster as usize)
                    .copied()
                    .unwrap_or('\u{FFFD}');
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
                        codepoint,
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
            for ch in piece.chars() {
                if ch == ' ' {
                    justify_stops.push(cursor_x);
                }
            }
        }

        let seg_end_x = cursor_x;
        if run.format.highlight.is_some() {
            decorations.push(super::types::TextDecoration {
                x: seg_start_x,
                y: baseline_y - line_ascent,
                width: seg_end_x - seg_start_x,
                height: line_ascent + line_descent,
                color: run.format.highlight.map(|c| c.to_argb()).unwrap_or(0xFFFFFF00),
                kind: super::types::DecorationKind::Highlight,
            });
        }
        if run.format.underline.is_some_and(|u| u != tw_model::UnderlineStyle::None) {
            decorations.push(super::types::TextDecoration {
                x: seg_start_x,
                y: baseline_y + (line_descent * 0.25),
                width: seg_end_x - seg_start_x,
                height: (size * 0.05).max(1.0),
                color,
                kind: super::types::DecorationKind::Underline,
            });
        }
        if run.format.strikethrough == Some(true) || is_deleted {
            decorations.push(super::types::TextDecoration {
                x: seg_start_x,
                y: baseline_y - line_ascent * 0.35,
                width: seg_end_x - seg_start_x,
                height: (size * 0.05).max(1.0),
                color,
                kind: super::types::DecorationKind::Strikethrough,
            });
        }

        run_map.push((seg_start_x, cursor_x, run.id, char_offset));
    }

    let ascent = line_ascent.max(default_size);
    let descent = line_descent.max(default_size * 0.25);
    let line_height = (ascent + descent + line_gap).max(default_size * 1.2);
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
        justify_stops,
        decorations,
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
    if lines.is_empty() {
        return;
    }
    let last = lines.len() - 1;
    for (index, line) in lines.iter_mut().enumerate() {
        let offset = match alignment {
            Alignment::Left => 0.0,
            Alignment::Center => (max_width - line.width) / 2.0,
            Alignment::Right => max_width - line.width,
            Alignment::Justify if index < last => {
                justify_line(line, max_width);
                0.0
            }
            Alignment::Justify => 0.0,
        };
        if offset.abs() < f32::EPSILON {
            continue;
        }
        line.x += offset;
        for g in &mut line.glyphs {
            g.x += offset;
        }
        for entry in &mut line.run_map {
            entry.0 += offset;
            entry.1 += offset;
        }
        for stop in &mut line.justify_stops {
            *stop += offset;
        }
        for deco in &mut line.decorations {
            deco.x += offset;
        }
    }
}

fn justify_line(line: &mut super::types::TextLine, target_width: f32) {
    if line.justify_stops.is_empty() || line.width >= target_width {
        return;
    }
    let extra_per = (target_width - line.width) / line.justify_stops.len() as f32;
    for stop in &line.justify_stops {
        for g in &mut line.glyphs {
            if g.x >= *stop - 0.01 {
                g.x += extra_per;
            }
        }
        for entry in &mut line.run_map {
            if entry.0 >= *stop - 0.01 {
                entry.0 += extra_per;
            }
            if entry.1 >= *stop - 0.01 {
                entry.1 += extra_per;
            }
        }
        for deco in &mut line.decorations {
            if deco.x >= *stop - 0.01 {
                deco.x += extra_per;
            }
        }
    }
    line.width = target_width;
}

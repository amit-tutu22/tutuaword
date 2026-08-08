use tw_model::{Alignment, FieldEvalContext, LineSpacing, Paragraph, RevisionType, TabStop, paragraph_layout_text, run_layout_text};
use tw_shape::{AtlasKey, GlyphAtlas, TextShaper};
use unicode_linebreak::{linebreaks, BreakOpportunity};

const MARKER_GUTTER: f32 = 24.0;

/// Word's default tab grid (`w:defaultTabStop` of 720 twips), measured from the
/// paragraph's text origin. Explicit `w:tabs` stops override the grid when set.
pub const DEFAULT_TAB_INTERVAL: f32 = 36.0;

/// Next tab stop strictly after `cursor_x`, preferring explicit stops over the
/// default grid.
fn next_tab_stop(
    cursor_x: f32,
    origin_x: f32,
    tab_interval: f32,
    tab_stops: &[TabStop],
) -> f32 {
    let mut explicit = None;
    for stop in tab_stops {
        let pos = origin_x + stop.position;
        if pos > cursor_x + 0.01 {
            explicit = Some(explicit.map_or(pos, |best: f32| best.min(pos)));
        }
    }
    if let Some(pos) = explicit {
        return pos;
    }
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
#[derive(Debug, Clone)]
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
    /// Explicit tab stops from paragraph formatting.
    pub tab_stops: Vec<TabStop>,
    /// Dynamic field evaluation (PAGE, DATE, …) during layout.
    pub field_context: Option<FieldEvalContext>,
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
            tab_stops: Vec::new(),
            field_context: None,
        }
    }

    pub fn with_tab_interval(mut self, tab_interval: f32) -> Self {
        self.tab_interval = tab_interval.max(1.0);
        self
    }

    pub fn with_tab_stops(mut self, tab_stops: Vec<TabStop>) -> Self {
        self.tab_stops = tab_stops;
        self
    }

    pub fn with_field_context(mut self, field_context: FieldEvalContext) -> Self {
        self.field_context = Some(field_context);
        self
    }

    pub fn indented_in(column_x: f32, indent: f32, y: f32, column_width: f32) -> Self {
        Self {
            x: column_x + indent,
            y,
            max_width: column_width - indent,
            tab_origin: column_x,
            tab_interval: DEFAULT_TAB_INTERVAL,
            tab_stops: Vec::new(),
            field_context: None,
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
        tab_stops,
        field_context,
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

    let text = paragraph_layout_text(para, frame.field_context.as_ref());
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
            decorative: false,
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
            current_y += emit_line_wrapped(
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
                    &tab_stops,
                    field_context,
                ),
                font_id,
                default_color,
                true,
                max_width,
                line_height,
                &mut lines,
            );
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
            &tab_stops,
            font_id,
            field_context,
        );

        if width <= max_width {
            last_fit = Some(byte_idx);
            continue;
        }

        // Close the line at the last opportunity that fit, if there was one.
        let mut pending_width = width;
        if let Some(fit) = last_fit.filter(|&fit| fit > line_start) {
            emit_line(
                shaper,
                atlas,
                atlas_ctx(
                    para,
                    &text,
                    line_start,
                    fit,
                    x,
                    current_y + ascent,
                    tab_origin,
                    tab_interval,
                    &tab_stops,
                    field_context,
                ),
                font_id,
                default_color,
                true,
                line_height,
                &mut lines,
            );
            current_y += line_height;
            line_start = fit;
            pending_width = measure_range(
                shaper,
                para,
                line_start,
                trim_trailing(&text, line_start, byte_idx),
                tab_origin - x,
                tab_interval,
                &tab_stops,
                font_id,
                field_context,
            );
        }

        // What is left before this opportunity can still exceed the column when
        // it holds no opportunity of its own. Closing the line at `fit` only
        // proves the text before `fit` fit, not the token after it.
        if pending_width > max_width {
            current_y += emit_line_wrapped(
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
                    &tab_stops,
                    field_context,
                ),
                font_id,
                default_color,
                true,
                max_width,
                line_height,
                &mut lines,
            );
            line_start = byte_idx;
            last_fit = None;
        } else {
            last_fit = Some(byte_idx);
        }
    }

    if line_start < text.len() {
        // Keep trailing whitespace on the paragraph's last line so a typed space
        // is visible and the caret can advance before the next character.
        current_y += emit_line_wrapped(
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
                &tab_stops,
                field_context,
            ),
            font_id,
            default_color,
            false,
            max_width,
            line_height,
            &mut lines,
        );
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
            decorative: false,
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
        Some(_) => single,
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
    tab_stops: &'a [TabStop],
    field_context: Option<FieldEvalContext>,
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
    tab_stops: &'a [TabStop],
    field_context: Option<FieldEvalContext>,
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
        tab_stops,
        field_context,
    }
}

fn emit_line(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    ctx: LineContext<'_>,
    font_id: Option<tw_shape::FontId>,
    default_color: u32,
    trim_end: bool,
    // Paragraph line-spacing rule must win over font-metric height from
    // `shape_line` — Exact is an absolute height.
    line_height: f32,
    lines: &mut Vec<super::types::TextLine>,
) {
    let end = if trim_end && ctx.end < ctx.text.len() {
        trim_trailing(ctx.text, ctx.start, ctx.end)
    } else {
        ctx.end
    };
    let line_text = &ctx.text[ctx.start..end];
    let (mut line, _) = shape_line(
        shaper,
        atlas,
        ctx.para,
        line_text,
        ctx.start,
        ctx.x,
        ctx.baseline_y,
        ctx.tab_origin,
        ctx.tab_interval,
        ctx.tab_stops,
        font_id,
        default_color,
        ctx.field_context,
    );
    line.line_height = line_height;
    lines.push(line);
}

/// Last character boundary in `start..end` whose text still fits `max_width`.
///
/// Only used for a token with no break opportunity of its own, so the choice is
/// between breaking mid-character-cluster and running past the margin. Advances
/// by at least one character: a column narrower than a single glyph must still
/// make progress rather than spin.
#[allow(clippy::too_many_arguments)]
fn break_overlong(
    shaper: &mut TextShaper,
    para: &Paragraph,
    text: &str,
    start: usize,
    end: usize,
    max_width: f32,
    tab_origin_offset: f32,
    tab_interval: f32,
    tab_stops: &[TabStop],
    font_id: Option<tw_shape::FontId>,
    field_context: Option<FieldEvalContext>,
) -> usize {
    // Interior boundaries only: `start` would make no progress and `end` is
    // already known not to fit.
    let boundaries: Vec<usize> = text[start..end]
        .char_indices()
        .skip(1)
        .map(|(offset, _)| start + offset)
        .collect();
    if boundaries.is_empty() {
        return end;
    }
    // Advances are non-negative, so width grows monotonically with the boundary
    // and the fitting boundaries form a prefix.
    let fitting = boundaries.partition_point(|&candidate| {
        measure_range(
            shaper,
            para,
            start,
            candidate,
            tab_origin_offset,
            tab_interval,
            tab_stops,
            font_id,
            field_context,
        ) <= max_width
    });
    if fitting == 0 {
        boundaries[0]
    } else {
        boundaries[fitting - 1]
    }
}

/// Emits `ctx.start..ctx.end`, splitting it across lines when it is wider than
/// `max_width` and holds no break opportunity to split on — a long URL or an
/// unspaced token. Word breaks those at the margin instead of letting them run
/// off the page. Returns the total baseline height the emitted lines consumed.
#[allow(clippy::too_many_arguments)]
fn emit_line_wrapped(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    ctx: LineContext<'_>,
    font_id: Option<tw_shape::FontId>,
    default_color: u32,
    trim_end: bool,
    max_width: f32,
    line_height: f32,
    lines: &mut Vec<super::types::TextLine>,
) -> f32 {
    let mut start = ctx.start;
    let mut baseline_y = ctx.baseline_y;
    let mut consumed = 0.0;
    while start < ctx.end {
        let measured_end = trim_trailing(ctx.text, start, ctx.end);
        let width = measure_range(
            shaper,
            ctx.para,
            start,
            measured_end,
            ctx.tab_origin - ctx.x,
            ctx.tab_interval,
            ctx.tab_stops,
            font_id,
            ctx.field_context,
        );
        if width <= max_width {
            break;
        }
        let break_at = break_overlong(
            shaper,
            ctx.para,
            ctx.text,
            start,
            ctx.end,
            max_width,
            ctx.tab_origin - ctx.x,
            ctx.tab_interval,
            ctx.tab_stops,
            font_id,
            ctx.field_context,
        );
        if break_at <= start || break_at >= ctx.end {
            break;
        }
        emit_line(
            shaper,
            atlas,
            atlas_ctx(
                ctx.para,
                ctx.text,
                start,
                break_at,
                ctx.x,
                baseline_y,
                ctx.tab_origin,
                ctx.tab_interval,
                ctx.tab_stops,
                ctx.field_context,
            ),
            font_id,
            default_color,
            true,
            line_height,
            lines,
        );
        baseline_y += line_height;
        consumed += line_height;
        start = break_at;
    }
    emit_line(
        shaper,
        atlas,
        atlas_ctx(
            ctx.para,
            ctx.text,
            start,
            ctx.end,
            ctx.x,
            baseline_y,
            ctx.tab_origin,
            ctx.tab_interval,
            ctx.tab_stops,
            ctx.field_context,
        ),
        font_id,
        default_color,
        trim_end,
        line_height,
        lines,
    );
    consumed + line_height
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
    tab_stops: &[TabStop],
    font_id: Option<tw_shape::FontId>,
    field_context: Option<FieldEvalContext>,
) -> f32 {
    let Some(fid) = font_id.or_else(|| shaper.default_font()) else {
        return 0.0;
    };
    // Widths are relative to the line origin, so the tab grid resolves against
    // the column origin expressed in the same space.
    let mut width = 0.0;
    for (segment_text, run, _) in run_segments_for_range(para, start_byte, end_byte, field_context) {
        if segment_text.is_empty() {
            continue;
        }
        let base_size = run.format.font_size.unwrap_or(12.0);
        let (size_scale, _) = script_scale(&run.format);
        let size = base_size * size_scale;
        let run_font = font_for(shaper, &run.format, fid);
        let mut shape_format = run.format.clone();
        shape_format.font_size = Some(size);
        for (piece_index, piece) in segment_text.split('\t').enumerate() {
            if piece_index > 0 {
                width = next_tab_stop(width, tab_origin_offset, tab_interval, tab_stops);
            }
            if piece.is_empty() {
                continue;
            }
            let shaped = shaper.shape(piece, &shape_format, run_font);
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
    marker_format: &tw_model::CharFormat,
) {
    if lines.is_empty() || marker.is_empty() {
        return;
    }
    let marker_para = Paragraph::with_text(marker.to_string());
    let baseline = lines[0].y;
    let font_id = shaper
        .default_font()
        .map(|fallback| font_for(shaper, marker_format, fallback));
    let marker_color = marker_format
        .color
        .map(|c| c.to_argb())
        .unwrap_or(default_color);
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
        &[],
        font_id,
        marker_color,
        None,
    );
    lines[0].glyphs.append(&mut marker_line.glyphs);
    lines[0].list_marker = Some(marker.to_string());
    let _ = MARKER_GUTTER;
}

fn script_scale(format: &tw_model::CharFormat) -> (f32, f32) {
    if format.superscript == Some(true) {
        (0.65, -0.35)
    } else if format.subscript == Some(true) {
        (0.65, 0.15)
    } else {
        (1.0, 0.0)
    }
}

/// A line with no glyphs, used when no face is available to shape with.
fn blank_line(
    para: &Paragraph,
    x: f32,
    baseline_y: f32,
    size: f32,
) -> super::types::TextLine {
    let ascent = size;
    let descent = size * 0.25;
    super::types::TextLine {
        y: baseline_y,
        x,
        width: 0.0,
        ascent,
        descent,
        line_height: (ascent + descent).max(size * 1.2),
        glyphs: Vec::new(),
        paragraph_id: para.id,
        run_map: para
            .runs
            .first()
            .map(|run| vec![(x, x, run.id, 0)])
            .unwrap_or_default(),
        list_marker: None,
        justify_stops: Vec::new(),
        decorations: Vec::new(),
        decorative: false,
    }
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
    tab_stops: &[TabStop],
    font_id: Option<tw_shape::FontId>,
    default_color: u32,
    field_context: Option<FieldEvalContext>,
) -> (super::types::TextLine, f32) {
    let default_size = para.runs.first().and_then(|r| r.format.font_size).unwrap_or(12.0);
    let Some(fid) = font_id.or_else(|| shaper.default_font()) else {
        // No face at all — a host that has registered no fonts yet. The document
        // still paginates against nominal metrics; it just has nothing to draw.
        return (blank_line(para, x, baseline_y, default_size), 0.0);
    };
    let mut cursor_x = x;
    let mut glyphs = Vec::new();
    let mut run_map = Vec::new();
    let line_end_byte = line_start_byte + line_text.len();

    let segments = run_segments_for_range(para, line_start_byte, line_end_byte, field_context);
    let mut line_ascent = 0.0f32;
    let mut line_descent = 0.0f32;
    let mut line_gap = 0.0f32;
    let mut justify_stops = Vec::new();
    let mut decorations = Vec::new();

    for (segment_text, run, char_offset) in segments {
        if segment_text.is_empty() {
            continue;
        }
        let base_size = run.format.font_size.unwrap_or(default_size);
        let (size_scale, baseline_frac) = script_scale(&run.format);
        let size = base_size * size_scale;
        let baseline_shift = baseline_frac * base_size;
        let run_font = font_for(shaper, &run.format, fid);
        let (run_ascent, run_descent, run_gap) = shaper.vertical_metrics(run_font, size);
        line_ascent = line_ascent.max(run_ascent + baseline_shift.abs());
        line_descent = line_descent.max(run_descent + baseline_shift.max(0.0));
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
                cursor_x = next_tab_stop(cursor_x, tab_origin, tab_interval, tab_stops);
            }
            if piece.is_empty() {
                continue;
            }

            let piece_chars: Vec<char> = piece.chars().collect();
            let mut shape_format = run.format.clone();
            shape_format.font_size = Some(size);
            let shaped = shaper.shape(piece, &shape_format, run_font);
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
                        y: baseline_y + baseline_shift + g.y_offset - entry.bearing_y,
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
                if let Some(spacing) = run.format.character_spacing {
                    cursor_x += spacing;
                }
                if codepoint == ' ' {
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
        if let Some(style) = run.format.underline {
            if style != tw_model::UnderlineStyle::None {
                let thickness = (size * 0.05).max(1.0);
                let base_y = baseline_y + (line_descent * 0.25);
                match style {
                    tw_model::UnderlineStyle::Double => {
                        decorations.push(super::types::TextDecoration {
                            x: seg_start_x,
                            y: base_y,
                            width: seg_end_x - seg_start_x,
                            height: thickness,
                            color,
                            kind: super::types::DecorationKind::DoubleUnderline,
                        });
                        decorations.push(super::types::TextDecoration {
                            x: seg_start_x,
                            y: base_y + thickness + 1.0,
                            width: seg_end_x - seg_start_x,
                            height: thickness,
                            color,
                            kind: super::types::DecorationKind::DoubleUnderline,
                        });
                    }
                    _ => {
                        decorations.push(super::types::TextDecoration {
                            x: seg_start_x,
                            y: base_y,
                            width: seg_end_x - seg_start_x,
                            height: thickness,
                            color,
                            kind: super::types::DecorationKind::Underline,
                        });
                    }
                }
            }
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
        decorative: false,
    };
    (line, width)
}

fn run_segments_for_range(
    para: &Paragraph,
    start_byte: usize,
    end_byte: usize,
    field_context: Option<FieldEvalContext>,
) -> Vec<(String, tw_model::Run, usize)> {
    let mut segments = Vec::new();
    let mut byte_cursor = 0usize;

    for run in &para.runs {
        let run_text = run_layout_text(run, field_context.as_ref());
        let run_start = byte_cursor;
        let run_end = byte_cursor + run_text.len();
        byte_cursor = run_end;

        if run_end <= start_byte || run_start >= end_byte {
            continue;
        }
        if run.format.hidden == Some(true) {
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
            _ => 0.0,
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

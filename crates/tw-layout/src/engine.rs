use crate::line::{apply_list_markers, layout_paragraph, ParagraphFrame};
use crate::tables::layout_table_slice;
use crate::types::{
    DocumentLayout, ImageLayout, LayoutBox, LineMap, PageIndex, PageLayout, ShapeLayout,
    TextLine,
};
use std::collections::HashMap;
use tw_model::{
    AnchorOrigin, Block, BreakType, Document, FieldEvalContext, HeaderFooter, HeaderFooterType,
    NodeId, Paragraph, Run, RunContent, SectionFormat, TextWrap, format_list_marker,
};
use tw_shape::{FontFaceSpec, FontId, FontRegistrationError, GlyphAtlas, TextShaper};

/// Maximum pages to synchronously reflow during incremental layout (R1.1).
const MAX_INCREMENTAL_REFLOW_PAGES: usize = 3;
/// Gap between a square-wrapped image and text beside it.
const IMAGE_TEXT_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct WrapObstacle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Flow state at a block boundary. Everything downstream of a boundary is a pure
/// function of this state, so two equal continuations imply an identical tail
/// layout — that is the convergence test that bounds incremental reflow.
///
/// Completed pages are deliberately *not* stored here: the prefix is taken from
/// `last_layout` at resume time, which keeps checkpoint memory O(1) per block and
/// stops a resume from resurrecting pages that a later pass already replaced.
#[derive(Debug, Clone)]
struct ColumnFlow {
    count: u32,
    gap: f32,
    width: f32,
    index: u32,
    x: f32,
}

impl ColumnFlow {
    fn from_format(format: &SectionFormat) -> Self {
        let count = format.columns.count.clamp(1, 3);
        let gap = if count > 1 {
            format.columns.gap.max(0.0)
        } else {
            0.0
        };
        let content = format.page_width - format.margin_left - format.margin_right;
        let total_gap = gap * (count.saturating_sub(1) as f32);
        let width = (content - total_gap) / count as f32;
        Self {
            count,
            gap,
            width: width.max(1.0),
            index: 0,
            x: format.margin_left,
        }
    }

    fn reset(&mut self, format: &SectionFormat) {
        *self = Self::from_format(format);
    }

    fn next_column(&mut self, format: &SectionFormat) -> bool {
        if self.index + 1 >= self.count {
            return false;
        }
        self.index += 1;
        self.x = format.margin_left + (self.width + self.gap) * self.index as f32;
        true
    }
}

#[derive(Debug, Clone)]
struct Continuation {
    page_index: PageIndex,
    y: f32,
    format: SectionFormat,
    column_flow: ColumnFlow,
    list_counters: HashMap<(u32, u32), u32>,
    current_boxes: Vec<LayoutBox>,
    current_lines: Vec<TextLine>,
    wrap_obstacles: Vec<WrapObstacle>,
}

fn page_geometry(format: &SectionFormat) -> [f32; 8] {
    [
        format.page_width,
        format.page_height,
        format.margin_top,
        format.margin_bottom,
        format.margin_left,
        format.margin_right,
        format.columns.count as f32,
        format.columns.gap,
    ]
}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        self.page_index == other.page_index
            && self.y == other.y
            && page_geometry(&self.format) == page_geometry(&other.format)
            && self.column_flow.index == other.column_flow.index
            && self.column_flow.x == other.column_flow.x
            && self.list_counters == other.list_counters
            && self.current_boxes == other.current_boxes
            && self.current_lines == other.current_lines
            && self.wrap_obstacles == other.wrap_obstacles
    }
}

fn advance_flow_y(
    y: &mut f32,
    column_flow: &mut ColumnFlow,
    format: &SectionFormat,
    pages: &mut Vec<PageLayout>,
    current_boxes: &mut Vec<LayoutBox>,
    current_lines: &mut Vec<TextLine>,
    page_index: &mut PageIndex,
    section_index: usize,
    section: &tw_model::Section,
    tab_interval: f32,
    doc: &Document,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    line_number: &mut u32,
    section_first_flush: &mut bool,
    wrap_obstacles: &mut Vec<WrapObstacle>,
) -> bool {
    if column_flow.next_column(format) {
        *y = format.margin_top;
        true
    } else {
        flush_page(
            pages,
            current_boxes,
            current_lines,
            *page_index,
            section_index,
            section,
            tab_interval,
            doc,
            shaper,
            atlas,
            line_number,
            section_first_flush,
        );
        *page_index += 1;
        column_flow.reset(format);
        wrap_obstacles.clear();
        *y = format.margin_top;
        false
    }
}

fn advance_flow_lines(
    lines: &mut [TextLine],
    line_idx: usize,
    y: &mut f32,
    column_flow: &mut ColumnFlow,
    format: &SectionFormat,
    pages: &mut Vec<PageLayout>,
    current_boxes: &mut Vec<LayoutBox>,
    current_lines: &mut Vec<TextLine>,
    page_index: &mut PageIndex,
    section_index: usize,
    section: &tw_model::Section,
    tab_interval: f32,
    doc: &Document,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    line_number: &mut u32,
    section_first_flush: &mut bool,
    wrap_obstacles: &mut Vec<WrapObstacle>,
) {
    let old_x = column_flow.x;
    let same_page = advance_flow_y(
        y,
        column_flow,
        format,
        pages,
        current_boxes,
        current_lines,
        page_index,
        section_index,
        section,
        tab_interval,
        doc,
        shaper,
        atlas,
        line_number,
        section_first_flush,
        wrap_obstacles,
    );
    if same_page {
        rebase_lines_to_column(lines, line_idx, *y, column_flow.x, old_x);
    } else {
        rebase_lines_from(lines, line_idx, *y);
    }
}

/// Shift remaining paragraph lines (and their glyphs/decorations) to a new page origin.
fn rebase_lines_from(lines: &mut [TextLine], start: usize, new_origin_y: f32) {
    if start >= lines.len() {
        return;
    }
    let dy = new_origin_y - lines[start].y;
    if dy.abs() < 0.01 {
        return;
    }
    for line in &mut lines[start..] {
        line.y += dy;
        for glyph in &mut line.glyphs {
            glyph.y += dy;
        }
        for decoration in &mut line.decorations {
            decoration.y += dy;
        }
    }
}

/// Move remaining lines into the next column on the same page.
fn rebase_lines_to_column(lines: &mut [TextLine], start: usize, new_y: f32, new_x: f32, old_x: f32) {
    if start >= lines.len() {
        return;
    }
    rebase_lines_from(lines, start, new_y);
    let dx = new_x - old_x;
    if dx.abs() < 0.01 {
        return;
    }
    for line in &mut lines[start..] {
        line.x += dx;
        for glyph in &mut line.glyphs {
            glyph.x += dx;
        }
        for decoration in &mut line.decorations {
            decoration.x += dx;
        }
    }
}

/// Shading fill and border strokes for one page batch of a paragraph's lines.
fn push_paragraph_decoration_boxes(
    boxes: &mut Vec<LayoutBox>,
    lines: &[TextLine],
    start: usize,
    count: usize,
    box_x: f32,
    box_width: f32,
    shading: Option<tw_model::Color>,
    borders: Option<&tw_model::BorderSet>,
) {
    if count == 0 || box_width <= 0.0 {
        return;
    }
    let first = &lines[start];
    let last = &lines[start + count - 1];
    let top = first.y - first.ascent;
    let bottom = last.y + last.descent;
    let height = (bottom - top).max(1.0);

    if let Some(fill) = shading.filter(|c| c.a > 0) {
        boxes.push(LayoutBox::Rect {
            x: box_x,
            y: top,
            width: box_width,
            height,
            color: fill.to_argb(),
        });
    }

    let Some(border_set) = borders.filter(|b| b.any()) else {
        return;
    };

    if let Some(spec) = &border_set.top {
        let w = spec.width.max(0.5);
        boxes.push(LayoutBox::Rect {
            x: box_x,
            y: top,
            width: box_width,
            height: w,
            color: spec.color.to_argb(),
        });
    }
    if let Some(spec) = &border_set.bottom {
        let w = spec.width.max(0.5);
        boxes.push(LayoutBox::Rect {
            x: box_x,
            y: bottom - w,
            width: box_width,
            height: w,
            color: spec.color.to_argb(),
        });
    }
    if let Some(spec) = &border_set.left {
        let w = spec.width.max(0.5);
        boxes.push(LayoutBox::Rect {
            x: box_x,
            y: top,
            width: w,
            height,
            color: spec.color.to_argb(),
        });
    }
    if let Some(spec) = &border_set.right {
        let w = spec.width.max(0.5);
        boxes.push(LayoutBox::Rect {
            x: box_x + box_width - w,
            y: top,
            width: w,
            height,
            color: spec.color.to_argb(),
        });
    }
}

pub struct LayoutEngine {
    shaper: TextShaper,
    atlas: GlyphAtlas,
    page_cache: HashMap<PageIndex, PageLayout>,
    line_maps: HashMap<PageIndex, LineMap>,
    dirty_pages: Vec<PageIndex>,
    last_layout: DocumentLayout,
    continuations: HashMap<(usize, usize), Continuation>,
    block_first_page: HashMap<(usize, usize), PageIndex>,
    incremental_from: Option<(usize, usize)>,
    /// Block the last capped pass stopped at; forward relayout resumes here.
    resume_at: Option<(usize, usize)>,
    /// First block whose stored continuation is stale because a capped pass never
    /// reached it. Resuming at or before this block is safe; beyond it is not.
    stale_from: Option<(usize, usize)>,
    /// First page whose geometry predates the edit that opened the pending reflow.
    /// Survives later converged passes, which rebuild a shorter prefix but leave
    /// the pages an earlier capped pass already rebuilt correctly in place.
    pending_reflow_page: Option<PageIndex>,
    last_pass_incremental: bool,
    last_relayout_pages: usize,
    last_relayout_start_page: usize,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    /// A layout engine over the operating system's installed fonts.
    pub fn new() -> Self {
        Self::with_shaper(TextShaper::new())
    }

    /// A layout engine whose only fonts are the ones the host registers from
    /// bytes with [`LayoutEngine::register_face`]. No system font scan, no
    /// filesystem — the web path.
    ///
    /// A document laid out before any face is registered paginates normally but
    /// produces lines with no glyphs; it does not panic.
    pub fn with_injected_fonts() -> Self {
        Self::with_shaper(TextShaper::with_injected_fonts())
    }

    pub fn with_shaper(shaper: TextShaper) -> Self {
        Self {
            shaper,
            atlas: GlyphAtlas::default(),
            page_cache: HashMap::new(),
            line_maps: HashMap::new(),
            dirty_pages: vec![0],
            last_layout: DocumentLayout::default(),
            continuations: HashMap::new(),
            block_first_page: HashMap::new(),
            incremental_from: None,
            resume_at: None,
            stale_from: None,
            pending_reflow_page: None,
            last_pass_incremental: false,
            last_relayout_pages: 0,
            last_relayout_start_page: 0,
        }
    }

    /// Registers a face from host-supplied bytes. See
    /// [`tw_shape::FontDatabase::register_face`].
    ///
    /// Cached layout is discarded: a newly resolvable family changes metrics on
    /// pages that were laid out without it.
    pub fn register_face(
        &mut self,
        spec: &FontFaceSpec,
        data: impl Into<Vec<u8>>,
    ) -> Result<FontId, FontRegistrationError> {
        let font = self.shaper.register_face(spec, data)?;
        self.invalidate_all();
        Ok(font)
    }

    /// Registers every face in `data` under the names in the font file. See
    /// [`tw_shape::FontDatabase::register_font_data`].
    pub fn register_font_data(
        &mut self,
        data: impl Into<Vec<u8>>,
    ) -> Result<Vec<FontId>, FontRegistrationError> {
        let fonts = self.shaper.register_font_data(data)?;
        self.invalidate_all();
        Ok(fonts)
    }

    pub fn shaper(&self) -> &TextShaper {
        &self.shaper
    }

    pub fn shaper_mut(&mut self) -> &mut TextShaper {
        &mut self.shaper
    }

    pub fn invalidate_all(&mut self) {
        self.dirty_pages = vec![0];
        self.page_cache.clear();
        self.line_maps.clear();
        self.last_layout = DocumentLayout::default();
        self.continuations.clear();
        self.block_first_page.clear();
        self.incremental_from = None;
        self.resume_at = None;
        self.stale_from = None;
        self.pending_reflow_page = None;
        self.last_pass_incremental = false;
        self.last_relayout_pages = 0;
        self.last_relayout_start_page = 0;
    }

    /// Mark blocks containing `node_ids` for incremental reflow on the next layout pass.
    pub fn invalidate_nodes(&mut self, doc: &Document, node_ids: &[NodeId]) {
        if node_ids.is_empty() {
            return;
        }
        let mut earliest: Option<(usize, usize)> = None;
        for &node_id in node_ids {
            let loc = doc
                .find_run_location(node_id)
                .map(|loc| (loc.section_index, loc.block_index))
                .or_else(|| doc.find_paragraph_location(node_id))
                .or_else(|| doc.find_block_location(node_id));
            if let Some(pos) = loc {
                earliest = Some(match earliest {
                    None => pos,
                    Some(cur) if pos < cur => pos,
                    Some(cur) => cur,
                });
            }
        }
        self.incremental_from = earliest;
    }

    /// Pages synchronously reflowed by the most recent layout pass.
    pub fn last_relayout_pages(&self) -> usize {
        self.last_relayout_pages
    }

    /// First page index rebuilt during the most recent incremental pass.
    pub fn last_relayout_start_page(&self) -> usize {
        self.last_relayout_start_page
    }

    /// First page index touched by the most recent layout pass.
    pub fn relayout_start_page(&self) -> PageIndex {
        self.dirty_pages.first().copied().unwrap_or(0)
    }

    /// Page indices rebuilt by the most recent layout pass.
    pub fn dirty_pages(&self) -> &[PageIndex] {
        &self.dirty_pages
    }

    pub fn last_pass_incremental(&self) -> bool {
        self.last_pass_incremental
    }

    /// True while the page cap left downstream pages carrying pre-edit content.
    pub fn has_pending_reflow(&self) -> bool {
        self.resume_at.is_some()
    }

    /// First page still carrying pre-edit geometry, or `None` when the layout is
    /// fully caught up. Pages before this index were rebuilt by some pass and are
    /// safe to hit test even while a forward reflow is owed.
    pub fn pending_reflow_page(&self) -> Option<PageIndex> {
        self.resume_at.and(self.pending_reflow_page)
    }

    /// Reflow the next chunk of pages left behind by a capped incremental pass.
    /// Returns `None` when nothing is pending.
    pub fn continue_layout(&mut self, doc: &Document) -> Option<DocumentLayout> {
        let resume = self.resume_at?;
        self.incremental_from = Some(resume);
        Some(self.layout_document(doc))
    }

    pub fn layout_document(&mut self, doc: &Document) -> DocumentLayout {
        let minor_font = doc
            .styles
            .defaults
            .char_format
            .font_family
            .as_deref()
            .unwrap_or(&doc.settings.theme.minor_font);
        self.shaper.configure_from_theme(minor_font, &doc.settings.theme.major_font);
        let tab_interval = doc.settings.default_tab_stop.max(1.0);
        let default_format = doc
            .sections
            .first()
            .map(|s| s.format.clone())
            .unwrap_or_default();

        let old_pages = std::mem::take(&mut self.last_layout.pages);
        let resume = self.incremental_from.take();
        let incremental_start = resume
            .filter(|from| self.can_resume_at(*from, old_pages.len()))
            .and_then(|from| self.continuations.get(&from).cloned().map(|cp| (cp, from)));
        let incremental = incremental_start.is_some();

        let (mut format, mut pages, mut current_boxes, mut current_lines, mut page_index, mut y, mut column_flow, mut list_counters, mut wrap_obstacles, start_section, start_block) =
            if let Some((cp, start)) = incremental_start {
                (
                    cp.format,
                    old_pages[..cp.page_index as usize].to_vec(),
                    cp.current_boxes,
                    cp.current_lines,
                    cp.page_index,
                    cp.y,
                    cp.column_flow,
                    cp.list_counters,
                    cp.wrap_obstacles,
                    start.0,
                    start.1,
                )
            } else {
                self.continuations.clear();
                self.block_first_page.clear();
                (
                    default_format.clone(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    0,
                    default_format.margin_top,
                    ColumnFlow::from_format(&default_format),
                    HashMap::new(),
                    Vec::new(),
                    0,
                    0,
                )
            };

        let mut line_number = default_format.line_numbers.start;
        let relayout_base = pages.len();
        // Convergence is only trustworthy against continuations a previous pass
        // actually reached; beyond `stale_from` the stored values are pre-edit.
        let convergence_limit = if incremental { self.stale_from } else { None };
        let mut converged_at: Option<PageIndex> = None;
        let mut resume_at: Option<(usize, usize)> = None;
        let mut current_section = doc.sections.first();
        let mut current_section_index = 0usize;
        let mut section_first_flush = true;

        'sections: for (section_idx, section) in doc.sections.iter().enumerate() {
            current_section = Some(section);
            current_section_index = section_idx;
            if section_idx < start_section {
                continue;
            }
            if section_idx > 0 && (section_idx > start_section || !incremental) {
                if !current_boxes.is_empty() {
                    flush_page(
                        &mut pages,
                        &mut current_boxes,
                        &mut current_lines,
                        page_index,
                        section_idx,
                        section,
                        tab_interval,
                        doc,
                        &mut self.shaper,
                        &mut self.atlas,
                        &mut line_number,
                        &mut section_first_flush,
                    );
                    page_index += 1;
                }
                format = section.format.clone();
                y = format.margin_top;
                column_flow.reset(&format);
                wrap_obstacles.clear();
                line_number = format.line_numbers.start;
                section_first_flush = true;
            }

            let content_bottom = format.page_height - format.margin_bottom;

            for (block_idx, block) in section.blocks.iter().enumerate() {
                if section_idx == start_section && block_idx < start_block {
                    continue;
                }

                let key = (section_idx, block_idx);
                let continuation = Continuation {
                    page_index,
                    y,
                    format: format.clone(),
                    column_flow: column_flow.clone(),
                    list_counters: list_counters.clone(),
                    current_boxes: current_boxes.clone(),
                    current_lines: current_lines.clone(),
                    wrap_obstacles: wrap_obstacles.clone(),
                };
                let previous = self.continuations.insert(key, continuation.clone());
                self.block_first_page.entry(key).or_insert(page_index);

                if incremental && key != (start_section, start_block) {
                    let comparable = convergence_limit.is_none_or(|limit| key < limit)
                        && pages.len() == page_index as usize;
                    if comparable && previous.as_ref() == Some(&continuation) {
                        converged_at = Some(page_index);
                        break 'sections;
                    }
                    if pages.len().saturating_sub(relayout_base) >= MAX_INCREMENTAL_REFLOW_PAGES {
                        resume_at = Some(key);
                        break 'sections;
                    }
                }

                let pages_before_block = pages.len();
                match block {
                    Block::Paragraph(para) => {
                        for segment in split_paragraph_at_page_breaks(para) {
                            if segment.page_break_before && !current_boxes.is_empty() {
                                flush_page(
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    page_index,
                                    section_idx,
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                    &mut line_number,
                                    &mut section_first_flush,
                                );
                                page_index += 1;
                                wrap_obstacles.clear();
                                column_flow.reset(&format);
                                y = format.margin_top;
                            }

                            if segment.column_break_before {
                                advance_flow_y(
                                    &mut y,
                                    &mut column_flow,
                                    &format,
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    &mut page_index,
                                    section_idx,
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                    &mut line_number,
                                    &mut section_first_flush,
                                    &mut wrap_obstacles,
                                );
                            }

                            let para = &segment.paragraph;
                            let resolved_para = doc
                                .styles
                                .resolve_para_format(para.style_id, &para.format);
                            let mut effective_para = para.clone();
                            effective_para.format = resolved_para.clone();
                            if effective_para.runs.is_empty() {
                                effective_para.runs.push(Run::new_text(""));
                            }
                            // Every run inherits the document defaults and the
                            // paragraph style, not just the first one.
                            for run in &mut effective_para.runs {
                                run.format = doc
                                    .styles
                                    .resolve_char_format(para.style_id, &run.format);
                            }

                            let level = para.format.numbering.and_then(|nr| {
                                doc.settings.numbering.get(nr.numbering_id).and_then(|d| {
                                    d.levels.iter().find(|l| l.level == nr.level)
                                })
                            });

                            let list_marker = para.format.numbering.map(|nr| {
                                let key = (nr.numbering_id, nr.level);
                                let start_at = level
                                    .map(|l| l.start.saturating_sub(1))
                                    .unwrap_or(0);
                                if para.format.num_restart == Some(true) {
                                    list_counters.insert(key, start_at);
                                }
                                let counter = list_counters.entry(key).or_insert(start_at);
                                let marker = doc
                                    .settings
                                    .numbering
                                    .get(nr.numbering_id)
                                    .map(|def| format_list_marker(def, nr.level, *counter))
                                    .unwrap_or_else(|| "•".into());
                                *counter += 1;
                                marker
                            });

                            // Indents override rather than accumulate. Direct
                            // paragraph formatting wins, then the numbering
                            // level, then the paragraph style -- Word ranks a
                            // list's own indents above the List Paragraph style
                            // it attaches to every list item.
                            let indent = para
                                .format
                                .indent_left
                                .or(level.map(|l| l.indent))
                                .or(resolved_para.indent_left)
                                .unwrap_or(0.0);
                            let hanging = para
                                .format
                                .indent_first_line
                                .map(|first| -first)
                                .or(level.map(|l| l.hanging))
                                .or(resolved_para.indent_first_line.map(|first| -first))
                                .unwrap_or(0.0)
                                .max(0.0);

                            y += resolved_para.space_before.unwrap_or(0.0);

                            if resolved_para.keep_with_next == Some(true) {
                                let next_min = next_paragraph_min_height(doc, &section.blocks, block_idx);
                                let rough_para = lines_estimated_min_height(&resolved_para);
                                if y + rough_para + next_min > content_bottom
                                    && !current_boxes.is_empty()
                                {
                                    advance_flow_y(
                                        &mut y,
                                        &mut column_flow,
                                        &format,
                                        &mut pages,
                                        &mut current_boxes,
                                        &mut current_lines,
                                        &mut page_index,
                                        section_idx,
                                        section,
                                        tab_interval,
                                        doc,
                                        &mut self.shaper,
                                        &mut self.atlas,
                                        &mut line_number,
                                        &mut section_first_flush,
                                        &mut wrap_obstacles,
                                    );
                                }
                            }

                            let field_ctx = FieldEvalContext::for_page(
                                page_index,
                                pages
                                    .len()
                                    .saturating_add(1)
                                    .max(page_index as usize + 1) as u32,
                            );
                            let (mut lines, _) = layout_paragraph(
                                &mut self.shaper,
                                &mut self.atlas,
                                &effective_para,
                                paragraph_frame_with_square_wrap(
                                    column_flow.x,
                                    indent,
                                    y,
                                    column_flow.width,
                                    &wrap_obstacles,
                                )
                                .with_tab_interval(tab_interval)
                                .with_tab_stops(
                                    resolved_para.tab_stops.clone().unwrap_or_default(),
                                )
                                .with_field_context(field_ctx),
                                tw_model::Color::BLACK.to_argb(),
                            );

                            if let Some(ref marker) = list_marker {
                                let mut marker_format = level
                                    .map(|l| l.char_format.clone())
                                    .unwrap_or_default();
                                marker_format = doc
                                    .styles
                                    .resolve_char_format(para.style_id, &marker_format);
                                apply_list_markers(
                                    &mut self.shaper,
                                    &mut self.atlas,
                                    &mut lines,
                                    marker,
                                    column_flow.x + indent - hanging,
                                    tw_model::Color::BLACK.to_argb(),
                                    &marker_format,
                                );
                                for line in &mut lines {
                                    line.list_marker = Some(marker.clone());
                                }
                            }

                            let keep_together = resolved_para.keep_together == Some(true);
                            let widow_control = resolved_para.widow_orphan_control != Some(false);

                            if keep_together && !lines.is_empty() {
                                let total: f32 = lines.iter().map(|l| l.line_height).sum();
                                if y + total > content_bottom && !current_boxes.is_empty() {
                                    advance_flow_lines(
                                        &mut lines,
                                        0,
                                        &mut y,
                                        &mut column_flow,
                                        &format,
                                        &mut pages,
                                        &mut current_boxes,
                                        &mut current_lines,
                                        &mut page_index,
                                        section_idx,
                                        section,
                                        tab_interval,
                                        doc,
                                        &mut self.shaper,
                                        &mut self.atlas,
                                        &mut line_number,
                                        &mut section_first_flush,
                                        &mut wrap_obstacles,
                                    );
                                }
                            }

                            let mut line_idx = 0usize;
                            'lines: while line_idx < lines.len() {
                                let available = content_bottom - y;
                                let mut count = 0usize;
                                let mut batch_height = 0.0f32;

                                while line_idx + count < lines.len() {
                                    let lh = lines[line_idx + count].line_height;
                                    if batch_height + lh > available && count > 0 {
                                        break;
                                    }
                                    if batch_height + lh > available
                                        && count == 0
                                        && !current_boxes.is_empty()
                                    {
                                        advance_flow_lines(
                                            &mut lines,
                                            line_idx,
                                            &mut y,
                                            &mut column_flow,
                                            &format,
                                            &mut pages,
                                            &mut current_boxes,
                                            &mut current_lines,
                                            &mut page_index,
                                            section_idx,
                                            section,
                                            tab_interval,
                                            doc,
                                            &mut self.shaper,
                                            &mut self.atlas,
                                            &mut line_number,
                                            &mut section_first_flush,
                                            &mut wrap_obstacles,
                                        );
                                        continue 'lines;
                                    }
                                    batch_height += lh;
                                    count += 1;
                                }

                                if count == 0 {
                                    count = 1;
                                    batch_height = lines[line_idx].line_height;
                                }

                                if widow_control && lines.len() > 1 {
                                    let remaining_after = lines.len() - line_idx - count;
                                    if count == 1
                                        && remaining_after >= 1
                                        && !current_boxes.is_empty()
                                    {
                                        advance_flow_lines(
                                            &mut lines,
                                            line_idx,
                                            &mut y,
                                            &mut column_flow,
                                            &format,
                                            &mut pages,
                                            &mut current_boxes,
                                            &mut current_lines,
                                            &mut page_index,
                                            section_idx,
                                            section,
                                            tab_interval,
                                            doc,
                                            &mut self.shaper,
                                            &mut self.atlas,
                                            &mut line_number,
                                            &mut section_first_flush,
                                            &mut wrap_obstacles,
                                        );
                                        continue 'lines;
                                    }
                                    if remaining_after == 1 && count > 1 {
                                        count -= 1;
                                        batch_height = lines[line_idx..line_idx + count]
                                            .iter()
                                            .map(|l| l.line_height)
                                            .sum();
                                    }
                                }

                                if keep_together && count < lines.len() - line_idx {
                                    if !current_boxes.is_empty() {
                                        advance_flow_lines(
                                            &mut lines,
                                            line_idx,
                                            &mut y,
                                            &mut column_flow,
                                            &format,
                                            &mut pages,
                                            &mut current_boxes,
                                            &mut current_lines,
                                            &mut page_index,
                                            section_idx,
                                            section,
                                            tab_interval,
                                            doc,
                                            &mut self.shaper,
                                            &mut self.atlas,
                                            &mut line_number,
                                            &mut section_first_flush,
                                            &mut wrap_obstacles,
                                        );
                                        continue 'lines;
                                    }
                                    count = lines.len() - line_idx;
                                    batch_height = lines[line_idx..]
                                        .iter()
                                        .map(|l| l.line_height)
                                        .sum();
                                }

                                let para_box_x = column_flow.x + indent;
                                let para_box_width = (column_flow.width
                                    - indent
                                    - resolved_para.indent_right.unwrap_or(0.0))
                                    .max(1.0);

                                push_paragraph_decoration_boxes(
                                    &mut current_boxes,
                                    &lines,
                                    line_idx,
                                    count,
                                    para_box_x,
                                    para_box_width,
                                    resolved_para.shading,
                                    resolved_para.borders.as_ref(),
                                );

                                for line in lines[line_idx..line_idx + count].iter().cloned() {
                                    current_lines.push(line.clone());
                                    current_boxes.push(LayoutBox::TextLine(line));
                                }
                                line_idx += count;
                                y += batch_height;

                                if line_idx < lines.len() {
                                    advance_flow_lines(
                                        &mut lines,
                                        line_idx,
                                        &mut y,
                                        &mut column_flow,
                                        &format,
                                        &mut pages,
                                        &mut current_boxes,
                                        &mut current_lines,
                                        &mut page_index,
                                        section_idx,
                                        section,
                                        tab_interval,
                                        doc,
                                        &mut self.shaper,
                                        &mut self.atlas,
                                        &mut line_number,
                                        &mut section_first_flush,
                                        &mut wrap_obstacles,
                                    );
                                }
                            }
                            y += resolved_para.space_after.unwrap_or(0.0);
                        }
                    }
                    Block::Table(table) => {
                        // Tables taller than the page are split row by row so
                        // they continue onto following pages instead of
                        // spilling past the bottom margin.
                        let mut start_row = 0usize;
                        while start_row < table.rows.len() {
                            let available = content_bottom - y;

                            // Not enough vertical room for a row — start a fresh page.
                            if available < crate::tables::MIN_ROW_HEIGHT
                                && y > format.margin_top + 0.01
                            {
                                advance_flow_y(
                                    &mut y,
                                    &mut column_flow,
                                    &format,
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    &mut page_index,
                                    section_idx,
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                    &mut line_number,
                                    &mut section_first_flush,
                                    &mut wrap_obstacles,
                                );
                                continue;
                            }

                            let slice = layout_table_slice(
                                &mut self.shaper,
                                &mut self.atlas,
                                table,
                                start_row,
                                column_flow.x,
                                y,
                                column_flow.width,
                                available,
                                tw_model::Color::BLACK.to_argb(),
                                tab_interval,
                            );

                            if slice.rows_placed == 0 {
                                break;
                            }

                            let overflows = y + slice.layout.height > content_bottom;
                            if overflows && !current_boxes.is_empty() {
                                advance_flow_y(
                                    &mut y,
                                    &mut column_flow,
                                    &format,
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    &mut page_index,
                                    section_idx,
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                    &mut line_number,
                                    &mut section_first_flush,
                                    &mut wrap_obstacles,
                                );
                                continue;
                            }

                            for cell in &slice.layout.cells {
                                for line in &cell.lines {
                                    current_lines.push(line.clone());
                                }
                            }
                            y += slice.layout.height + 8.0;
                            start_row += slice.rows_placed;
                            current_boxes.push(LayoutBox::Table(slice.layout));

                            if start_row < table.rows.len() {
                                flush_page(
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    page_index,
                                    section_idx,
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                    &mut line_number,
                                    &mut section_first_flush,
                                );
                                page_index += 1;
                                y = format.margin_top;
                                wrap_obstacles.clear();
                            }
                        }
                    }
                    Block::ImageBlock(image) => {
                        let encoded = std::sync::Arc::new(image.data.bytes.clone());
                        let (frame_w, frame_h) = image.effective_display_size();

                        // Floating images are placed absolutely and take no
                        // room in the text flow; inline ones occupy a band.
                        if let Some(anchor) = image.anchor {
                            let (ax, ay) = anchor_position(&format, anchor);
                            current_boxes.push(LayoutBox::Image(layout_image(
                                image, ax, ay, encoded.clone(),
                            )));
                            if image.wrap == TextWrap::Square {
                                wrap_obstacles.push(WrapObstacle {
                                    x: ax,
                                    y: ay,
                                    width: frame_w,
                                    height: frame_h,
                                });
                            }
                            continue;
                        }

                        if y + frame_h > content_bottom
                            && !current_boxes.is_empty()
                        {
                            flush_page(
                                &mut pages,
                                &mut current_boxes,
                                &mut current_lines,
                                page_index,
                                section_idx,
                                section,
                                tab_interval,
                                doc,
                                &mut self.shaper,
                                &mut self.atlas,
                                &mut line_number,
                                &mut section_first_flush,
                            );
                            page_index += 1;
                            y = format.margin_top;
                            wrap_obstacles.clear();
                        }

                        let img = layout_image(image, format.margin_left, y, encoded);
                        y += frame_h + 8.0;
                        current_boxes.push(LayoutBox::Image(img));
                    }
                    Block::ShapeBlock(shape) => {
                        let height = shape.shape.height;
                        if y + height > content_bottom && !current_boxes.is_empty() {
                            flush_page(
                                &mut pages,
                                &mut current_boxes,
                                &mut current_lines,
                                page_index,
                                section_idx,
                                section,
                                tab_interval,
                                doc,
                                &mut self.shaper,
                                &mut self.atlas,
                                &mut line_number,
                                &mut section_first_flush,
                            );
                            page_index += 1;
                            y = format.margin_top;
                            wrap_obstacles.clear();
                        }
                        if let Some(preview) = shape
                            .preview_image
                            .as_ref()
                            .filter(|img| !img.bytes.is_empty())
                        {
                            let encoded = std::sync::Arc::new(preview.bytes.clone());
                            current_boxes.push(LayoutBox::Image(layout_shape_preview(
                                shape,
                                preview,
                                format.margin_left,
                                y,
                                encoded,
                            )));
                        } else {
                            current_boxes.push(LayoutBox::Shape(layout_shape(
                                shape,
                                format.margin_left,
                                y,
                            )));
                        }
                        let padding = 6.0;
                        let inner_width = (shape.shape.width - padding * 2.0).max(12.0);
                        let inner = layout_shape_paragraphs(
                            doc,
                            shape,
                            &mut self.shaper,
                            &mut self.atlas,
                            format.margin_left + padding,
                            y + padding,
                            inner_width,
                            tab_interval,
                        );
                        current_boxes.extend(inner);
                        if let Some(label) = layout_diagram_label(
                            shape,
                            &mut self.shaper,
                            &mut self.atlas,
                            format.margin_left,
                            y,
                        ) {
                            current_boxes.push(label);
                        }
                        y += height + 8.0;
                    }
                    _ => {}
                }

                let _ = pages_before_block;
            }
        }

        let built_end = if let Some(page_from) = converged_at {
            // The tail is provably identical to the previous pass; take it verbatim.
            let from = (page_from as usize).min(old_pages.len());
            let built_end = pages.len();
            pages.extend(old_pages[from..].iter().cloned());
            built_end
        } else {
            if !current_boxes.is_empty() || pages.is_empty() {
                if let Some(section) = current_section {
                    flush_page(
                        &mut pages,
                        &mut current_boxes,
                        &mut current_lines,
                        page_index,
                        current_section_index,
                        section,
                        tab_interval,
                        doc,
                        &mut self.shaper,
                        &mut self.atlas,
                        &mut line_number,
                        &mut section_first_flush,
                    );
                }
            }
            let built_end = pages.len();
            // Only a capped pass leaves a tail behind; a pass that ran to the end of
            // the document is authoritative about the final page count.
            if resume_at.is_some() && pages.len() < old_pages.len() {
                pages.extend(old_pages[pages.len()..].iter().cloned());
            }
            built_end
        };
        for (idx, page) in pages.iter_mut().enumerate() {
            page.page_index = idx as PageIndex;
        }

        self.last_relayout_pages = if incremental {
            built_end.saturating_sub(relayout_base)
        } else {
            pages.len().max(1)
        };
        self.last_relayout_start_page = if incremental { relayout_base } else { 0 };

        self.dirty_pages = if incremental {
            (relayout_base..built_end).map(|i| i as PageIndex).collect()
        } else {
            (0..pages.len()).map(|i| i as PageIndex).collect()
        };

        if incremental {
            self.page_cache.retain(|idx, _| (*idx as usize) < pages.len());
            self.line_maps.retain(|idx, _| (*idx as usize) < pages.len());
            for idx in relayout_base..built_end {
                if let Some(page) = pages.get(idx) {
                    self.cache_page(page);
                }
            }
        } else {
            self.page_cache.clear();
            self.line_maps.clear();
            for page in &pages {
                self.cache_page(page);
            }
        }

        self.last_pass_incremental = incremental;
        if !incremental {
            self.resume_at = None;
            self.stale_from = None;
            self.pending_reflow_page = None;
        } else if resume_at.is_some() {
            self.resume_at = resume_at;
            self.stale_from = resume_at;
            // Pages up to `built_end` were just rebuilt, so they are current; past
            // it the tail is pre-edit. `can_resume_at` refuses to start beyond the
            // previous stale floor, so this window can never skip over a page that
            // no pass has rebuilt.
            self.pending_reflow_page = Some(built_end as PageIndex);
        } else if converged_at.is_none() {
            // Reached the end of the document: every continuation is current again.
            self.resume_at = None;
            self.stale_from = None;
            self.pending_reflow_page = None;
        }
        // A converged pass leaves `pending_reflow_page` alone: it proved the tail is
        // identical to the previous pass, so whatever was stale before still is and
        // whatever was valid before still is.

        self.last_layout = DocumentLayout { pages: append_endnotes_to_last_page(pages, doc, &mut self.shaper, &mut self.atlas, tab_interval) };
        self.last_layout.clone()
    }

    /// A resume is only sound when the stored continuation was produced by a pass
    /// that actually reached this block, and the prefix it refers to still exists.
    fn can_resume_at(&self, key: (usize, usize), available_pages: usize) -> bool {
        let Some(continuation) = self.continuations.get(&key) else {
            return false;
        };
        if continuation.page_index as usize > available_pages {
            return false;
        }
        self.stale_from.is_none_or(|stale| key <= stale)
    }

    fn cache_page(&mut self, page: &PageLayout) {
        let mut lines = Vec::new();
        for b in &page.boxes {
            match b {
                LayoutBox::TextLine(l) if !l.decorative => lines.push(l.clone()),
                LayoutBox::Table(t) => {
                    for cell in &t.cells {
                        lines.extend(cell.lines.iter().filter(|l| !l.decorative).cloned());
                    }
                }
                _ => {}
            }
        }
        self.line_maps.insert(page.page_index, LineMap { lines });
        self.page_cache.insert(page.page_index, page.clone());
    }

    pub fn page_count(&self) -> usize {
        self.last_layout.pages.len()
    }

    pub fn page_layout(&self, page: PageIndex) -> Option<&PageLayout> {
        self.page_cache.get(&page)
    }

    pub fn line_map(&self, page: PageIndex) -> Option<&LineMap> {
        self.line_maps.get(&page)
    }

    pub fn atlas(&self) -> &GlyphAtlas {
        &self.atlas
    }

    pub fn section_format(&self, doc: &Document) -> SectionFormat {
        doc.sections
            .first()
            .map(|s| s.format.clone())
            .unwrap_or_default()
    }

    pub fn document_layout(&self) -> &DocumentLayout {
        &self.last_layout
    }
}

fn flush_page(
    pages: &mut Vec<PageLayout>,
    boxes: &mut Vec<LayoutBox>,
    lines: &mut Vec<TextLine>,
    page_index: PageIndex,
    section_index: usize,
    section: &tw_model::Section,
    tab_interval: f32,
    doc: &Document,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    line_number: &mut u32,
    section_first_flush: &mut bool,
) {
    let format = &section.format;
    let content_width = format.page_width - format.margin_left - format.margin_right;
    let mut page_boxes = std::mem::take(boxes);
    let body_line_ys: Vec<f32> = page_boxes
        .iter()
        .filter_map(|b| match b {
            LayoutBox::TextLine(line) => Some(line.y),
            _ => None,
        })
        .collect();
    let margin_color = tw_model::Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    }
    .to_argb();
    let field_ctx = FieldEvalContext::for_page(
        page_index,
        pages
            .len()
            .saturating_add(1)
            .max(page_index as usize + 1) as u32,
    );

    let is_section_first_page = *section_first_flush;
    *section_first_flush = false;
    let page_number = page_index.saturating_add(1);
    let hf_type = HeaderFooterType::for_page_layout(
        page_number,
        is_section_first_page,
        format.different_first_page,
        doc.settings.even_and_odd_headers,
    );

    let header_y = format.margin_top * 0.25;
    if let Some(header_boxes) = layout_header_footer_band(
        doc.resolved_header(section_index, hf_type),
        format,
        doc,
        shaper,
        atlas,
        format.margin_left,
        header_y,
        content_width,
        tab_interval,
        margin_color,
        field_ctx,
        true,
    ) {
        for item in header_boxes.into_iter().rev() {
            page_boxes.insert(0, item);
        }
    }

    let footer_y = format.page_height - format.margin_bottom * 0.75;
    if let Some(footnote_boxes) = layout_footnote_band(
        doc,
        &page_boxes,
        format,
        shaper,
        atlas,
        format.margin_left,
        content_width,
        tab_interval,
        margin_color,
        field_ctx,
    ) {
        page_boxes.extend(footnote_boxes);
    }
    if let Some(footer_boxes) = layout_header_footer_band(
        doc.resolved_footer(section_index, hf_type),
        format,
        doc,
        shaper,
        atlas,
        format.margin_left,
        footer_y,
        content_width,
        tab_interval,
        margin_color,
        field_ctx,
        false,
    ) {
        page_boxes.extend(footer_boxes);
    }

    prepend_page_decorations(
        &mut page_boxes,
        format,
        &body_line_ys,
        line_number,
        shaper,
        atlas,
        tab_interval,
    );

    page_boxes.extend(layout_comment_margin_markers(
        &page_boxes,
        doc,
        format,
    ));

    pages.push(PageLayout {
        page_index,
        width: format.page_width,
        height: format.page_height,
        content_top: format.margin_top,
        content_left: format.margin_left,
        content_width,
        content_height: format.page_height - format.margin_top - format.margin_bottom,
        boxes: page_boxes,
    });
    lines.clear();
}

fn layout_footnote_band(
    doc: &Document,
    page_boxes: &[LayoutBox],
    format: &tw_model::SectionFormat,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    x: f32,
    content_width: f32,
    tab_interval: f32,
    margin_color: u32,
    field_ctx: FieldEvalContext,
) -> Option<Vec<LayoutBox>> {
    let note_ids = footnote_ids_on_page(page_boxes, doc);
    if note_ids.is_empty() {
        return None;
    }

    let separator_color = tw_model::Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    }
    .to_argb();
    let mut y = format.page_height - format.margin_bottom;
    let mut out = vec![LayoutBox::Rect {
        x,
        y,
        width: content_width * 0.2,
        height: 1.0,
        color: separator_color,
    }];
    y += 8.0;

    for note_id in note_ids {
        let Some(footnote) = doc.footnote_by_id(note_id) else {
            continue;
        };
        let display_number = footnote_display_number(page_boxes, doc, note_id);

        let mut prefixed_blocks = footnote.blocks.clone();
        if let Some(Block::Paragraph(para)) = prefixed_blocks.first_mut() {
            let mut first = para.clone();
            if first.runs.is_empty() {
                first.runs.push(Run::new_text(""));
            }
            let prefix = format!("{} ", display_number);
            if let Some(run) = first.runs.first_mut() {
                if let RunContent::Text(text) = &mut run.content {
                    if !text.starts_with(&prefix) {
                        text.insert_str(0, &prefix);
                    }
                }
            }
            prefixed_blocks[0] = Block::Paragraph(first);
        }

        let band = layout_margin_blocks(
            doc,
            &prefixed_blocks,
            shaper,
            atlas,
            x,
            y,
            content_width,
            tab_interval,
            margin_color,
            field_ctx,
        );
        for item in band {
            if let LayoutBox::TextLine(line) = item {
                y = line.y + line.ascent + line.descent + 4.0;
                out.push(LayoutBox::TextLine(line));
            }
        }
        y += 4.0;
    }

    Some(out)
}

fn footnote_display_number(page_boxes: &[LayoutBox], doc: &Document, note_id: i32) -> u32 {
    for item in page_boxes {
        let LayoutBox::TextLine(line) = item else {
            continue;
        };
        for (_, _, run_id, _) in &line.run_map {
            let Some(run) = doc.run_by_id(*run_id) else {
                continue;
            };
            if let RunContent::FootnoteRef(note) = &run.content {
                if note.note_id == note_id {
                    return note.display_number.unwrap_or(note_id.max(1) as u32);
                }
            }
        }
    }
    note_id.max(1) as u32
}

fn footnote_ids_on_page(page_boxes: &[LayoutBox], doc: &Document) -> Vec<i32> {
    let mut ids = Vec::new();
    for item in page_boxes {
        let LayoutBox::TextLine(line) = item else {
            continue;
        };
        for (_, _, run_id, _) in &line.run_map {
            let Some(run) = doc.run_by_id(*run_id) else {
                continue;
            };
            if let tw_model::RunContent::FootnoteRef(note) = &run.content {
                if !ids.contains(&note.note_id) {
                    ids.push(note.note_id);
                }
            }
        }
    }
    ids
}

fn append_endnotes_to_last_page(
    mut pages: Vec<PageLayout>,
    doc: &Document,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    tab_interval: f32,
) -> Vec<PageLayout> {
    if doc.endnotes.is_empty() {
        return pages;
    }
    let page_count = pages.len() as u32;
    let Some(last_index) = pages.len().checked_sub(1) else {
        return pages;
    };
    let page_index = pages[last_index].page_index;
    let Some(last) = pages.get_mut(last_index) else {
        return pages;
    };
    let format = doc
        .sections
        .last()
        .map(|s| s.format.clone())
        .unwrap_or_default();
    let x = format.margin_left;
    let content_width = format.page_width - format.margin_left - format.margin_right;
    let margin_color = tw_model::Color::BLACK.to_argb();
    let field_ctx = FieldEvalContext::for_page(page_index, page_count);

    let mut y = format.page_height - format.margin_bottom;
    let separator_color = tw_model::Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    }
    .to_argb();
    last.boxes.push(LayoutBox::Rect {
        x,
        y,
        width: content_width * 0.2,
        height: 1.0,
        color: separator_color,
    });
    y += 8.0;

    let mut number = 1u32;
    for endnote in &doc.endnotes {
        let mut prefixed_blocks = endnote.blocks.clone();
        if let Some(Block::Paragraph(para)) = prefixed_blocks.first_mut() {
            let mut first = para.clone();
            let prefix = format!("{} ", roman_numeral(number));
            if let Some(run) = first.runs.first_mut() {
                if let RunContent::Text(text) = &mut run.content {
                    if !text.starts_with(&prefix) {
                        text.insert_str(0, &prefix);
                    }
                }
            }
            prefixed_blocks[0] = Block::Paragraph(first);
        }
        number += 1;

        let band = layout_margin_blocks(
            doc,
            &prefixed_blocks,
            shaper,
            atlas,
            x,
            y,
            content_width,
            tab_interval,
            margin_color,
            field_ctx,
        );
        for item in band {
            if let LayoutBox::TextLine(line) = item {
                y = line.y + line.ascent + line.descent + 4.0;
                last.boxes.push(LayoutBox::TextLine(line));
            }
        }
        y += 4.0;
    }

    pages
}

fn roman_numeral(mut n: u32) -> String {
    const VALUES: [(u32, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    for (value, symbol) in VALUES {
        while n >= value {
            out.push_str(symbol);
            n -= value;
        }
    }
    if out.is_empty() {
        "i".into()
    } else {
        out
    }
}

/// Right-margin markers for comment anchors (F17.S3).
fn layout_comment_margin_markers(
    page_boxes: &[LayoutBox],
    doc: &Document,
    format: &tw_model::SectionFormat,
) -> Vec<LayoutBox> {
    const COMMENT_MARKER_COLOR: u32 = 0xFFFFA500;
    let marker_x = format.page_width - format.margin_right + 4.0;
    let mut markers = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for item in page_boxes {
        let LayoutBox::TextLine(line) = item else {
            continue;
        };
        for (_, _, run_id, _) in &line.run_map {
            let Some(run) = doc.run_by_id(*run_id) else {
                continue;
            };
            let tw_model::RunContent::CommentRef(c) = &run.content else {
                continue;
            };
            if !seen.insert(c.comment_id) {
                continue;
            }
            markers.push(LayoutBox::Rect {
                x: marker_x,
                y: line.y - line.ascent,
                width: 6.0,
                height: line.line_height.max(12.0),
                color: COMMENT_MARKER_COLOR,
            });
        }
    }
    markers
}

fn layout_header_footer_band(
    hf: Option<&HeaderFooter>,
    format: &SectionFormat,
    doc: &Document,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    x: f32,
    y: f32,
    content_width: f32,
    tab_interval: f32,
    margin_color: u32,
    field_ctx: FieldEvalContext,
    is_header: bool,
) -> Option<Vec<LayoutBox>> {
    if let Some(hf) = hf {
        if !hf.blocks.is_empty() {
            return Some(layout_margin_blocks(
                doc,
                &hf.blocks,
                shaper,
                atlas,
                x,
                y,
                content_width,
                tab_interval,
                margin_color,
                field_ctx,
            ));
        }
        if let Some(ref text) = hf.plain_text {
            let para = tw_model::Paragraph::with_text(text.clone());
            let (lines, _) = layout_paragraph(
                shaper,
                atlas,
                &para,
                ParagraphFrame::new(x, y, content_width)
                    .with_tab_interval(tab_interval)
                    .with_field_context(field_ctx),
                margin_color,
            );
            return Some(lines.into_iter().map(LayoutBox::TextLine).collect());
        }
    }

    if is_header {
        if !format.header_blocks.is_empty() {
            return Some(layout_margin_blocks(
                doc,
                &format.header_blocks,
                shaper,
                atlas,
                x,
                y,
                content_width,
                tab_interval,
                margin_color,
                field_ctx,
            ));
        }
        if let Some(ref header) = format.header_text {
            let para = tw_model::Paragraph::with_text(header.clone());
            let (lines, _) = layout_paragraph(
                shaper,
                atlas,
                &para,
                ParagraphFrame::new(x, y, content_width)
                    .with_tab_interval(tab_interval)
                    .with_field_context(field_ctx),
                margin_color,
            );
            return Some(lines.into_iter().map(LayoutBox::TextLine).collect());
        }
    } else if !format.footer_blocks.is_empty() {
        return Some(layout_margin_blocks(
            doc,
            &format.footer_blocks,
            shaper,
            atlas,
            x,
            y,
            content_width,
            tab_interval,
            margin_color,
            field_ctx,
        ));
    } else if let Some(ref footer) = format.footer_text {
        let para = tw_model::Paragraph::with_text(footer.clone());
        let (lines, _) = layout_paragraph(
            shaper,
            atlas,
            &para,
            ParagraphFrame::new(x, y, content_width)
                .with_tab_interval(tab_interval)
                .with_field_context(field_ctx),
            margin_color,
        );
        return Some(lines.into_iter().map(LayoutBox::TextLine).collect());
    }

    None
}

fn prepend_page_decorations(
    page_boxes: &mut Vec<LayoutBox>,
    format: &SectionFormat,
    body_line_ys: &[f32],
    line_number: &mut u32,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    tab_interval: f32,
) {
    let mut back_layers = Vec::new();

    if let Some(color) = format.page_color {
        back_layers.push(LayoutBox::Rect {
            x: 0.0,
            y: 0.0,
            width: format.page_width,
            height: format.page_height,
            color: color.to_argb(),
        });
    }

    if let Some(wm) = &format.watermark {
        if !wm.text.is_empty() {
            let (rect_x, rect_y, rect_w, rect_h) = format.watermark_band();
            back_layers.push(LayoutBox::Rect {
                x: rect_x,
                y: rect_y,
                width: rect_w,
                height: rect_h,
                color: wm.background_color().to_argb(),
            });
            back_layers.extend(layout_styled_label(
                shaper,
                atlas,
                &wm.text,
                rect_x,
                rect_y + rect_h * 0.35,
                rect_w,
                WATERMARK_FONT_PT,
                wm.color,
                tab_interval,
            ));
        }
    }

    if format.line_numbers.enabled {
        let gutter_x = format.line_number_gutter_x();
        for &y in body_line_ys {
            let num_text = line_number.to_string();
            *line_number += 1;
            back_layers.extend(layout_styled_label(
                shaper,
                atlas,
                &num_text,
                gutter_x,
                y - LINE_NUMBER_BASELINE_OFFSET,
                16.0,
                LINE_NUMBER_FONT_PT,
                LINE_NUMBER_COLOR,
                tab_interval,
            ));
        }
    }

    if let Some(borders) = &format.page_borders {
        if borders.any() {
            push_page_border_rects(&mut back_layers, format, borders);
        }
    }

    for layer in back_layers.into_iter().rev() {
        page_boxes.insert(0, layer);
    }
}

fn push_page_border_rects(
    layers: &mut Vec<LayoutBox>,
    format: &SectionFormat,
    borders: &tw_model::BorderSet,
) {
    let inset = SectionFormat::PAGE_BORDER_INSET;
    let x0 = inset;
    let y0 = inset;
    let x1 = (format.page_width - inset).max(x0);
    let y1 = (format.page_height - inset).max(y0);
    let span_w = (x1 - x0).max(0.0);
    let span_h = (y1 - y0).max(0.0);

    if let Some(top) = borders.top {
        let w = top.width.max(0.5);
        layers.push(LayoutBox::Rect {
            x: x0,
            y: y0,
            width: span_w,
            height: w,
            color: top.color.to_argb(),
        });
    }
    if let Some(bottom) = borders.bottom {
        let w = bottom.width.max(0.5);
        layers.push(LayoutBox::Rect {
            x: x0,
            y: y1 - w,
            width: span_w,
            height: w,
            color: bottom.color.to_argb(),
        });
    }
    if let Some(left) = borders.left {
        let w = left.width.max(0.5);
        layers.push(LayoutBox::Rect {
            x: x0,
            y: y0,
            width: w,
            height: span_h,
            color: left.color.to_argb(),
        });
    }
    if let Some(right) = borders.right {
        let w = right.width.max(0.5);
        layers.push(LayoutBox::Rect {
            x: x1 - w,
            y: y0,
            width: w,
            height: span_h,
            color: right.color.to_argb(),
        });
    }
}

const WATERMARK_FONT_PT: f32 = 54.0;
const LINE_NUMBER_FONT_PT: f32 = 9.0;
const LINE_NUMBER_BASELINE_OFFSET: f32 = 10.0;
const LINE_NUMBER_COLOR: tw_model::Color = tw_model::Color {
    r: 128,
    g: 128,
    b: 128,
    a: 255,
};

fn layout_styled_label(
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    font_size: f32,
    color: tw_model::Color,
    tab_interval: f32,
) -> Vec<LayoutBox> {
    let mut para = Paragraph::with_text(text.to_string());
    if let Some(run) = para.runs.first_mut() {
        run.format.font_size = Some(font_size);
        run.format.color = Some(color);
    }
    let (lines, _) = layout_paragraph(
        shaper,
        atlas,
        &para,
        ParagraphFrame::new(x, y, width).with_tab_interval(tab_interval),
        color.to_argb(),
    );
    lines.into_iter().map(LayoutBox::TextLine).collect()
}

fn layout_margin_blocks(
    doc: &Document,
    blocks: &[Block],
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    x: f32,
    mut y: f32,
    content_width: f32,
    tab_interval: f32,
    color: u32,
    field_context: FieldEvalContext,
) -> Vec<LayoutBox> {
    let mut out = Vec::new();
    for block in blocks {
        match block {
            Block::Paragraph(para) => {
                let resolved_para = doc
                    .styles
                    .resolve_para_format(para.style_id, &para.format);
                let mut effective = para.clone();
                effective.format = resolved_para.clone();
                if effective.runs.is_empty() {
                    effective.runs.push(Run::new_text(""));
                }
                for run in &mut effective.runs {
                    run.format = doc
                        .styles
                        .resolve_char_format(para.style_id, &run.format);
                }
                let (lines, height) = layout_paragraph(
                    shaper,
                    atlas,
                    &effective,
                    ParagraphFrame::new(x, y, content_width)
                        .with_tab_interval(tab_interval)
                        .with_tab_stops(
                            resolved_para.tab_stops.clone().unwrap_or_default(),
                        )
                        .with_field_context(field_context),
                    color,
                );
                for line in lines {
                    out.push(LayoutBox::TextLine(line));
                }
                y += height;
            }
            Block::ImageBlock(image) => {
                let encoded = std::sync::Arc::new(image.data.bytes.clone());
                let layout = layout_image(image, x, y, encoded);
                let height = layout.height;
                out.push(LayoutBox::Image(layout));
                y += height;
            }
            Block::Table(_) => {}
            Block::ShapeBlock(shape) => {
                if let Some(preview) = shape
                    .preview_image
                    .as_ref()
                    .filter(|img| !img.bytes.is_empty())
                {
                    let encoded = std::sync::Arc::new(preview.bytes.clone());
                    out.push(LayoutBox::Image(layout_shape_preview(
                        shape, preview, x, y, encoded,
                    )));
                } else {
                    out.push(LayoutBox::Shape(layout_shape(shape, x, y)));
                }
                let padding = 6.0;
                let inner_width = (shape.shape.width - padding * 2.0).max(12.0);
                out.extend(layout_shape_paragraphs(
                    doc,
                    shape,
                    shaper,
                    atlas,
                    x + padding,
                    y + padding,
                    inner_width,
                    tab_interval,
                ));
                if let Some(label) = layout_diagram_label(shape, shaper, atlas, x, y) {
                    out.push(label);
                }
                y += shape.shape.height + 8.0;
            }
            _ => {}
        }
    }
    out
}

struct ParagraphSegment {
    paragraph: Paragraph,
    page_break_before: bool,
    column_break_before: bool,
}

/// Minimum vertical space the next paragraph likely needs (keep-with-next).
fn next_paragraph_min_height(doc: &Document, blocks: &[Block], idx: usize) -> f32 {
    blocks
        .get(idx + 1)
        .and_then(|block| {
            if let Block::Paragraph(para) = block {
                let resolved = doc.styles.resolve_para_format(para.style_id, &para.format);
                Some(resolved.space_before.unwrap_or(0.0) + 14.0)
            } else {
                None
            }
        })
        .unwrap_or(0.0)
}

/// Rough single-line height before layout (keep-with-next pre-check).
fn lines_estimated_min_height(para: &tw_model::ParaFormat) -> f32 {
    para.space_before.unwrap_or(0.0) + 14.0
}

fn layout_shape(shape: &tw_model::ShapeBlock, x: f32, y: f32) -> ShapeLayout {
    ShapeLayout {
        x,
        y,
        width: shape.shape.width,
        height: shape.shape.height,
        shape_id: shape.id,
        shape_type: shape.shape.shape_type,
        fill: shape.style.fill,
        stroke: shape.style.stroke,
        stroke_width: shape.style.stroke_width,
        chart_data: shape.chart_data.clone(),
        diagram_kind: shape.diagram_kind,
    }
}

fn layout_shape_preview(
    shape: &tw_model::ShapeBlock,
    preview: &tw_model::ImageData,
    x: f32,
    y: f32,
    encoded: std::sync::Arc<Vec<u8>>,
) -> ImageLayout {
    let selection_shape_kind = match shape.shape.shape_type {
        tw_model::ShapeKind::Diagram | tw_model::ShapeKind::Chart => {
            Some(shape.shape.shape_type)
        }
        _ => None,
    };
    ImageLayout {
        x,
        y,
        width: shape.shape.width,
        height: shape.shape.height,
        image_id: shape.id,
        asset_id: preview.asset_id.clone(),
        encoded,
        rotation_deg: 0.0,
        opacity: 1.0,
        crop_left: 0.0,
        crop_top: 0.0,
        crop_right: 0.0,
        crop_bottom: 0.0,
        selection_shape_kind,
    }
}

fn layout_diagram_label(
    shape: &tw_model::ShapeBlock,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    x: f32,
    y: f32,
) -> Option<LayoutBox> {
    use tw_model::{Alignment, ShapeKind};

    let label = match shape.shape.shape_type {
        ShapeKind::Diagram => "SmartArt",
        // Word inserts a titled chart surface; caption doubles as the title.
        ShapeKind::Chart => {
            if shape.chart_data.is_some() {
                "Chart Title"
            } else {
                "Chart"
            }
        }
        _ => return None,
    };

    if shape
        .preview_image
        .as_ref()
        .is_some_and(|img| !img.bytes.is_empty())
    {
        return None;
    }

    let mut para = tw_model::Paragraph::with_text(label);
    para.format.alignment = Some(Alignment::Center);
    // Keep captions in the top band so chart/diagram previews can use the body.
    let label_y = y + shape.shape.height * 0.08;
    let (lines, _) = crate::line::layout_paragraph(
        shaper,
        atlas,
        &para,
        ParagraphFrame::new(x, label_y, shape.shape.width),
        0xFF506070,
    );
    lines.into_iter().next().map(|mut line| {
        // Captions sit inside the shape bounds; keep them out of caret hit-testing.
        line.decorative = true;
        line.run_map.clear();
        LayoutBox::TextLine(line)
    })
}

fn layout_shape_paragraphs(
    doc: &Document,
    shape: &tw_model::ShapeBlock,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
    x: f32,
    mut y: f32,
    content_width: f32,
    tab_interval: f32,
) -> Vec<LayoutBox> {
    let mut out = Vec::new();
    for para in &shape.paragraphs {
        let resolved_para = doc
            .styles
            .resolve_para_format(para.style_id, &para.format);
        let mut effective = para.clone();
        effective.format = resolved_para.clone();
        if effective.runs.is_empty() {
            effective.runs.push(Run::new_text(""));
        }
        for run in &mut effective.runs {
            run.format = doc
                .styles
                .resolve_char_format(para.style_id, &run.format);
        }
        let color = effective
            .runs
            .first()
            .and_then(|r| r.format.color)
            .map(|c| c.to_argb())
            .unwrap_or(0xFF000000);
        let (lines, height) = layout_paragraph(
            shaper,
            atlas,
            &effective,
            ParagraphFrame::new(x, y, content_width)
                .with_tab_interval(tab_interval)
                .with_tab_stops(resolved_para.tab_stops.clone().unwrap_or_default()),
            color,
        );
        for line in lines {
            out.push(LayoutBox::TextLine(line));
        }
        y += height;
    }
    out
}

fn layout_image(
    image: &tw_model::ImageBlock,
    x: f32,
    y: f32,
    encoded: std::sync::Arc<Vec<u8>>,
) -> ImageLayout {
    let (width, height) = image.effective_display_size();
    let t = &image.transform;
    ImageLayout {
        x,
        y,
        width,
        height,
        image_id: image.id,
        asset_id: image.data.asset_id.clone(),
        encoded,
        rotation_deg: t.rotation_deg,
        opacity: t.opacity,
        crop_left: t.crop_left,
        crop_top: t.crop_top,
        crop_right: t.crop_right,
        crop_bottom: t.crop_bottom,
        selection_shape_kind: None,
    }
}

/// Resolves an anchor offset against the box it is measured from.
fn anchor_position(format: &tw_model::SectionFormat, anchor: tw_model::ImageAnchor) -> (f32, f32) {
    let x = match anchor.origin_x {
        AnchorOrigin::Page => anchor.x,
        AnchorOrigin::Column | AnchorOrigin::Margin => format.margin_left + anchor.x,
    };
    let y = match anchor.origin_y {
        AnchorOrigin::Page => anchor.y,
        AnchorOrigin::Column | AnchorOrigin::Margin => format.margin_top + anchor.y,
    };
    (x.max(0.0), y.max(0.0))
}

fn square_wrap_inset_at_y(
    column_x: f32,
    column_width: f32,
    y: f32,
    obstacles: &[WrapObstacle],
) -> f32 {
    let mut left_inset = 0.0f32;
    for obs in obstacles {
        if y >= obs.y - 0.5 && y < obs.y + obs.height {
            let obs_right = obs.x + obs.width + IMAGE_TEXT_GAP;
            if obs.x <= column_x + left_inset + 1.0 && obs_right > column_x {
                left_inset = left_inset.max(obs_right - column_x);
            }
        }
    }
    left_inset.min(column_width * 0.85)
}

fn paragraph_frame_with_square_wrap(
    column_x: f32,
    indent: f32,
    y: f32,
    column_width: f32,
    obstacles: &[WrapObstacle],
) -> ParagraphFrame {
    let left_inset = square_wrap_inset_at_y(column_x, column_width, y, obstacles);
    ParagraphFrame::indented_in(column_x + left_inset, indent, y, column_width - left_inset)
}

fn split_paragraph_at_page_breaks(para: &tw_model::Paragraph) -> Vec<ParagraphSegment> {
    let mut segments = Vec::new();
    // Start with an empty run list — Paragraph::new() would inject a fresh
    // empty run whose NodeId is not in the document model, breaking hit-test
    // → edit routing for blank paragraphs.
    let mut current = Paragraph {
        id: para.id,
        format: para.format.clone(),
        style_id: para.style_id,
        runs: Vec::new(),
    };
    let mut page_break_before = para.format.page_break_before == Some(true);
    let mut column_break_before = false;

    for run in &para.runs {
        match &run.content {
            RunContent::Break(BreakType::Page) => {
                if !current.runs.is_empty() || page_break_before {
                    segments.push(ParagraphSegment {
                        paragraph: current,
                        page_break_before,
                        column_break_before,
                    });
                    current = Paragraph {
                        id: para.id,
                        format: para.format.clone(),
                        style_id: para.style_id,
                        runs: Vec::new(),
                    };
                    page_break_before = true;
                    column_break_before = false;
                } else {
                    page_break_before = true;
                }
            }
            RunContent::Break(BreakType::Column) => {
                if !current.runs.is_empty() || page_break_before || column_break_before {
                    segments.push(ParagraphSegment {
                        paragraph: current,
                        page_break_before,
                        column_break_before,
                    });
                    current = Paragraph {
                        id: para.id,
                        format: para.format.clone(),
                        style_id: para.style_id,
                        runs: Vec::new(),
                    };
                    page_break_before = false;
                    column_break_before = true;
                } else {
                    column_break_before = true;
                }
            }
            RunContent::Break(BreakType::Line) => {
                current.runs.push(Run {
                    id: run.id,
                    format: run.format.clone(),
                    content: RunContent::Text("\n".into()),
                    revision: run.revision.clone(),
                });
            }
            RunContent::Text(text) => {
                current.runs.push(run.clone());
                let _ = text;
            }
            RunContent::Tab => {
                current.runs.push(Run {
                    id: run.id,
                    format: run.format.clone(),
                    content: RunContent::Text("\t".into()),
                    revision: run.revision.clone(),
                });
            }
            RunContent::Hyperlink { text, target } => {
                current.runs.push(Run {
                    id: run.id,
                    format: run.format.clone(),
                    content: RunContent::Hyperlink {
                        target: target.clone(),
                        text: text.clone(),
                    },
                    revision: run.revision.clone(),
                });
            }
            RunContent::Field(field) => {
                current.runs.push(Run {
                    id: run.id,
                    format: run.format.clone(),
                    content: RunContent::Field(field.clone()),
                    revision: run.revision.clone(),
                });
            }
            RunContent::InlineImage(_)
            | RunContent::FootnoteRef(_)
            | RunContent::EndnoteRef(_)
            | RunContent::CitationRef(_)
            | RunContent::CommentRef(_)
            | RunContent::Bookmark(_)
            | RunContent::OfficeMath { .. } => {
                current.runs.push(run.clone());
            }
            _ => {
                current.runs.push(run.clone());
            }
        }
    }

    if !current.runs.is_empty() || page_break_before || column_break_before || segments.is_empty() {
        segments.push(ParagraphSegment {
            paragraph: current,
            page_break_before,
            column_break_before,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::{Block, BreakType, Document, Paragraph, Run, RunContent, Table};

    fn long_document(paragraphs: usize) -> Document {
        let mut doc = Document::new();
        doc.sections[0].blocks.clear();
        for i in 0..paragraphs {
            doc.sections[0]
                .blocks
                .push(Block::Paragraph(Paragraph::with_text(format!(
                    "Paragraph {i} with enough text to fill some vertical space on the page."
                ))));
        }
        doc
    }

    #[test]
    fn paginates_long_document_without_runaway_page_count() {
        let doc = long_document(80);
        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        assert!(layout.pages.len() > 1, "expected multiple pages");
        assert!(
            layout.pages.len() <= 30,
            "80 short paragraphs should not exceed 30 pages, got {}",
            layout.pages.len()
        );
    }

    #[test]
    fn wraps_paragraph_into_lines_that_fill_the_column() {
        let mut doc = Document::new();
        doc.sections[0].blocks.clear();
        let words = (0..60).map(|i| format!("word{i}")).collect::<Vec<_>>();
        doc.sections[0]
            .blocks
            .push(Block::Paragraph(Paragraph::with_text(words.join(" "))));

        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        let lines: Vec<_> = layout.pages[0]
            .boxes
            .iter()
            .filter_map(|b| match b {
                LayoutBox::TextLine(l) => Some(l),
                _ => None,
            })
            .collect();

        assert_eq!(layout.pages.len(), 1, "60 words should fit on one page");
        assert!(
            lines.len() < 12,
            "expected greedy wrapping, got {} lines for 60 words",
            lines.len()
        );
        let content_width = layout.pages[0].content_width;
        for line in &lines {
            assert!(
                line.width <= content_width + 1.0,
                "line width {} exceeds column {}",
                line.width,
                content_width
            );
        }
    }

    /// A token with no break opportunity inside it must still be broken at the
    /// margin. Left to run, it prints past the page edge and is unreadable on a
    /// narrow viewport.
    #[test]
    fn breaks_a_word_wider_than_the_column_at_the_margin() {
        let mut doc = Document::new();
        doc.sections[0].blocks.clear();
        // No spaces, hyphens, or punctuation: UAX #14 offers no break at all.
        let word = "W".repeat(400);
        doc.sections[0]
            .blocks
            .push(Block::Paragraph(Paragraph::with_text(word.clone())));

        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        let content_width = layout.pages[0].content_width;
        let lines: Vec<_> = layout
            .pages
            .iter()
            .flat_map(|p| p.boxes.iter())
            .filter_map(|b| match b {
                LayoutBox::TextLine(l) => Some(l),
                _ => None,
            })
            .collect();

        assert!(
            lines.len() > 1,
            "a 400-character word must break across lines, got {}",
            lines.len()
        );
        for line in &lines {
            assert!(
                line.width <= content_width + 1.0,
                "line width {} exceeds column {}",
                line.width,
                content_width
            );
        }
        // Breaking must not drop or duplicate glyphs.
        let glyphs: usize = lines.iter().map(|l| l.glyphs.len()).sum();
        assert_eq!(glyphs, word.len(), "every character should survive the break");
    }

    /// The same token mid-paragraph: the words around it wrap normally and the
    /// long one is broken rather than overflowing.
    #[test]
    fn breaks_an_overlong_token_surrounded_by_normal_words() {
        let mut doc = Document::new();
        doc.sections[0].blocks.clear();
        let text = format!("alpha beta {} gamma delta", "X".repeat(300));
        doc.sections[0]
            .blocks
            .push(Block::Paragraph(Paragraph::with_text(text)));

        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        let content_width = layout.pages[0].content_width;
        for line in layout
            .pages
            .iter()
            .flat_map(|p| p.boxes.iter())
            .filter_map(|b| match b {
                LayoutBox::TextLine(l) => Some(l),
                _ => None,
            })
        {
            assert!(
                line.width <= content_width + 1.0,
                "line width {} exceeds column {}",
                line.width,
                content_width
            );
        }
    }

    #[test]
    fn table_layout_produces_grid_lines() {
        let mut doc = Document::new();
        doc.sections[0]
            .blocks
            .push(Block::Table(Table::new(3, 3)));

        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        let has_table = layout.pages.iter().any(|p| {
            p.boxes.iter().any(|b| matches!(b, LayoutBox::Table(_)))
        });
        assert!(has_table);
    }

    #[test]
    fn layouts_all_sections_not_only_first() {
        let mut doc = Document::new();
        doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
            "Section one paragraph.",
        ))];
        let mut section_two = tw_model::Section::new();
        section_two.blocks = vec![Block::Paragraph(Paragraph::with_text(
            "Section two paragraph.",
        ))];
        doc.sections.push(section_two);

        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        let text: String = layout
            .pages
            .iter()
            .flat_map(|p| &p.boxes)
            .filter_map(|b| match b {
                LayoutBox::TextLine(l) => Some(
                    l.glyphs
                        .iter()
                        .map(|g| g.codepoint)
                        .collect::<String>(),
                ),
                _ => None,
            })
            .collect();
        assert!(text.contains('S'));
        assert!(text.contains('t'));
        assert!(text.contains('w'));
        assert!(text.contains('o'));
    }
}

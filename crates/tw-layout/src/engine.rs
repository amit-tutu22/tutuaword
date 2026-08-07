use crate::line::{apply_list_markers, layout_paragraph, ParagraphFrame};
use crate::tables::layout_table_slice;
use crate::types::{
    DocumentLayout, ImageLayout, LayoutBox, LineMap, PageIndex, PageLayout, TextLine,
};
use std::collections::HashMap;
use tw_model::{
    Block, BreakType, Document, NodeId, Paragraph, Run, RunContent, SectionFormat,
    format_list_marker,
};
use tw_shape::{FontFaceSpec, FontId, FontRegistrationError, GlyphAtlas, TextShaper};

/// Maximum pages to synchronously reflow during incremental layout (R1.1).
const MAX_INCREMENTAL_REFLOW_PAGES: usize = 3;

/// Flow state at a block boundary. Everything downstream of a boundary is a pure
/// function of this state, so two equal continuations imply an identical tail
/// layout — that is the convergence test that bounds incremental reflow.
///
/// Completed pages are deliberately *not* stored here: the prefix is taken from
/// `last_layout` at resume time, which keeps checkpoint memory O(1) per block and
/// stops a resume from resurrecting pages that a later pass already replaced.
#[derive(Debug, Clone)]
struct Continuation {
    page_index: PageIndex,
    y: f32,
    format: SectionFormat,
    list_counters: HashMap<(u32, u32), u32>,
    current_boxes: Vec<LayoutBox>,
    current_lines: Vec<TextLine>,
}

fn page_geometry(format: &SectionFormat) -> [f32; 6] {
    [
        format.page_width,
        format.page_height,
        format.margin_top,
        format.margin_bottom,
        format.margin_left,
        format.margin_right,
    ]
}

impl PartialEq for Continuation {
    fn eq(&self, other: &Self) -> bool {
        self.page_index == other.page_index
            && self.y == other.y
            && page_geometry(&self.format) == page_geometry(&other.format)
            && self.list_counters == other.list_counters
            && self.current_boxes == other.current_boxes
            && self.current_lines == other.current_lines
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
                .map(|(si, bi, _)| (si, bi))
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

        let (mut format, mut pages, mut current_boxes, mut current_lines, mut page_index, mut y, mut list_counters, start_section, start_block) =
            if let Some((cp, start)) = incremental_start {
                (
                    cp.format,
                    old_pages[..cp.page_index as usize].to_vec(),
                    cp.current_boxes,
                    cp.current_lines,
                    cp.page_index,
                    cp.y,
                    cp.list_counters,
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
                    HashMap::new(),
                    0,
                    0,
                )
            };

        let relayout_base = pages.len();
        // Convergence is only trustworthy against continuations a previous pass
        // actually reached; beyond `stale_from` the stored values are pre-edit.
        let convergence_limit = if incremental { self.stale_from } else { None };
        let mut converged_at: Option<PageIndex> = None;
        let mut resume_at: Option<(usize, usize)> = None;
        let mut current_section = doc.sections.first();

        'sections: for (section_idx, section) in doc.sections.iter().enumerate() {
            current_section = Some(section);
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
                        section,
                        tab_interval,
                        doc,
                        &mut self.shaper,
                        &mut self.atlas,
                    );
                    page_index += 1;
                }
                format = section.format.clone();
                y = format.margin_top;
            }

            let content_width = format.page_width - format.margin_left - format.margin_right;
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
                    list_counters: list_counters.clone(),
                    current_boxes: current_boxes.clone(),
                    current_lines: current_lines.clone(),
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
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                );
                                page_index += 1;
                                y = format.margin_top;
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
                                    flush_page(
                                        &mut pages,
                                        &mut current_boxes,
                                        &mut current_lines,
                                        page_index,
                                        section,
                                        tab_interval,
                                        doc,
                                        &mut self.shaper,
                                        &mut self.atlas,
                                    );
                                    page_index += 1;
                                    y = format.margin_top;
                                }
                            }

                            let (mut lines, _) = layout_paragraph(
                                &mut self.shaper,
                                &mut self.atlas,
                                &effective_para,
                                ParagraphFrame::indented_in(
                                    format.margin_left,
                                    indent,
                                    y,
                                    content_width,
                                )
                                .with_tab_interval(tab_interval)
                                .with_tab_stops(resolved_para.tab_stops.clone()),
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
                                    format.margin_left + indent - hanging,
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
                                    flush_page(
                                        &mut pages,
                                        &mut current_boxes,
                                        &mut current_lines,
                                        page_index,
                                        section,
                                        tab_interval,
                                        doc,
                                        &mut self.shaper,
                                        &mut self.atlas,
                                    );
                                    page_index += 1;
                                    y = format.margin_top;
                                    rebase_lines_from(&mut lines, 0, y);
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
                                        flush_page(
                                            &mut pages,
                                            &mut current_boxes,
                                            &mut current_lines,
                                            page_index,
                                            section,
                                            tab_interval,
                                            doc,
                                            &mut self.shaper,
                                            &mut self.atlas,
                                        );
                                        page_index += 1;
                                        y = format.margin_top;
                                        rebase_lines_from(&mut lines, line_idx, y);
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
                                        flush_page(
                                            &mut pages,
                                            &mut current_boxes,
                                            &mut current_lines,
                                            page_index,
                                            section,
                                            tab_interval,
                                            doc,
                                            &mut self.shaper,
                                            &mut self.atlas,
                                        );
                                        page_index += 1;
                                        y = format.margin_top;
                                        rebase_lines_from(&mut lines, line_idx, y);
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
                                        flush_page(
                                            &mut pages,
                                            &mut current_boxes,
                                            &mut current_lines,
                                            page_index,
                                            section,
                                            tab_interval,
                                            doc,
                                            &mut self.shaper,
                                            &mut self.atlas,
                                        );
                                        page_index += 1;
                                        y = format.margin_top;
                                        rebase_lines_from(&mut lines, line_idx, y);
                                        continue 'lines;
                                    }
                                    count = lines.len() - line_idx;
                                    batch_height = lines[line_idx..]
                                        .iter()
                                        .map(|l| l.line_height)
                                        .sum();
                                }

                                for line in lines[line_idx..line_idx + count].iter().cloned() {
                                    current_lines.push(line.clone());
                                    current_boxes.push(LayoutBox::TextLine(line));
                                }
                                line_idx += count;
                                y += batch_height;

                                if line_idx < lines.len() {
                                    flush_page(
                                        &mut pages,
                                        &mut current_boxes,
                                        &mut current_lines,
                                        page_index,
                                        section,
                                        tab_interval,
                                        doc,
                                        &mut self.shaper,
                                        &mut self.atlas,
                                    );
                                    page_index += 1;
                                    y = format.margin_top;
                                    rebase_lines_from(&mut lines, line_idx, y);
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
                                flush_page(
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    page_index,
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                );
                                page_index += 1;
                                y = format.margin_top;
                                continue;
                            }

                            let slice = layout_table_slice(
                                &mut self.shaper,
                                &mut self.atlas,
                                table,
                                start_row,
                                format.margin_left,
                                y,
                                content_width,
                                available,
                                tw_model::Color::BLACK.to_argb(),
                                tab_interval,
                            );

                            if slice.rows_placed == 0 {
                                break;
                            }

                            let overflows = y + slice.layout.height > content_bottom;
                            if overflows && !current_boxes.is_empty() {
                                flush_page(
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    page_index,
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                );
                                page_index += 1;
                                y = format.margin_top;
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
                                    section,
                                    tab_interval,
                                    doc,
                                    &mut self.shaper,
                                    &mut self.atlas,
                                );
                                page_index += 1;
                                y = format.margin_top;
                            }
                        }
                    }
                    Block::ImageBlock(image) => {
                        let encoded = std::sync::Arc::new(image.data.bytes.clone());

                        // Floating images are placed absolutely and take no
                        // room in the text flow; inline ones occupy a band.
                        if let Some(anchor) = image.anchor {
                            let (ax, ay) = anchor_position(&format, anchor);
                            current_boxes.push(LayoutBox::Image(ImageLayout {
                                x: ax,
                                y: ay,
                                width: image.display_width,
                                height: image.display_height,
                                image_id: image.id,
                                asset_id: image.data.asset_id.clone(),
                                encoded,
                            }));
                            continue;
                        }

                        if y + image.display_height > content_bottom
                            && !current_boxes.is_empty()
                        {
                            flush_page(
                                &mut pages,
                                &mut current_boxes,
                                &mut current_lines,
                                page_index,
                                section,
                                tab_interval,
                                doc,
                                &mut self.shaper,
                                &mut self.atlas,
                            );
                            page_index += 1;
                            y = format.margin_top;
                        }

                        let img = ImageLayout {
                            x: format.margin_left,
                            y,
                            width: image.display_width,
                            height: image.display_height,
                            image_id: image.id,
                            asset_id: image.data.asset_id.clone(),
                            encoded,
                        };
                        y += img.height + 8.0;
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
                                section,
                                tab_interval,
                                doc,
                                &mut self.shaper,
                                &mut self.atlas,
                            );
                            page_index += 1;
                            y = format.margin_top;
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
                        section,
                        tab_interval,
                        doc,
                        &mut self.shaper,
                        &mut self.atlas,
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

        self.last_layout = DocumentLayout { pages };
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
                LayoutBox::TextLine(l) => lines.push(l.clone()),
                LayoutBox::Table(t) => {
                    for cell in &t.cells {
                        lines.extend(cell.lines.clone());
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
    section: &tw_model::Section,
    tab_interval: f32,
    doc: &Document,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
) {
    let format = &section.format;
    let content_width = format.page_width - format.margin_left - format.margin_right;
    let mut page_boxes = std::mem::take(boxes);
    let margin_color = tw_model::Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    }
    .to_argb();

    let default_header = section
        .headers
        .get(&tw_model::HeaderFooterType::Default);
    if let Some(hf) = default_header {
        if !hf.blocks.is_empty() {
            let header_boxes = layout_margin_blocks(
                doc,
                &hf.blocks,
                shaper,
                atlas,
                format.margin_left,
                format.margin_top * 0.25,
                content_width,
                tab_interval,
                margin_color,
            );
            for item in header_boxes.into_iter().rev() {
                page_boxes.insert(0, item);
            }
        } else if let Some(ref header) = hf.plain_text {
            let header_para = tw_model::Paragraph::with_text(header.clone());
            let (header_lines, _) = layout_paragraph(
                shaper,
                atlas,
                &header_para,
                ParagraphFrame::new(
                    format.margin_left,
                    format.margin_top * 0.25,
                    content_width,
                )
                .with_tab_interval(tab_interval),
                margin_color,
            );
            for line in header_lines {
                page_boxes.insert(0, LayoutBox::TextLine(line));
            }
        }
    } else if !format.header_blocks.is_empty() {
        let header_boxes = layout_margin_blocks(
            doc,
            &format.header_blocks,
            shaper,
            atlas,
            format.margin_left,
            format.margin_top * 0.25,
            content_width,
            tab_interval,
            margin_color,
        );
        for item in header_boxes.into_iter().rev() {
            page_boxes.insert(0, item);
        }
    } else if let Some(ref header) = format.header_text {
        let header_para = tw_model::Paragraph::with_text(header.clone());
        let (header_lines, _) = layout_paragraph(
            shaper,
            atlas,
            &header_para,
            ParagraphFrame::new(
                format.margin_left,
                format.margin_top * 0.25,
                content_width,
            )
            .with_tab_interval(tab_interval),
            margin_color,
        );
        for line in header_lines {
            page_boxes.insert(0, LayoutBox::TextLine(line));
        }
    }

    let default_footer = section
        .footers
        .get(&tw_model::HeaderFooterType::Default);
    if let Some(hf) = default_footer {
        let footer_y = format.page_height - format.margin_bottom * 0.75;
        if !hf.blocks.is_empty() {
            page_boxes.extend(layout_margin_blocks(
                doc,
                &hf.blocks,
                shaper,
                atlas,
                format.margin_left,
                footer_y,
                content_width,
                tab_interval,
                margin_color,
            ));
        } else if let Some(ref footer) = hf.plain_text {
            let footer_para = tw_model::Paragraph::with_text(footer.clone());
            let (footer_lines, _) = layout_paragraph(
                shaper,
                atlas,
                &footer_para,
                ParagraphFrame::new(format.margin_left, footer_y, content_width)
                    .with_tab_interval(tab_interval),
                margin_color,
            );
            for line in footer_lines {
                page_boxes.push(LayoutBox::TextLine(line));
            }
        }
    } else if !format.footer_blocks.is_empty() {
        let footer_y = format.page_height - format.margin_bottom * 0.75;
        page_boxes.extend(layout_margin_blocks(
            doc,
            &format.footer_blocks,
            shaper,
            atlas,
            format.margin_left,
            footer_y,
            content_width,
            tab_interval,
            margin_color,
        ));
    } else if let Some(ref footer) = format.footer_text {
        let footer_para = tw_model::Paragraph::with_text(footer.clone());
        let footer_y = format.page_height - format.margin_bottom * 0.75;
        let (footer_lines, _) = layout_paragraph(
            shaper,
            atlas,
            &footer_para,
            ParagraphFrame::new(format.margin_left, footer_y, content_width)
                .with_tab_interval(tab_interval),
            margin_color,
        );
        for line in footer_lines {
            page_boxes.push(LayoutBox::TextLine(line));
        }
    }

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
                        .with_tab_stops(resolved_para.tab_stops.clone()),
                    color,
                );
                for line in lines {
                    out.push(LayoutBox::TextLine(line));
                }
                y += height;
            }
            Block::ImageBlock(image) => {
                let encoded = std::sync::Arc::new(image.data.bytes.clone());
                out.push(LayoutBox::Image(ImageLayout {
                    x,
                    y,
                    width: image.display_width,
                    height: image.display_height,
                    image_id: image.id,
                    asset_id: image.data.asset_id.clone(),
                    encoded,
                }));
                y += image.display_height;
            }
            Block::Table(_) => {}
            Block::ShapeBlock(shape) => {
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

/// Resolves an anchor offset against the box it is measured from.
fn anchor_position(format: &tw_model::SectionFormat, anchor: tw_model::ImageAnchor) -> (f32, f32) {
    use tw_model::AnchorOrigin;
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

    for run in &para.runs {
        match &run.content {
            RunContent::Break(BreakType::Page) => {
                if !current.runs.is_empty() || page_break_before {
                    segments.push(ParagraphSegment {
                        paragraph: current,
                        page_break_before,
                    });
                    current = Paragraph {
                        id: para.id,
                        format: para.format.clone(),
                        style_id: para.style_id,
                        runs: Vec::new(),
                    };
                    page_break_before = true;
                } else {
                    page_break_before = true;
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
            RunContent::Break(BreakType::Column) => {}
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
            | RunContent::CommentRef(_)
            | RunContent::Bookmark(_) => {
                current.runs.push(run.clone());
            }
            _ => {
                current.runs.push(run.clone());
            }
        }
    }

    if !current.runs.is_empty() || page_break_before || segments.is_empty() {
        segments.push(ParagraphSegment {
            paragraph: current,
            page_break_before,
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

use crate::line::{apply_list_markers, layout_paragraph, ParagraphFrame};
use crate::tables::layout_table_slice;
use crate::types::{
    DocumentLayout, ImageLayout, LayoutBox, LineMap, PageIndex, PageLayout, TextLine,
};
use std::collections::HashMap;
use tw_model::{
    Block, BreakType, Document, Paragraph, Run, RunContent, SectionFormat, format_list_marker,
};
use tw_shape::{GlyphAtlas, TextShaper};

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
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            shaper: TextShaper::new(),
            atlas: GlyphAtlas::default(),
            page_cache: HashMap::new(),
            line_maps: HashMap::new(),
            dirty_pages: vec![0],
            last_layout: DocumentLayout::default(),
        }
    }

    pub fn invalidate_all(&mut self) {
        self.dirty_pages = vec![0];
        self.page_cache.clear();
        self.line_maps.clear();
        self.last_layout = DocumentLayout::default();
    }

    pub fn layout_document(&mut self, doc: &Document) -> DocumentLayout {
        self.shaper.configure_from_theme(
            &doc.settings.theme.minor_font,
            &doc.settings.theme.major_font,
        );
        let tab_interval = doc.settings.default_tab_stop.max(1.0);
        let mut format = doc
            .sections
            .first()
            .map(|s| s.format.clone())
            .unwrap_or_default();

        let mut pages: Vec<PageLayout> = Vec::new();
        let mut current_boxes: Vec<LayoutBox> = Vec::new();
        let mut current_lines: Vec<TextLine> = Vec::new();
        let mut page_index: PageIndex = 0;
        let mut y = format.margin_top;

        let mut list_counters: HashMap<(u32, u32), u32> = HashMap::new();

        for (section_idx, section) in doc.sections.iter().enumerate() {
            if section_idx > 0 {
                if !current_boxes.is_empty() {
                    flush_page(
                        &mut pages,
                        &mut current_boxes,
                        &mut current_lines,
                        page_index,
                        &format,
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
                match block {
                    Block::Paragraph(para) => {
                        for segment in split_paragraph_at_page_breaks(para) {
                            if segment.page_break_before && !current_boxes.is_empty() {
                                flush_page(
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    page_index,
                                    &format,
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
                                        &format,
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
                                        &format,
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
                                            &format,
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
                                            &format,
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
                                            &format,
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
                                        &format,
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
                                    &format,
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
                                    &format,
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
                                    &format,
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
                                &format,
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
                }
            }
        }

        if !current_boxes.is_empty() || pages.is_empty() {
            flush_page(
                &mut pages,
                &mut current_boxes,
                &mut current_lines,
                page_index,
                &format,
    tab_interval,
    doc,
    &mut self.shaper,
                &mut self.atlas,
            );
        }

        self.page_cache.clear();
        self.line_maps.clear();
        for page in &pages {
            self.page_cache.insert(page.page_index, page.clone());
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
        }

        self.dirty_pages.clear();
        self.last_layout = DocumentLayout {
            pages: pages.clone(),
        };
        self.last_layout.clone()
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
    format: &SectionFormat,
    tab_interval: f32,
    doc: &Document,
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
) {
    let content_width = format.page_width - format.margin_left - format.margin_right;
    let mut page_boxes = std::mem::take(boxes);
    let margin_color = tw_model::Color {
        r: 128,
        g: 128,
        b: 128,
        a: 255,
    }
    .to_argb();

    if !format.header_blocks.is_empty() {
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

    if !format.footer_blocks.is_empty() {
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

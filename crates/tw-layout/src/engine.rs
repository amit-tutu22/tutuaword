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
        let section = doc.sections.first();
        let format = section.map(|s| s.format.clone()).unwrap_or_default();
        let content_width = format.page_width - format.margin_left - format.margin_right;
        let content_bottom = format.page_height - format.margin_bottom;

        let mut pages: Vec<PageLayout> = Vec::new();
        let mut current_boxes: Vec<LayoutBox> = Vec::new();
        let mut current_lines: Vec<TextLine> = Vec::new();
        let mut page_index: PageIndex = 0;
        let mut y = format.margin_top;

        let mut list_counters: HashMap<(u32, u32), u32> = HashMap::new();

        if let Some(section) = section {
            for block in &section.blocks {
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
                            if effective_para.runs.is_empty() {
                                effective_para.runs.push(Run::new_text(""));
                            }
                            let resolved = doc.styles.resolve_char_format(
                                para.style_id,
                                &effective_para.runs[0].format,
                            );
                            effective_para.runs[0].format = resolved;

                            let list_marker = para.format.numbering.map(|nr| {
                                let key = (nr.numbering_id, nr.level);
                                let counter = list_counters.entry(key).or_insert(0);
                                let marker = doc
                                    .settings
                                    .numbering
                                    .get(nr.numbering_id)
                                    .map(|def| format_list_marker(def, nr.level, *counter))
                                    .unwrap_or_else(|| "•".into());
                                *counter += 1;
                                marker
                            });

                            let level = para.format.numbering.and_then(|nr| {
                                doc.settings.numbering.get(nr.numbering_id).and_then(|d| {
                                    d.levels.iter().find(|l| l.level == nr.level)
                                })
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

                            loop {
                                let (mut lines, height) = layout_paragraph(
                                    &mut self.shaper,
                                    &mut self.atlas,
                                    &effective_para,
                                    ParagraphFrame::indented_in(
                                        format.margin_left,
                                        indent,
                                        y,
                                        content_width,
                                    ),
                                    tw_model::Color::BLACK.to_argb(),
                                );

                                if let Some(ref marker) = list_marker {
                                    apply_list_markers(
                                        &mut self.shaper,
                                        &mut self.atlas,
                                        &mut lines,
                                        marker,
                                        format.margin_left + indent - hanging,
                                        tw_model::Color::BLACK.to_argb(),
                                    );
                                    for line in &mut lines {
                                        line.list_marker = Some(marker.clone());
                                    }
                                }

                                if y + height > content_bottom && !current_boxes.is_empty() {
                                    flush_page(
                                        &mut pages,
                                        &mut current_boxes,
                                        &mut current_lines,
                                        page_index,
                                        &format,
                                        &mut self.shaper,
                                        &mut self.atlas,
                                    );
                                    page_index += 1;
                                    y = format.margin_top;
                                    continue;
                                }

                                for line in lines {
                                    current_lines.push(line.clone());
                                    current_boxes.push(LayoutBox::TextLine(line));
                                }
                                y += height + resolved_para.space_after.unwrap_or(0.0);
                                break;
                            }
                        }
                    }
                    Block::Table(table) => {
                        // Tables taller than the page are split row by row so
                        // they continue onto following pages instead of
                        // spilling past the bottom margin.
                        let mut start_row = 0usize;
                        while start_row < table.rows.len() {
                            let slice = layout_table_slice(
                                &mut self.shaper,
                                &mut self.atlas,
                                table,
                                start_row,
                                format.margin_left,
                                y,
                                content_width,
                                content_bottom - y,
                                tw_model::Color::BLACK.to_argb(),
                            );

                            let overflows = y + slice.layout.height > content_bottom;
                            if overflows && !current_boxes.is_empty() {
                                flush_page(
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    page_index,
                                    &format,
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
                            start_row += slice.rows_placed.max(1);
                            current_boxes.push(LayoutBox::Table(slice.layout));

                            if start_row < table.rows.len() {
                                flush_page(
                                    &mut pages,
                                    &mut current_boxes,
                                    &mut current_lines,
                                    page_index,
                                    &format,
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
    shaper: &mut TextShaper,
    atlas: &mut GlyphAtlas,
) {
    let content_width = format.page_width - format.margin_left - format.margin_right;
    let mut page_boxes = std::mem::take(boxes);

    if let Some(ref header) = format.header_text {
        let header_para = tw_model::Paragraph::with_text(header.clone());
        let (header_lines, _) = layout_paragraph(
            shaper,
            atlas,
            &header_para,
            ParagraphFrame::new(
                format.margin_left,
                format.margin_top * 0.25,
                content_width,
            ),
            tw_model::Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            }
            .to_argb(),
        );
        for line in header_lines {
            page_boxes.insert(0, LayoutBox::TextLine(line));
        }
    }

    if let Some(ref footer) = format.footer_text {
        let footer_para = tw_model::Paragraph::with_text(footer.clone());
        let footer_y = format.page_height - format.margin_bottom * 0.75;
        let (footer_lines, _) = layout_paragraph(
            shaper,
            atlas,
            &footer_para,
            ParagraphFrame::new(format.margin_left, footer_y, content_width),
            tw_model::Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            }
            .to_argb(),
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

struct ParagraphSegment {
    paragraph: Paragraph,
    page_break_before: bool,
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
    let mut current = Paragraph::new();
    current.id = para.id;
    current.format = para.format.clone();
    current.style_id = para.style_id;
    let mut page_break_before = para.format.page_break_before == Some(true);

    for run in &para.runs {
        match &run.content {
            RunContent::Break(BreakType::Page) => {
                if !current.runs.is_empty() || page_break_before {
                    segments.push(ParagraphSegment {
                        paragraph: current,
                        page_break_before,
                    });
                    current = Paragraph::new();
                    current.format = para.format.clone();
                    current.style_id = para.style_id;
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
}

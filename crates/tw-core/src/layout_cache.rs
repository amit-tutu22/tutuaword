use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tw_edit::{DocPosition, DocRange};
use tw_layout::{HitTestResult, LayoutEngine, LineMap};
use tw_model::{CharFormat, Document, NodeId, ParaFormat};
use tw_text::TextBuffer;

#[derive(Clone, Default)]
pub struct LayoutCache {
    line_maps: HashMap<u32, LineMap>,
    page_epochs: HashMap<u32, u64>,
    pending_pages: HashSet<u32>,
    layout_epoch: u64,
    page_count: u32,
    document: Document,
    buffer: TextBuffer,
}

impl LayoutCache {
    pub fn update_from_engine(&mut self, engine: &LayoutEngine) {
        self.page_count = engine.page_count() as u32;
        self.line_maps.clear();
        for page in 0..self.page_count {
            if let Some(map) = engine.line_map(page) {
                self.line_maps.insert(page, map.clone());
            }
        }
    }

    pub fn update_from_session(
        &mut self,
        engine: &LayoutEngine,
        document: &Document,
        buffer: &TextBuffer,
    ) {
        self.layout_epoch += 1;
        self.update_from_engine(engine);
        self.page_epochs.clear();
        self.pending_pages.clear();
        for page in 0..self.page_count {
            self.page_epochs.insert(page, self.layout_epoch);
        }
        self.document = document.clone();
        self.buffer = buffer.clone();
    }

    /// Incremental layout update: refresh relayouted pages, mark downstream pending.
    pub fn update_from_session_incremental(
        &mut self,
        engine: &LayoutEngine,
        document: &Document,
        buffer: &TextBuffer,
        epoch: u64,
        relayout_start: u32,
        page_count: u32,
    ) {
        self.layout_epoch = epoch;
        self.page_count = page_count.max(1);
        self.document = document.clone();
        self.buffer = buffer.clone();

        let relayout_end = relayout_start + engine.last_relayout_pages() as u32;
        for page in relayout_start..relayout_end.min(page_count) {
            if let Some(map) = engine.line_map(page) {
                self.line_maps.insert(page, map.clone());
                self.page_epochs.insert(page, epoch);
                self.pending_pages.remove(&page);
            }
        }
        for page in relayout_end..page_count {
            if !self.line_maps.contains_key(&page) {
                self.pending_pages.insert(page);
            }
        }
    }

    pub fn is_page_stale(&self, page: u32) -> bool {
        self.pending_pages.contains(&page)
    }

    pub fn page_epoch(&self, page: u32) -> Option<u64> {
        self.page_epochs.get(&page).copied()
    }

    pub fn text_in_range(
        &self,
        start_run: NodeId,
        start_offset: usize,
        end_run: NodeId,
        end_offset: usize,
    ) -> Option<String> {
        let range = DocRange {
            start: DocPosition {
                run_id: start_run,
                char_offset: start_offset,
            },
            end: DocPosition {
                run_id: end_run,
                char_offset: end_offset,
            },
        };
        tw_edit::text_in_range(&self.document, &self.buffer, &range).ok()
    }

    /// Resolved character and paragraph format at a caret position.
    pub fn format_at(&self, run_id: NodeId) -> Option<(CharFormat, ParaFormat, Option<String>)> {
        let (si, bi, ri) = self.document.find_run_location(run_id)?;
        let block = self.document.sections.get(si)?.blocks.get(bi)?;
        let para = block.paragraph()?;
        let run = para.runs.get(ri)?;
        let char_format = self
            .document
            .styles
            .resolve_char_format(para.style_id, &run.format);
        let para_format = self
            .document
            .styles
            .resolve_para_format(para.style_id, &para.format);
        let style_name = para.style_id.and_then(|id| {
            self.document
                .styles
                .paragraph_styles
                .get(&id)
                .map(|style| style.name.clone())
        });
        Some((char_format, para_format, style_name))
    }

    pub fn hit_test(&self, page: u32, x: f32, y: f32) -> Option<HitTestResult> {
        if self.is_page_stale(page) {
            return None;
        }
        if let Some(map) = self.line_maps.get(&page) {
            if let Some(mut result) = map.hit_test(x, y) {
                result.page = page;
                return Some(result);
            }
            if map.lines.is_empty() {
                return self.empty_page_tail_hit(page);
            }
        }
        None
    }

    /// Last editable position in the document (for select-all / paste-at-end).
    pub fn document_tail_hit(&self, page: u32) -> Option<HitTestResult> {
        let run_id = last_text_run(&self.document)?;
        let char_offset = self.buffer.len(run_id);
        Some(HitTestResult {
            page,
            run_id,
            char_offset,
        })
    }

    /// When a page has no laid-out lines, anchor at the end of the document so
    /// the user can keep typing; the requested page is recorded for caret display.
    fn empty_page_tail_hit(&self, page: u32) -> Option<HitTestResult> {
        self.document_tail_hit(page)
    }

    pub fn caret_geometry(&self, page: u32, x: f32, y: f32) -> Option<(f32, f32, f32)> {
        let map = self.line_maps.get(&page)?;
        for line in &map.lines {
            if y >= line.y - line.ascent && y <= line.y + line.descent {
                for &(x_start, x_end, _, _) in &line.run_map {
                    if x >= x_start && x <= x_end {
                        return Some((x, line.y, line.ascent + line.descent));
                    }
                }
                return Some((line.x, line.y, line.ascent + line.descent));
            }
        }
        map.lines.first().map(|line| (line.x, line.y, line.ascent + line.descent))
    }

    pub fn caret_at(&self, page: u32, run_id: tw_model::NodeId, char_offset: usize) -> Option<(f32, f32, f32)> {
        if let Some(map) = self.line_maps.get(&page) {
            if let Some(geom) = map.caret_at(run_id, char_offset) {
                return Some(geom);
            }
        }
        for map in self.line_maps.values() {
            if let Some(geom) = map.caret_at(run_id, char_offset) {
                return Some(geom);
            }
        }
        None
    }

    pub fn selection_rects(&self, page: u32, start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> Vec<f32> {
        let Some(map) = self.line_maps.get(&page) else {
            return Vec::new();
        };
        let mut rects = Vec::new();
        let mut in_selection = false;
        for line in &map.lines {
            let line_top = line.y - line.ascent;
            let line_bottom = line.y + line.descent;
            if end_y < line_top || start_y > line_bottom {
                continue;
            }
            let sel_x_start = if start_y <= line_top { line.x } else { start_x };
            let sel_x_end = if end_y >= line_bottom { line.x + line.width } else { end_x };
            if sel_x_end > sel_x_start {
                rects.extend_from_slice(&[sel_x_start, line_top, sel_x_end - sel_x_start, line_bottom - line_top]);
            }
            in_selection = true;
        }
        let _ = in_selection;
        rects
    }

    pub fn page_count(&self) -> u32 {
        self.page_count.max(1)
    }
}

fn last_text_run(doc: &Document) -> Option<NodeId> {
    for section in doc.sections.iter().rev() {
        for block in section.blocks.iter().rev() {
            let para = block.paragraph()?;
            let run = para.runs.last()?;
            return Some(run.id);
        }
    }
    None
}

pub type SharedLayoutCache = Arc<RwLock<LayoutCache>>;

pub fn new_shared_layout_cache() -> SharedLayoutCache {
    Arc::new(RwLock::new(LayoutCache::default()))
}

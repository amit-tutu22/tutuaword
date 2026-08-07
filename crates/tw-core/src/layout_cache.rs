use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tw_edit::{run_char_len_by_id, text_in_range, DocPosition, DocRange};
use tw_layout::{HitTestResult, LayoutEngine, LineMap};
use tw_model::{CharFormat, Document, NodeId, ParaFormat};

#[derive(Clone)]
pub struct LayoutCache {
    line_maps: HashMap<u32, LineMap>,
    page_epochs: HashMap<u32, u64>,
    pending_pages: HashSet<u32>,
    layout_epoch: u64,
    page_count: u32,
    document: Arc<Document>,
    document_version: u64,
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

    pub fn update_from_session(&mut self, engine: &LayoutEngine, document: Arc<Document>) {
        self.layout_epoch += 1;
        self.document_version = self.layout_epoch;
        self.update_from_engine(engine);
        self.page_epochs.clear();
        self.pending_pages.clear();
        for page in 0..self.page_count {
            self.page_epochs.insert(page, self.layout_epoch);
        }
        self.document = document;
    }

    /// Incremental layout update: refresh relayouted pages and record whether the
    /// engine still owes a forward reflow for the pages after them.
    pub fn update_from_session_incremental(
        &mut self,
        engine: &LayoutEngine,
        document: Arc<Document>,
        epoch: u64,
        page_count: u32,
        pending_forward_reflow: bool,
    ) {
        self.layout_epoch = epoch;
        self.document_version = epoch;
        self.page_count = page_count.max(1);
        self.document = document;

        self.line_maps.retain(|page, _| *page < page_count);
        self.page_epochs.retain(|page, _| *page < page_count);

        let mut relayout_end = engine.last_relayout_start_page() as u32;
        for &page in engine.dirty_pages() {
            if page >= page_count {
                continue;
            }
            if let Some(map) = engine.line_map(page) {
                self.line_maps.insert(page, map.clone());
                self.page_epochs.insert(page, epoch);
            }
            relayout_end = relayout_end.max(page + 1);
        }

        // Pages past the reflow window still carry pre-edit geometry until the
        // background pass reaches them; hit testing must not trust them. The floor
        // comes from the engine rather than this pass's `relayout_end`: a converged
        // pass rebuilds only a short prefix but does not re-invalidate the pages an
        // earlier capped pass already brought up to date.
        self.pending_pages.clear();
        if pending_forward_reflow {
            let stale_from = engine
                .pending_reflow_page()
                .map_or(relayout_end, |page| page as u32);
            self.pending_pages.extend(stale_from..page_count);
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Cheap Arc handoff for read-only snapshot consumers.
    pub fn document_snapshot(&self) -> Arc<Document> {
        Arc::clone(&self.document)
    }

    pub fn document_version(&self) -> u64 {
        self.document_version
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
        text_in_range(&self.document, &range).ok()
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
        let char_offset = run_char_len_by_id(&self.document, run_id);
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

    /// Width of the first laid-out line on a page, if layout has run.
    pub fn first_line_width(&self, page: u32) -> f32 {
        self.line_maps
            .get(&page)
            .and_then(|map| map.lines.first())
            .map(|line| line.width)
            .unwrap_or(0.0)
    }
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self {
            line_maps: HashMap::new(),
            page_epochs: HashMap::new(),
            pending_pages: HashSet::new(),
            layout_epoch: 0,
            page_count: 0,
            document: Arc::new(Document::default()),
            document_version: 0,
        }
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

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tw_layout::{HitTestResult, LayoutEngine, LineMap};

#[derive(Debug, Clone, Default)]
pub struct LayoutCache {
    line_maps: HashMap<u32, LineMap>,
    page_count: u32,
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

    pub fn hit_test(&self, page: u32, x: f32, y: f32) -> Option<HitTestResult> {
        let map = self.line_maps.get(&page)?;
        let mut result = map.hit_test(x, y)?;
        result.page = page;
        Some(result)
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
        let map = self.line_maps.get(&page)?;
        map.caret_at(run_id, char_offset)
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

pub type SharedLayoutCache = Arc<RwLock<LayoutCache>>;

pub fn new_shared_layout_cache() -> SharedLayoutCache {
    Arc::new(RwLock::new(LayoutCache::default()))
}

use parking_lot::RwLock;
use std::sync::Arc;
use tw_render::DisplayList;

#[derive(Debug, Clone)]
pub struct SinglePageSnapshot {
    pub version: u64,
    pub bytes: Arc<Vec<u8>>,
    pub page_width: f32,
    pub page_height: f32,
}

#[derive(Debug, Clone)]
pub struct PageSnapshot {
    pub version: u64,
    pub page_index: u32,
    pub page_count: u32,
    pub pages: Arc<Vec<Arc<SinglePageSnapshot>>>,
    /// Current page display list bytes (Arc handoff; not a deep clone of page payload).
    pub bytes: Arc<Vec<u8>>,
    pub page_width: f32,
    pub page_height: f32,
    pub atlas_generation: u64,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub atlas_bytes: Arc<Vec<u8>>,
    pub document_text: String,
    pub document_properties_json: String,
    pub read_only: bool,
}

impl PageSnapshot {
    pub fn page(&self, index: u32) -> Option<Arc<SinglePageSnapshot>> {
        self.pages.get(index as usize).cloned()
    }

    pub fn total_stored_page_bytes(&self) -> usize {
        self.pages.iter().map(|p| p.bytes.len()).sum()
    }
}

pub struct SnapshotBuffer {
    front: RwLock<Arc<PageSnapshot>>,
    back: RwLock<Arc<PageSnapshot>>,
}

impl SnapshotBuffer {
    pub fn new() -> Self {
        let empty = Arc::new(empty_snapshot());
        Self {
            front: RwLock::new(empty.clone()),
            back: RwLock::new(empty),
        }
    }

    pub fn publish(&self, snapshot: PageSnapshot) {
        let snapshot = Arc::new(snapshot);
        let mut back = self.back.write();
        *back = snapshot.clone();
        let mut front = self.front.write();
        std::mem::swap(&mut *back, &mut *front);
    }

    pub fn read(&self) -> Arc<PageSnapshot> {
        self.front.read().clone()
    }

    pub fn set_current_page(&self, page: u32) {
        let current = self.read();
        let page_index = page.min(current.page_count.saturating_sub(1));
        if page_index == current.page_index {
            return;
        }
        let updated = Arc::new(current.with_page_index(page_index));
        *self.front.write() = updated;
    }

    /// Incrementally update metadata and only the pages listed in `updated_indices`.
    pub fn publish_incremental(
        &self,
        version: u64,
        page_index: u32,
        page_count: u32,
        updated_indices: &[u32],
        updated_pages: &[Arc<SinglePageSnapshot>],
        atlas_generation: u64,
        atlas_width: u32,
        atlas_height: u32,
        atlas_bytes: Arc<Vec<u8>>,
        document_text: String,
        document_properties_json: String,
        read_only: bool,
    ) {
        debug_assert_eq!(updated_indices.len(), updated_pages.len());
        let current = self.read();
        let mut pages: Vec<Arc<SinglePageSnapshot>> = current.pages.as_ref().clone();
        if pages.len() < page_count as usize {
            pages.resize_with(page_count as usize, || {
                Arc::new(SinglePageSnapshot {
                    version: 0,
                    bytes: Arc::new(Vec::new()),
                    page_width: 612.0,
                    page_height: 792.0,
                })
            });
        } else if pages.len() > page_count as usize {
            pages.truncate(page_count as usize);
        }
        for (idx, page) in updated_indices.iter().zip(updated_pages.iter()) {
            if (*idx as usize) < pages.len() {
                pages[*idx as usize] = page.clone();
            }
        }
        let pages = Arc::new(pages);
        let snapshot = snapshot_from_page_arc(
            pages,
            page_index,
            version,
            atlas_generation,
            atlas_width,
            atlas_height,
            atlas_bytes,
            document_text,
            document_properties_json,
            read_only,
        );
        self.publish(snapshot);
    }
}

impl PageSnapshot {
    fn with_page_index(self: &Arc<Self>, page_index: u32) -> PageSnapshot {
        let page_index = page_index.min(self.page_count.saturating_sub(1));
        let (bytes, page_width, page_height) = if let Some(current) = self.pages.get(page_index as usize) {
            (
                current.bytes.clone(),
                current.page_width,
                current.page_height,
            )
        } else {
            (Arc::new(Vec::new()), 612.0, 792.0)
        };
        PageSnapshot {
            version: self.version,
            page_index,
            page_count: self.page_count,
            pages: self.pages.clone(),
            bytes,
            page_width,
            page_height,
            atlas_generation: self.atlas_generation,
            atlas_width: self.atlas_width,
            atlas_height: self.atlas_height,
            atlas_bytes: self.atlas_bytes.clone(),
            document_text: self.document_text.clone(),
            document_properties_json: self.document_properties_json.clone(),
            read_only: self.read_only,
        }
    }
}

fn empty_snapshot() -> PageSnapshot {
    snapshot_from_page_arc(
        Arc::new(vec![Arc::new(SinglePageSnapshot {
            version: 0,
            bytes: Arc::new(Vec::new()),
            page_width: 612.0,
            page_height: 792.0,
        })]),
        0,
        0,
        0,
        0,
        0,
        Arc::new(Vec::new()),
        String::new(),
        String::from("{}"),
        false,
    )
}

pub fn snapshot_from_page_arc(
    pages: Arc<Vec<Arc<SinglePageSnapshot>>>,
    page_index: u32,
    version: u64,
    atlas_generation: u64,
    atlas_width: u32,
    atlas_height: u32,
    atlas_bytes: Arc<Vec<u8>>,
    document_text: String,
    document_properties_json: String,
    read_only: bool,
) -> PageSnapshot {
    let page_count = pages.len().max(1) as u32;
    let page_index = page_index.min(page_count.saturating_sub(1));
    let (bytes, page_width, page_height) = if let Some(current) = pages.get(page_index as usize) {
        (
            current.bytes.clone(),
            current.page_width,
            current.page_height,
        )
    } else {
        (Arc::new(Vec::new()), 612.0, 792.0)
    };
    PageSnapshot {
        version,
        page_index,
        page_count,
        pages,
        bytes,
        page_width,
        page_height,
        atlas_generation,
        atlas_width,
        atlas_height,
        atlas_bytes,
        document_text,
        document_properties_json,
        read_only,
    }
}

pub fn snapshot_from_pages(
    pages: Vec<SinglePageSnapshot>,
    page_index: u32,
    version: u64,
    atlas_generation: u64,
    atlas_width: u32,
    atlas_height: u32,
    atlas_bytes: Arc<Vec<u8>>,
    document_text: String,
    document_properties_json: String,
    read_only: bool,
) -> PageSnapshot {
    let arc_pages: Vec<Arc<SinglePageSnapshot>> = pages.into_iter().map(Arc::new).collect();
    snapshot_from_page_arc(
        Arc::new(arc_pages),
        page_index,
        version,
        atlas_generation,
        atlas_width,
        atlas_height,
        atlas_bytes,
        document_text,
        document_properties_json,
        read_only,
    )
}

pub fn snapshot_from_display_list(
    list: &DisplayList,
    page_index: u32,
    page_count: u32,
    document_text: String,
    document_properties_json: String,
    read_only: bool,
) -> PageSnapshot {
    let _ = page_count;
    snapshot_from_pages(
        vec![SinglePageSnapshot {
            version: list.version,
            bytes: Arc::new(tw_render::DisplayListBuilder::to_page_bytes(list)),
            page_width: list.page_width,
            page_height: list.page_height,
        }],
        page_index,
        list.version,
        0,
        0,
        0,
        Arc::new(Vec::new()),
        document_text,
        document_properties_json,
        read_only,
    )
}

pub fn document_plain_text(doc: &tw_model::Document) -> String {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .map(|p| p.visible_text())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn document_properties_json(doc: &tw_model::Document, layout_page_count: u32) -> String {
    let mut props = doc.properties.clone();
    if props.page_count.is_none() {
        props.page_count = Some(layout_page_count);
    }
    serde_json::to_string(&props).unwrap_or_else(|_| "{}".into())
}

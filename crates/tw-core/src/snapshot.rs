use parking_lot::RwLock;
use tw_render::DisplayList;

#[derive(Debug, Clone, Default)]
pub struct SinglePageSnapshot {
    pub bytes: Vec<u8>,
    pub page_width: f32,
    pub page_height: f32,
}

#[derive(Debug, Clone)]
pub struct PageSnapshot {
    pub version: u64,
    pub page_index: u32,
    pub page_count: u32,
    pub pages: Vec<SinglePageSnapshot>,
    pub bytes: Vec<u8>,
    pub page_width: f32,
    pub page_height: f32,
    pub document_text: String,
}

impl PageSnapshot {
    pub fn with_page_index(mut self, page_index: u32) -> Self {
        self.page_index = page_index.min(self.page_count.saturating_sub(1));
        if let Some(current) = self.pages.get(self.page_index as usize) {
            self.bytes = current.bytes.clone();
            self.page_width = current.page_width;
            self.page_height = current.page_height;
        }
        self
    }
}

pub struct SnapshotBuffer {
    front: RwLock<PageSnapshot>,
    back: RwLock<PageSnapshot>,
}

impl SnapshotBuffer {
    pub fn new() -> Self {
        let empty = PageSnapshot {
            version: 0,
            page_index: 0,
            page_count: 1,
            pages: vec![SinglePageSnapshot {
                bytes: Vec::new(),
                page_width: 612.0,
                page_height: 792.0,
            }],
            bytes: Vec::new(),
            page_width: 612.0,
            page_height: 792.0,
            document_text: String::new(),
        };
        Self {
            front: RwLock::new(empty.clone()),
            back: RwLock::new(empty),
        }
    }

    pub fn publish(&self, snapshot: PageSnapshot) {
        let mut back = self.back.write();
        *back = snapshot;
        let mut front = self.front.write();
        std::mem::swap(&mut *back, &mut *front);
    }

    pub fn read(&self) -> PageSnapshot {
        self.front.read().clone()
    }

    pub fn set_current_page(&self, page: u32) {
        let mut front = self.front.write();
        *front = front.clone().with_page_index(page);
    }
}

pub fn snapshot_from_pages(
    pages: Vec<SinglePageSnapshot>,
    page_index: u32,
    version: u64,
    document_text: String,
) -> PageSnapshot {
    let page_count = pages.len().max(1) as u32;
    let mut snapshot = PageSnapshot {
        version,
        page_index,
        page_count,
        pages,
        bytes: Vec::new(),
        page_width: 612.0,
        page_height: 792.0,
        document_text,
    };
    snapshot = snapshot.with_page_index(page_index);
    snapshot
}

pub fn snapshot_from_display_list(
    list: &DisplayList,
    page_index: u32,
    page_count: u32,
    document_text: String,
) -> PageSnapshot {
    let _ = page_count;
    snapshot_from_pages(
        vec![SinglePageSnapshot {
            bytes: tw_render::DisplayListBuilder::to_bytes(list),
            page_width: list.page_width,
            page_height: list.page_height,
        }],
        page_index,
        list.version,
        document_text,
    )
}

pub fn document_plain_text(doc: &tw_model::Document) -> String {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .map(|p| p.full_text())
        .collect::<Vec<_>>()
        .join("\n")
}

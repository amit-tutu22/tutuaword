//! Debug counters for FFI buffer clone sizes (B0 measurement hooks).

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct TransferStats {
    pub page_dl_bytes_cloned: AtomicU64,
    pub atlas_bytes_cloned: AtomicU64,
    pub image_payload_bytes_embedded: AtomicU64,
    pub page_dl_transfers: AtomicU64,
    pub atlas_transfers: AtomicU64,
    pub arc_unwrap_moves: AtomicU64,
    pub arc_shared_clones: AtomicU64,
}

static STATS: TransferStats = TransferStats {
    page_dl_bytes_cloned: AtomicU64::new(0),
    atlas_bytes_cloned: AtomicU64::new(0),
    image_payload_bytes_embedded: AtomicU64::new(0),
    page_dl_transfers: AtomicU64::new(0),
    atlas_transfers: AtomicU64::new(0),
    arc_unwrap_moves: AtomicU64::new(0),
    arc_shared_clones: AtomicU64::new(0),
};

pub fn reset() {
    STATS.page_dl_bytes_cloned.store(0, Ordering::Relaxed);
    STATS.atlas_bytes_cloned.store(0, Ordering::Relaxed);
    STATS.image_payload_bytes_embedded.store(0, Ordering::Relaxed);
    STATS.page_dl_transfers.store(0, Ordering::Relaxed);
    STATS.atlas_transfers.store(0, Ordering::Relaxed);
    STATS.arc_unwrap_moves.store(0, Ordering::Relaxed);
    STATS.arc_shared_clones.store(0, Ordering::Relaxed);
}

pub fn record_page_dl_transfer(cloned_bytes: usize, page_bytes: &[u8]) {
    STATS
        .page_dl_bytes_cloned
        .fetch_add(cloned_bytes as u64, Ordering::Relaxed);
    STATS.page_dl_transfers.fetch_add(1, Ordering::Relaxed);
    let embedded = image_payload_bytes_in_page_wire(page_bytes);
    STATS
        .image_payload_bytes_embedded
        .fetch_add(embedded as u64, Ordering::Relaxed);
}

pub fn record_atlas_transfer(cloned_bytes: usize) {
    STATS
        .atlas_bytes_cloned
        .fetch_add(cloned_bytes as u64, Ordering::Relaxed);
    STATS.atlas_transfers.fetch_add(1, Ordering::Relaxed);
}

pub fn record_arc_move() {
    STATS.arc_unwrap_moves.fetch_add(1, Ordering::Relaxed);
}

pub fn record_arc_clone() {
    STATS.arc_shared_clones.fetch_add(1, Ordering::Relaxed);
}

/// Sum encoded image payload lengths embedded in a page display-list wire blob.
pub fn image_payload_bytes_in_page_wire(bytes: &[u8]) -> usize {
    if bytes.len() < 8 {
        return 0;
    }
    let file_version = u32::from_le_bytes(bytes[0..4].try_into().unwrap_or([0; 4]));
    let mut offset = 4 + 8 + 4 + 4; // version + layout version + page w/h

    offset = skip_glyph_batch(bytes, offset);
    offset = skip_rect_batch(bytes, offset);
    offset = skip_path_batch(bytes, offset);

    let Some(image_count) = read_u32_at(bytes, offset) else {
        return 0;
    };
    offset += 4;
    offset = offset.saturating_add(image_count as usize * 2 * 4); // transforms
    offset = offset.saturating_add(image_count as usize * 2 * 4); // sizes

    for _ in 0..image_count {
        let Some(len) = read_u32_at(bytes, offset) else {
            return 0;
        };
        offset += 4 + len as usize;
    }
    if file_version >= 5 {
        for _ in 0..image_count {
            let Some(len) = read_u32_at(bytes, offset) else {
                return 0;
            };
            offset += 4 + len as usize;
        }
    }

    let mut total = 0usize;
    for _ in 0..image_count {
        let Some(len) = read_u32_at(bytes, offset) else {
            return total;
        };
        offset += 4;
        total += len as usize;
        offset = offset.saturating_add(len as usize);
    }
    total
}

fn read_u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?))
}

fn skip_glyph_batch(bytes: &[u8], mut offset: usize) -> usize {
    let Some(count) = read_u32_at(bytes, offset) else {
        return offset;
    };
    offset + 4 + count as usize * (2 + 4) * 4 + count as usize * 4
}

fn skip_rect_batch(bytes: &[u8], mut offset: usize) -> usize {
    let Some(count) = read_u32_at(bytes, offset) else {
        return offset;
    };
    offset + 4 + count as usize * 4 * 4 + count as usize * 4
}

fn skip_path_batch(bytes: &[u8], mut offset: usize) -> usize {
    let Some(count) = read_u32_at(bytes, offset) else {
        return offset;
    };
    offset + 4 + count as usize * 4 * 4 + count as usize * 4
}

/// Snapshot counters for test hooks and `tw_get_transfer_stats`.
pub fn snapshot() -> (u64, u64, u64, u64, u64, u64, u64) {
    (
        STATS.page_dl_bytes_cloned.load(Ordering::Relaxed),
        STATS.atlas_bytes_cloned.load(Ordering::Relaxed),
        STATS.image_payload_bytes_embedded.load(Ordering::Relaxed),
        STATS.page_dl_transfers.load(Ordering::Relaxed),
        STATS.atlas_transfers.load(Ordering::Relaxed),
        STATS.arc_unwrap_moves.load(Ordering::Relaxed),
        STATS.arc_shared_clones.load(Ordering::Relaxed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_render::{DisplayList, DisplayListBuilder, ImageBatch};

    #[test]
    fn image_payload_meter_reads_batch() {
        let list = DisplayList {
            version: 1,
            page_width: 612.0,
            page_height: 792.0,
            atlas_width: 0,
            atlas_height: 0,
            atlas_pixels: Vec::new(),
            atlas_batch: Default::default(),
            rect_batch: Default::default(),
            image_batch: ImageBatch {
                transforms: vec![0.0, 0.0],
                sizes: vec![10.0, 10.0],
                asset_ids: vec!["a1".into()],
                image_ids: vec!["id".into()],
                payloads: vec![vec![1, 2, 3, 4, 5]],
                rotations: vec![0.0],
                opacities: vec![1.0],
                crop_rects: vec![0.0, 0.0, 0.0, 0.0],
            },
            path_batch: Default::default(),
            shape_selection_batch: Default::default(),
            formatting_marks_batch: Default::default(),
        };
        let bytes = DisplayListBuilder::to_page_bytes(&list);
        assert_eq!(image_payload_bytes_in_page_wire(&bytes), 0, "v9 omits embedded payloads");
    }
}

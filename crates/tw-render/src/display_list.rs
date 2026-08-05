use tw_layout::{LayoutBox, PageLayout, TextLine};
use tw_shape::GlyphAtlas;

/// v2 added path and image batches; v3 added encoded image payloads.
pub const DISPLAY_LIST_VERSION: u32 = 3;

#[derive(Debug, Clone)]
pub struct DisplayList {
    pub version: u64,
    pub page_width: f32,
    pub page_height: f32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub atlas_pixels: Vec<u8>,
    pub atlas_batch: AtlasBatch,
    pub rect_batch: RectBatch,
    pub image_batch: ImageBatch,
    pub path_batch: PathBatch,
}

#[derive(Debug, Clone, Default)]
pub struct AtlasBatch {
    pub transforms: Vec<f32>,
    pub rects: Vec<f32>,
    pub colors: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct RectBatch {
    pub rects: Vec<f32>,
    pub colors: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct ImageBatch {
    pub transforms: Vec<f32>,
    pub sizes: Vec<f32>,
    pub asset_ids: Vec<String>,
    /// Source bytes (PNG, JPEG, ...) per image, decoded by the platform.
    /// Empty for an image whose asset could not be resolved.
    pub payloads: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Default)]
pub struct PathBatch {
    pub points: Vec<f32>,
    pub colors: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct DisplayListSnapshot {
    pub version: u64,
    pub page_index: u32,
    pub page_count: u32,
    pub display_list: DisplayList,
}

pub struct DisplayListBuilder;

impl DisplayListBuilder {
    pub fn from_page(page: &PageLayout, atlas: &GlyphAtlas, version: u64) -> DisplayList {
        let mut atlas_batch = AtlasBatch::default();
        let mut rect_batch = RectBatch::default();
        let mut image_batch = ImageBatch::default();
        let mut path_batch = PathBatch::default();

        for layout_box in &page.boxes {
            match layout_box {
                LayoutBox::TextLine(line) => {
                    append_line_decorations(line, &mut rect_batch);
                    append_line_glyphs(line, &mut atlas_batch);
                }
                LayoutBox::Rect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => append_rect(*x, *y, *width, *height, *color, &mut rect_batch),
                LayoutBox::Image(img) => {
                    image_batch.transforms.push(img.x);
                    image_batch.transforms.push(img.y);
                    image_batch.sizes.push(img.width);
                    image_batch.sizes.push(img.height);
                    image_batch.asset_ids.push(img.asset_id.clone());
                    image_batch.payloads.push(img.encoded.as_ref().clone());
                    if img.encoded.is_empty() {
                        // Nothing to decode, so mark the slot the way an empty
                        // frame reads in Word.
                        append_rect(
                            img.x,
                            img.y,
                            img.width,
                            img.height,
                            0xFFE0E0E0,
                            &mut rect_batch,
                        );
                    }
                }
                LayoutBox::Table(table) => {
                    for cell in &table.cells {
                        if let Some(bg) = cell.background {
                            append_rect(cell.x, cell.y, cell.width, cell.height, bg, &mut rect_batch);
                        }
                        for line in &cell.lines {
                            append_line_decorations(line, &mut rect_batch);
                            append_line_glyphs(line, &mut atlas_batch);
                        }
                    }
                    for chunk in table.grid_lines.chunks(4) {
                        if chunk.len() == 4 {
                            path_batch.points.extend_from_slice(chunk);
                            let color_idx = path_batch.colors.len();
                            path_batch.colors.push(
                                table
                                    .grid_line_colors
                                    .get(color_idx)
                                    .copied()
                                    .unwrap_or(0xFF000000),
                            );
                        }
                    }
                }
            }
        }

        DisplayList {
            version,
            page_width: page.width,
            page_height: page.height,
            atlas_width: atlas.width,
            atlas_height: atlas.height,
            atlas_pixels: atlas.pixels_rgba().to_vec(),
            atlas_batch,
            rect_batch,
            image_batch,
            path_batch,
        }
    }

    pub fn to_bytes(list: &DisplayList) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&DISPLAY_LIST_VERSION.to_le_bytes());
        bytes.extend_from_slice(&list.version.to_le_bytes());
        bytes.extend_from_slice(&list.page_width.to_le_bytes());
        bytes.extend_from_slice(&list.page_height.to_le_bytes());
        bytes.extend_from_slice(&list.atlas_width.to_le_bytes());
        bytes.extend_from_slice(&list.atlas_height.to_le_bytes());

        let atlas_len = list.atlas_pixels.len() as u32;
        bytes.extend_from_slice(&atlas_len.to_le_bytes());
        bytes.extend_from_slice(&list.atlas_pixels);

        write_glyph_batch(&mut bytes, &list.atlas_batch);
        write_rect_batch(&mut bytes, &list.rect_batch);
        write_path_batch(&mut bytes, &list.path_batch);
        write_image_batch(&mut bytes, &list.image_batch);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<DisplayList> {
        if bytes.len() < 28 {
            return None;
        }
        let mut offset = 0usize;

        fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
            let end = *offset + 4;
            let v = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
            *offset = end;
            Some(v)
        }

        fn read_u64(bytes: &[u8], offset: &mut usize) -> Option<u64> {
            let end = *offset + 8;
            let v = u64::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
            *offset = end;
            Some(v)
        }

        fn read_f32(bytes: &[u8], offset: &mut usize) -> Option<f32> {
            let end = *offset + 4;
            let v = f32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
            *offset = end;
            Some(v)
        }

        let file_version = read_u32(bytes, &mut offset)?;
        let version = read_u64(bytes, &mut offset)?;
        let page_width = read_f32(bytes, &mut offset)?;
        let page_height = read_f32(bytes, &mut offset)?;
        let atlas_width = read_u32(bytes, &mut offset)?;
        let atlas_height = read_u32(bytes, &mut offset)?;
        let atlas_len = read_u32(bytes, &mut offset)? as usize;
        if offset + atlas_len > bytes.len() {
            return None;
        }
        let atlas_pixels = bytes[offset..offset + atlas_len].to_vec();
        offset += atlas_len;

        let atlas_batch = read_glyph_batch(bytes, &mut offset)?;
        let rect_batch = read_rect_batch(bytes, &mut offset)?;

        let (path_batch, image_batch) = if file_version >= 2 {
            (
                read_path_batch(bytes, &mut offset).unwrap_or_default(),
                read_image_batch(bytes, &mut offset, file_version).unwrap_or_default(),
            )
        } else {
            (PathBatch::default(), ImageBatch::default())
        };

        Some(DisplayList {
            version,
            page_width,
            page_height,
            atlas_width,
            atlas_height,
            atlas_pixels,
            atlas_batch,
            rect_batch,
            image_batch,
            path_batch,
        })
    }
}

fn append_line_decorations(line: &TextLine, batch: &mut RectBatch) {
    for deco in &line.decorations {
        append_rect(deco.x, deco.y, deco.width, deco.height, deco.color, batch);
    }
}

fn append_line_glyphs(line: &TextLine, batch: &mut AtlasBatch) {
    for g in &line.glyphs {
        batch.transforms.push(g.x);
        batch.transforms.push(g.y);
        batch.rects.push(g.atlas_x);
        batch.rects.push(g.atlas_y);
        batch.rects.push(g.atlas_w);
        batch.rects.push(g.atlas_h);
        batch.colors.push(g.color);
    }
}

fn append_rect(x: f32, y: f32, w: f32, h: f32, color: u32, batch: &mut RectBatch) {
    batch.rects.extend_from_slice(&[x, y, w, h]);
    batch.colors.push(color);
}

fn write_glyph_batch(bytes: &mut Vec<u8>, batch: &AtlasBatch) {
    let glyph_count = (batch.transforms.len() / 2) as u32;
    bytes.extend_from_slice(&glyph_count.to_le_bytes());
    for val in &batch.transforms {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    for val in &batch.rects {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    for val in &batch.colors {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
}

fn read_glyph_batch(bytes: &[u8], offset: &mut usize) -> Option<AtlasBatch> {
    fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
        let end = *offset + 4;
        let v = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    fn read_f32(bytes: &[u8], offset: &mut usize) -> Option<f32> {
        let end = *offset + 4;
        let v = f32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    let glyph_count = read_u32(bytes, offset)? as usize;
    let mut transforms = Vec::with_capacity(glyph_count * 2);
    for _ in 0..glyph_count * 2 {
        transforms.push(read_f32(bytes, offset)?);
    }
    let mut rects = Vec::with_capacity(glyph_count * 4);
    for _ in 0..glyph_count * 4 {
        rects.push(read_f32(bytes, offset)?);
    }
    let mut colors = Vec::with_capacity(glyph_count);
    for _ in 0..glyph_count {
        colors.push(read_u32(bytes, offset)?);
    }
    Some(AtlasBatch {
        transforms,
        rects,
        colors,
    })
}

fn write_rect_batch(bytes: &mut Vec<u8>, batch: &RectBatch) {
    let rect_count = (batch.rects.len() / 4) as u32;
    bytes.extend_from_slice(&rect_count.to_le_bytes());
    for val in &batch.rects {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    for val in &batch.colors {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
}

fn read_rect_batch(bytes: &[u8], offset: &mut usize) -> Option<RectBatch> {
    fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
        let end = *offset + 4;
        let v = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    fn read_f32(bytes: &[u8], offset: &mut usize) -> Option<f32> {
        let end = *offset + 4;
        let v = f32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    let rect_count = read_u32(bytes, offset).unwrap_or(0) as usize;
    let mut rects = Vec::with_capacity(rect_count * 4);
    for _ in 0..rect_count * 4 {
        if *offset + 4 > bytes.len() {
            break;
        }
        rects.push(read_f32(bytes, offset)?);
    }
    let mut colors = Vec::with_capacity(rect_count);
    for _ in 0..rect_count {
        if *offset + 4 > bytes.len() {
            break;
        }
        colors.push(read_u32(bytes, offset)?);
    }
    Some(RectBatch { rects, colors })
}

fn write_path_batch(bytes: &mut Vec<u8>, batch: &PathBatch) {
    let line_count = (batch.points.len() / 4) as u32;
    bytes.extend_from_slice(&line_count.to_le_bytes());
    for val in &batch.points {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    for val in &batch.colors {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
}

fn read_path_batch(bytes: &[u8], offset: &mut usize) -> Option<PathBatch> {
    fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
        let end = *offset + 4;
        let v = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    fn read_f32(bytes: &[u8], offset: &mut usize) -> Option<f32> {
        let end = *offset + 4;
        let v = f32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    let line_count = read_u32(bytes, offset)? as usize;
    let mut points = Vec::with_capacity(line_count * 4);
    for _ in 0..line_count * 4 {
        points.push(read_f32(bytes, offset)?);
    }
    let mut colors = Vec::with_capacity(line_count);
    for _ in 0..line_count {
        colors.push(read_u32(bytes, offset)?);
    }
    Some(PathBatch { points, colors })
}

fn write_image_batch(bytes: &mut Vec<u8>, batch: &ImageBatch) {
    let image_count = (batch.transforms.len() / 2) as u32;
    bytes.extend_from_slice(&image_count.to_le_bytes());
    for val in &batch.transforms {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    for val in &batch.sizes {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    for id in &batch.asset_ids {
        let bytes_id = id.as_bytes();
        bytes.extend_from_slice(&(bytes_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(bytes_id);
    }
    for payload in &batch.payloads {
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
    }
}

fn read_image_batch(bytes: &[u8], offset: &mut usize, file_version: u32) -> Option<ImageBatch> {
    fn read_u32(bytes: &[u8], offset: &mut usize) -> Option<u32> {
        let end = *offset + 4;
        let v = u32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    fn read_f32(bytes: &[u8], offset: &mut usize) -> Option<f32> {
        let end = *offset + 4;
        let v = f32::from_le_bytes(bytes.get(*offset..end)?.try_into().ok()?);
        *offset = end;
        Some(v)
    }
    let image_count = read_u32(bytes, offset)? as usize;
    let mut transforms = Vec::with_capacity(image_count * 2);
    for _ in 0..image_count * 2 {
        transforms.push(read_f32(bytes, offset)?);
    }
    let mut sizes = Vec::with_capacity(image_count * 2);
    for _ in 0..image_count * 2 {
        sizes.push(read_f32(bytes, offset)?);
    }
    let mut asset_ids = Vec::with_capacity(image_count);
    for _ in 0..image_count {
        let len = read_u32(bytes, offset)? as usize;
        let end = *offset + len;
        let id = std::str::from_utf8(bytes.get(*offset..end)?).ok()?.to_string();
        *offset = end;
        asset_ids.push(id);
    }
    let mut payloads = vec![Vec::new(); image_count];
    if file_version >= 3 {
        for slot in payloads.iter_mut() {
            let len = read_u32(bytes, offset)? as usize;
            let end = *offset + len;
            *slot = bytes.get(*offset..end)?.to_vec();
            *offset = end;
        }
    }
    Some(ImageBatch {
        transforms,
        sizes,
        asset_ids,
        payloads,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_layout::LayoutEngine;
    use tw_model::Document;

    #[test]
    fn display_list_round_trip() {
        let doc = Document::with_paragraph("Hello World");
        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        let page = layout.pages.first().unwrap();
        let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);
        let bytes = DisplayListBuilder::to_bytes(&list);
        let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.atlas_batch.transforms.len(), list.atlas_batch.transforms.len());
    }
}

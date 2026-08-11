use tw_layout::{LayoutBox, PageLayout, ShapeLayout, TableLayout, TextLine};
use tw_shape::GlyphAtlas;

/// v2 added path and image batches; v3 added encoded image payloads; v4 drops embedded atlas pixels;
/// v5 adds stable image block ids for selection/resize;
/// v6 adds rotation, opacity, and crop metadata per image (F10.S4);
/// v7 adds read-only shape selection bounds for SmartArt placeholders (F12.S3).
pub const DISPLAY_LIST_VERSION: u32 = 7;

/// ARGB fill for read-only imported shape placeholders (F11.S1).
pub const SHAPE_PLACEHOLDER_COLOR: u32 = 0xFFD0DCE8;

/// Separate atlas resource wire format version.
pub const ATLAS_RESOURCE_VERSION: u32 = 1;

/// Atlas pixels transferred independently from page display lists (R1.2).
#[derive(Debug, Clone)]
pub struct AtlasResource {
    pub generation: u64,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

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
    pub shape_selection_batch: ShapeSelectionBatch,
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
    /// Block ids parallel to asset_ids (wire format v5+).
    pub image_ids: Vec<String>,
    /// Source bytes (PNG, JPEG, ...) per image, decoded by the platform.
    /// Empty for an image whose asset could not be resolved.
    pub payloads: Vec<Vec<u8>>,
    /// Clockwise rotation in degrees (wire format v6+).
    pub rotations: Vec<f32>,
    /// Opacity 0..1 (wire format v6+).
    pub opacities: Vec<f32>,
    /// Crop fractions l,t,r,b per image (wire format v6+).
    pub crop_rects: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct PathBatch {
    pub points: Vec<f32>,
    pub colors: Vec<u32>,
}

/// Read-only diagram/shape bounds for hit-testing (F12.S3).
#[derive(Debug, Clone, Default)]
pub struct ShapeSelectionBatch {
    pub shape_ids: Vec<String>,
    pub rects: Vec<f32>,
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
        let mut shape_selection_batch = ShapeSelectionBatch::default();

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
                    if matches!(
                        img.selection_shape_kind,
                        Some(tw_model::ShapeKind::Diagram) | Some(tw_model::ShapeKind::Chart)
                    ) {
                        shape_selection_batch
                            .shape_ids
                            .push(img.image_id.to_string());
                        shape_selection_batch.rects.extend_from_slice(&[
                            img.x, img.y, img.width, img.height,
                        ]);
                    }
                    image_batch.transforms.push(img.x);
                    image_batch.transforms.push(img.y);
                    image_batch.sizes.push(img.width);
                    image_batch.sizes.push(img.height);
                    image_batch.asset_ids.push(img.asset_id.clone());
                    image_batch.image_ids.push(img.image_id.to_string());
                    image_batch.payloads.push(img.encoded.as_ref().clone());
                    image_batch.rotations.push(img.rotation_deg);
                    image_batch.opacities.push(img.opacity);
                    image_batch.crop_rects.push(img.crop_left);
                    image_batch.crop_rects.push(img.crop_top);
                    image_batch.crop_rects.push(img.crop_right);
                    image_batch.crop_rects.push(img.crop_bottom);
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
                LayoutBox::Shape(shape) => {
                    // All inserted shapes participate in object selection (F11–F13).
                    shape_selection_batch
                        .shape_ids
                        .push(shape.shape_id.to_string());
                    shape_selection_batch
                        .rects
                        .extend_from_slice(&[shape.x, shape.y, shape.width, shape.height]);
                    if shape.fill.is_none() && shape.stroke.is_none() {
                        match shape.shape_type {
                            tw_model::ShapeKind::Chart if shape.chart_data.is_some() => {
                                append_chart_surface(shape, &mut rect_batch);
                                append_chart_preview(shape, &mut rect_batch, &mut path_batch);
                            }
                            tw_model::ShapeKind::Diagram => {
                                append_shape_placeholder(shape, &mut rect_batch);
                                append_diagram_preview(
                                    shape,
                                    &mut rect_batch,
                                    &mut path_batch,
                                );
                            }
                            _ => append_shape_placeholder(shape, &mut rect_batch),
                        }
                    } else {
                        append_shape_geometry(shape, &mut rect_batch, &mut path_batch);
                    }
                    if shape.shape_type == tw_model::ShapeKind::WordArt {
                        append_word_art_path(shape, &mut path_batch);
                    }
                }
                LayoutBox::Table(table) => {
                    shape_selection_batch
                        .shape_ids
                        .push(table.table_id.to_string());
                    shape_selection_batch.rects.extend_from_slice(&[
                        table.x,
                        table.y,
                        table.width,
                        table.height,
                    ]);
                    append_table_layout(table, &mut rect_batch, &mut path_batch, &mut atlas_batch);
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
            shape_selection_batch,
        }
    }

    /// Build a page display list without embedding atlas pixels (v4 page payload).
    pub fn from_page_without_atlas(page: &PageLayout, version: u64) -> DisplayList {
        let mut list = Self::from_page(page, &GlyphAtlas::default(), version);
        list.atlas_width = 0;
        list.atlas_height = 0;
        list.atlas_pixels.clear();
        list
    }

    /// Serialize a page display list without embedded atlas pixels (wire format v4).
    pub fn to_page_bytes(list: &DisplayList) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&DISPLAY_LIST_VERSION.to_le_bytes());
        bytes.extend_from_slice(&list.version.to_le_bytes());
        bytes.extend_from_slice(&list.page_width.to_le_bytes());
        bytes.extend_from_slice(&list.page_height.to_le_bytes());

        write_glyph_batch(&mut bytes, &list.atlas_batch);
        write_rect_batch(&mut bytes, &list.rect_batch);
        write_path_batch(&mut bytes, &list.path_batch);
        write_image_batch(&mut bytes, &list.image_batch);
        write_shape_selection_batch(&mut bytes, &list.shape_selection_batch);

        bytes
    }

    /// Compare two serialized page display lists ignoring the layout version
    /// stamp, so a relayout that reproduced a page can be detected as a no-op.
    pub fn page_bytes_match_content(a: &[u8], b: &[u8]) -> bool {
        const VERSION_START: usize = std::mem::size_of::<u32>();
        const VERSION_END: usize = VERSION_START + std::mem::size_of::<u64>();
        a.len() == b.len()
            && a.len() >= VERSION_END
            && a[..VERSION_START] == b[..VERSION_START]
            && a[VERSION_END..] == b[VERSION_END..]
    }

    /// Serialize atlas pixels as a standalone resource for FFI transfer.
    pub fn atlas_to_bytes(atlas: &GlyphAtlas, generation: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ATLAS_RESOURCE_VERSION.to_le_bytes());
        bytes.extend_from_slice(&generation.to_le_bytes());
        bytes.extend_from_slice(&atlas.width.to_le_bytes());
        bytes.extend_from_slice(&atlas.height.to_le_bytes());
        let pixels = atlas.pixels_rgba();
        bytes.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
        bytes.extend_from_slice(pixels);
        bytes
    }

    pub fn atlas_from_bytes(bytes: &[u8]) -> Option<AtlasResource> {
        if bytes.len() < 24 {
            return None;
        }
        let mut offset = 0usize;
        let file_version = read_u32(bytes, &mut offset)?;
        if file_version != ATLAS_RESOURCE_VERSION {
            return None;
        }
        let generation = read_u64(bytes, &mut offset)?;
        let width = read_u32(bytes, &mut offset)?;
        let height = read_u32(bytes, &mut offset)?;
        let pixel_len = read_u32(bytes, &mut offset)? as usize;
        if offset + pixel_len > bytes.len() {
            return None;
        }
        let pixels = bytes[offset..offset + pixel_len].to_vec();
        Some(AtlasResource {
            generation,
            width,
            height,
            pixels,
        })
    }

    pub fn to_bytes(list: &DisplayList) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Legacy v3 wire format with embedded atlas pixels (PDF export, tests).
        const LEGACY_V3: u32 = 3;
        bytes.extend_from_slice(&LEGACY_V3.to_le_bytes());
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
        write_shape_selection_batch(&mut bytes, &list.shape_selection_batch);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<DisplayList> {
        if bytes.len() < 20 {
            return None;
        }
        let mut offset = 0usize;

        let file_version = read_u32(bytes, &mut offset)?;
        let version = read_u64(bytes, &mut offset)?;
        let page_width = read_f32(bytes, &mut offset)?;
        let page_height = read_f32(bytes, &mut offset)?;

        let (atlas_width, atlas_height, atlas_pixels) = if file_version >= 4 {
            (0, 0, Vec::new())
        } else {
            let atlas_width = read_u32(bytes, &mut offset)?;
            let atlas_height = read_u32(bytes, &mut offset)?;
            let atlas_len = read_u32(bytes, &mut offset)? as usize;
            if offset + atlas_len > bytes.len() {
                return None;
            }
            let atlas_pixels = bytes[offset..offset + atlas_len].to_vec();
            offset += atlas_len;
            (atlas_width, atlas_height, atlas_pixels)
        };

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

        let shape_selection_batch = if file_version >= 7 {
            read_shape_selection_batch(bytes, &mut offset).unwrap_or_default()
        } else {
            ShapeSelectionBatch::default()
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
            shape_selection_batch,
        })
    }
}

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

fn append_table_layout(
    table: &TableLayout,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
    atlas_batch: &mut AtlasBatch,
) {
    for cell in &table.cells {
        if let Some(bg) = cell.background {
            append_rect(cell.x, cell.y, cell.width, cell.height, bg, rect_batch);
        }
        for line in &cell.lines {
            append_line_decorations(line, rect_batch);
            append_line_glyphs(line, atlas_batch);
        }
        for nested in &cell.nested_tables {
            append_table_layout(nested, rect_batch, path_batch, atlas_batch);
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

fn append_path_line(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: u32,
    batch: &mut PathBatch,
) {
    batch.points.extend_from_slice(&[x1, y1, x2, y2]);
    batch.colors.push(color);
}

fn append_shape_placeholder(shape: &ShapeLayout, rect_batch: &mut RectBatch) {
    append_rect(
        shape.x,
        shape.y,
        shape.width,
        shape.height,
        SHAPE_PLACEHOLDER_COLOR,
        rect_batch,
    );
    let border = 0xFF8099B3;
    append_rect(shape.x, shape.y, shape.width, 1.0, border, rect_batch);
    append_rect(
        shape.x,
        shape.y + shape.height - 1.0,
        shape.width,
        1.0,
        border,
        rect_batch,
    );
    append_rect(shape.x, shape.y, 1.0, shape.height, border, rect_batch);
    append_rect(
        shape.x + shape.width - 1.0,
        shape.y,
        1.0,
        shape.height,
        border,
        rect_batch,
    );
}

fn append_diagram_preview(
    shape: &ShapeLayout,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    match shape.diagram_kind {
        tw_model::DiagramKind::Hierarchy => {
            append_diagram_hierarchy_preview(shape, rect_batch, path_batch)
        }
        tw_model::DiagramKind::Cycle => {
            append_diagram_cycle_preview(shape, rect_batch, path_batch)
        }
        tw_model::DiagramKind::Process => {
            append_diagram_process_preview(shape, rect_batch, path_batch)
        }
    }
}

/// Word-like process SmartArt: three nodes with connecting arrows.
fn append_diagram_process_preview(
    shape: &ShapeLayout,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    const NODE_FILL: u32 = 0xFF5B9BD5;
    const ARROW: u32 = 0xFF2F5496;
    let pad_x = shape.width * 0.08;
    let pad_top = shape.height * 0.28;
    let pad_bottom = shape.height * 0.18;
    let body_h = (shape.height - pad_top - pad_bottom).max(24.0);
    let gap = shape.width * 0.06;
    let node_w = ((shape.width - pad_x * 2.0 - gap * 2.0) / 3.0).max(28.0);
    let node_h = body_h.min(shape.height * 0.42).max(20.0);
    let y = shape.y + pad_top + (body_h - node_h) * 0.5;

    for i in 0..3 {
        let x = shape.x + pad_x + i as f32 * (node_w + gap);
        append_rect(x, y, node_w, node_h, NODE_FILL, rect_batch);
        if i < 2 {
            let ax1 = x + node_w + 4.0;
            let ax2 = x + node_w + gap - 4.0;
            let ay = y + node_h * 0.5;
            append_path_line(ax1, ay, ax2, ay, ARROW, path_batch);
            append_path_line(ax2 - 6.0, ay - 5.0, ax2, ay, ARROW, path_batch);
            append_path_line(ax2 - 6.0, ay + 5.0, ax2, ay, ARROW, path_batch);
        }
    }
}

fn append_diagram_hierarchy_preview(
    shape: &ShapeLayout,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    const NODE_FILL: u32 = 0xFF5B9BD5;
    const LINE: u32 = 0xFF2F5496;
    let top_w = shape.width * 0.28;
    let top_h = shape.height * 0.16;
    let top_x = shape.x + (shape.width - top_w) * 0.5;
    let top_y = shape.y + shape.height * 0.28;
    append_rect(top_x, top_y, top_w, top_h, NODE_FILL, rect_batch);

    let mid_y = top_y + top_h + 12.0;
    append_path_line(
        top_x + top_w * 0.5,
        top_y + top_h,
        top_x + top_w * 0.5,
        mid_y,
        LINE,
        path_batch,
    );

    let child_w = shape.width * 0.22;
    let child_h = shape.height * 0.16;
    let gap = shape.width * 0.06;
    let row_w = child_w * 3.0 + gap * 2.0;
    let row_x = shape.x + (shape.width - row_w) * 0.5;
    append_path_line(row_x + child_w * 0.5, mid_y, row_x + row_w - child_w * 0.5, mid_y, LINE, path_batch);

    for i in 0..3 {
        let x = row_x + i as f32 * (child_w + gap);
        append_path_line(x + child_w * 0.5, mid_y, x + child_w * 0.5, mid_y + 8.0, LINE, path_batch);
        append_rect(x, mid_y + 8.0, child_w, child_h, NODE_FILL, rect_batch);
    }
}

fn append_diagram_cycle_preview(
    shape: &ShapeLayout,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    const NODE_FILL: u32 = 0xFF5B9BD5;
    const RING: u32 = 0xFF2F5496;
    let cx = shape.x + shape.width * 0.5;
    let cy = shape.y + shape.height * 0.55;
    let radius = (shape.width.min(shape.height) * 0.22).max(28.0);
    let segments = 36;
    let mut prev = (cx + radius, cy);
    for i in 1..=segments {
        let t = std::f32::consts::TAU * i as f32 / segments as f32;
        let next = (cx + radius * t.cos(), cy + radius * t.sin());
        append_path_line(prev.0, prev.1, next.0, next.1, RING, path_batch);
        prev = next;
    }
    for i in 0..3 {
        let t = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * i as f32 / 3.0;
        let nx = cx + radius * t.cos() - 18.0;
        let ny = cy + radius * t.sin() - 12.0;
        append_rect(nx, ny, 36.0, 24.0, NODE_FILL, rect_batch);
    }
}

const CHART_SERIES_COLORS: [u32; 4] = [0xFF4472C4, 0xFFED7D31, 0xFFA5A5A5, 0xFFFFC000];
const CHART_AXIS: u32 = 0xFF595959;
const CHART_GRID: u32 = 0xFFD9D9D9;
const CHART_BORDER: u32 = 0xFFB0B0B0;
const CHART_SURFACE: u32 = 0xFFFFFFFF;

fn append_chart_surface(shape: &ShapeLayout, rect_batch: &mut RectBatch) {
    append_rect(
        shape.x,
        shape.y,
        shape.width,
        shape.height,
        CHART_SURFACE,
        rect_batch,
    );
    append_rect(shape.x, shape.y, shape.width, 1.0, CHART_BORDER, rect_batch);
    append_rect(
        shape.x,
        shape.y + shape.height - 1.0,
        shape.width,
        1.0,
        CHART_BORDER,
        rect_batch,
    );
    append_rect(shape.x, shape.y, 1.0, shape.height, CHART_BORDER, rect_batch);
    append_rect(
        shape.x + shape.width - 1.0,
        shape.y,
        1.0,
        shape.height,
        CHART_BORDER,
        rect_batch,
    );
}

/// Word-like chart preview from sample / edited data (column, bar, line, pie).
fn append_chart_preview(
    shape: &ShapeLayout,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    let Some(data) = shape.chart_data.as_ref() else {
        return;
    };
    if data.series.is_empty() || data.categories.is_empty() {
        return;
    }

    // Legend swatches (Series 1 / Series 2) along the top-right.
    let legend_y = shape.y + shape.height * 0.08;
    let mut legend_x = shape.x + shape.width - 18.0;
    for (idx, _) in data.series.iter().enumerate().take(4).rev() {
        let color = CHART_SERIES_COLORS[idx % CHART_SERIES_COLORS.len()];
        append_rect(legend_x - 28.0, legend_y, 12.0, 8.0, color, rect_batch);
        legend_x -= 40.0;
    }

    match data.kind {
        tw_model::ChartKind::Pie => append_chart_pie(shape, data, rect_batch, path_batch),
        tw_model::ChartKind::Bar => append_chart_bar(shape, data, rect_batch, path_batch),
        tw_model::ChartKind::Line => append_chart_line(shape, data, rect_batch, path_batch),
        tw_model::ChartKind::Column => append_chart_column(shape, data, rect_batch, path_batch),
    }
}

fn chart_plot_rect(shape: &ShapeLayout) -> (f32, f32, f32, f32) {
    let pad_l = shape.width * 0.12;
    let pad_r = shape.width * 0.08;
    let pad_t = shape.height * 0.22;
    let pad_b = shape.height * 0.14;
    (
        shape.x + pad_l,
        shape.y + pad_t,
        (shape.width - pad_l - pad_r).max(20.0),
        (shape.height - pad_t - pad_b).max(20.0),
    )
}

fn chart_max_value(data: &tw_model::ChartData) -> f64 {
    data.series
        .iter()
        .flat_map(|s| s.values.iter().copied())
        .fold(0.0_f64, f64::max)
        .max(1.0)
}

fn append_chart_axes_and_grid(
    plot_x: f32,
    plot_y: f32,
    plot_w: f32,
    plot_h: f32,
    path_batch: &mut PathBatch,
) {
    for i in 1..4 {
        let y = plot_y + plot_h * i as f32 / 4.0;
        append_path_line(plot_x, y, plot_x + plot_w, y, CHART_GRID, path_batch);
    }
    append_path_line(plot_x, plot_y, plot_x, plot_y + plot_h, CHART_AXIS, path_batch);
    append_path_line(
        plot_x,
        plot_y + plot_h,
        plot_x + plot_w,
        plot_y + plot_h,
        CHART_AXIS,
        path_batch,
    );
}

fn append_chart_column(
    shape: &ShapeLayout,
    data: &tw_model::ChartData,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    let (plot_x, plot_y, plot_w, plot_h) = chart_plot_rect(shape);
    append_chart_axes_and_grid(plot_x, plot_y, plot_w, plot_h, path_batch);
    let max_v = chart_max_value(data);
    let n_cats = data.categories.len() as f32;
    let n_series = data.series.len().max(1) as f32;
    let slot = plot_w / n_cats;
    let cluster = (slot * 0.7).max(8.0);
    let bar_w = (cluster / n_series).max(3.0) - 1.0;

    for (ci, _) in data.categories.iter().enumerate() {
        for (si, series) in data.series.iter().enumerate() {
            let Some(&value) = series.values.get(ci) else {
                continue;
            };
            let h = ((value / max_v) as f32 * (plot_h - 4.0)).max(2.0);
            let cluster_x = plot_x + slot * ci as f32 + (slot - cluster) * 0.5;
            let x = cluster_x + si as f32 * (bar_w + 1.0);
            let y = plot_y + plot_h - h;
            let color = CHART_SERIES_COLORS[si % CHART_SERIES_COLORS.len()];
            append_rect(x, y, bar_w, h, color, rect_batch);
        }
    }
}

fn append_chart_bar(
    shape: &ShapeLayout,
    data: &tw_model::ChartData,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    let (plot_x, plot_y, plot_w, plot_h) = chart_plot_rect(shape);
    append_chart_axes_and_grid(plot_x, plot_y, plot_w, plot_h, path_batch);
    let max_v = chart_max_value(data);
    let n_cats = data.categories.len() as f32;
    let n_series = data.series.len().max(1) as f32;
    let slot = plot_h / n_cats;
    let cluster = (slot * 0.7).max(8.0);
    let bar_h = (cluster / n_series).max(3.0) - 1.0;

    for (ci, _) in data.categories.iter().enumerate() {
        for (si, series) in data.series.iter().enumerate() {
            let Some(&value) = series.values.get(ci) else {
                continue;
            };
            let w = ((value / max_v) as f32 * (plot_w - 4.0)).max(2.0);
            let cluster_y = plot_y + slot * ci as f32 + (slot - cluster) * 0.5;
            let y = cluster_y + si as f32 * (bar_h + 1.0);
            let color = CHART_SERIES_COLORS[si % CHART_SERIES_COLORS.len()];
            append_rect(plot_x + 1.0, y, w, bar_h, color, rect_batch);
        }
    }
}

fn append_chart_line(
    shape: &ShapeLayout,
    data: &tw_model::ChartData,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    let (plot_x, plot_y, plot_w, plot_h) = chart_plot_rect(shape);
    append_chart_axes_and_grid(plot_x, plot_y, plot_w, plot_h, path_batch);
    let max_v = chart_max_value(data);
    let n = (data.categories.len().max(1) - 1) as f32;

    for (si, series) in data.series.iter().enumerate() {
        let color = CHART_SERIES_COLORS[si % CHART_SERIES_COLORS.len()];
        let mut prev: Option<(f32, f32)> = None;
        for (ci, &value) in series.values.iter().enumerate() {
            let x = if n <= 0.0 {
                plot_x + plot_w * 0.5
            } else {
                plot_x + plot_w * (ci as f32 / n)
            };
            let y = plot_y + plot_h - ((value / max_v) as f32 * (plot_h - 4.0)).max(2.0);
            if let Some((px, py)) = prev {
                append_path_line(px, py, x, y, color, path_batch);
            }
            append_rect(x - 2.5, y - 2.5, 5.0, 5.0, color, rect_batch);
            prev = Some((x, y));
        }
    }
}

fn append_chart_pie(
    shape: &ShapeLayout,
    data: &tw_model::ChartData,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    // Use first series values as slice sizes (Word pie of categories).
    let Some(series) = data.series.first() else {
        return;
    };
    let total: f64 = series.values.iter().sum::<f64>().max(1.0);
    let cx = shape.x + shape.width * 0.42;
    let cy = shape.y + shape.height * 0.55;
    let radius = (shape.width.min(shape.height) * 0.28).max(24.0);

    let mut angle = -std::f32::consts::FRAC_PI_2;
    for (i, &value) in series.values.iter().enumerate() {
        let sweep = (value / total) as f32 * std::f32::consts::TAU;
        let color = CHART_SERIES_COLORS[i % CHART_SERIES_COLORS.len()];
        // Dense radial strokes approximate a filled wedge with the path batch.
        let steps = ((sweep / std::f32::consts::TAU) * 48.0).ceil().max(4.0) as i32;
        for s in 0..=steps {
            let t = angle + sweep * s as f32 / steps as f32;
            let x2 = cx + radius * t.cos();
            let y2 = cy + radius * t.sin();
            append_path_line(cx, cy, x2, y2, color, path_batch);
        }
        // Slice boundary.
        let x2 = cx + radius * (angle + sweep).cos();
        let y2 = cy + radius * (angle + sweep).sin();
        append_path_line(cx, cy, x2, y2, CHART_AXIS, path_batch);
        angle += sweep;
    }

    // Outer ring.
    let segments = 48;
    let mut prev = (cx + radius, cy);
    for i in 1..=segments {
        let t = std::f32::consts::TAU * i as f32 / segments as f32 - std::f32::consts::FRAC_PI_2;
        let next = (cx + radius * t.cos(), cy + radius * t.sin());
        append_path_line(prev.0, prev.1, next.0, next.1, CHART_AXIS, path_batch);
        prev = next;
    }

    // Category color key on the right.
    let mut key_y = shape.y + shape.height * 0.30;
    for (i, _) in data.categories.iter().enumerate().take(4) {
        let color = CHART_SERIES_COLORS[i % CHART_SERIES_COLORS.len()];
        append_rect(
            shape.x + shape.width * 0.72,
            key_y,
            10.0,
            8.0,
            color,
            rect_batch,
        );
        key_y += 16.0;
    }
}

fn append_shape_geometry(
    shape: &ShapeLayout,
    rect_batch: &mut RectBatch,
    path_batch: &mut PathBatch,
) {
    use tw_model::ShapeKind;

    let stroke = shape.stroke.unwrap_or(0xFF000000);
    match shape.shape_type {
        ShapeKind::Rectangle => {
            if let Some(fill) = shape.fill {
                append_rect(shape.x, shape.y, shape.width, shape.height, fill, rect_batch);
            }
            append_path_line(shape.x, shape.y, shape.x + shape.width, shape.y, stroke, path_batch);
            append_path_line(
                shape.x + shape.width,
                shape.y,
                shape.x + shape.width,
                shape.y + shape.height,
                stroke,
                path_batch,
            );
            append_path_line(
                shape.x + shape.width,
                shape.y + shape.height,
                shape.x,
                shape.y + shape.height,
                stroke,
                path_batch,
            );
            append_path_line(shape.x, shape.y + shape.height, shape.x, shape.y, stroke, path_batch);
        }
        ShapeKind::Line => {
            append_path_line(
                shape.x,
                shape.y,
                shape.x + shape.width,
                shape.y + shape.height,
                stroke,
                path_batch,
            );
        }
        ShapeKind::Ellipse => {
            if let Some(fill) = shape.fill {
                append_rect(shape.x, shape.y, shape.width, shape.height, fill, rect_batch);
            }
            let cx = shape.x + shape.width / 2.0;
            let cy = shape.y + shape.height / 2.0;
            let rx = shape.width / 2.0;
            let ry = shape.height / 2.0;
            let segments = 36;
            let mut prev = (
                cx + rx,
                cy,
            );
            for i in 1..=segments {
                let t = std::f32::consts::TAU * i as f32 / segments as f32;
                let next = (cx + rx * t.cos(), cy + ry * t.sin());
                append_path_line(prev.0, prev.1, next.0, next.1, stroke, path_batch);
                prev = next;
            }
        }
        _ => append_shape_placeholder(shape, rect_batch),
    }
}

fn append_word_art_path(shape: &ShapeLayout, path_batch: &mut PathBatch) {
    let color = 0xFF1F4E79;
    let cx = shape.x + shape.width / 2.0;
    let base_y = shape.y + shape.height * 0.72;
    let rx = shape.width * 0.42;
    let segments = 24;
    let mut prev = (cx - rx, base_y);
    for i in 1..=segments {
        let t = std::f32::consts::PI * i as f32 / segments as f32;
        let next = (cx - rx * t.cos(), base_y - rx * 0.35 * t.sin());
        append_path_line(prev.0, prev.1, next.0, next.1, color, path_batch);
        prev = next;
    }
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
    for id in &batch.image_ids {
        let bytes_id = id.as_bytes();
        bytes.extend_from_slice(&(bytes_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(bytes_id);
    }
    for payload in &batch.payloads {
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(payload);
    }
    if !batch.rotations.is_empty() {
        for val in &batch.rotations {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        for val in &batch.opacities {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        for val in &batch.crop_rects {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
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
    let mut image_ids = Vec::new();
    if file_version >= 5 {
        image_ids = Vec::with_capacity(image_count);
        for _ in 0..image_count {
            let len = read_u32(bytes, offset)? as usize;
            let end = *offset + len;
            let id = std::str::from_utf8(bytes.get(*offset..end)?).ok()?.to_string();
            *offset = end;
            image_ids.push(id);
        }
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
    let mut rotations = Vec::new();
    let mut opacities = Vec::new();
    let mut crop_rects = Vec::new();
    if file_version >= 6 && *offset + image_count * 4 * 6 <= bytes.len() {
        rotations = Vec::with_capacity(image_count);
        for _ in 0..image_count {
            rotations.push(read_f32(bytes, offset)?);
        }
        opacities = Vec::with_capacity(image_count);
        for _ in 0..image_count {
            opacities.push(read_f32(bytes, offset)?);
        }
        crop_rects = Vec::with_capacity(image_count * 4);
        for _ in 0..image_count * 4 {
            crop_rects.push(read_f32(bytes, offset)?);
        }
    }
    Some(ImageBatch {
        transforms,
        sizes,
        asset_ids,
        image_ids,
        payloads,
        rotations,
        opacities,
        crop_rects,
    })
}

fn write_shape_selection_batch(bytes: &mut Vec<u8>, batch: &ShapeSelectionBatch) {
    let shape_count = batch.shape_ids.len() as u32;
    bytes.extend_from_slice(&shape_count.to_le_bytes());
    for val in &batch.rects {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    for id in &batch.shape_ids {
        let bytes_id = id.as_bytes();
        bytes.extend_from_slice(&(bytes_id.len() as u32).to_le_bytes());
        bytes.extend_from_slice(bytes_id);
    }
}

fn read_shape_selection_batch(bytes: &[u8], offset: &mut usize) -> Option<ShapeSelectionBatch> {
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
    let shape_count = read_u32(bytes, offset)? as usize;
    let mut rects = Vec::with_capacity(shape_count * 4);
    for _ in 0..shape_count * 4 {
        rects.push(read_f32(bytes, offset)?);
    }
    let mut shape_ids = Vec::with_capacity(shape_count);
    for _ in 0..shape_count {
        let len = read_u32(bytes, offset)? as usize;
        let end = *offset + len;
        let id = std::str::from_utf8(bytes.get(*offset..end)?).ok()?.to_string();
        *offset = end;
        shape_ids.push(id);
    }
    Some(ShapeSelectionBatch { shape_ids, rects })
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

    #[test]
    fn v4_page_bytes_omit_atlas_pixels() {
        let doc = Document::with_paragraph("Hello World");
        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(&doc);
        let page = layout.pages.first().unwrap();
        let list = DisplayListBuilder::from_page_without_atlas(page, 2);
        let bytes = DisplayListBuilder::to_page_bytes(&list);
        assert_eq!(u32::from_le_bytes(bytes[0..4].try_into().unwrap()), 7);
        let decoded = DisplayListBuilder::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.version, 2);
        assert!(decoded.atlas_pixels.is_empty());
        assert_eq!(decoded.atlas_width, 0);
        assert_eq!(
            decoded.atlas_batch.transforms.len(),
            list.atlas_batch.transforms.len()
        );
    }

    #[test]
    fn atlas_resource_round_trip() {
        let doc = Document::with_paragraph("Hello");
        let mut engine = LayoutEngine::new();
        engine.layout_document(&doc);
        let atlas = engine.atlas();
        let generation = atlas.generation;
        let bytes = DisplayListBuilder::atlas_to_bytes(atlas, generation);
        let decoded = DisplayListBuilder::atlas_from_bytes(&bytes).unwrap();
        assert_eq!(decoded.generation, generation);
        assert_eq!(decoded.width, atlas.width);
        assert_eq!(decoded.height, atlas.height);
        assert_eq!(decoded.pixels.len(), atlas.pixels_rgba().len());
    }
}

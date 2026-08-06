use tw_model::ImageData;

/// Converts vector or metafile images to raster bytes the renderer can decode.
pub fn normalize_image(mut data: ImageData) -> ImageData {
    if is_svg(&data) {
        if let Some((width, height, png)) = rasterize_svg(&data.bytes) {
            data.mime_type = "image/png".into();
            data.width_px = width;
            data.height_px = height;
            data.bytes = png;
        }
    }
    data
}

fn is_svg(data: &ImageData) -> bool {
    if data.mime_type.contains("svg") {
        return true;
    }
    let head = data.bytes.get(..128.min(data.bytes.len())).unwrap_or(&data.bytes);
    let text = String::from_utf8_lossy(head);
    text.contains("<svg") || text.contains(":svg")
}

fn rasterize_svg(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(bytes, &opt).ok()?;
    let size = tree.size();
    let width = size.width().ceil().max(1.0) as u32;
    let height = size.height().ceil().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());
    let png = pixmap.encode_png().ok()?;
    Some((width, height, png))
}

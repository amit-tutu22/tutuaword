//! SVG rasterization for the rendering pipeline (R3.3 — moved out of tw-docx).

use tw_model::{Block, Document, ImageData, RunContent};

/// Converts vector images (SVG) to raster bytes the layout/renderer can decode.
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

/// Walk the document and rasterize any SVG image payloads in place.
pub fn normalize_document_images(doc: &mut Document) {
    for section in &mut doc.sections {
        normalize_blocks(&mut section.blocks);
    }
}

fn normalize_blocks(blocks: &mut [Block]) {
    for block in blocks {
        match block {
            Block::Paragraph(para) => {
                for run in &mut para.runs {
                    if let RunContent::InlineImage(img) = &mut run.content {
                        img.image = normalize_image(img.image.clone());
                    }
                }
            }
            Block::Table(table) => {
                for row in &mut table.rows {
                    for cell in &mut row.cells {
                        normalize_blocks(&mut cell.blocks);
                    }
                }
            }
            Block::ImageBlock(image) => {
                image.data = normalize_image(image.data.clone());
            }
            _ => {}
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

  const MINI_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="red"/></svg>"#;

    #[test]
    fn svg_bytes_rasterize_to_png() {
        let data = ImageData {
            asset_id: "svg".into(),
            mime_type: "image/svg+xml".into(),
            width_px: 0,
            height_px: 0,
            bytes: MINI_SVG.to_vec(),
        };
        let out = normalize_image(data);
        assert_eq!(out.mime_type, "image/png");
        assert!(out.width_px > 0);
        assert!(out.height_px > 0);
        assert!(out.bytes.len() > 8);
        assert_eq!(&out.bytes[0..4], &[0x89, 0x50, 0x4e, 0x47]);
    }
}

//! Software page rasterizer for Word-baseline PNG comparison (Layer 2 QA).

use tiny_skia::{Pixmap, Transform};

use tw_layout::{LayoutBox, LayoutEngine, PageLayout, PositionedGlyph, TableLayout, TextLine};
use tw_model::Document;
use tw_shape::GlyphAtlas;

/// Rasterize page 0 of [doc] to RGBA PNG bytes.
pub fn rasterize_document_page(doc: &Document, page_index: usize) -> Option<Vec<u8>> {
    let mut engine = LayoutEngine::new();
    let layout = engine.layout_document(doc);
    let page = layout.pages.get(page_index)?;
    Some(rasterize_page(page, engine.atlas()))
}

/// Rasterize a laid-out page using glyph atlas pixels already populated by layout.
///
/// The atlas must come from the same engine that produced [page]; its entries
/// are what the page's glyphs index into.
pub fn rasterize_page(page: &PageLayout, atlas: &GlyphAtlas) -> Vec<u8> {
    let width = page.width.max(1.0).ceil() as u32;
    let height = page.height.max(1.0).ceil() as u32;
    let mut pixmap = Pixmap::new(width, height).unwrap_or_else(|| Pixmap::new(1, 1).unwrap());
    pixmap.fill(tiny_skia::Color::WHITE);

    for layout_box in &page.boxes {
        match layout_box {
            LayoutBox::Rect {
                x,
                y,
                width,
                height,
                color,
            } => fill_rect(&mut pixmap, *x, *y, *width, *height, *color),
            LayoutBox::TextLine(line) => draw_text_line(&mut pixmap, atlas, line),
            LayoutBox::Image(img) => draw_image(&mut pixmap, img),
            LayoutBox::Shape(shape) => {
                if let Some(fill) = shape.fill {
                    fill_rect(&mut pixmap, shape.x, shape.y, shape.width, shape.height, fill);
                }
                if let Some(stroke) = shape.stroke {
                    stroke_rect(
                        &mut pixmap,
                        shape.x,
                        shape.y,
                        shape.width,
                        shape.height,
                        shape.stroke_width.max(1.0),
                        stroke,
                    );
                }
            }
            LayoutBox::Table(table) => draw_table(&mut pixmap, atlas, table),
        }
    }

    pixmap.encode_png().unwrap_or_default()
}

fn draw_text_line(pixmap: &mut Pixmap, atlas: &GlyphAtlas, line: &TextLine) {
    for glyph in &line.glyphs {
        blit_glyph(pixmap, atlas, glyph);
    }
    for deco in &line.decorations {
        fill_rect(pixmap, deco.x, deco.y, deco.width, deco.height, deco.color);
    }
}

fn draw_table(pixmap: &mut Pixmap, atlas: &GlyphAtlas, table: &TableLayout) {
    for cell in &table.cells {
        if let Some(bg) = cell.background {
            fill_rect(pixmap, cell.x, cell.y, cell.width, cell.height, bg);
        }
        for line in &cell.lines {
            draw_text_line(pixmap, atlas, line);
        }
        for nested in &cell.nested_tables {
            draw_table(pixmap, atlas, nested);
        }
    }
    for (index, chunk) in table.grid_lines.chunks(4).enumerate() {
        let [x1, y1, x2, y2] = chunk else {
            continue;
        };
        let color = table
            .grid_line_colors
            .get(index)
            .copied()
            .unwrap_or(0xFF00_0000);
        // Grid lines are axis-aligned segments; give them a hairline thickness.
        fill_rect(
            pixmap,
            x1.min(*x2),
            y1.min(*y2),
            (x2 - x1).abs().max(1.0),
            (y2 - y1).abs().max(1.0),
            color,
        );
    }
}

fn draw_image(pixmap: &mut Pixmap, img: &tw_layout::ImageLayout) {
    if let Ok(decoded) = Pixmap::decode_png(img.encoded.as_slice()) {
        let sx = img.width / decoded.width().max(1) as f32;
        let sy = img.height / decoded.height().max(1) as f32;
        if !(sx.is_finite() && sy.is_finite()) || sx <= 0.0 || sy <= 0.0 {
            fill_rect(pixmap, img.x, img.y, img.width, img.height, 0xFFE0E0E0);
            return;
        }
        let paint = tiny_skia::PixmapPaint {
            opacity: img.opacity.clamp(0.0, 1.0),
            ..Default::default()
        };
        pixmap.draw_pixmap(
            0,
            0,
            decoded.as_ref(),
            &paint,
            Transform::from_row(sx, 0.0, 0.0, sy, img.x, img.y),
            None,
        );
        return;
    }
    // JPEG and other encodings have no decoder here; a neutral box keeps the
    // page geometry honest instead of leaving a hole.
    fill_rect(pixmap, img.x, img.y, img.width, img.height, 0xFFE0E0E0);
}

/// Blit one glyph's atlas coverage, modulated by the glyph color, over the page.
///
/// Atlas mask pixels are premultiplied white (RGB tracks alpha), matching the
/// Modulate blend the GPU renderer uses; color glyphs carry their own palette
/// and are tinted with opaque white by layout, so one path covers both.
fn blit_glyph(pixmap: &mut Pixmap, atlas: &GlyphAtlas, glyph: &PositionedGlyph) {
    let src_w = glyph.atlas_w.round() as i64;
    let src_h = glyph.atlas_h.round() as i64;
    if src_w <= 0 || src_h <= 0 {
        return;
    }
    let dst_w = pixmap.width() as i64;
    let dst_h = pixmap.height() as i64;
    let atlas_w = atlas.width as i64;
    let atlas_h = atlas.height as i64;
    let src_x = glyph.atlas_x.round() as i64;
    let src_y = glyph.atlas_y.round() as i64;
    let dst_x = glyph.x.round() as i64;
    // `PositionedGlyph::y` is already the bitmap top (baseline minus bearing).
    let dst_y = glyph.y.round() as i64;

    let tint_a = ((glyph.color >> 24) & 0xFF) as f32 / 255.0;
    let tint_r = ((glyph.color >> 16) & 0xFF) as f32 / 255.0;
    let tint_g = ((glyph.color >> 8) & 0xFF) as f32 / 255.0;
    let tint_b = (glyph.color & 0xFF) as f32 / 255.0;
    if tint_a <= 0.0 {
        return;
    }

    let src = atlas.pixels_rgba();
    let dst = pixmap.data_mut();
    for row in 0..src_h {
        let sy = src_y + row;
        let dy = dst_y + row;
        if sy < 0 || sy >= atlas_h || dy < 0 || dy >= dst_h {
            continue;
        }
        for col in 0..src_w {
            let sx = src_x + col;
            let dx = dst_x + col;
            if sx < 0 || sx >= atlas_w || dx < 0 || dx >= dst_w {
                continue;
            }
            let si = (((sy * atlas_w) + sx) * 4) as usize;
            let Some(px) = src.get(si..si + 4) else {
                continue;
            };
            let cov = px[3] as f32 / 255.0;
            if cov <= 0.0 {
                continue;
            }
            let alpha = cov * tint_a;
            let sr = (px[0] as f32 / 255.0) * tint_r * tint_a;
            let sg = (px[1] as f32 / 255.0) * tint_g * tint_a;
            let sb = (px[2] as f32 / 255.0) * tint_b * tint_a;

            let di = (((dy * dst_w) + dx) * 4) as usize;
            let Some(out) = dst.get_mut(di..di + 4) else {
                continue;
            };
            let inv = 1.0 - alpha;
            out[0] = ((sr * 255.0) + out[0] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            out[1] = ((sg * 255.0) + out[1] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            out[2] = ((sb * 255.0) + out[2] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            out[3] = ((alpha * 255.0) + out[3] as f32 * inv).round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// Source-over fill of an axis-aligned rect.
///
/// Written directly into the pixmap rather than through tiny-skia's rasterizer,
/// which takes an anti-aliased hairline path for sub-pixel rects (underlines,
/// table rules) and asserts on some page coordinates.
fn fill_rect(pixmap: &mut Pixmap, x: f32, y: f32, width: f32, height: f32, argb: u32) {
    if width <= 0.0 || height <= 0.0 || !(x.is_finite() && y.is_finite()) {
        return;
    }
    let alpha = ((argb >> 24) & 0xFF) as f32 / 255.0;
    if alpha <= 0.0 {
        return;
    }
    let src_r = ((argb >> 16) & 0xFF) as f32 * alpha;
    let src_g = ((argb >> 8) & 0xFF) as f32 * alpha;
    let src_b = (argb & 0xFF) as f32 * alpha;

    let dst_w = pixmap.width() as i64;
    let dst_h = pixmap.height() as i64;
    // Hairlines round to nothing otherwise, so keep every rect at least 1 px.
    let x0 = (x.round() as i64).clamp(0, dst_w);
    let y0 = (y.round() as i64).clamp(0, dst_h);
    let x1 = (((x + width).round() as i64).max(x0 + 1)).clamp(0, dst_w);
    let y1 = (((y + height).round() as i64).max(y0 + 1)).clamp(0, dst_h);

    let inv = 1.0 - alpha;
    let data = pixmap.data_mut();
    for py in y0..y1 {
        for px in x0..x1 {
            let i = (((py * dst_w) + px) * 4) as usize;
            let Some(out) = data.get_mut(i..i + 4) else {
                continue;
            };
            out[0] = (src_r + out[0] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            out[1] = (src_g + out[1] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            out[2] = (src_b + out[2] as f32 * inv).round().clamp(0.0, 255.0) as u8;
            out[3] = ((alpha * 255.0) + out[3] as f32 * inv)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
}

fn stroke_rect(
    pixmap: &mut Pixmap,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    thickness: f32,
    argb: u32,
) {
    fill_rect(pixmap, x, y, width, thickness, argb);
    fill_rect(pixmap, x, y + height - thickness, width, thickness, argb);
    fill_rect(pixmap, x, y, thickness, height, argb);
    fill_rect(pixmap, x + width - thickness, y, thickness, height, argb);
}

/// Fraction of pixels that differ between two RGBA PNG images (0..1).
pub fn pixel_diff_ratio(a: &[u8], b: &[u8]) -> f64 {
    let (Some(img_a), Some(img_b)) = (decode_png_rgba(a), decode_png_rgba(b)) else {
        return 1.0;
    };
    if img_a.0 != img_b.0 || img_a.1 != img_b.1 {
        return 1.0;
    }
    let (w, h, pa) = img_a;
    let pb = img_b.2;
    let mut diff = 0u64;
    let total = (w * h) as u64;
    for i in 0..total as usize {
        let ai = i * 4;
        if pa[ai..ai + 3] != pb[ai..ai + 3] {
            diff += 1;
        }
    }
    diff as f64 / total as f64
}

fn decode_png_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let pixmap = Pixmap::decode_png(bytes).ok()?;
    Some((pixmap.width(), pixmap.height(), pixmap.data().to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::Document;

    fn ink_ratio(png: &[u8]) -> f64 {
        let (w, h, data) = decode_png_rgba(png).expect("decode");
        let total = (w * h) as f64;
        let inked = data
            .chunks(4)
            .filter(|px| px[0] < 200 || px[1] < 200 || px[2] < 200)
            .count() as f64;
        inked / total
    }

    #[test]
    fn rasterize_simple_page_produces_png() {
        let doc = Document::with_paragraph("Hello baseline");
        let png = rasterize_document_page(&doc, 0).expect("page 0");
        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
    }

    #[test]
    fn identical_pngs_have_zero_diff() {
        let doc = Document::with_paragraph("Same");
        let a = rasterize_document_page(&doc, 0).unwrap();
        let b = rasterize_document_page(&doc, 0).unwrap();
        assert!(pixel_diff_ratio(&a, &b) < 0.001);
    }

    #[test]
    fn different_docs_have_nonzero_diff() {
        let a = rasterize_document_page(&Document::with_paragraph("Alpha"), 0).unwrap();
        let b = rasterize_document_page(&Document::with_paragraph("Beta"), 0).unwrap();
        assert!(pixel_diff_ratio(&a, &b) > 0.0001);
    }

    #[test]
    fn glyph_coverage_is_thinner_than_solid_boxes() {
        // Filled glyph boxes would ink the whole em square; real coverage is a
        // fraction of it, which is what a Word pixel diff needs.
        let doc = Document::with_paragraph("iiii iiii iiii");
        let png = rasterize_document_page(&doc, 0).unwrap();
        let ratio = ink_ratio(&png);
        assert!(ratio > 0.0, "expected painted glyphs, got {ratio}");
        assert!(
            ratio < 0.01,
            "glyph coverage should be sparse, not solid boxes (got {ratio})"
        );
    }

    #[test]
    fn blank_page_has_no_ink() {
        let doc = Document::with_paragraph("");
        let png = rasterize_document_page(&doc, 0).unwrap();
        assert_eq!(ink_ratio(&png), 0.0);
    }
}

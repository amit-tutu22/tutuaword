use std::collections::HashMap;

use swash::scale::image::{Content, Image};
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::{FontRef, GlyphId};

use crate::{FontDatabase, FontId};

const RENDER_SOURCES: [Source; 3] = [
    Source::ColorOutline(0),
    Source::ColorBitmap(StrikeWith::BestFit),
    Source::Outline,
];

/// A glyph bitmap in premultiplied RGBA, ready to be packed into the atlas.
///
/// Alpha-mask glyphs are stored as premultiplied white so the renderer can tint
/// them with the run color; `is_color` marks bitmaps that already carry their
/// own color (emoji) and must not be tinted.
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub is_color: bool,
}

impl RasterizedGlyph {
    pub fn empty() -> Self {
        Self {
            rgba: Vec::new(),
            width: 0,
            height: 0,
            bearing_x: 0.0,
            bearing_y: 0.0,
            is_color: false,
        }
    }
}

struct CachedFont {
    data: Vec<u8>,
    offset: u32,
}

pub struct GlyphRasterizer {
    scale_context: ScaleContext,
    font_cache: HashMap<u32, CachedFont>,
}

impl Default for GlyphRasterizer {
    fn default() -> Self {
        Self::new()
    }
}

impl GlyphRasterizer {
    pub fn new() -> Self {
        Self {
            scale_context: ScaleContext::new(),
            font_cache: HashMap::new(),
        }
    }

    pub fn rasterize_glyph(
        &mut self,
        fonts: &FontDatabase,
        font_id: FontId,
        glyph_id: u32,
        size: f32,
    ) -> RasterizedGlyph {
        let Some(face) = fonts.face(font_id) else {
            return RasterizedGlyph::empty();
        };
        let cache_key = font_id.key();

        if !self.font_cache.contains_key(&cache_key) {
            let Some(data) = fonts.load_face_data(font_id) else {
                return RasterizedGlyph::empty();
            };
            let Some(font) = FontRef::from_index(&data, face.index as usize) else {
                return RasterizedGlyph::empty();
            };
            let offset = font.offset;
            self.font_cache.insert(cache_key, CachedFont { data, offset });
        }

        let cached = self.font_cache.get(&cache_key).expect("font cache");
        let Some(font) = FontRef::from_offset(&cached.data, cached.offset) else {
            return RasterizedGlyph::empty();
        };

        let mut scaler = self
            .scale_context
            .builder(font)
            .size(size)
            .hint(false)
            .build();

        let Some(image) =
            Render::new(&RENDER_SOURCES).render(&mut scaler, glyph_id as GlyphId)
        else {
            return RasterizedGlyph::empty();
        };

        let width = image.placement.width;
        let height = image.placement.height;
        let is_color = matches!(image.content, Content::Color);
        if width == 0 || height == 0 {
            return RasterizedGlyph {
                rgba: Vec::new(),
                width: 0,
                height: 0,
                bearing_x: image.placement.left as f32,
                bearing_y: image.placement.top as f32,
                is_color,
            };
        }

        RasterizedGlyph {
            rgba: swash_image_to_rgba(&image),
            width,
            height,
            bearing_x: image.placement.left as f32,
            bearing_y: image.placement.top as f32,
            is_color,
        }
    }
}

fn swash_image_to_rgba(image: &Image) -> Vec<u8> {
    let pixel_count = (image.placement.width * image.placement.height) as usize;
    let mut rgba = vec![0u8; pixel_count * 4];

    match image.content {
        Content::Mask => {
            // Premultiplied white: RGB tracks alpha so a Modulate blend against
            // the run color yields correctly premultiplied tinted glyphs.
            for (i, alpha) in image.data.iter().enumerate().take(pixel_count) {
                let base = i * 4;
                rgba[base] = *alpha;
                rgba[base + 1] = *alpha;
                rgba[base + 2] = *alpha;
                rgba[base + 3] = *alpha;
            }
        }
        Content::SubpixelMask | Content::Color => {
            let len = rgba.len().min(image.data.len());
            rgba[..len].copy_from_slice(&image.data[..len]);
        }
    }

    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph_for(ch: char, size: f32) -> RasterizedGlyph {
        let mut fonts = FontDatabase::new();
        let mut rasterizer = GlyphRasterizer::new();
        let font_id = fonts.resolve(None).expect("system font");
        let data = fonts.load_face_data(font_id).expect("font data");
        let face = swash::FontRef::from_index(&data, 0).expect("font ref");
        let glyph_id = face.charmap().map(ch);
        rasterizer.rasterize_glyph(&fonts, font_id, glyph_id as u32, size)
    }

    #[test]
    fn rasterizes_glyph_with_ink_and_metrics() {
        let glyph = glyph_for('W', 24.0);

        assert!(glyph.width > 0 && glyph.height > 0, "expected a bitmap");
        assert_eq!(glyph.rgba.len(), (glyph.width * glyph.height * 4) as usize);
        assert!(
            glyph.rgba.chunks(4).any(|px| px[3] > 0),
            "expected non-empty glyph coverage"
        );
        assert!(
            glyph.bearing_y > 0.0,
            "uppercase glyph should sit above the baseline, got {}",
            glyph.bearing_y
        );
    }

    #[test]
    fn mask_glyphs_are_premultiplied_white() {
        let glyph = glyph_for('W', 24.0);
        assert!(!glyph.is_color);
        for px in glyph.rgba.chunks(4) {
            assert_eq!(
                [px[0], px[1], px[2]],
                [px[3], px[3], px[3]],
                "mask pixels must be premultiplied white"
            );
        }
    }

    #[test]
    fn larger_size_yields_larger_bitmap() {
        let small = glyph_for('W', 12.0);
        let large = glyph_for('W', 48.0);
        assert!(large.width > small.width && large.height > small.height);
    }
}

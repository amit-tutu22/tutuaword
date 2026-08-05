use crate::{FontDatabase, FontId, GlyphRasterizer, RasterizedGlyph};
use rustybuzz::{Direction, UnicodeBuffer};
use tw_model::CharFormat;

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub glyph_id: u32,
    pub cluster: u32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub font_key: u32,
}

#[derive(Debug, Clone)]
pub struct ShapedRun {
    pub glyphs: Vec<GlyphInfo>,
    pub cluster_to_glyph: Vec<usize>,
}

pub struct TextShaper {
    fonts: FontDatabase,
    rasterizer: GlyphRasterizer,
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}

impl TextShaper {
    pub fn new() -> Self {
        Self {
            fonts: FontDatabase::new(),
            rasterizer: GlyphRasterizer::new(),
        }
    }

    pub fn fonts_mut(&mut self) -> &mut FontDatabase {
        &mut self.fonts
    }

    pub fn shape(&self, text: &str, format: &CharFormat, font_id: FontId) -> ShapedRun {
        let size = format.font_size.unwrap_or(12.0);
        let face_info = self.fonts.face(font_id);
        let Some(face_info) = face_info else {
            return ShapedRun {
                glyphs: Vec::new(),
                cluster_to_glyph: vec![0; text.chars().count()],
            };
        };

        let font_data = self.fonts.load_face_data(font_id).unwrap_or_default();
        let face = match rustybuzz::Face::from_slice(&font_data, face_info.index) {
            Some(f) => f,
            None => {
                return ShapedRun {
                    glyphs: Vec::new(),
                    cluster_to_glyph: vec![0; text.chars().count()],
                };
            }
        };

        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);

        let output = rustybuzz::shape(&face, &[], buffer);
        let scale = size / face.units_per_em() as f32;
        let font_key = font_id.key();

        let glyphs: Vec<GlyphInfo> = output
            .glyph_infos()
            .iter()
            .zip(output.glyph_positions())
            .map(|(info, pos)| GlyphInfo {
                glyph_id: info.glyph_id,
                cluster: info.cluster,
                x_offset: pos.x_offset as f32 * scale,
                y_offset: pos.y_offset as f32 * scale,
                x_advance: pos.x_advance as f32 * scale,
                font_key,
            })
            .collect();

        let char_count = text.chars().count();
        let mut cluster_to_glyph = vec![0; char_count.max(1)];
        for (gi, g) in glyphs.iter().enumerate() {
            let ci = g.cluster as usize;
            if ci < char_count {
                cluster_to_glyph[ci] = gi;
            }
        }

        ShapedRun {
            glyphs,
            cluster_to_glyph,
        }
    }

    pub fn default_font(&mut self) -> Option<FontId> {
        self.fonts.resolve(None)
    }

    pub fn rasterize_glyph(
        &mut self,
        font_id: FontId,
        glyph_id: u32,
        size: f32,
    ) -> RasterizedGlyph {
        self.rasterizer
            .rasterize_glyph(&self.fonts, font_id, glyph_id, size)
    }
}

use crate::{FontDatabase, FontId, GlyphRasterizer, RasterizedGlyph};
use rustybuzz::{Direction, UnicodeBuffer};
use tw_model::CharFormat;

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub glyph_id: u32,
    /// Index of the source character, not its byte offset.
    pub cluster: u32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    /// The face this glyph came from, which is not always the run's font: a
    /// character the run font lacks is shaped with a fallback.
    pub font: FontId,
}

impl GlyphInfo {
    pub fn font_key(&self) -> u32 {
        self.font.key()
    }
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

    pub fn shape(&mut self, text: &str, format: &CharFormat, font_id: FontId) -> ShapedRun {
        let size = format.font_size.unwrap_or(12.0);
        let char_count = text.chars().count();

        let mut glyphs = Vec::new();
        for (segment, segment_font, char_offset) in self.split_by_coverage(text, font_id) {
            self.shape_segment(&segment, size, segment_font, char_offset, &mut glyphs);
        }

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

    /// Splits text into runs of characters the same face can render, so a
    /// character missing from the run's font is shaped with a fallback rather
    /// than coming out as `.notdef`. Returns each piece with its face and the
    /// character index it starts at.
    fn split_by_coverage(&mut self, text: &str, font_id: FontId) -> Vec<(String, FontId, usize)> {
        let mut segments: Vec<(String, FontId, usize)> = Vec::new();

        for (char_index, ch) in text.chars().enumerate() {
            let font = if self.fonts.covers(font_id, ch) {
                font_id
            } else if ch.is_whitespace() {
                // Nothing to draw, so keep it with the current segment rather
                // than splitting the run around an invisible character.
                segments.last().map(|(_, f, _)| *f).unwrap_or(font_id)
            } else {
                self.fonts.fallback_for(ch).unwrap_or(font_id)
            };

            match segments.last_mut() {
                Some((buffer, existing, _)) if *existing == font => buffer.push(ch),
                _ => segments.push((ch.to_string(), font, char_index)),
            }
        }

        if segments.is_empty() {
            segments.push((String::new(), font_id, 0));
        }
        segments
    }

    fn shape_segment(
        &mut self,
        text: &str,
        size: f32,
        font_id: FontId,
        char_offset: usize,
        out: &mut Vec<GlyphInfo>,
    ) {
        let Some(face_index) = self.fonts.face(font_id).map(|f| f.index) else {
            return;
        };
        let Some(font_data) = self.fonts.face_data(font_id) else {
            return;
        };
        let Some(face) = rustybuzz::Face::from_slice(&font_data, face_index) else {
            return;
        };

        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);

        let output = rustybuzz::shape(&face, &[], buffer);
        let scale = size / face.units_per_em() as f32;

        // rustybuzz reports clusters as byte offsets; the rest of the engine
        // indexes by character.
        let char_of_byte: Vec<usize> = {
            let mut map = vec![0usize; text.len() + 1];
            for (char_index, (byte_index, _)) in text.char_indices().enumerate() {
                map[byte_index] = char_index;
            }
            map[text.len()] = text.chars().count();
            let mut last = 0;
            for slot in map.iter_mut() {
                if *slot == 0 {
                    *slot = last;
                } else {
                    last = *slot;
                }
            }
            map
        };

        out.extend(
            output
                .glyph_infos()
                .iter()
                .zip(output.glyph_positions())
                .map(|(info, pos)| GlyphInfo {
                    glyph_id: info.glyph_id,
                    cluster: (char_offset
                        + char_of_byte
                            .get(info.cluster as usize)
                            .copied()
                            .unwrap_or(0)) as u32,
                    x_offset: pos.x_offset as f32 * scale,
                    y_offset: pos.y_offset as f32 * scale,
                    x_advance: pos.x_advance as f32 * scale,
                    font: font_id,
                }),
        );
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

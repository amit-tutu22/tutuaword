use crate::{AtlasKey, RasterizedGlyph};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AtlasEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub is_color: bool,
}

#[derive(Debug, Clone)]
pub struct GlyphAtlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    /// Monotonically increased whenever a new glyph entry is rasterized into the atlas.
    pub generation: u64,
    entries: HashMap<AtlasKey, AtlasEntry>,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::new(2048, 2048)
    }
}

impl GlyphAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
            generation: 0,
            entries: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
        }
    }

    pub fn get(&self, key: &AtlasKey) -> Option<&AtlasEntry> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: AtlasKey, glyph: &RasterizedGlyph) -> AtlasEntry {
        if let Some(entry) = self.entries.get(&key) {
            return entry.clone();
        }

        let (width, height) = (glyph.width, glyph.height);
        if width == 0 || height == 0 {
            return AtlasEntry {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                bearing_x: glyph.bearing_x,
                bearing_y: glyph.bearing_y,
                is_color: glyph.is_color,
            };
        }

        if self.cursor_y + self.row_height + height + 1 > self.height {
            self.grow();
        }
        if self.cursor_x + width > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height + 1;
            self.row_height = 0;
        }

        let x = self.cursor_x;
        let y = self.cursor_y;
        self.row_height = self.row_height.max(height);

        for row in 0..height {
            for col in 0..width {
                let src = ((row * width + col) * 4) as usize;
                let dst = (((y + row) * self.width + x + col) * 4) as usize;
                if dst + 3 < self.pixels.len() && src + 3 < glyph.rgba.len() {
                    self.pixels[dst..dst + 4].copy_from_slice(&glyph.rgba[src..src + 4]);
                }
            }
        }

        self.cursor_x += width + 1;

        let entry = AtlasEntry {
            x,
            y,
            width,
            height,
            bearing_x: glyph.bearing_x,
            bearing_y: glyph.bearing_y,
            is_color: glyph.is_color,
        };
        self.entries.insert(key, entry.clone());
        self.generation = self.generation.saturating_add(1);
        entry
    }

    pub fn pixels_rgba(&self) -> &[u8] {
        &self.pixels
    }

    fn grow(&mut self) {
        let new_height = (self.height * 2).min(8192);
        if new_height <= self.height {
            // Atlas full: reset and bump generation so clients refresh UVs.
            self.pixels.fill(0);
            self.entries.clear();
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.row_height = 0;
            self.generation = self.generation.saturating_add(1);
            return;
        }
        self.height = new_height;
        self.pixels.resize((self.width * self.height * 4) as usize, 0);
        self.generation = self.generation.saturating_add(1);
    }
}

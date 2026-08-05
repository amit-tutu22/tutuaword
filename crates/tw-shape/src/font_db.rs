use fontdb::{Database, ID, Source};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId {
    id: ID,
    key: u32,
}

impl FontId {
    pub fn key(self) -> u32 {
        self.key
    }
}

pub struct FontDatabase {
    pub db: Database,
    default_family: String,
    key_map: HashMap<ID, u32>,
    next_key: u32,
}

impl Default for FontDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl FontDatabase {
    pub fn new() -> Self {
        let mut db = Database::new();
        db.load_system_fonts();
        Self {
            db,
            default_family: "Arial".into(),
            key_map: HashMap::new(),
            next_key: 1,
        }
    }

    fn assign_key(&mut self, id: ID) -> u32 {
        *self.key_map.entry(id).or_insert_with(|| {
            let key = self.next_key;
            self.next_key += 1;
            key
        })
    }

    pub fn resolve(&mut self, family: Option<&str>) -> Option<FontId> {
        let family = family.unwrap_or(&self.default_family);
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            ..Default::default()
        };
        let id = self.db.query(&query)?;
        let key = self.assign_key(id);
        Some(FontId { id, key })
    }

    pub fn face(&self, font_id: FontId) -> Option<&fontdb::FaceInfo> {
        self.db.face(font_id.id)
    }

    pub fn load_face_data(&self, font_id: FontId) -> Option<Vec<u8>> {
        let face = self.db.face(font_id.id)?;
        match &face.source {
            Source::File(path) => std::fs::read(path).ok(),
            Source::Binary(data) => Some(data.as_ref().as_ref().to_vec()),
            Source::SharedFile(_path, data) => Some(data.as_ref().as_ref().to_vec()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtlasKey {
    pub font_key: u32,
    pub glyph_id: u32,
    pub size_bits: u32,
}

impl AtlasKey {
    pub fn new(font_key: u32, glyph_id: u32, size: f32) -> Self {
        Self {
            font_key,
            glyph_id,
            size_bits: size.to_bits(),
        }
    }
}

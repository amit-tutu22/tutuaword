use fontdb::{Database, ID, Source};
use std::collections::HashMap;
use std::sync::Arc;

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

/// Families tried first when the requested font lacks a character, before
/// falling back to scanning every installed face. These cover the bulk of what
/// documents actually reach for: currency and symbols, then broad Unicode.
const FALLBACK_FAMILIES: &[&str] = &[
    "Helvetica",
    "Arial Unicode MS",
    "Lucida Grande",
    "Apple Symbols",
    "Segoe UI Symbol",
    "DejaVu Sans",
    "Noto Sans",
];

pub struct FontDatabase {
    pub db: Database,
    default_family: String,
    key_map: HashMap<ID, u32>,
    next_key: u32,
    /// Resolved fallback per character. Scanning the whole face list is slow,
    /// so each miss is paid once.
    fallback_cache: HashMap<char, Option<FontId>>,
    /// Coverage is queried once per character per run; parsing the face every
    /// time would dominate shaping.
    coverage_cache: HashMap<(u32, char), bool>,
    face_data_cache: HashMap<u32, Arc<Vec<u8>>>,
    /// Resolved face per family, indexed by [`style_slot`]. Every run asks for
    /// its font, so the database query has to be paid only once per style.
    family_cache: HashMap<String, [Option<Option<FontId>>; 4]>,
}

/// Index into a family's cached regular/bold/italic/bold-italic faces.
fn style_slot(bold: bool, italic: bool) -> usize {
    usize::from(bold) | (usize::from(italic) << 1)
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
            fallback_cache: HashMap::new(),
            coverage_cache: HashMap::new(),
            face_data_cache: HashMap::new(),
            family_cache: HashMap::new(),
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
        self.resolve_styled(family, false, false)
    }

    /// Resolves a family at the requested weight and slant. A family the system
    /// does not have falls back to the default family in the same style, which
    /// is what Word does for a missing font.
    pub fn resolve_styled(
        &mut self,
        family: Option<&str>,
        bold: bool,
        italic: bool,
    ) -> Option<FontId> {
        let family = family
            .map(str::trim)
            .filter(|f| !f.is_empty())
            .unwrap_or(&self.default_family)
            .to_owned();
        let slot = style_slot(bold, italic);

        if let Some(cached) = self.family_cache.get(&family) {
            if let Some(resolved) = cached[slot] {
                return resolved;
            }
        }

        let resolved = self
            .query_family(&family, bold, italic)
            .or_else(|| {
                let default = self.default_family.clone();
                (family != default).then(|| self.query_family(&default, bold, italic))?
            });

        self.family_cache.entry(family).or_default()[slot] = Some(resolved);
        resolved
    }

    fn query_family(&mut self, family: &str, bold: bool, italic: bool) -> Option<FontId> {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: if bold {
                fontdb::Weight::BOLD
            } else {
                fontdb::Weight::NORMAL
            },
            style: if italic {
                fontdb::Style::Italic
            } else {
                fontdb::Style::Normal
            },
            stretch: fontdb::Stretch::Normal,
        };
        let id = self.db.query(&query)?;
        // fontdb matches on family name alone, so an unknown family can come
        // back with an unrelated face. Only accept a genuine name match.
        let matches_name = self
            .db
            .face(id)?
            .families
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case(family));
        if !matches_name {
            return None;
        }
        let key = self.assign_key(id);
        Some(FontId { id, key })
    }

    pub fn face(&self, font_id: FontId) -> Option<&fontdb::FaceInfo> {
        self.db.face(font_id.id)
    }

    /// True when the face has a glyph for `ch`.
    pub fn covers(&mut self, font_id: FontId, ch: char) -> bool {
        if let Some(cached) = self.coverage_cache.get(&(font_id.key(), ch)) {
            return *cached;
        }
        let covered = self.face_covers(font_id.id, ch);
        self.coverage_cache.insert((font_id.key(), ch), covered);
        covered
    }

    fn face_covers(&self, id: ID, ch: char) -> bool {
        self.db
            .with_face_data(id, |data, index| {
                rustybuzz::Face::from_slice(data, index)
                    .and_then(|face| face.glyph_index(ch))
                    .is_some()
            })
            .unwrap_or(false)
    }

    /// Finds an installed face that can render `ch`, for characters the
    /// requested font is missing. Results are cached, including misses.
    pub fn fallback_for(&mut self, ch: char) -> Option<FontId> {
        if let Some(cached) = self.fallback_cache.get(&ch) {
            return *cached;
        }

        let resolved = self.search_fallback(ch);
        self.fallback_cache.insert(ch, resolved);
        resolved
    }

    fn search_fallback(&mut self, ch: char) -> Option<FontId> {
        for family in FALLBACK_FAMILIES {
            if let Some(id) = self.resolve(Some(family)) {
                if self.covers(id, ch) {
                    return Some(id);
                }
            }
        }

        // Nothing preferred has it, so take the first installed face that does.
        let ids: Vec<ID> = self.db.faces().map(|face| face.id).collect();
        let found = ids.into_iter().find(|id| self.face_covers(*id, ch))?;
        let key = self.assign_key(found);
        Some(FontId { id: found, key })
    }

    pub fn load_face_data(&self, font_id: FontId) -> Option<Vec<u8>> {
        let face = self.db.face(font_id.id)?;
        match &face.source {
            Source::File(path) => std::fs::read(path).ok(),
            Source::Binary(data) => Some(data.as_ref().as_ref().to_vec()),
            Source::SharedFile(_path, data) => Some(data.as_ref().as_ref().to_vec()),
        }
    }

    /// Face bytes kept in memory across calls, since shaping re-reads them for
    /// every text segment.
    pub fn face_data(&mut self, font_id: FontId) -> Option<Arc<Vec<u8>>> {
        if let Some(cached) = self.face_data_cache.get(&font_id.key()) {
            return Some(Arc::clone(cached));
        }
        let data = Arc::new(self.load_face_data(font_id)?);
        self.face_data_cache
            .insert(font_id.key(), Arc::clone(&data));
        Some(data)
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

use fontdb::{Database, FaceInfo, ID, Language, Source, Stretch, Style, Weight};
use std::collections::HashMap;
use std::fmt;
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

/// CSS numeric weight of a regular face.
pub const WEIGHT_REGULAR: u16 = 400;
/// CSS numeric weight of a bold face.
pub const WEIGHT_BOLD: u16 = 700;

/// How a host wants a face registered from bytes to be labelled.
///
/// The names inside the font file are ignored, so a host may register
/// `DejaVuSans.ttf` bytes as `Calibri` and have a document written against
/// Calibri resolve without shipping Calibri itself. Use
/// [`FontDatabase::register_font_data`] instead to keep the file's own names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFaceSpec {
    /// Family name documents will ask for. Matched case-insensitively.
    pub family: String,
    /// CSS numeric weight: 400 regular, 700 bold.
    pub weight: u16,
    pub italic: bool,
    /// Face index inside a TrueType/OpenType collection. 0 for a single-face file.
    pub index: u32,
}

impl FontFaceSpec {
    /// A regular, upright face of `family`.
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            weight: WEIGHT_REGULAR,
            italic: false,
            index: 0,
        }
    }

    pub fn weight(mut self, weight: u16) -> Self {
        self.weight = weight;
        self
    }

    pub fn bold(mut self) -> Self {
        self.weight = WEIGHT_BOLD;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn index(mut self, index: u32) -> Self {
        self.index = index;
        self
    }
}

/// Why a host's font bytes could not be registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontRegistrationError {
    /// The bytes are not a font face this engine can shape with.
    UnreadableFontData,
    /// The requested face index is past the end of the collection.
    FaceIndexOutOfRange { index: u32, face_count: u32 },
    /// The engine executor could not be reached — it has already shut down.
    RegistrationNotSupported,
}

impl fmt::Display for FontRegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnreadableFontData => f.write_str("font data could not be parsed"),
            Self::FaceIndexOutOfRange { index, face_count } => write!(
                f,
                "face index {index} is out of range for a collection of {face_count} faces"
            ),
            Self::RegistrationNotSupported => {
                f.write_str("font registration is not supported by this executor")
            }
        }
    }
}

impl std::error::Error for FontRegistrationError {}

/// Families tried first when the requested font lacks a character, before
/// falling back to scanning every available face. These cover the bulk of what
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
    document_fallbacks: Vec<String>,
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
    /// A database preloaded with the operating system's installed fonts.
    ///
    /// On `wasm32` there is no font directory to scan and no filesystem to scan
    /// it with, so this is identical to [`FontDatabase::empty`] and the host has
    /// to supply faces itself.
    pub fn new() -> Self {
        let mut fonts = Self::empty();
        fonts.load_system_fonts();
        fonts
    }

    /// A database with no faces: no system scan, no filesystem access. Every
    /// face comes from [`FontDatabase::register_face`] or
    /// [`FontDatabase::register_font_data`].
    ///
    /// This is what a web host uses, where the only source of font bytes is the
    /// host itself.
    pub fn empty() -> Self {
        Self {
            db: Database::new(),
            default_family: "Arial".into(),
            key_map: HashMap::new(),
            next_key: 1,
            fallback_cache: HashMap::new(),
            coverage_cache: HashMap::new(),
            face_data_cache: HashMap::new(),
            family_cache: HashMap::new(),
            document_fallbacks: Vec::new(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_system_fonts(&mut self) {
        self.db.load_system_fonts();
        self.invalidate_lookups();
    }

    /// No-op: `fontdb`'s `fs` feature is off on `wasm32` (see this crate's
    /// target-specific dependency section), so `load_system_fonts` does not
    /// exist there to call.
    #[cfg(target_arch = "wasm32")]
    fn load_system_fonts(&mut self) {}

    /// Registers `data` as one face under the host-supplied family, weight, and
    /// slant, ignoring the names inside the font file.
    ///
    /// The bytes are parsed once to reject data nothing downstream could shape,
    /// then shared with the shaper and rasterizer without a further copy.
    pub fn register_face(
        &mut self,
        spec: &FontFaceSpec,
        data: impl Into<Vec<u8>>,
    ) -> Result<FontId, FontRegistrationError> {
        let shared = Arc::new(data.into());
        let face_count = rustybuzz::ttf_parser::fonts_in_collection(&shared).unwrap_or(1);
        if spec.index >= face_count {
            return Err(FontRegistrationError::FaceIndexOutOfRange {
                index: spec.index,
                face_count,
            });
        }
        if rustybuzz::Face::from_slice(&shared, spec.index).is_none() {
            return Err(FontRegistrationError::UnreadableFontData);
        }

        let id = self.db.push_face_info(FaceInfo {
            id: ID::dummy(),
            source: binary_source(&shared),
            index: spec.index,
            families: vec![(spec.family.clone(), Language::English_UnitedStates)],
            post_script_name: post_script_name(spec),
            style: if spec.italic {
                Style::Italic
            } else {
                Style::Normal
            },
            weight: Weight(spec.weight),
            stretch: Stretch::Normal,
            monospaced: false,
        });
        Ok(self.adopt(id, shared))
    }

    /// Registers every face in `data` under the family, weight, and slant
    /// recorded in the font file itself.
    ///
    /// Use this when the host ships a font under its real name; use
    /// [`FontDatabase::register_face`] to rename it or to correct metadata the
    /// file gets wrong.
    pub fn register_font_data(
        &mut self,
        data: impl Into<Vec<u8>>,
    ) -> Result<Vec<FontId>, FontRegistrationError> {
        let shared = Arc::new(data.into());
        let ids = self.db.load_font_source(binary_source(&shared));
        if ids.is_empty() {
            return Err(FontRegistrationError::UnreadableFontData);
        }
        Ok(ids
            .iter()
            .map(|&id| self.adopt(id, Arc::clone(&shared)))
            .collect())
    }

    /// Gives a newly inserted face a stable key and pre-seeds its bytes, so no
    /// later lookup has to go back to the source for data we already hold.
    fn adopt(&mut self, id: ID, data: Arc<Vec<u8>>) -> FontId {
        let key = self.assign_key(id);
        self.face_data_cache.insert(key, data);
        self.invalidate_lookups();
        FontId { id, key }
    }

    /// Family and per-character fallback resolutions are memoised including
    /// misses, so a newly available face has to clear them. Coverage is keyed by
    /// face and stays valid.
    fn invalidate_lookups(&mut self) {
        self.family_cache.clear();
        self.fallback_cache.clear();
    }

    /// Number of faces available to resolve against.
    pub fn face_count(&self) -> usize {
        self.db.len()
    }

    /// True when no face is available, so nothing can be shaped at all.
    pub fn is_empty(&self) -> bool {
        self.db.len() == 0
    }

    /// Family names available to resolve against, deduplicated and sorted.
    pub fn families(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .db
            .faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    /// Word theme + common Office families used before scanning every face.
    pub fn configure_document(&mut self, minor_font: &str, major_font: &str) {
        self.default_family = minor_font.to_string();
        self.document_fallbacks = vec![
            minor_font.to_string(),
            major_font.to_string(),
            "Calibri".into(),
            "Cambria".into(),
            "Aptos".into(),
            "Times New Roman".into(),
            "Arial".into(),
            "Helvetica".into(),
        ];
        self.invalidate_lookups();
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

    /// Resolves a family at the requested weight and slant.
    ///
    /// The chain is: the family itself, then its Office aliases, then the
    /// document default family (the theme minor font), then — only if none of
    /// those exist — any available face in the closest style. The last step
    /// matters for injected fonts, where the default family is as likely to be
    /// missing as the requested one; returning `None` there would render the
    /// document blank, which is worse than rendering it in the wrong typeface.
    /// `None` comes back only from a database with no faces at all.
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
            .or_else(|| self.query_aliases(&family, bold, italic))
            .or_else(|| {
                let default = self.default_family.clone();
                (family != default).then(|| self.query_family(&default, bold, italic))?
            })
            .or_else(|| self.last_resort(bold, italic));

        self.family_cache.entry(family).or_default()[slot] = Some(resolved);
        resolved
    }

    fn query_family(&mut self, family: &str, bold: bool, italic: bool) -> Option<FontId> {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: if bold { Weight::BOLD } else { Weight::NORMAL },
            style: if italic {
                Style::Italic
            } else {
                Style::Normal
            },
            stretch: Stretch::Normal,
        };
        let id = self
            .db
            .query(&query)
            // fontdb matches on family name alone, so an unknown family can come
            // back with an unrelated face. Only accept a genuine name match.
            .filter(|id| self.face_has_family(*id, family))
            // fontdb's own query is case-sensitive. A host registering faces
            // from bytes picks the family label itself, so `calibri` and
            // `Calibri` have to name the same face.
            .or_else(|| self.match_family_ignoring_case(family, bold, italic))?;
        let key = self.assign_key(id);
        Some(FontId { id, key })
    }

    fn face_has_family(&self, id: ID, family: &str) -> bool {
        self.db.face(id).is_some_and(|face| {
            face.families
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case(family))
        })
    }

    fn match_family_ignoring_case(&self, family: &str, bold: bool, italic: bool) -> Option<ID> {
        let candidates: Vec<ID> = self
            .db
            .faces()
            .filter(|face| {
                face.families
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case(family))
            })
            .map(|face| face.id)
            .collect();
        self.closest_style(&candidates, bold, italic)
    }

    /// Any available face, in the closest style — the last step of
    /// [`FontDatabase::resolve_styled`].
    fn last_resort(&mut self, bold: bool, italic: bool) -> Option<FontId> {
        let candidates: Vec<ID> = self.db.faces().map(|face| face.id).collect();
        let id = self.closest_style(&candidates, bold, italic)?;
        let key = self.assign_key(id);
        Some(FontId { id, key })
    }

    /// Face nearest the requested weight and slant. Slant dominates: an upright
    /// face at the wrong weight reads far better than a wrongly slanted one.
    fn closest_style(&self, candidates: &[ID], bold: bool, italic: bool) -> Option<ID> {
        const SLANT_PENALTY: i32 = 10_000;
        let target_weight = i32::from(if bold { WEIGHT_BOLD } else { WEIGHT_REGULAR });
        candidates.iter().copied().min_by_key(|id| {
            let (face_italic, weight) = self
                .db
                .face(*id)
                .map(|face| (face.style != Style::Normal, i32::from(face.weight.0)))
                .unwrap_or((false, i32::from(WEIGHT_REGULAR)));
            let slant = if face_italic == italic {
                0
            } else {
                SLANT_PENALTY
            };
            slant + (weight - target_weight).abs()
        })
    }

    pub fn face(&self, font_id: FontId) -> Option<&fontdb::FaceInfo> {
        self.db.face(font_id.id)
    }

    /// Ascent, descent, and line gap in points for a given font size.
    pub fn vertical_metrics(&self, font_id: FontId, size: f32) -> (f32, f32, f32) {
        self.db
            .with_face_data(font_id.id, |data, index| {
                rustybuzz::Face::from_slice(data, index).map(|face| {
                    let scale = size / face.units_per_em() as f32;
                    let ascent = face.ascender() as f32 * scale;
                    let descent = (-face.descender() as f32) * scale;
                    let line_gap = face.line_gap() as f32 * scale;
                    (ascent, descent, line_gap.max(0.0))
                })
            })
            .flatten()
            .unwrap_or((size, size * 0.25, size * 0.1))
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
                    // Glyph 0 is `.notdef` — treat as uncovered so fallback can
                    // replace empty tofu boxes (common for Symbol/PUA bullets).
                    .map(|gid| gid.0 != 0)
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// Finds an available face that can render `ch`, for characters the
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
        let families: Vec<String> = self
            .document_fallbacks
            .iter()
            .cloned()
            .chain(FALLBACK_FAMILIES.iter().map(|s| s.to_string()))
            .collect();
        for family in &families {
            if let Some(id) = self.resolve(Some(family.as_str())) {
                if self.covers(id, ch) {
                    return Some(id);
                }
            }
        }

        // Nothing preferred has it, so take the first available face that does.
        // With injected fonts that list is the host's registrations; with system
        // fonts it is everything installed.
        let ids: Vec<ID> = self.db.faces().map(|face| face.id).collect();
        let found = ids.into_iter().find(|id| self.face_covers(*id, ch))?;
        let key = self.assign_key(found);
        Some(FontId { id: found, key })
    }

    fn query_aliases(&mut self, family: &str, bold: bool, italic: bool) -> Option<FontId> {
        for alias in family_aliases(family) {
            if let Some(id) = self.query_family(alias, bold, italic) {
                return Some(id);
            }
        }
        None
    }

    /// Resolves a layout/render font key back to a [`FontId`].
    pub fn font_id_for_key(&self, key: u32) -> Option<FontId> {
        self.key_map
            .iter()
            .find(|(_, &mapped)| mapped == key)
            .map(|(&id, &mapped)| FontId { id, key: mapped })
    }

    /// Face bytes for a layout glyph's `font_id` key (PDF embedding / export).
    pub fn load_face_data_by_key(&self, key: u32) -> Option<Vec<u8>> {
        let font_id = self.font_id_for_key(key)?;
        self.load_face_data(font_id)
    }

    /// Collection face index for a layout font key (`0` for single-face files).
    pub fn face_index_for_key(&self, key: u32) -> Option<u32> {
        let font_id = self.font_id_for_key(key)?;
        Some(self.db.face(font_id.id)?.index)
    }

    /// Bytes suitable for PDF `/FontFile2` / `/FontFile3`: a single SFNT face.
    ///
    /// TrueType Collections (`.ttc`) are expanded to one face so exporters do
    /// not have to ship the whole collection.
    pub fn load_embeddable_sfnt_by_key(&self, key: u32) -> Option<Vec<u8>> {
        let data = self.load_face_data_by_key(key)?;
        let index = self.face_index_for_key(key).unwrap_or(0);
        extract_embeddable_sfnt(&data, index)
    }

    /// Primary family name for a layout font key, when known.
    pub fn family_name_for_key(&self, key: u32) -> Option<String> {
        let font_id = self.font_id_for_key(key)?;
        let face = self.db.face(font_id.id)?;
        if let Some((name, _)) = face.families.first() {
            return Some(name.clone());
        }
        if face.post_script_name.is_empty() {
            None
        } else {
            Some(face.post_script_name.clone())
        }
    }

    pub fn load_face_data(&self, font_id: FontId) -> Option<Vec<u8>> {
        // Registered faces already hold their bytes, so a host-injected face
        // never reaches the source at all.
        if let Some(cached) = self.face_data_cache.get(&font_id.key()) {
            return Some(cached.as_ref().clone());
        }
        let face = self.db.face(font_id.id)?;
        match &face.source {
            Source::Binary(data) => Some(data.as_ref().as_ref().to_vec()),
            // `Source::File` and `Source::SharedFile` exist only under fontdb's
            // `fs` feature, which this crate does not enable on wasm32.
            #[cfg(not(target_arch = "wasm32"))]
            Source::File(path) => std::fs::read(path).ok(),
            #[cfg(not(target_arch = "wasm32"))]
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

/// Returns a single-font SFNT suitable for PDF embedding.
///
/// Passes through TrueType / OpenType CFF; extracts one face from a TTC.
fn extract_embeddable_sfnt(data: &[u8], face_index: u32) -> Option<Vec<u8>> {
    if data.len() < 4 {
        return None;
    }
    match &data[0..4] {
        [0x00, 0x01, 0x00, 0x00] | b"true" | b"typ1" | b"OTTO" => Some(data.to_vec()),
        b"ttcf" => extract_face_from_ttc(data, face_index),
        _ => None,
    }
}

fn extract_face_from_ttc(data: &[u8], face_index: u32) -> Option<Vec<u8>> {
    if data.len() < 12 || &data[0..4] != b"ttcf" {
        return None;
    }
    let num_fonts = u32::from_be_bytes(data[8..12].try_into().ok()?);
    if face_index >= num_fonts {
        return None;
    }
    let offset_entry = 12 + (face_index as usize) * 4;
    if offset_entry + 4 > data.len() {
        return None;
    }
    let face_offset = u32::from_be_bytes(data[offset_entry..offset_entry + 4].try_into().ok()?) as usize;
    if face_offset + 12 > data.len() {
        return None;
    }

    let num_tables = u16::from_be_bytes(data[face_offset + 4..face_offset + 6].try_into().ok()?) as usize;
    let table_dir_start = face_offset + 12;
    let table_dir_end = table_dir_start + num_tables * 16;
    if table_dir_end > data.len() {
        return None;
    }

    let mut tables: Vec<( [u8; 4], u32, usize, usize)> = Vec::with_capacity(num_tables);
    for i in 0..num_tables {
        let e = table_dir_start + i * 16;
        let tag: [u8; 4] = data[e..e + 4].try_into().ok()?;
        let checksum = u32::from_be_bytes(data[e + 4..e + 8].try_into().ok()?);
        let offset = u32::from_be_bytes(data[e + 8..e + 12].try_into().ok()?) as usize;
        let length = u32::from_be_bytes(data[e + 12..e + 16].try_into().ok()?) as usize;
        if offset.checked_add(length)? > data.len() {
            return None;
        }
        tables.push((tag, checksum, offset, length));
    }

    // Offset table (12) + directory (16 * n) then table payloads, 4-byte aligned.
    let mut out = Vec::new();
    out.extend_from_slice(&data[face_offset..face_offset + 12]);
    let mut dir = vec![0u8; num_tables * 16];
    let mut cursor = 12 + num_tables * 16;
    for (i, (tag, checksum, _offset, length)) in tables.iter().enumerate() {
        while cursor % 4 != 0 {
            cursor += 1;
        }
        let entry = i * 16;
        dir[entry..entry + 4].copy_from_slice(tag);
        dir[entry + 4..entry + 8].copy_from_slice(&checksum.to_be_bytes());
        dir[entry + 8..entry + 12].copy_from_slice(&(cursor as u32).to_be_bytes());
        dir[entry + 12..entry + 16].copy_from_slice(&(*length as u32).to_be_bytes());
        cursor += length;
    }
    out.extend_from_slice(&dir);

    for (_, _, offset, length) in &tables {
        while out.len() % 4 != 0 {
            out.push(0);
        }
        out.extend_from_slice(&data[*offset..*offset + *length]);
    }
    Some(out)
}

/// Wraps host bytes for `fontdb` without copying them: the database and this
/// crate's own byte cache share one allocation.
fn binary_source(data: &Arc<Vec<u8>>) -> Source {
    Source::Binary(data.clone())
}

/// A synthetic PostScript name. Nothing in this engine queries by PostScript
/// name, but `fontdb` requires the field and DOCX font embedding may read it.
fn post_script_name(spec: &FontFaceSpec) -> String {
    let compact: String = spec.family.chars().filter(|c| !c.is_whitespace()).collect();
    let style = match (spec.weight >= WEIGHT_BOLD, spec.italic) {
        (true, true) => "BoldItalic",
        (true, false) => "Bold",
        (false, true) => "Italic",
        (false, false) => "Regular",
    };
    format!("{compact}-{style}")
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

fn family_aliases(family: &str) -> &'static [&'static str] {
    match family.to_ascii_lowercase().as_str() {
        s if s == "calibri" => &["Carlito", "Helvetica Neue", "Arial"],
        s if s == "cambria" => &["Caladea", "Georgia", "Times New Roman"],
        s if s == "arial" => &["Helvetica", "Liberation Sans"],
        s if s == "times new roman" => &["Times", "Liberation Serif", "Georgia"],
        s if s == "aptos" => &["Segoe UI", "Calibri", "Arial"],
        _ => &[],
    }
}

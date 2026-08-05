//! Writes the image parts, relationships, and content-type defaults an
//! exported document needs, so that a `w:drawing` in `document.xml` resolves
//! to real bytes when the file is reopened.

use std::collections::{HashMap, HashSet};

use tw_model::ImageData;

use crate::xml_util::{read_own_attr, split_elements};
use crate::DocxPackage;

const IMAGE_RELATIONSHIP_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";
const RELS_PART: &str = "word/_rels/document.xml.rels";
const CONTENT_TYPES_PART: &str = "[Content_Types].xml";

pub struct MediaWriter {
    /// Relationship target (relative to `word/`) → relationship id, as found in
    /// the source package.
    targets: HashMap<String, String>,
    /// Part name → relationship id, for assets referenced during this export.
    referenced: HashMap<String, String>,
    /// Part names already spoken for, so a generated one cannot overwrite an
    /// existing part.
    taken: HashSet<String>,
    parts: Vec<(String, Vec<u8>)>,
    relationships: Vec<(String, String)>,
    next_relationship: u32,
    next_drawing: u32,
}

impl MediaWriter {
    pub fn new(package: &DocxPackage) -> Self {
        let existing = package
            .parts
            .get(RELS_PART)
            .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
            .unwrap_or_default();

        let next_relationship = existing
            .keys()
            .filter_map(|id| id.strip_prefix("rId")?.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
            + 1;

        let targets = existing
            .into_iter()
            .map(|(id, target)| (normalize_target(&target), id))
            .collect();

        Self {
            targets,
            referenced: HashMap::new(),
            taken: package.parts.keys().cloned().collect(),
            parts: Vec::new(),
            relationships: Vec::new(),
            next_relationship,
            next_drawing: 1,
        }
    }

    /// Relationship id a `<a:blip r:embed>` should point at, registering the
    /// media part if it is not already in the package. `None` when the image
    /// carries no bytes and so cannot be written.
    pub fn reference(&mut self, image: &ImageData) -> Option<String> {
        if image.bytes.is_empty() {
            return None;
        }
        let part_name = self.part_name_for(image);
        if let Some(id) = self.referenced.get(&part_name) {
            return Some(id.clone());
        }

        let target = part_name
            .strip_prefix("word/")
            .unwrap_or(&part_name)
            .to_string();
        let relationship_id = match self.targets.get(&target) {
            Some(id) => id.clone(),
            None => {
                let id = format!("rId{}", self.next_relationship);
                self.next_relationship += 1;
                self.relationships.push((id.clone(), target.clone()));
                self.targets.insert(target, id.clone());
                id
            }
        };

        self.parts.push((part_name.clone(), image.bytes.clone()));
        self.referenced.insert(part_name, relationship_id.clone());
        Some(relationship_id)
    }

    pub fn next_drawing_id(&mut self) -> u32 {
        let id = self.next_drawing;
        self.next_drawing += 1;
        id
    }

    pub fn commit(self, package: &mut DocxPackage) {
        if self.parts.is_empty() {
            return;
        }

        let mut extensions: Vec<String> = Vec::new();
        for (name, bytes) in self.parts {
            if let Some(extension) = extension_of(&name) {
                if !extensions.contains(&extension) {
                    extensions.push(extension);
                }
            }
            package.parts.insert(name.clone(), bytes);
            package.mark_modified(name);
        }

        if !self.relationships.is_empty() {
            let existing = package
                .parts
                .get(RELS_PART)
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                .unwrap_or_else(|| DEFAULT_RELS.to_string());
            let updated = append_relationships(&existing, &self.relationships);
            package.parts.insert(RELS_PART.into(), updated.into_bytes());
            package.mark_modified(RELS_PART.into());
        }

        let content_types = package
            .parts
            .get(CONTENT_TYPES_PART)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_else(|| DEFAULT_CONTENT_TYPES.to_string());
        if let Some(updated) = ensure_default_extensions(&content_types, &extensions) {
            package
                .parts
                .insert(CONTENT_TYPES_PART.into(), updated.into_bytes());
            package.mark_modified(CONTENT_TYPES_PART.into());
        }
    }

    /// Imported images keep the part name they came from, so re-exporting
    /// reuses the same media part instead of duplicating it.
    fn part_name_for(&mut self, image: &ImageData) -> String {
        if image.asset_id.starts_with("word/media/") {
            return image.asset_id.clone();
        }
        let extension = extension_for_mime(&image.mime_type);
        let mut index = 1;
        loop {
            let name = format!("word/media/image{index}.{extension}");
            if self.taken.insert(name.clone()) {
                return name;
            }
            index += 1;
        }
    }
}

fn parse_relationships(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for element in split_elements(xml, "Relationship") {
        let (Some(id), Some(target)) = (
            read_own_attr(element, "Id"),
            read_own_attr(element, "Target"),
        ) else {
            continue;
        };
        map.insert(id.to_string(), target.to_string());
    }
    map
}

fn normalize_target(target: &str) -> String {
    target.trim_start_matches("./").to_string()
}

fn append_relationships(xml: &str, relationships: &[(String, String)]) -> String {
    let mut additions = String::new();
    for (id, target) in relationships {
        additions.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{IMAGE_RELATIONSHIP_TYPE}" Target="{target}"/>"#
        ));
    }

    if let Some(index) = xml.rfind("</Relationships>") {
        let mut out = String::with_capacity(xml.len() + additions.len());
        out.push_str(&xml[..index]);
        out.push_str(&additions);
        out.push_str(&xml[index..]);
        return out;
    }
    // A self-closing `<Relationships/>` has no place to insert into.
    if let Some(index) = xml.rfind("/>") {
        let mut out = String::with_capacity(xml.len() + additions.len() + 20);
        out.push_str(&xml[..index]);
        out.push('>');
        out.push_str(&additions);
        out.push_str("</Relationships>");
        return out;
    }
    format!("{DEFAULT_RELS_OPEN}{additions}</Relationships>")
}

/// Adds a `<Default Extension=...>` for each image extension the package does
/// not already declare. `None` when nothing needed adding.
fn ensure_default_extensions(xml: &str, extensions: &[String]) -> Option<String> {
    let missing: Vec<&String> = extensions
        .iter()
        .filter(|ext| !declares_extension(xml, ext))
        .collect();
    if missing.is_empty() {
        return None;
    }

    let mut additions = String::new();
    for extension in missing {
        additions.push_str(&format!(
            r#"<Default Extension="{extension}" ContentType="{}"/>"#,
            content_type_for_extension(extension)
        ));
    }

    if let Some(index) = xml.rfind("</Types>") {
        let mut out = String::with_capacity(xml.len() + additions.len());
        out.push_str(&xml[..index]);
        out.push_str(&additions);
        out.push_str(&xml[index..]);
        return Some(out);
    }
    if let Some(index) = xml.rfind("/>") {
        let mut out = String::new();
        out.push_str(&xml[..index]);
        out.push('>');
        out.push_str(&additions);
        out.push_str("</Types>");
        return Some(out);
    }
    None
}

fn declares_extension(xml: &str, extension: &str) -> bool {
    split_elements(xml, "Default")
        .into_iter()
        .filter_map(|element| read_own_attr(element, "Extension"))
        .any(|declared| declared.eq_ignore_ascii_case(extension))
}

fn extension_of(part_name: &str) -> Option<String> {
    part_name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
}

fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpeg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/webp" => "webp",
        "image/tiff" => "tiff",
        "image/x-emf" => "emf",
        "image/x-wmf" => "wmf",
        _ => "png",
    }
}

fn content_type_for_extension(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "tif" | "tiff" => "image/tiff",
        "emf" => "image/x-emf",
        "wmf" => "image/x-wmf",
        _ => "image/png",
    }
}

const DEFAULT_RELS_OPEN: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#;

const DEFAULT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#;

const DEFAULT_CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn png(asset_id: &str) -> ImageData {
        ImageData {
            asset_id: asset_id.into(),
            mime_type: "image/png".into(),
            width_px: 1,
            height_px: 1,
            bytes: vec![0x89, b'P', b'N', b'G'],
        }
    }

    #[test]
    fn an_image_without_bytes_cannot_be_referenced() {
        let mut writer = MediaWriter::new(&DocxPackage::minimal());
        let empty = ImageData::placeholder(10, 10);

        assert_eq!(writer.reference(&empty), None);
    }

    #[test]
    fn an_imported_image_reuses_its_original_relationship() {
        let mut package = DocxPackage::minimal();
        package.parts.insert(
            RELS_PART.into(),
            br#"<Relationships><Relationship Id="rId9" Type="i" Target="media/image1.png"/></Relationships>"#.to_vec(),
        );
        let mut writer = MediaWriter::new(&package);

        assert_eq!(
            writer.reference(&png("word/media/image1.png")).as_deref(),
            Some("rId9")
        );
    }

    #[test]
    fn a_new_image_gets_an_id_that_does_not_collide() {
        let mut package = DocxPackage::minimal();
        package.parts.insert(
            RELS_PART.into(),
            br#"<Relationships><Relationship Id="rId4" Type="i" Target="styles.xml"/></Relationships>"#
                .to_vec(),
        );
        let mut writer = MediaWriter::new(&package);

        assert_eq!(writer.reference(&png("fresh")).as_deref(), Some("rId5"));
    }

    #[test]
    fn referencing_the_same_asset_twice_writes_one_relationship() {
        let mut writer = MediaWriter::new(&DocxPackage::minimal());
        let image = png("word/media/image1.png");

        let first = writer.reference(&image);
        let second = writer.reference(&image);

        assert_eq!(first, second);
        assert_eq!(writer.relationships.len(), 1);
    }

    #[test]
    fn committing_adds_the_part_relationship_and_content_type() {
        let mut package = DocxPackage::minimal();
        let mut writer = MediaWriter::new(&package);
        writer.reference(&png("word/media/image1.png"));
        writer.commit(&mut package);

        assert!(package.parts.contains_key("word/media/image1.png"));
        let rels = String::from_utf8(package.parts[RELS_PART].clone()).unwrap();
        assert!(rels.contains("media/image1.png"), "{rels}");
        let types = String::from_utf8(package.parts[CONTENT_TYPES_PART].clone()).unwrap();
        assert!(types.contains(r#"Extension="png""#), "{types}");
    }

    #[test]
    fn an_already_declared_extension_is_not_declared_twice() {
        let xml = r#"<Types><Default Extension="png" ContentType="image/png"/></Types>"#;

        assert_eq!(ensure_default_extensions(xml, &["png".into()]), None);
    }
}

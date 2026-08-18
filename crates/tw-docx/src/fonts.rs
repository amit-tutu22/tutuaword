//! Embedded font extraction from `word/fontTable.xml` and `word/fonts/*.odttf`.

use std::collections::HashMap;

use tw_shape::FontFaceSpec;

use crate::xml_util::{read_attr_value, read_own_attr, split_elements};
use crate::DocxPackage;

/// A deobfuscated font payload ready for [`tw_shape::FontDatabase::register_face`].
#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    pub spec: FontFaceSpec,
    pub data: Vec<u8>,
}

/// Parse `fontTable.xml`, resolve `word/fonts/*.odttf` parts, and deobfuscate.
pub fn extract_embedded_fonts(package: &DocxPackage) -> Vec<EmbeddedFont> {
    let Some(font_table_bytes) = package.parts.get("word/fontTable.xml") else {
        return Vec::new();
    };
    let font_table = String::from_utf8_lossy(font_table_bytes);
    let rels = package
        .parts
        .get("word/_rels/fontTable.xml.rels")
        .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
        .unwrap_or_default();

    let mut out = Vec::new();
    for element in split_elements(&font_table, "w:font") {
        let Some(family) = read_own_attr(element, "w:name") else {
            continue;
        };
        for (embed_tag, weight, italic) in [
            ("w:embedRegular", WEIGHT_REGULAR, false),
            ("w:embedBold", WEIGHT_BOLD, false),
            ("w:embedItalic", WEIGHT_REGULAR, true),
            ("w:embedBoldItalic", WEIGHT_BOLD, true),
        ] {
            if !element.contains(embed_tag) {
                continue;
            }
            let Some(font_key) = read_attr_value(element, embed_tag, "w:fontKey") else {
                continue;
            };
            let Some(rel_id) = read_attr_value(element, embed_tag, "r:id") else {
                continue;
            };
            let Some(target) = rels.get(&rel_id) else {
                continue;
            };
            let part = normalize_font_part(target);
            let Some(obfuscated) = package.parts.get(&part) else {
                continue;
            };
            let data = deobfuscate_odttf(obfuscated, &font_key);
            if data.len() < 128 {
                continue;
            }
            let mut spec = FontFaceSpec::new(family).weight(weight);
            if italic {
                spec = spec.italic();
            }
            out.push(EmbeddedFont { spec, data });
        }
    }
    out
}

const WEIGHT_REGULAR: u16 = 400;
const WEIGHT_BOLD: u16 = 700;

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

fn normalize_font_part(target: &str) -> String {
    if target.starts_with("word/") {
        target.to_string()
    } else {
        format!("word/{}", target.trim_start_matches("./"))
    }
}

/// Office Open XML font obfuscation: XOR the first 32 bytes with reordered GUID bytes.
pub fn deobfuscate_odttf(data: &[u8], font_key: &str) -> Vec<u8> {
    let mut out = data.to_vec();
    let Some(key) = guid_xor_key(font_key) else {
        return out;
    };
    for (i, byte) in out.iter_mut().take(32).enumerate() {
        *byte ^= key[i % 16];
    }
    out
}

fn guid_xor_key(font_key: &str) -> Option<[u8; 16]> {
    let trimmed = font_key.trim().trim_matches(|c| c == '{' || c == '}');
    let uuid = uuid::Uuid::parse_str(trimmed).ok()?;
    let b = uuid.as_bytes();
    Some([
        b[3], b[2], b[1], b[0], b[5], b[4], b[7], b[6], b[8], b[9], b[10], b[11], b[12], b[13],
        b[14], b[15],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deobfuscate_round_trip_xor() {
        let key = "{00112233-4455-6677-8899-AABBCCDDEEFF}";
        let mut data = b"0123456789ABCDEF0123456789ABCDEF".to_vec();
        data.extend(std::iter::repeat_n(0u8, 64));
        let obfuscated = {
            let mut copy = data.clone();
            let xor = guid_xor_key(key).unwrap();
            for (i, byte) in copy.iter_mut().take(32).enumerate() {
                *byte ^= xor[i % 16];
            }
            copy
        };
        let restored = deobfuscate_odttf(&obfuscated, key);
        assert_eq!(&restored[..32], &data[..32]);
    }
}

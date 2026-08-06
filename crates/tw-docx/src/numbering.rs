//! Serializes `word/numbering.xml` from the document's numbering catalog.

use tw_model::{
    CharFormat, ListMarkerFormat, ListSuffix, NumberingCatalog, NumberingDefinition,
};

use crate::export::{escape_xml, to_twips};

pub fn serialize_numbering_xml(catalog: &NumberingCatalog) -> String {
    if catalog.definitions.is_empty() {
        return String::new();
    }

    let mut body = String::new();
    for def in catalog.definitions.values() {
        body.push_str(&serialize_abstract_num(def));
        body.push_str(&serialize_num(def.id, def.id));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{body}</w:numbering>"#
    )
}

fn serialize_abstract_num(def: &NumberingDefinition) -> String {
    let mut levels = String::new();
    for lvl in &def.levels {
        let suffix = match lvl.suffix {
            ListSuffix::Tab => "tab",
            ListSuffix::Space => "space",
            ListSuffix::Nothing => "nothing",
        };
        let marker = lvl
            .marker_text
            .clone()
            .unwrap_or_else(|| default_lvl_text(lvl.format).into());
        let rpr = serialize_level_rpr(&lvl.char_format);
        levels.push_str(&format!(
            r#"<w:lvl w:ilvl="{}"><w:start w:val="{}"/><w:numFmt w:val="{}"/><w:suff w:val="{}"/><w:lvlText w:val="{}"/><w:lvlJc w:val="left"/><w:pPr><w:ind w:left="{}" w:hanging="{}"/></w:pPr>{rpr}</w:lvl>"#,
            lvl.level,
            lvl.start.max(1),
            num_fmt(lvl.format),
            suffix,
            escape_xml(&marker),
            to_twips(lvl.indent),
            to_twips(lvl.hanging),
        ));
    }
    format!(
        r#"<w:abstractNum w:abstractNumId="{}">{levels}</w:abstractNum>"#,
        def.id
    )
}

fn serialize_level_rpr(format: &CharFormat) -> String {
    let mut inner = String::new();
    if let Some(ref family) = format.font_family {
        inner.push_str(&format!(
            r#"<w:rFonts w:ascii="{0}" w:hAnsi="{0}"/>"#,
            escape_xml(family)
        ));
    }
    if format.bold == Some(true) {
        inner.push_str("<w:b/>");
    }
    if format.italic == Some(true) {
        inner.push_str("<w:i/>");
    }
    if let Some(size) = format.font_size {
        let half_points = (size * 2.0).round() as i32;
        inner.push_str(&format!(r#"<w:sz w:val="{half_points}"/>"#));
    }
    if let Some(color) = format.color {
        inner.push_str(&format!(
            r#"<w:color w:val="{:02X}{:02X}{:02X}"/>"#,
            color.r, color.g, color.b
        ));
    }
    if inner.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{inner}</w:rPr>")
    }
}

fn serialize_num(num_id: u32, abstract_id: u32) -> String {
    format!(
        r#"<w:num w:numId="{num_id}"><w:abstractNumId w:val="{abstract_id}"/></w:num>"#
    )
}

fn num_fmt(format: ListMarkerFormat) -> &'static str {
    match format {
        ListMarkerFormat::Bullet => "bullet",
        ListMarkerFormat::Decimal => "decimal",
        ListMarkerFormat::LowerAlpha => "lowerLetter",
        ListMarkerFormat::UpperAlpha => "upperLetter",
        ListMarkerFormat::LowerRoman => "lowerRoman",
        ListMarkerFormat::UpperRoman => "upperRoman",
    }
}

fn default_lvl_text(format: ListMarkerFormat) -> &'static str {
    match format {
        ListMarkerFormat::Bullet => "•",
        ListMarkerFormat::Decimal => "%1.",
        ListMarkerFormat::LowerAlpha => "%1.",
        ListMarkerFormat::UpperAlpha => "%1.",
        ListMarkerFormat::LowerRoman => "%1.",
        ListMarkerFormat::UpperRoman => "%1.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::NumberingCatalog;

    #[test]
    fn exports_default_bullet_and_numbered_definitions() {
        let xml = serialize_numbering_xml(&NumberingCatalog::with_defaults());
        assert!(xml.contains("<w:abstractNum w:abstractNumId=\"1\""));
        assert!(xml.contains("<w:num w:numId=\"2\""));
        assert!(xml.contains(r#"w:numFmt w:val="decimal""#));
    }
}

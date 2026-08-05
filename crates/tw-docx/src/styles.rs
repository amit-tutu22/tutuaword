use tw_model::{
    Alignment, CharFormat, Color, LineSpacing, NumberingRef, ParaFormat, SectionFormat, StyleId,
    CharacterStyle, DocumentTheme, ListLevel, ListMarkerFormat, ListSuffix,
    NumberingCatalog, NumberingDefinition, ParagraphStyle, StyleSheet,
};

use crate::xml_util::{
    half_points_to_points, read_attr_value, read_numeric_attr, read_own_attr, split_elements,
    twips_to_points,
};

pub fn parse_styles_xml(xml: &str) -> StyleSheet {
    let mut sheet = StyleSheet::with_defaults();

    for chunk in split_elements(xml, "w:style") {
        let style_type = read_own_attr(chunk, "w:type").unwrap_or_default().to_string();
        let style_id_str = read_own_attr(chunk, "w:styleId").unwrap_or_default().to_string();
        let name = read_tag_text_in(chunk, "w:name", "w:val").unwrap_or_else(|| style_id_str.clone());
        if style_id_str.is_empty() {
            continue;
        }
        let id = StyleId::new();
        sheet.ooxml_style_ids.insert(style_id_str.clone(), id);

        let based_on = read_tag_text_in(chunk, "w:basedOn", "w:val")
            .and_then(|s| sheet.ooxml_style_ids.get(&s).copied());

        match style_type.as_str() {
            "paragraph" => {
                let para_format = parse_para_properties(chunk);
                let char_format = parse_char_properties(chunk);
                sheet.paragraph_styles.insert(
                    id,
                    ParagraphStyle {
                        id,
                        name,
                        based_on,
                        char_format,
                        para_format,
                        next_style: None,
                    },
                );
            }
            "character" => {
                let char_format = parse_char_properties(chunk);
                sheet.character_styles.insert(
                    id,
                    CharacterStyle {
                        id,
                        name,
                        based_on,
                        char_format,
                    },
                );
            }
            "table" | "numbering" => {}
            _ => {}
        }
    }

    if let Some(doc_defaults) = chunk_after(xml, "<w:docDefaults") {
        sheet.defaults.char_format = parse_char_properties(doc_defaults);
    }

    sheet
}

pub fn parse_numbering_xml(xml: &str) -> NumberingCatalog {
    let mut catalog = NumberingCatalog::with_defaults();
    let mut abstract_defs: std::collections::HashMap<u32, NumberingDefinition> =
        std::collections::HashMap::new();

    for chunk in split_elements(xml, "w:abstractNum") {
        let abs_id = read_own_attr(chunk, "w:abstractNumId")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let mut levels = Vec::new();
        for lvl in split_elements(chunk, "w:lvl") {
            let level = read_own_attr(lvl, "w:ilvl")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let num_fmt = read_tag_text_in(lvl, "w:numFmt", "w:val").unwrap_or_default();
            let format = match num_fmt.as_str() {
                "decimal" => ListMarkerFormat::Decimal,
                "lowerLetter" => ListMarkerFormat::LowerAlpha,
                "upperLetter" => ListMarkerFormat::UpperAlpha,
                "lowerRoman" => ListMarkerFormat::LowerRoman,
                "upperRoman" => ListMarkerFormat::UpperRoman,
                _ => ListMarkerFormat::Bullet,
            };
            let indent = read_numeric_attr(lvl, "w:ind", "w:left")
                .map(twips_to_points)
                .unwrap_or(36.0 * (level as f32 + 1.0));
            let hanging = read_numeric_attr(lvl, "w:ind", "w:hanging")
                .map(twips_to_points)
                .unwrap_or(18.0);
            levels.push(ListLevel {
                level,
                format,
                indent,
                hanging,
                suffix: ListSuffix::Tab,
            });
        }
        abstract_defs.insert(
            abs_id,
            NumberingDefinition {
                id: abs_id,
                name: format!("List {abs_id}"),
                levels,
            },
        );
    }

    for chunk in split_elements(xml, "w:num") {
        let num_id = read_own_attr(chunk, "w:numId")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        if let Some(abs_id) = read_tag_text_in(chunk, "w:abstractNumId", "w:val")
            .and_then(|v| v.parse().ok())
        {
            if let Some(def) = abstract_defs.get(&abs_id) {
                catalog.definitions.insert(
                    num_id,
                    NumberingDefinition {
                        id: num_id,
                        name: def.name.clone(),
                        levels: def.levels.clone(),
                    },
                );
            }
        }
    }

    catalog
}

pub fn parse_theme_xml(xml: &str) -> DocumentTheme {
    let mut theme = DocumentTheme::default();
    if let Some(color) = read_tag_text_in(xml, "a:dk1", "val").or_else(|| {
        xml.find("a:dk1")
            .and_then(|_| read_attr_value(xml, "a:srgbClr", "val"))
    }) {
        if color.len() == 6 {
            if let Ok(v) = u8::from_str_radix(&color[0..2], 16) {
                theme.text1.r = v;
            }
            if let Ok(v) = u8::from_str_radix(&color[2..4], 16) {
                theme.text1.g = v;
            }
            if let Ok(v) = u8::from_str_radix(&color[4..6], 16) {
                theme.text1.b = v;
            }
        }
    }
    if let Some(font) = read_attr_value(xml, "a:latin", "typeface") {
        theme.major_font = font;
    }
    theme
}

pub fn parse_para_properties(xml: &str) -> ParaFormat {
    let mut format = ParaFormat::default();
    if let Some(jc) = read_tag_text_in(xml, "w:jc", "w:val") {
        format.alignment = Some(parse_alignment(&jc));
    }
    if let Some(before) = read_numeric_attr(xml, "w:spacing", "w:before") {
        format.space_before = Some(twips_to_points(before));
    }
    if let Some(after) = read_numeric_attr(xml, "w:spacing", "w:after") {
        format.space_after = Some(twips_to_points(after));
    }
    if let Some(line) = read_numeric_attr(xml, "w:spacing", "w:line") {
        format.line_spacing = Some(LineSpacing::Multiple(line / 240.0));
    }
    if let Some(left) = read_numeric_attr(xml, "w:ind", "w:left") {
        format.indent_left = Some(twips_to_points(left));
    }
    if let Some(first) = read_numeric_attr(xml, "w:ind", "w:firstLine") {
        format.indent_first_line = Some(twips_to_points(first));
    }
    // A hanging indent is a negative first-line indent; the two are exclusive.
    if let Some(hanging) = read_numeric_attr(xml, "w:ind", "w:hanging") {
        format.indent_first_line = Some(-twips_to_points(hanging));
    }
    if xml.contains("<w:pageBreakBefore") && !xml.contains("w:val=\"0\"") {
        format.page_break_before = Some(true);
    }
    if let Some(num_id) = read_tag_text_in(xml, "w:numId", "w:val").and_then(|v| v.parse().ok()) {
        let level = read_tag_text_in(xml, "w:ilvl", "w:val")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        format.numbering = Some(NumberingRef { numbering_id: num_id, level });
    }
    format
}

pub fn parse_char_properties(xml: &str) -> CharFormat {
    let mut format = CharFormat::default();
    if xml.contains("<w:b") && !xml.contains("w:val=\"0\"") {
        format.bold = Some(true);
    }
    if xml.contains("<w:i") && !xml.contains("w:val=\"0\"") {
        format.italic = Some(true);
    }
    if xml.contains("<w:u ") || xml.contains("<w:u/>") {
        format.underline = Some(tw_model::UnderlineStyle::Single);
    }
    if xml.contains("<w:strike") && !xml.contains("w:val=\"0\"") {
        format.strikethrough = Some(true);
    }
    if let Some(sz) = read_numeric_attr(xml, "w:sz", "w:val") {
        format.font_size = Some(half_points_to_points(sz));
    }
    if let Some(font) = read_attr_value(xml, "w:rFonts", "w:ascii")
        .or_else(|| read_attr_value(xml, "w:rFonts", "w:hAnsi"))
    {
        format.font_family = Some(font);
    }
    if let Some(color) = read_tag_text_in(xml, "w:color", "w:val") {
        format.color = parse_color(&color);
    }
    if let Some(highlight) = read_tag_text_in(xml, "w:highlight", "w:val") {
        format.highlight = parse_highlight(&highlight);
    }
    format
}

pub fn parse_section_properties(xml: &str) -> SectionFormat {
    let mut format = SectionFormat::default();
    if let Some(w) = read_numeric_attr(xml, "w:pgSz", "w:w") {
        format.page_width = twips_to_points(w);
    }
    if let Some(h) = read_numeric_attr(xml, "w:pgSz", "w:h") {
        format.page_height = twips_to_points(h);
    }
    if let Some(v) = read_numeric_attr(xml, "w:pgMar", "w:top") {
        format.margin_top = twips_to_points(v);
    }
    if let Some(v) = read_numeric_attr(xml, "w:pgMar", "w:bottom") {
        format.margin_bottom = twips_to_points(v);
    }
    if let Some(v) = read_numeric_attr(xml, "w:pgMar", "w:left") {
        format.margin_left = twips_to_points(v);
    }
    if let Some(v) = read_numeric_attr(xml, "w:pgMar", "w:right") {
        format.margin_right = twips_to_points(v);
    }
    format
}

fn parse_alignment(value: &str) -> Alignment {
    match value {
        "center" => Alignment::Center,
        "right" => Alignment::Right,
        "both" | "distribute" => Alignment::Justify,
        _ => Alignment::Left,
    }
}

fn parse_color(value: &str) -> Option<Color> {
    if value == "auto" || value.len() != 6 {
        return None;
    }
    Some(Color {
        r: u8::from_str_radix(&value[0..2], 16).ok()?,
        g: u8::from_str_radix(&value[2..4], 16).ok()?,
        b: u8::from_str_radix(&value[4..6], 16).ok()?,
        a: 255,
    })
}

fn parse_highlight(value: &str) -> Option<Color> {
    match value {
        "yellow" => Some(Color { r: 255, g: 255, b: 0, a: 255 }),
        "green" => Some(Color { r: 0, g: 255, b: 0, a: 255 }),
        "cyan" => Some(Color { r: 0, g: 255, b: 255, a: 255 }),
        _ => None,
    }
}

fn read_tag_text_in(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = xml.find(&open)?;
    read_attr_value(&xml[start..], tag, attr)
}

fn chunk_after<'a>(xml: &'a str, marker: &str) -> Option<&'a str> {
    let start = xml.find(marker)?;
    Some(&xml[start..])
}

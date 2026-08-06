use tw_model::{
    Alignment, CharFormat, Color, LineSpacing, NumberingRef, ParaFormat, SectionFormat, StyleId,
    CharacterStyle, DocumentTheme, ListLevel, ListMarkerFormat, ListSuffix, TabAlignment, TabStop,
    NumberingCatalog, NumberingDefinition, ParagraphStyle, StyleSheet, TableStyle, BorderSpec,
};

use crate::xml_util::{
    half_points_to_points, read_attr_value, read_int_attr, read_numeric_attr, read_own_attr,
    read_toggle, split_elements, twips_to_points,
};

/// Marks a `font_family` that names a theme slot rather than a real family.
pub const THEME_FONT_PREFIX: &str = "+";

/// Substitutes theme font references for the families the theme names. Runs
/// after the whole document is parsed, since `styles.xml` is read before
/// `theme1.xml`.
pub fn resolve_theme_fonts(doc: &mut tw_model::Document) {
    let theme = doc.settings.theme.clone();
    let substitute = |format: &mut CharFormat| {
        let Some(family) = format.font_family.as_deref() else {
            return;
        };
        let Some(slot) = family.strip_prefix(THEME_FONT_PREFIX) else {
            return;
        };
        format.font_family = Some(if slot.starts_with("major") {
            theme.major_font.clone()
        } else {
            theme.minor_font.clone()
        });
    };

    substitute(&mut doc.styles.defaults.char_format);
    for style in doc.styles.paragraph_styles.values_mut() {
        substitute(&mut style.char_format);
    }
    for style in doc.styles.character_styles.values_mut() {
        substitute(&mut style.char_format);
    }
    for section in &mut doc.sections {
        for block in &mut section.blocks {
            visit_block_runs(block, &substitute);
        }
    }
}

fn visit_block_runs(block: &mut tw_model::Block, substitute: &impl Fn(&mut CharFormat)) {
    match block {
        tw_model::Block::Paragraph(para) => {
            for run in &mut para.runs {
                substitute(&mut run.format);
            }
        }
        tw_model::Block::Table(table) => {
            for row in &mut table.rows {
                for cell in &mut row.cells {
                    for block in &mut cell.blocks {
                        visit_block_runs(block, substitute);
                    }
                }
            }
        }
        tw_model::Block::ImageBlock(_) => {}
    }
}

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
            "table" => {
                let border = parse_tbl_borders(chunk);
                sheet.ooxml_table_style_ids.insert(style_id_str.clone(), id);
                sheet.table_styles.insert(
                    id,
                    TableStyle {
                        id,
                        name,
                        border,
                    },
                );
            }
            "numbering" => {}
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
            let suffix = match read_tag_text_in(lvl, "w:suff", "w:val").as_deref() {
                Some("space") => ListSuffix::Space,
                Some("nothing") => ListSuffix::Nothing,
                _ => ListSuffix::Tab,
            };
            let marker_text = read_tag_text_in(lvl, "w:lvlText", "w:val");
            let start = read_tag_text_in(lvl, "w:start", "w:val")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let char_format = parse_char_properties(lvl);
            levels.push(ListLevel {
                level,
                format,
                indent,
                hanging,
                suffix,
                marker_text,
                start,
                char_format,
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
    if let Some(font) = theme_font_latin(xml, "a:majorFont") {
        theme.major_font = font;
    }
    if let Some(font) = theme_font_latin(xml, "a:minorFont") {
        theme.minor_font = font;
    }
    theme
}

fn theme_font_latin(xml: &str, marker: &str) -> Option<String> {
    let start = xml.find(&format!("<{marker}"))?;
    let chunk = &xml[start..];
    let end = chunk.find(&format!("</{marker}>")).unwrap_or(chunk.len());
    read_attr_value(&chunk[..end], "a:latin", "typeface")
}

fn parse_tbl_borders(style_xml: &str) -> Option<BorderSpec> {
    if !style_xml.contains("w:tblBorders") {
        return None;
    }
    let width = ["w:top", "w:left", "w:bottom", "w:right"]
        .iter()
        .filter_map(|edge| read_int_attr(style_xml, edge, "w:sz"))
        .filter(|v| *v > 0)
        .map(|v| v as f32 / 8.0)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))?;
    let color = ["w:top", "w:left", "w:bottom", "w:right"]
        .iter()
        .find_map(|edge| {
            read_attr_value(style_xml, edge, "w:color").and_then(|v| parse_fill_color(&v))
        })
        .unwrap_or(Color::BLACK);
    Some(BorderSpec { width, color })
}

fn parse_fill_color(value: &str) -> Option<Color> {
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
        format.line_spacing = Some(parse_line_spacing(line, xml));
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
    format.tab_stops = parse_tab_stops(xml);
    if read_toggle(xml, "w:keepNext").unwrap_or(false) {
        format.keep_with_next = Some(true);
    }
    if read_toggle(xml, "w:keepLines").unwrap_or(false) {
        format.keep_together = Some(true);
    }
    if xml.contains("<w:widowControl") {
        format.widow_orphan_control = read_toggle(xml, "w:widowControl");
    }
    format
}

/// Default character formatting declared on a paragraph via `w:pPr/w:rPr`.
pub fn parse_para_default_char_format(ppr_xml: &str) -> CharFormat {
    split_elements(ppr_xml, "w:rPr")
        .into_iter()
        .next()
        .map(parse_char_properties)
        .unwrap_or_default()
}

fn parse_tab_stops(xml: &str) -> Vec<TabStop> {
    let mut stops = Vec::new();
    for tab in split_elements(xml, "w:tab") {
        let Some(pos) = read_own_attr(tab, "w:pos")
            .and_then(|v| v.parse::<f32>().ok())
            .map(twips_to_points)
        else {
            continue;
        };
        let alignment = match read_own_attr(tab, "w:val").as_deref() {
            Some("center") => TabAlignment::Center,
            Some("right") => TabAlignment::Right,
            Some("decimal") => TabAlignment::Decimal,
            Some("bar") => TabAlignment::Bar,
            _ => TabAlignment::Left,
        };
        stops.push(TabStop { position: pos, alignment });
    }
    stops.sort_by(|a, b| a.position.total_cmp(&b.position));
    stops
}

pub fn parse_char_properties(xml: &str) -> CharFormat {
    // Character properties live in `w:rPr`. Reading the whole chunk would pick
    // up paragraph properties too, so a `<w:ind>` would read as italic.
    let xml = split_elements(xml, "w:rPr")
        .into_iter()
        .next()
        .unwrap_or(xml);

    let mut format = CharFormat::default();
    format.bold = read_toggle(xml, "w:b");
    format.italic = read_toggle(xml, "w:i");
    if read_toggle(xml, "w:strike").unwrap_or(false) {
        format.strikethrough = Some(true);
    }
    if let Some(underline) = read_attr_value(xml, "w:u", "w:val") {
        if underline != "none" {
            format.underline = Some(tw_model::UnderlineStyle::Single);
        }
    } else if split_elements(xml, "w:u").first().is_some() {
        format.underline = Some(tw_model::UnderlineStyle::Single);
    }
    if let Some(sz) = read_numeric_attr(xml, "w:sz", "w:val") {
        format.font_size = Some(half_points_to_points(sz));
    }
    if let Some(font) = read_attr_value(xml, "w:rFonts", "w:ascii")
        .or_else(|| read_attr_value(xml, "w:rFonts", "w:hAnsi"))
    {
        format.font_family = Some(font);
    } else if let Some(theme) = read_attr_value(xml, "w:rFonts", "w:asciiTheme")
        .or_else(|| read_attr_value(xml, "w:rFonts", "w:hAnsiTheme"))
    {
        // The theme part may not be parsed yet, so record the reference and let
        // `resolve_theme_fonts` substitute the real family afterwards.
        format.font_family = Some(format!("{THEME_FONT_PREFIX}{theme}"));
    }
    if let Some(color) = read_tag_text_in(xml, "w:color", "w:val") {
        format.color = parse_color(&color);
    }
    if let Some(highlight) = read_tag_text_in(xml, "w:highlight", "w:val") {
        format.highlight = parse_highlight(&highlight);
    }
    if let Some(align) = read_tag_text_in(xml, "w:vertAlign", "w:val") {
        match align.as_str() {
            "superscript" => {
                format.superscript = Some(true);
                format.subscript = Some(false);
            }
            "subscript" => {
                format.subscript = Some(true);
                format.superscript = Some(false);
            }
            "baseline" => {
                format.superscript = Some(false);
                format.subscript = Some(false);
            }
            _ => {}
        }
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

/// OOXML `w:line` meaning depends on `w:lineRule`: multiples of a line under
/// `auto`, absolute twips under `exact` / `atLeast`.
fn parse_line_spacing(line: f32, xml: &str) -> LineSpacing {
    let rule = read_attr_value(xml, "w:spacing", "w:lineRule").unwrap_or_default();
    match rule.as_str() {
        "exact" => LineSpacing::Exactly(twips_to_points(line)),
        "atLeast" => LineSpacing::AtLeast(twips_to_points(line)),
        _ if (line - 240.0).abs() < f32::EPSILON => LineSpacing::Single,
        _ if (line - 480.0).abs() < f32::EPSILON => LineSpacing::Double,
        _ => LineSpacing::Multiple(line / 240.0),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // Run properties must be read as elements, not substrings: `<w:i` also
    // appears in `<w:ind>` and `<w:iCs>`, `<w:b` in `<w:bCs>` and `<w:bdr>`.

    #[test]
    fn an_indent_does_not_make_text_italic() {
        let xml = r#"<w:pPr><w:ind w:left="720"/></w:pPr><w:rPr><w:sz w:val="24"/></w:rPr>"#;

        assert_eq!(parse_char_properties(xml).italic, None);
    }

    #[test]
    fn complex_script_bold_alone_does_not_bold_the_run() {
        let xml = r#"<w:rPr><w:bCs/><w:sz w:val="24"/></w:rPr>"#;

        assert_eq!(parse_char_properties(xml).bold, None);
    }

    #[test]
    fn a_bold_toggle_is_read() {
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:b/></w:rPr>"#).bold,
            Some(true)
        );
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:b w:val="1"/></w:rPr>"#).bold,
            Some(true)
        );
    }

    #[test]
    fn a_bold_toggle_can_be_switched_off() {
        // A style may enable bold and the run turn it back off.
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:b w:val="0"/></w:rPr>"#).bold,
            Some(false)
        );
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:b w:val="false"/></w:rPr>"#).bold,
            Some(false)
        );
    }

    #[test]
    fn an_unrelated_zero_valued_property_does_not_disable_bold() {
        let xml = r#"<w:rPr><w:b/><w:spacing w:val="0"/></w:rPr>"#;

        assert_eq!(parse_char_properties(xml).bold, Some(true));
    }

    #[test]
    fn italic_is_read_from_its_own_element() {
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:i/></w:rPr>"#).italic,
            Some(true)
        );
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:iCs/></w:rPr>"#).italic,
            None
        );
    }

    #[test]
    fn paragraph_properties_do_not_leak_into_the_run_format() {
        // A style chunk holds both; only w:rPr describes the characters.
        let xml = r#"<w:pPr><w:ind w:left="720"/><w:jc w:val="center"/></w:pPr>
                     <w:rPr><w:rFonts w:ascii="Georgia"/></w:rPr>"#;
        let format = parse_char_properties(xml);

        assert_eq!(format.font_family.as_deref(), Some("Georgia"));
        assert_eq!(format.italic, None);
        assert_eq!(format.bold, None);
    }

    #[test]
    fn an_underline_of_none_is_not_an_underline() {
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:u w:val="none"/></w:rPr>"#).underline,
            None
        );
        assert!(
            parse_char_properties(r#"<w:rPr><w:u w:val="single"/></w:rPr>"#)
                .underline
                .is_some()
        );
    }

    #[test]
    fn a_theme_font_is_recorded_as_a_reference() {
        let xml = r#"<w:rPr><w:rFonts w:asciiTheme="minorHAnsi"/></w:rPr>"#;

        assert_eq!(
            parse_char_properties(xml).font_family.as_deref(),
            Some("+minorHAnsi")
        );
    }

    #[test]
    fn an_explicit_family_wins_over_a_theme_reference() {
        let xml = r#"<w:rPr><w:rFonts w:ascii="Georgia" w:hAnsiTheme="minorHAnsi"/></w:rPr>"#;

        assert_eq!(
            parse_char_properties(xml).font_family.as_deref(),
            Some("Georgia")
        );
    }

    #[test]
    fn theme_references_resolve_to_the_documents_theme() {
        let mut doc = tw_model::Document::with_paragraph("Hi");
        doc.settings.theme.minor_font = "Aptos".into();
        doc.settings.theme.major_font = "Aptos Display".into();
        doc.styles.defaults.char_format.font_family = Some("+minorHAnsi".into());
        doc.sections[0].blocks = vec![tw_model::Block::Paragraph({
            let mut para = tw_model::Paragraph::with_text("Hi");
            para.runs[0].format.font_family = Some("+majorHAnsi".into());
            para
        })];

        resolve_theme_fonts(&mut doc);

        assert_eq!(
            doc.styles.defaults.char_format.font_family.as_deref(),
            Some("Aptos")
        );
        let tw_model::Block::Paragraph(para) = &doc.sections[0].blocks[0] else {
            panic!("expected a paragraph");
        };
        assert_eq!(para.runs[0].format.font_family.as_deref(), Some("Aptos Display"));
    }

    #[test]
    fn a_real_family_is_left_alone_by_theme_resolution() {
        let mut doc = tw_model::Document::with_paragraph("Hi");
        doc.styles.defaults.char_format.font_family = Some("Times New Roman".into());

        resolve_theme_fonts(&mut doc);

        assert_eq!(
            doc.styles.defaults.char_format.font_family.as_deref(),
            Some("Times New Roman")
        );
    }

    #[test]
    fn auto_line_spacing_is_a_multiple_of_a_line() {
        let xml = r#"<w:spacing w:line="360" w:lineRule="auto"/>"#;
        assert_eq!(
            parse_para_properties(xml).line_spacing,
            Some(LineSpacing::Multiple(1.5))
        );
    }

    #[test]
    fn exact_line_spacing_is_in_points_not_multiples() {
        // 480 twips = 24 pt — must not become Multiple(2.0) which would balloon layout.
        let xml = r#"<w:spacing w:line="480" w:lineRule="exact"/>"#;
        assert_eq!(
            parse_para_properties(xml).line_spacing,
            Some(LineSpacing::Exactly(24.0))
        );
    }

    #[test]
    fn at_least_line_spacing_is_in_points() {
        let xml = r#"<w:spacing w:line="240" w:lineRule="atLeast"/>"#;
        assert_eq!(
            parse_para_properties(xml).line_spacing,
            Some(LineSpacing::AtLeast(12.0))
        );
    }

    #[test]
    fn explicit_tab_stops_are_parsed_in_twips() {
        let xml = r#"<w:tabs><w:tab w:val="right" w:pos="2880"/></w:tabs>"#;
        let stops = parse_para_properties(xml).tab_stops;
        assert_eq!(stops.len(), 1);
        assert!((stops[0].position - 144.0).abs() < 0.01);
        assert_eq!(stops[0].alignment, TabAlignment::Right);
    }

    #[test]
    fn superscript_and_subscript_import() {
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:vertAlign w:val="superscript"/></w:rPr>"#)
                .superscript,
            Some(true)
        );
        assert_eq!(
            parse_char_properties(r#"<w:rPr><w:vertAlign w:val="subscript"/></w:rPr>"#).subscript,
            Some(true)
        );
    }
}

//! Shared XML parsing helpers for OOXML parts.

pub fn read_attr_value(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("<{tag}");
    let start = xml.find(&pattern)?;
    let fragment = &xml[start..];
    let attr_pattern = format!("{attr}=\"");
    let attr_start = fragment.find(&attr_pattern)? + attr_pattern.len();
    let rest = &fragment[attr_start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn read_numeric_attr(xml: &str, tag: &str, attr: &str) -> Option<f32> {
    read_attr_value(xml, tag, attr).and_then(|v| v.parse().ok())
}

pub fn read_int_attr(xml: &str, tag: &str, attr: &str) -> Option<i32> {
    read_attr_value(xml, tag, attr).and_then(|v| v.parse().ok())
}

/// Bodies of the `<tag ...> ... </tag>` elements found in `xml`, each slice
/// running from just after the tag name to just before its closing tag.
///
/// Unlike a plain `split("<w:tc")`, this ignores tags that merely share a name
/// prefix, so `w:tc` does not match `w:tcPr` and `w:p` does not match `w:pPr`.
/// Nested elements of the same name are not supported.
pub fn split_elements<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = xml;

    while let Some(i) = rest.find(&open) {
        let after = &rest[i + open.len()..];
        if !is_name_boundary(after) {
            rest = after;
            continue;
        }
        let Some(tag_end) = after.find('>') else {
            break;
        };
        if after[..tag_end].ends_with('/') {
            out.push(&after[..tag_end]);
            rest = &after[tag_end + 1..];
        } else {
            let end = after.find(&close).unwrap_or(after.len());
            out.push(&after[..end]);
            rest = &after[end..];
        }
    }
    out
}

/// Reads an attribute from an element's own start tag, given a body slice
/// produced by [`split_elements`]. Descendant attributes are ignored.
pub fn read_own_attr<'a>(element_body: &'a str, attr: &str) -> Option<&'a str> {
    let head = &element_body[..element_body.find('>').unwrap_or(element_body.len())];
    let pattern = format!("{attr}=\"");
    let start = head.find(&pattern)? + pattern.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// Reads an OOXML on/off property such as `<w:b/>`, `<w:b w:val="0"/>`, or
/// `<w:i w:val="true"/>`. Returns `None` when the element is absent.
///
/// Matching the tag name exactly matters here: a substring test for `<w:i`
/// also hits `<w:ind>` and `<w:iCs>`, and `<w:b` hits `<w:bCs>` and `<w:bdr>`.
pub fn read_toggle(xml: &str, tag: &str) -> Option<bool> {
    let element = split_elements(xml, tag).into_iter().next()?;
    Some(match read_own_attr(element, "w:val") {
        Some(value) => !matches!(value, "0" | "false" | "off"),
        None => true,
    })
}

fn is_name_boundary(after_tag_name: &str) -> bool {
    matches!(
        after_tag_name.as_bytes().first(),
        Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
    )
}

pub fn skip_tag(xml: &str) -> &str {
    xml.find('>').map(|i| &xml[i + 1..]).unwrap_or("")
}

pub fn advance_past_tag<'a>(xml: &'a str, tag: &str) -> &'a str {
    let close = format!("</{tag}>");
    xml.find(&close)
        .map(|i| &xml[i + close.len()..])
        .unwrap_or("")
}

/// The next run-level child inside a paragraph body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunLevelTag {
    Run,
    Insert,
    Delete,
    Hyperlink,
    FieldSimple,
    BookmarkStart,
    OfficeMath,
}

/// Finds the earliest run-level element at the current parse position.
pub fn next_run_level_tag(xml: &str) -> Option<(usize, RunLevelTag)> {
    let candidates = [
        xml.find("<w:r").map(|i| (i, RunLevelTag::Run)),
        xml.find("<w:ins").map(|i| (i, RunLevelTag::Insert)),
        xml.find("<w:del").map(|i| (i, RunLevelTag::Delete)),
        xml.find("<w:hyperlink").map(|i| (i, RunLevelTag::Hyperlink)),
        xml.find("<w:fldSimple").map(|i| (i, RunLevelTag::FieldSimple)),
        xml.find("<w:bookmarkStart").map(|i| (i, RunLevelTag::BookmarkStart)),
        xml.find("<m:oMath").map(|i| (i, RunLevelTag::OfficeMath)),
    ]
    .into_iter()
    .flatten()
    .filter(|(i, tag)| {
        let after = &xml[*i..];
        let prefix = match tag {
            RunLevelTag::Run => "<w:r",
            RunLevelTag::Insert => "<w:ins",
            RunLevelTag::Delete => "<w:del",
            RunLevelTag::Hyperlink => "<w:hyperlink",
            RunLevelTag::FieldSimple => "<w:fldSimple",
            RunLevelTag::BookmarkStart => "<w:bookmarkStart",
            RunLevelTag::OfficeMath => "<m:oMath",
        };
        after
            .get(prefix.len()..)
            .map(|rest| is_name_boundary(rest))
            .unwrap_or(false)
    })
    .min_by_key(|(i, _)| *i);
    candidates
}

/// Returns the whole element starting at `xml` and the remainder after it.
pub fn take_element<'a>(xml: &'a str, tag: &str) -> Option<(&'a str, &'a str)> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let after = &xml[start..];
    if !is_name_boundary(&after[open.len()..]) {
        return None;
    }
    let head_end = after.find('>')?;
    if after[..head_end].ends_with('/') {
        let end = head_end + 1;
        return Some((&after[..end], &after[end..]));
    }

    let mut depth = 0usize;
    let mut scan = 0usize;
    while scan < after.len() {
        // Only inspect tag starts; byte-wise `scan += 1` breaks on multi-byte
        // UTF-8 inside `<w:t>` (e.g. U+2002 en space).
        let rel = after[scan..].find('<').unwrap_or(after.len() - scan);
        if rel > 0 {
            scan += rel;
        }
        if scan >= after.len() {
            break;
        }
        if after[scan..].starts_with(&open) && is_name_boundary(&after[scan + open.len()..]) {
            depth += 1;
            scan += open.len();
            continue;
        }
        if after[scan..].starts_with(&close) {
            depth = depth.saturating_sub(1);
            scan += close.len();
            if depth == 0 {
                return Some((&after[..scan], &after[scan..]));
            }
            continue;
        }
        scan += 1;
    }
    None
}

/// Reads an attribute from an element's opening tag given the full element slice.
pub fn read_attr_on_element(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = xml.find(&open)?;
    let after = &xml[start..];
    let head_end = after.find('>')?;
    read_attr_value(&after[..=head_end], tag, attr)
}

pub fn read_tag_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let start = xml.find(&open)?;
    let after_open = &xml[start..];
    let gt = after_open.find('>')?;
    let content_start = start + gt + 1;
    let rel_end = xml[content_start..].find(&close)?;
    Some(xml[content_start..content_start + rel_end].to_string())
}

pub fn decode_xml_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

pub fn extract_body_xml(document_xml: &str) -> &str {
    if let Some(start) = document_xml.find("<w:body") {
        let after = &document_xml[start..];
        if let Some(gt) = after.find('>') {
            let body = &after[gt + 1..];
            if let Some(end) = body.find("</w:body>") {
                return &body[..end];
            }
        }
    }
    document_xml
}

pub fn extract_embedded_sect_pr(paragraph_xml: &str) -> Option<&str> {
    let ppr_start = paragraph_xml.find("<w:pPr")?;
    let ppr_end = paragraph_xml[ppr_start..]
        .find("</w:pPr>")
        .map(|i| i + ppr_start)
        .unwrap_or(paragraph_xml.len());
    let ppr = &paragraph_xml[ppr_start..ppr_end];
    if !ppr.contains("<w:sectPr") {
        return None;
    }
    let sect_start = ppr.find("<w:sectPr")?;
    let sect_rest = &ppr[sect_start..];
    let sect_end = sect_rest.find("</w:sectPr>")? + "</w:sectPr>".len();
    Some(&sect_rest[..sect_end])
}

/// Iterate top-level block tags (`w:p`, `w:tbl`, `w:sectPr`) in document order.
pub fn iter_body_blocks(body_xml: &str) -> Vec<(&str, BlockKind)> {
    let mut blocks = Vec::new();
    let mut rest = body_xml;
    while !rest.is_empty() {
        let next_p = rest.find("<w:p");
        let next_tbl = rest.find("<w:tbl");
        let next_sect = rest.find("<w:sectPr");
        let next_math = rest.find("<m:oMathPara");

        let mut candidates = [
            next_p.map(|i| (i, BlockKind::Paragraph)),
            next_tbl.map(|i| (i, BlockKind::Table)),
            next_sect.map(|i| (i, BlockKind::SectionProps)),
            next_math
                .filter(|i| {
                    rest[*i..]
                        .get("<m:oMathPara".len()..)
                        .map(is_name_boundary)
                        .unwrap_or(false)
                })
                .map(|i| (i, BlockKind::MathPara)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        if candidates.is_empty() {
            break;
        }
        candidates.sort_by_key(|(i, _)| *i);
        let (offset, kind) = candidates[0];
        rest = &rest[offset..];

        let close_tag = match kind {
            BlockKind::Paragraph => "</w:p>",
            BlockKind::Table => "</w:tbl>",
            BlockKind::SectionProps => "</w:sectPr>",
            BlockKind::MathPara => "</m:oMathPara>",
        };
        if let Some(end) = rest.find(close_tag) {
            let chunk = &rest[..end + close_tag.len()];
            blocks.push((chunk, kind));
            rest = &rest[end + close_tag.len()..];
        } else {
            break;
        }
    }
    blocks
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    Table,
    SectionProps,
    MathPara,
}

pub fn twips_to_points(twips: f32) -> f32 {
    twips / 20.0
}

pub fn half_points_to_points(half_pts: f32) -> f32 {
    half_pts / 2.0
}

pub fn extract_plain_text(xml: &str) -> String {
    extract_text(xml, true)
}

/// Text of a single run. Unlike [`extract_plain_text`] this leaves page breaks
/// out, because a page break is modelled as its own `RunContent` and folding it
/// in as a newline as well would count it twice.
pub fn extract_run_text(xml: &str) -> String {
    extract_text(xml, false)
}

fn extract_text(xml: &str, page_breaks_as_newlines: bool) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(tag_start) = rest.find('<') {
        if tag_start > 0 {
            rest = &rest[tag_start..];
        }
        if rest.starts_with("<w:tab") {
            out.push('\t');
            rest = skip_tag(rest);
        } else if rest.starts_with("<w:br") {
            let head = &rest[..rest.find('>').map(|i| i + 1).unwrap_or(rest.len())];
            let is_page = head.contains("\"page\"") || head.contains("'page'");
            if page_breaks_as_newlines || !is_page {
                out.push('\n');
            }
            rest = skip_tag(rest);
        } else if rest.starts_with("<w:t") || rest.starts_with("<w:delText") {
            let tag = if rest.starts_with("<w:delText") {
                "w:delText"
            } else {
                "w:t"
            };
            if let Some(text) = read_tag_text(rest, tag) {
                out.push_str(&decode_xml_entities(&text));
            }
            rest = advance_past_tag(rest, tag);
        } else {
            rest = &rest[1..];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_elements_ignores_tags_sharing_a_name_prefix() {
        let xml = "<w:tc><w:tcPr><w:gridSpan w:val=\"2\"/></w:tcPr><w:p>A</w:p></w:tc>\
                   <w:tc><w:p>B</w:p></w:tc>";
        let cells = split_elements(xml, "w:tc");
        assert_eq!(cells.len(), 2);
        assert!(cells[0].contains("gridSpan"));
        assert!(cells[1].contains("B"));
    }

    #[test]
    fn split_elements_handles_self_closing_tags() {
        let xml = "<w:gridCol w:w=\"2880\"/><w:gridCol w:w=\"1440\"/>";
        let cols = split_elements(xml, "w:gridCol");
        assert_eq!(cols.len(), 2);
        assert_eq!(read_own_attr(cols[0], "w:w"), Some("2880"));
        assert_eq!(read_own_attr(cols[1], "w:w"), Some("1440"));
    }

    #[test]
    fn split_elements_skips_paragraph_properties() {
        let xml = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:t>X</w:t></w:r></w:p>";
        assert_eq!(split_elements(xml, "w:p").len(), 1);
        assert_eq!(split_elements(xml, "w:r").len(), 1);
    }

    #[test]
    fn read_own_attr_ignores_descendant_attributes() {
        let body = " w:abstractNumId=\"3\"><w:lvl w:ilvl=\"0\"/>";
        assert_eq!(read_own_attr(body, "w:abstractNumId"), Some("3"));
        assert_eq!(read_own_attr(body, "w:ilvl"), None);
    }

    #[test]
    fn iter_body_blocks_preserves_order() {
        let body = "<w:p><w:r><w:t>A</w:t></w:r></w:p><w:tbl></w:tbl><w:p><w:r><w:t>B</w:t></w:r></w:p>";
        let blocks = iter_body_blocks(body);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].1, BlockKind::Paragraph);
        assert_eq!(blocks[1].1, BlockKind::Table);
        assert_eq!(blocks[2].1, BlockKind::Paragraph);
    }

    #[test]
    fn take_element_handles_multibyte_text_between_tags() {
        let xml = format!(r#"<w:r><w:t>Hi{}there</w:t></w:r>"#, '\u{2002}');
        let (element, rest) = take_element(&xml, "w:r").expect("element");
        assert!(element.contains('\u{2002}'));
        assert!(rest.is_empty());
    }

    #[test]
    fn iter_body_blocks_preserves_order_with_math_para() {
        let body = "<w:p><w:r><w:t>A</w:t></w:r></w:p>\
                    <m:oMathPara xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">\
                    <m:oMath><m:r><m:t>E=mc2</m:t></m:r></m:oMath></m:oMathPara>\
                    <w:tbl></w:tbl>";
        let blocks = iter_body_blocks(body);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].1, BlockKind::Paragraph);
        assert_eq!(blocks[1].1, BlockKind::MathPara);
        assert_eq!(blocks[2].1, BlockKind::Table);
    }

    #[test]
    fn next_run_level_tag_finds_omath_before_run() {
        let xml = "<m:oMath xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">\
                   <m:r><m:t>x</m:t></m:r></m:oMath><w:r><w:t>y</w:t></w:r>";
        let (_, tag) = next_run_level_tag(xml).expect("tag");
        assert_eq!(tag, RunLevelTag::OfficeMath);
    }
}

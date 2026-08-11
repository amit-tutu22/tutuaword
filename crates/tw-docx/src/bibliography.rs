//! `word/bibliography.xml` import/export (F16.S3).

use tw_model::BibliographySource;

pub const BIBLIOGRAPHY_PART: &str = "word/bibliography.xml";

pub fn serialize_bibliography_xml(sources: &[BibliographySource]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><b:Sources xmlns:b="http://schemas.openxmlformats.org/officeDocument/2006/bibliography">"#,
    );
    for source in sources {
        xml.push_str("<b:Source>");
        xml.push_str(&format!("<b:Tag>{}</b:Tag>", escape_xml(&source.key)));
        xml.push_str("<b:Author><b:Author><b:NameList><b:Person>");
        if let Some((last, first)) = split_author(&source.author) {
            xml.push_str(&format!("<b:Last>{}</b:Last>", escape_xml(last)));
            if !first.is_empty() {
                xml.push_str(&format!("<b:First>{}</b:First>", escape_xml(first)));
            }
        } else {
            xml.push_str(&format!("<b:Last>{}</b:Last>", escape_xml(&source.author)));
        }
        xml.push_str("</b:Person></b:NameList></b:Author></b:Author>");
        xml.push_str(&format!("<b:Title>{}</b:Title>", escape_xml(&source.title)));
        xml.push_str(&format!("<b:Year>{}</b:Year>", escape_xml(&source.year)));
        xml.push_str("</b:Source>");
    }
    xml.push_str("</b:Sources>");
    xml
}

pub fn parse_bibliography_xml(xml: &str) -> Vec<BibliographySource> {
    let mut sources = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<b:Source>") {
        let after = &rest[start + "<b:Source>".len()..];
        let end = after.find("</b:Source>").unwrap_or(after.len());
        let block = &after[..end];
        let key = read_tag(block, "b:Tag").unwrap_or_default();
        if key.is_empty() {
            rest = &after[end..];
            continue;
        }
        let author = read_author(block).unwrap_or_else(|| "Unknown".into());
        let title = read_tag(block, "b:Title").unwrap_or_else(|| "Untitled".into());
        let year = read_tag(block, "b:Year").unwrap_or_else(|| "n.d.".into());
        sources.push(BibliographySource::new(key, author, title, year));
        rest = &after[end..];
    }
    sources
}

pub fn parse_citation_key(instr: &str) -> Option<String> {
    let upper = instr.trim().to_ascii_uppercase();
    if !upper.starts_with("CITATION") {
        return None;
    }
    instr
        .split_whitespace()
        .nth(1)
        .map(|key| key.trim_matches('"').to_string())
        .filter(|k| !k.is_empty())
}

fn split_author(author: &str) -> Option<(&str, &str)> {
    let mut parts = author.splitn(2, ',');
    let last = parts.next()?.trim();
    let first = parts.next().unwrap_or("").trim();
    Some((last, first))
}

fn read_author(block: &str) -> Option<String> {
    let last = read_tag(block, "b:Last")?;
    let first = read_tag(block, "b:First").unwrap_or_default();
    if first.is_empty() {
        Some(last)
    } else {
        Some(format!("{last}, {first}"))
    }
}

fn read_tag(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&close)? + start;
    Some(unescape_xml(&block[start..end]))
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_bibliography_xml() {
        let sources = vec![BibliographySource::new(
            "Smith2020",
            "Smith, John",
            "Example Research",
            "2020",
        )];
        let xml = serialize_bibliography_xml(&sources);
        let parsed = parse_bibliography_xml(&xml);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].key, "Smith2020");
        assert_eq!(parsed[0].author, "Smith, John");
        assert_eq!(parsed[0].title, "Example Research");
        assert_eq!(parsed[0].year, "2020");
    }

    #[test]
    fn parse_citation_instruction() {
        assert_eq!(
            parse_citation_key(" CITATION Smith2020 \\l 1033 "),
            Some("Smith2020".into())
        );
    }
}

use tw_model::DocumentProperties;

pub fn parse_core_properties(xml: &str) -> DocumentProperties {
    DocumentProperties {
        title: read_element_text(xml, "dc:title"),
        author: read_element_text(xml, "dc:creator"),
        page_count: None,
    }
}

pub fn parse_app_properties(xml: &str) -> DocumentProperties {
    DocumentProperties {
        title: None,
        author: None,
        page_count: read_element_text(xml, "Pages").and_then(|v| v.parse().ok()),
    }
}

/// Returns true when Word edit protection is enforced as read-only.
pub fn parse_read_only_from_settings(xml: &str) -> bool {
    if !xml.contains("w:documentProtection") {
        return false;
    }
    let enforced = xml.contains("w:enforcement=\"1\"")
        || xml.contains("w:enforcement=\"true\"")
        || xml.contains("w:enforcement=\"on\"");
    if !enforced {
        return false;
    }
    xml.contains("w:edit=\"readOnly\"")
}

fn read_element_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    let text = decode_xml_entities(xml[start..end].trim());
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn decode_xml_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_core_title_and_author() {
        let xml = r#"<?xml version="1.0"?>
<cp:coreProperties xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>Quarterly Report</dc:title>
  <dc:creator>Jane Author</dc:creator>
</cp:coreProperties>"#;
        let props = parse_core_properties(xml);
        assert_eq!(props.title.as_deref(), Some("Quarterly Report"));
        assert_eq!(props.author.as_deref(), Some("Jane Author"));
    }

    #[test]
    fn parses_app_page_count() {
        let xml = r#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Pages>7</Pages>
</Properties>"#;
        let props = parse_app_properties(xml);
        assert_eq!(props.page_count, Some(7));
    }

    #[test]
    fn detects_read_only_protection() {
        let xml = r#"<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:documentProtection w:edit="readOnly" w:enforcement="1"/>
</w:settings>"#;
        assert!(parse_read_only_from_settings(xml));
    }
}

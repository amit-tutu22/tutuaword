//! Read-only OMML preview helpers (F14.S2) and minimal OMML builders (F14.S3).

/// OMML namespace used in generated and preserved math markup.
pub const OMML_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

/// Word-like equation preview background (light blue-gray).
pub const MATH_FRAME_ARGB: u32 = 0xFFE8EEF7;

/// Slight size bump for math preview glyphs vs body text.
pub const MATH_PREVIEW_SCALE: f32 = 1.08;

/// Extract visible text from OMML by concatenating all `m:t` elements in document order.
pub fn extract_omml_preview_text(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<m:t") {
        let after = &rest[start..];
        if !is_tag_boundary(after, "<m:t") {
            rest = &rest[start + 1..];
            continue;
        }
        let gt = after.find('>').unwrap_or(after.len());
        let content_start = gt + 1;
        let Some(close) = after[content_start..].find("</m:t>") else {
            break;
        };
        let text = &after[content_start..content_start + close];
        out.push_str(&decode_xml_entities(text));
        rest = &after[content_start + close..];
    }
    out
}

fn is_tag_boundary(after_open: &str, prefix: &str) -> bool {
    after_open
        .get(prefix.len()..)
        .and_then(|rest| rest.as_bytes().first())
        .is_some_and(|b| matches!(b, b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'))
}

fn decode_xml_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Escape plain text for insertion into `m:t`.
pub fn escape_omml_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// One `m:r`/`m:t` run.
pub fn omml_run(text: &str) -> String {
    format!(
        r#"<m:r><m:t xml:space="preserve">{}</m:t></m:r>"#,
        escape_omml_text(text)
    )
}

/// Inline `<m:oMath>` wrapper around one or more OMML fragments.
pub fn build_inline_omath(content: &str) -> String {
    format!(r#"<m:oMath xmlns:m="{OMML_NS}">{content}</m:oMath>"#)
}

/// Block-level `<m:oMathPara>` for centered display equations.
pub fn build_display_omath_para(inner_omath: &str) -> String {
    let inner = if inner_omath.contains("<m:oMath") {
        inner_omath.to_string()
    } else {
        build_inline_omath(&omml_run(inner_omath))
    };
    format!(r#"<m:oMathPara xmlns:m="{OMML_NS}">{inner}</m:oMathPara>"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_omml_preview_text_concatenates_m_t() {
        let xml = r#"<m:oMath><m:r><m:t>E=mc</m:t></m:r><m:r><m:t>2</m:t></m:r></m:oMath>"#;
        assert_eq!(extract_omml_preview_text(xml), "E=mc2");
    }

    #[test]
    fn extract_omml_preview_text_skips_omath_para_wrapper() {
        let xml = r#"<m:oMathPara><m:oMath><m:r><m:t>x</m:t></m:r></m:oMath></m:oMathPara>"#;
        assert_eq!(extract_omml_preview_text(xml), "x");
    }

    #[test]
    fn build_inline_omath_wraps_text_run() {
        let xml = build_inline_omath(&omml_run("E=mc2"));
        assert!(xml.contains("<m:oMath"));
        assert!(xml.contains("E=mc2"));
        assert_eq!(extract_omml_preview_text(&xml), "E=mc2");
    }

    #[test]
    fn build_display_omath_para_wraps_inline() {
        let xml = build_display_omath_para("x+y");
        assert!(xml.contains("<m:oMathPara"));
        assert_eq!(extract_omml_preview_text(&xml), "x+y");
    }
}

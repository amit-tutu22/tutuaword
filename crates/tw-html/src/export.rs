use tw_model::{Block, Document, StyleSheet};

pub fn export_html(doc: &Document) -> String {
    let mut out = String::from(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>\n",
    );
    if let Some(section) = doc.sections.first() {
        for block in &section.blocks {
            if let Block::Paragraph(para) = block {
                let tag = html_block_tag(&doc.styles, para.style_id);
                out.push_str(&format!("<{tag}>"));
                for run in &para.runs {
                    let mut text = escape_html(run.text());
                    if run.format.bold == Some(true) {
                        text = format!("<strong>{text}</strong>");
                    }
                    if run.format.italic == Some(true) {
                        text = format!("<em>{text}</em>");
                    }
                    if run.revision.is_some() {
                        text = format!("<mark>{text}</mark>");
                    }
                    out.push_str(&text);
                }
                out.push_str(&format!("</{tag}>\n"));
            }
        }
    }
    out.push_str("</body></html>");
    out
}

/// Map paragraph style → HTML heading tag. Only Heading 1–6 become `<hN>`;
/// Quote/Caption/Normal and unknown styles stay `<p>` (F23.S3).
fn html_block_tag(styles: &StyleSheet, style_id: Option<tw_model::StyleId>) -> &'static str {
    let Some(id) = style_id else {
        return "p";
    };
    let Some(style) = styles.paragraph_styles.get(&id) else {
        return "p";
    };
    match style.name.as_str() {
        "Heading 1" => "h1",
        "Heading 2" => "h2",
        "Heading 3" => "h3",
        "Heading 4" => "h4",
        "Heading 5" => "h5",
        "Heading 6" => "h6",
        _ => "p",
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

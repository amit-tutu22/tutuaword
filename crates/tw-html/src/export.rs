use tw_model::{Block, Document};

pub fn export_html(doc: &Document) -> String {
    let mut out = String::from(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>\n",
    );
    if let Some(section) = doc.sections.first() {
        for block in &section.blocks {
            if let Block::Paragraph(para) = block {
                let tag = if para.style_id.is_some() { "h1" } else { "p" };
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

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

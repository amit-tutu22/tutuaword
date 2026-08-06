use thiserror::Error;
use tw_layout::{LayoutBox, LayoutEngine, PageLayout, TextLine};
use tw_model::Document;
use tw_render::DisplayListBuilder;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("pdf export failed: {0}")]
    ExportFailed(String),
}

#[derive(Debug, Clone, Default)]
pub struct PdfExportOptions {
    pub embed_fonts: bool,
}

pub trait PdfExporter: Send + Sync {
    fn export(&self, doc: &Document, options: &PdfExportOptions) -> Result<Vec<u8>, PdfError>;
}

pub struct DisplayListPdfExporter;

impl PdfExporter for DisplayListPdfExporter {
    fn export(&self, doc: &Document, _options: &PdfExportOptions) -> Result<Vec<u8>, PdfError> {
        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(doc);
        let mut pdf = MinimalPdfWriter::new();

        for page in &layout.pages {
            let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);
            pdf.add_page(page.width, page.height, &list, page);
        }

        Ok(pdf.finish())
    }
}

/// Legacy name kept for compatibility.
pub struct PrintPdfExporter;

impl PdfExporter for PrintPdfExporter {
    fn export(&self, doc: &Document, options: &PdfExportOptions) -> Result<Vec<u8>, PdfError> {
        DisplayListPdfExporter.export(doc, options)
    }
}

struct MinimalPdfWriter {
    pages: Vec<(f32, f32, String)>,
}

impl MinimalPdfWriter {
    fn new() -> Self {
        Self { pages: Vec::new() }
    }

    fn add_page(&mut self, width: f32, height: f32, list: &tw_render::DisplayList, page: &PageLayout) {
        let mut content = String::new();
        append_page_text(&mut content, height, page);

        for chunk in list.rect_batch.rects.chunks(4) {
            if chunk.len() == 4 {
                let [x, y, w, h] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                content.push_str(&format!(
                    "q 0.9 0.9 0.9 rg {} {} {} {} re f Q\n",
                    x,
                    height - y - h,
                    w,
                    h
                ));
            }
        }

        for chunk in list.path_batch.points.chunks(4) {
            if chunk.len() == 4 {
                content.push_str(&format!(
                    "q 0 0 0 RG 0.5 w {} {} m {} {} l S Q\n",
                    chunk[0],
                    height - chunk[1],
                    chunk[2],
                    height - chunk[3]
                ));
            }
        }

        self.pages.push((width, height, content));
    }

    fn finish(self) -> Vec<u8> {
        let mut objects: Vec<String> = Vec::new();
        objects.push("1 0 obj<< /Type /Catalog /Pages 2 0 R >>endobj".into());
        let kids: String = (3..3 + self.pages.len())
            .map(|i| format!("{i} 0 R"))
            .collect::<Vec<_>>()
            .join(" ");
        objects.push(format!("2 0 obj<< /Type /Pages /Kids [{kids}] /Count {} >>endobj", self.pages.len()));

        let mut next_id = 3;
        let mut xref_positions = vec![0usize];
        let mut body = String::new();

        for (width, height, content) in &self.pages {
            let content_id = next_id;
            next_id += 1;
            let page_id = next_id;
            next_id += 1;

            objects.push(format!(
                "{page_id} 0 obj<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width:.2} {height:.2}] /Contents {content_id} 0 R /Resources << /Font << /F1 {next_id} 0 R >> >> >>endobj"
            ));
            objects.push(format!(
                "{content_id} 0 obj<< /Length {} >>stream\n{content}\nendstream\nendobj",
                content.len()
            ));
        }

        let font_id = next_id;
        objects.push(format!(
            "{font_id} 0 obj<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>endobj"
        ));

        let mut pdf = String::from("%PDF-1.4\n");
        for obj in objects {
            xref_positions.push(pdf.len());
            pdf.push_str(&obj);
            pdf.push('\n');
        }

        let xref_start = pdf.len();
        pdf.push_str(&format!("xref\n0 {}\n", xref_positions.len()));
        pdf.push_str("0000000000 65535 f \n");
        for pos in &xref_positions[1..] {
            pdf.push_str(&format!("{pos:010} 00000 n \n"));
        }
        pdf.push_str(&format!("trailer<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF", xref_positions.len()));

        pdf.into_bytes()
    }
}

fn escape_pdf_text(text: &str) -> String {
    text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)")
}

fn append_page_text(content: &mut String, page_height: f32, page: &PageLayout) {
    for layout_box in &page.boxes {
        match layout_box {
            LayoutBox::TextLine(line) => append_line_text(content, page_height, line),
            LayoutBox::Table(table) => {
                for cell in &table.cells {
                    for line in &cell.lines {
                        append_line_text(content, page_height, line);
                    }
                }
            }
            _ => {}
        }
    }
}

fn append_line_text(content: &mut String, page_height: f32, line: &TextLine) {
    let mut text = String::new();
    for glyph in &line.glyphs {
        if glyph.codepoint.is_control() {
            continue;
        }
        text.push(glyph.codepoint);
    }
    if text.is_empty() {
        return;
    }
    let pdf_y = page_height - line.y;
    content.push_str(&format!(
        "BT /F1 12 Tf 1 0 0 1 {:.2} {:.2} Tm ({}) Tj ET\n",
        line.x,
        pdf_y,
        escape_pdf_text(&text)
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_edit::{apply, Command, EditSession};
    use tw_model::NodeId;

    #[test]
    fn export_document_with_table_produces_pdf() {
        let mut session = EditSession::new();
        let block_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .id;
        apply(
            &mut session.document,
            &mut session.buffer,
            Command::InsertTable {
                after_block_id: block_id,
                rows: 3,
                cols: 3,
            },
        )
        .unwrap();

        let exporter = DisplayListPdfExporter;
        let pdf = exporter.export(&session.document, &PdfExportOptions::default()).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 100);
    }
}

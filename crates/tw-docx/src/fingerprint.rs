//! A cheap content hash of a [`Document`], used to decide whether the
//! passthrough export in [`crate::export`] is still safe to take.
//!
//! Passthrough returns the original file byte for byte, which is only correct
//! while the document is unchanged. Relying on callers to flag every edit
//! makes a missed flag silently discard the user's work, so the export
//! compares fingerprints instead of trusting the flag alone.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use tw_model::{Block, Document, Paragraph, RunContent};

pub fn document_fingerprint(doc: &Document) -> u64 {
    let mut hasher = DefaultHasher::new();
    doc.sections.len().hash(&mut hasher);
    for section in &doc.sections {
        let format = &section.format;
        for value in [
            format.page_width,
            format.page_height,
            format.margin_top,
            format.margin_bottom,
            format.margin_left,
            format.margin_right,
        ] {
            value.to_bits().hash(&mut hasher);
        }
        format.header_text.hash(&mut hasher);
        format.footer_text.hash(&mut hasher);
        hash_blocks(&section.blocks, &mut hasher);
    }
    hasher.finish()
}

fn hash_blocks(blocks: &[Block], hasher: &mut DefaultHasher) {
    blocks.len().hash(hasher);
    for block in blocks {
        match block {
            Block::Paragraph(para) => {
                0u8.hash(hasher);
                hash_paragraph(para, hasher);
            }
            Block::Table(table) => {
                1u8.hash(hasher);
                table.format.column_widths.len().hash(hasher);
                table.rows.len().hash(hasher);
                for row in &table.rows {
                    row.cells.len().hash(hasher);
                    for cell in &row.cells {
                        cell.format.colspan.hash(hasher);
                        cell.format.rowspan.hash(hasher);
                        hash_blocks(&cell.blocks, hasher);
                    }
                }
            }
            Block::ImageBlock(image) => {
                2u8.hash(hasher);
                image.data.asset_id.hash(hasher);
                image.data.bytes.len().hash(hasher);
                image.display_width.to_bits().hash(hasher);
                image.display_height.to_bits().hash(hasher);
            }
            Block::ShapeBlock(shape) => {
                3u8.hash(hasher);
                shape.shape.width.to_bits().hash(hasher);
                shape.shape.height.to_bits().hash(hasher);
            }
            _ => {
                255u8.hash(hasher);
            }
        }
    }
}

fn hash_paragraph(para: &Paragraph, hasher: &mut DefaultHasher) {
    para.style_id.hash(hasher);
    para.format.alignment.hash(hasher);
    para.format.numbering.hash(hasher);
    para.runs.len().hash(hasher);
    for run in &para.runs {
        match &run.content {
            RunContent::Text(text) => {
                0u8.hash(hasher);
                text.hash(hasher);
            }
            RunContent::Tab => 1u8.hash(hasher),
            RunContent::Break(kind) => {
                2u8.hash(hasher);
                kind.hash(hasher);
            }
            RunContent::Hyperlink { text, .. } => {
                3u8.hash(hasher);
                text.hash(hasher);
            }
            RunContent::Field(field) => {
                4u8.hash(hasher);
                field.display_text.hash(hasher);
            }
            RunContent::InlineImage(img) => {
                5u8.hash(hasher);
                img.image.asset_id.hash(hasher);
            }
            RunContent::FootnoteRef(note) => {
                6u8.hash(hasher);
                note.note_id.hash(hasher);
            }
            RunContent::CommentRef(c) => {
                7u8.hash(hasher);
                c.comment_id.hash(hasher);
            }
            RunContent::Bookmark(b) => {
                8u8.hash(hasher);
                b.name.hash(hasher);
            }
            _ => {
                9u8.hash(hasher);
            }
        }
        let format = &run.format;
        format.bold.hash(hasher);
        format.italic.hash(hasher);
        format.underline.hash(hasher);
        format.strikethrough.hash(hasher);
        format.all_caps.hash(hasher);
        format.small_caps.hash(hasher);
        format.hidden.hash(hasher);
        format.font_family.hash(hasher);
        format.font_size.map(f32::to_bits).hash(hasher);
        format.character_spacing.map(f32::to_bits).hash(hasher);
        format.color.hash(hasher);
        run.revision.is_some().hash(hasher);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::Block;

    #[test]
    fn an_unchanged_document_keeps_its_fingerprint() {
        let doc = Document::with_paragraph("Hello");

        assert_eq!(document_fingerprint(&doc), document_fingerprint(&doc.clone()));
    }

    #[test]
    fn editing_text_changes_the_fingerprint() {
        let doc = Document::with_paragraph("Hello");
        let before = document_fingerprint(&doc);

        let mut edited = doc.clone();
        if let Block::Paragraph(para) = &mut edited.sections[0].blocks[0] {
            *para.runs[0].text_mut().unwrap() = "Goodbye".into();
        }

        assert_ne!(before, document_fingerprint(&edited));
    }

    #[test]
    fn toggling_bold_changes_the_fingerprint() {
        let doc = Document::with_paragraph("Hello");
        let before = document_fingerprint(&doc);

        let mut edited = doc.clone();
        if let Block::Paragraph(para) = &mut edited.sections[0].blocks[0] {
            para.runs[0].format.bold = Some(true);
        }

        assert_ne!(before, document_fingerprint(&edited));
    }

    #[test]
    fn deleting_a_block_changes_the_fingerprint() {
        let mut doc = Document::with_paragraph("One");
        doc.sections[0]
            .blocks
            .push(Block::Paragraph(tw_model::Paragraph::with_text("Two")));
        let before = document_fingerprint(&doc);

        doc.sections[0].blocks.pop();

        assert_ne!(before, document_fingerprint(&doc));
    }

    #[test]
    fn changing_a_page_margin_changes_the_fingerprint() {
        let doc = Document::with_paragraph("Hello");
        let before = document_fingerprint(&doc);

        let mut edited = doc.clone();
        edited.sections[0].format.margin_left = 90.0;

        assert_ne!(before, document_fingerprint(&edited));
    }
}

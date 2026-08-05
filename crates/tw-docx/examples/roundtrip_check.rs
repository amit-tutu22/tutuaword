//! Temporary verification harness: import each .docx given on the command
//! line, export it, re-import, and report what changed.

use std::fs;

use tw_model::{Block, Document};

fn outline(doc: &Document) -> (usize, usize, usize, usize, usize) {
    let mut paragraphs = 0;
    let mut tables = 0;
    let mut images = 0;
    let mut image_bytes = 0;
    let mut chars = 0;
    for section in &doc.sections {
        for block in &section.blocks {
            match block {
                Block::Paragraph(p) => {
                    paragraphs += 1;
                    chars += p.full_text().chars().count();
                }
                Block::Table(t) => {
                    tables += 1;
                    for row in &t.rows {
                        for cell in &row.cells {
                            for b in &cell.blocks {
                                if let Block::Paragraph(p) = b {
                                    chars += p.full_text().chars().count();
                                }
                            }
                        }
                    }
                }
                Block::ImageBlock(i) => {
                    images += 1;
                    image_bytes += i.data.bytes.len();
                }
            }
        }
    }
    (paragraphs, tables, images, image_bytes, chars)
}

fn main() {
    for path in std::env::args().skip(1) {
        let Ok(bytes) = fs::read(&path) else {
            println!("SKIP  {path} (unreadable)");
            continue;
        };
        let imported = match tw_docx::import(&bytes) {
            Ok(r) => r,
            Err(e) => {
                println!("SKIP  {path} (import failed: {e})");
                continue;
            }
        };
        let before = outline(&imported.document);

        let mut package = imported.package.clone();
        package.mark_modified("word/document.xml".into());
        let exported = match tw_docx::export(&imported.document, &package) {
            Ok(b) => b,
            Err(e) => {
                println!("FAIL  {path} (export failed: {e})");
                continue;
            }
        };
        let after = match tw_docx::import(&exported) {
            Ok(r) => outline(&r.document),
            Err(e) => {
                println!("FAIL  {path} (re-import failed: {e})");
                continue;
            }
        };

        let name = path.rsplit('/').next().unwrap_or(&path);
        let status = if before == after { "OK  " } else { "DIFF" };
        println!(
            "{status}  {name}\n        paras {}/{}  tables {}/{}  images {}/{}  imgbytes {}/{}  chars {}/{}",
            before.0, after.0, before.1, after.1, before.2, after.2, before.3, after.3, before.4, after.4
        );
    }
}

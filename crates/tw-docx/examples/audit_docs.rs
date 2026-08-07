//! Import + layout audit for real-world DOCX files passed on the command line.

use std::fs;

use tw_layout::LayoutEngine;
use tw_model::{Block, Document, LineSpacing};

fn outline(doc: &Document) -> AuditStats {
    let mut stats = AuditStats::default();
    for section in &doc.sections {
        let fmt = &section.format;
        if fmt.header_text.is_some() || !fmt.header_blocks.is_empty() {
            stats.has_header = true;
        }
        if fmt.footer_text.is_some() || !fmt.footer_blocks.is_empty() {
            stats.has_footer = true;
        }
        stats.header_blocks += fmt.header_blocks.len();
        stats.footer_blocks += fmt.footer_blocks.len();

        for block in &section.blocks {
            match block {
                Block::Paragraph(p) => {
                    stats.paragraphs += 1;
                    stats.chars += p.full_text().chars().count();
                    audit_para(&p.format, &mut stats);
                    for run in &p.runs {
                        stats.runs += 1;
                        if run.format.superscript == Some(true) {
                            stats.superscript_runs += 1;
                        }
                        if run.format.subscript == Some(true) {
                            stats.subscript_runs += 1;
                        }
                        if run.format.bold == Some(true) {
                            stats.bold_runs += 1;
                        }
                    }
                }
                Block::Table(t) => {
                    stats.tables += 1;
                    stats.table_rows += t.rows.len();
                    for row in &t.rows {
                        for cell in &row.cells {
                            if cell.format.rowspan > 1 {
                                stats.rowspan_cells += 1;
                            }
                            if cell.format.colspan > 1 {
                                stats.colspan_cells += 1;
                            }
                            if cell.format.background.is_some() {
                                stats.shaded_cells += 1;
                            }
                            if cell.format.border.is_some() {
                                stats.bordered_cells += 1;
                            }
                            for b in &cell.blocks {
                                if let Block::Paragraph(p) = b {
                                    stats.chars += p.full_text().chars().count();
                                }
                            }
                        }
                    }
                }
                Block::ImageBlock(i) => {
                    stats.images += 1;
                    stats.image_bytes += i.data.bytes.len();
                    if i.data.bytes.is_empty() {
                        stats.empty_images += 1;
                    }
                }
                Block::ShapeBlock(_) => {}
                _ => {}
            }
        }
    }
    stats
}

fn audit_para(format: &tw_model::ParaFormat, stats: &mut AuditStats) {
    if !format.tab_stops.is_empty() {
        stats.paras_with_tab_stops += 1;
    }
    match &format.line_spacing {
        Some(LineSpacing::Exactly(_)) => stats.exact_line_spacing += 1,
        Some(LineSpacing::AtLeast(_)) => stats.at_least_line_spacing += 1,
        Some(LineSpacing::Multiple(m)) if *m > 3.0 => stats.large_multiple_spacing += 1,
        _ => {}
    }
    if format.numbering.is_some() {
        stats.numbered_paras += 1;
    }
}

#[derive(Default)]
struct AuditStats {
    paragraphs: usize,
    runs: usize,
    chars: usize,
    tables: usize,
    table_rows: usize,
    images: usize,
    image_bytes: usize,
    empty_images: usize,
    has_header: bool,
    has_footer: bool,
    header_blocks: usize,
    footer_blocks: usize,
    paras_with_tab_stops: usize,
    exact_line_spacing: usize,
    at_least_line_spacing: usize,
    large_multiple_spacing: usize,
    numbered_paras: usize,
    rowspan_cells: usize,
    colspan_cells: usize,
    shaded_cells: usize,
    bordered_cells: usize,
    superscript_runs: usize,
    subscript_runs: usize,
    bold_runs: usize,
}

fn main() {
    for path in std::env::args().skip(1) {
        let name = path.rsplit('/').next().unwrap_or(&path);
        let Ok(bytes) = fs::read(&path) else {
            println!("=== {name}\n  ERROR: unreadable\n");
            continue;
        };
        let size_kb = bytes.len() as f64 / 1024.0;

        let imported = match tw_docx::import(&bytes) {
            Ok(r) => r,
            Err(e) => {
                println!("=== {name}\n  IMPORT FAILED: {e}\n");
                continue;
            }
        };

        let doc = &imported.document;
        let stats = outline(doc);

        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(doc);
        let pages = layout.pages.len();

        println!("=== {name}");
        println!("  file size: {size_kb:.1} KB");
        println!(
            "  import: {} paras, {} runs, {} chars, {} tables ({} rows), {} images ({} KB, {} unresolved)",
            stats.paragraphs,
            stats.runs,
            stats.chars,
            stats.tables,
            stats.table_rows,
            stats.images,
            stats.image_bytes / 1024,
            stats.empty_images,
        );
        println!("  layout: {pages} pages");
        println!(
            "  header: {} ({} blocks)  footer: {} ({} blocks)",
            stats.has_header, stats.header_blocks, stats.has_footer, stats.footer_blocks
        );
        if stats.paras_with_tab_stops > 0
            || stats.exact_line_spacing > 0
            || stats.at_least_line_spacing > 0
            || stats.numbered_paras > 0
        {
            println!(
                "  spacing/lists: {} tab-stop paras, {} exact, {} atLeast, {} numbered",
                stats.paras_with_tab_stops,
                stats.exact_line_spacing,
                stats.at_least_line_spacing,
                stats.numbered_paras,
            );
        }
        if stats.rowspan_cells > 0
            || stats.colspan_cells > 0
            || stats.shaded_cells > 0
            || stats.bordered_cells > 0
        {
            println!(
                "  tables detail: {} rowspan, {} colspan, {} shaded, {} bordered cells",
                stats.rowspan_cells,
                stats.colspan_cells,
                stats.shaded_cells,
                stats.bordered_cells,
            );
        }
        if stats.superscript_runs > 0 || stats.subscript_runs > 0 {
            println!(
                "  scripts: {} superscript, {} subscript runs",
                stats.superscript_runs, stats.subscript_runs
            );
        }

        let mut notes = Vec::new();
        if stats.empty_images > 0 {
            notes.push(format!(
                "{} image(s) have no embedded bytes (may not render)",
                stats.empty_images
            ));
        }
        if stats.large_multiple_spacing > 0 {
            notes.push(format!(
                "{} para(s) with very large multiple line spacing (>3x)",
                stats.large_multiple_spacing
            ));
        }
        if stats.chars == 0 && stats.images == 0 {
            notes.push("document appears empty after import".into());
        }
        if pages > 50 && stats.chars < 5000 {
            notes.push(format!(
                "suspicious pagination: {pages} pages for only {} chars",
                stats.chars
            ));
        }
        if !notes.is_empty() {
            println!("  warnings:");
            for n in notes {
                println!("    - {n}");
            }
        }
        println!();
    }
}

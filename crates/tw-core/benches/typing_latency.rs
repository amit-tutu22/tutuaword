//! Measures the edit + layout pipeline budget from docs/performance-budgets.md.
//!
//! `typing_incremental_relayout` is the gate: it keeps one warm engine and one
//! document and times a single keystroke plus the incremental reflow it triggers,
//! which is the path a user actually waits on. `cold_full_document_layout` is the
//! open/import path and is tracked separately -- it is orders of magnitude slower
//! by design and must not be read as typing latency.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};
use tw_edit::{apply, Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::{Document, NodeId, Paragraph};

const PARAGRAPHS: usize = 400;
/// Characters accumulated before the untimed reset trims the paragraph back, so a
/// long criterion run cannot silently grow the document out from under the measurement.
const RESET_INTERVAL: u64 = 64;

fn multi_page_document() -> Document {
    let mut doc = Document::with_paragraph("Typing latency benchmark corpus.");
    let section = &mut doc.sections[0];
    for i in 0..PARAGRAPHS {
        let text = format!(
            "Paragraph {i} of the benchmark corpus. The quick brown fox jumps over \
             the lazy dog, then turns around and does it again so the line wraps at \
             least twice within the page content width."
        );
        section
            .blocks
            .push(tw_model::Block::Paragraph(Paragraph::with_text(text)));
    }
    doc
}

/// A run roughly a third of the way in, so the reflow has pages on both sides.
fn edit_target(doc: &Document) -> NodeId {
    let index = PARAGRAPHS / 3;
    doc.sections[0].blocks[index]
        .paragraph()
        .expect("paragraph block")
        .runs[0]
        .id
}

fn typing_incremental_relayout(c: &mut Criterion) {
    let doc = multi_page_document();
    let run_id = edit_target(&doc);
    let mut session = EditSession::from_document(doc);
    let mut layout = LayoutEngine::new();
    // Warm: the steady state we care about is an engine that already holds a
    // layout and only has to reflow around the caret.
    let pages = layout.layout_document(&session.document).pages.len();
    assert!(pages > 10, "corpus should span many pages, got {pages}");

    c.bench_function("typing_incremental_relayout", |b| {
        b.iter_custom(|iters| {
            let mut elapsed = Duration::ZERO;
            for i in 0..iters {
                let start = Instant::now();
                apply(
                    &mut session.document,
                    Command::InsertText {
                        run_id,
                        offset: 10,
                        text: "X".into(),
                    },
                )
                .expect("insert");
                layout.invalidate_nodes(&session.document, &[run_id]);
                black_box(layout.layout_document(&session.document));
                elapsed += start.elapsed();

                if i % RESET_INTERVAL == RESET_INTERVAL - 1 {
                    apply(
                        &mut session.document,
                        Command::DeleteRange {
                            run_id,
                            start: 10,
                            end: 10 + RESET_INTERVAL as usize,
                        },
                    )
                    .expect("reset delete");
                    layout.invalidate_nodes(&session.document, &[run_id]);
                    layout.layout_document(&session.document);
                }
            }
            elapsed
        });
    });
}

fn cold_full_document_layout(c: &mut Criterion) {
    let doc = multi_page_document();

    c.bench_function("cold_full_document_layout", |b| {
        b.iter(|| {
            let mut layout = LayoutEngine::new();
            black_box(layout.layout_document(&doc));
        });
    });
}

criterion_group!(benches, typing_incremental_relayout, cold_full_document_layout);
criterion_main!(benches);

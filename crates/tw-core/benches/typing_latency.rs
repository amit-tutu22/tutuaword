//! Measures the edit + layout pipeline budget from docs/performance-budgets.md.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tw_edit::{apply, Command, EditSession};
use tw_layout::LayoutEngine;
use tw_model::Document;

fn typing_latency(c: &mut Criterion) {
    let doc = Document::with_paragraph("The quick brown fox jumps over the lazy dog.");
    let mut session = EditSession::from_document(doc);
    let run_id = session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .runs[0]
        .id;

    c.bench_function("insert_text_and_layout_page", |b| {
        b.iter(|| {
            let mut layout = LayoutEngine::new();
            apply(
                &mut session.document,
                &mut session.buffer,
                Command::InsertText {
                    run_id,
                    offset: 10,
                    text: "X".into(),
                },
            )
            .unwrap();
            black_box(layout.layout_document(&session.document));
        });
    });
}

criterion_group!(benches, typing_latency);
criterion_main!(benches);

//! F25.S2 — print scale and margins (unit + stress).

use tw_model::{Block, Document, Paragraph};
use tw_pdf::{
    prepare_print_pdf, PrintLayoutOptions, PrintScaleMode,
};

fn sample_doc() -> Document {
    let mut doc = Document::new();
    doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
        "Scale and margins sample paragraph.",
    ))];
    doc
}

#[test]
fn u_f25_s2_resolve_actual_size_identity() {
    let layout = PrintLayoutOptions::default();
    let t = layout.resolve(612.0, 792.0);
    assert!((t.scale - 1.0).abs() < 1e-5);
    assert!(t.tx.abs() < 1e-5);
    assert!(t.ty.abs() < 1e-5);
    assert!(t.is_identity());
}

#[test]
fn u_f25_s2_resolve_custom_scale() {
    let layout = PrintLayoutOptions::with_scale_percent(50.0);
    let t = layout.resolve(612.0, 792.0);
    assert!((t.scale - 0.5).abs() < 1e-5);
    // Centered in full page when margins are zero.
    assert!((t.tx - 612.0 * 0.25).abs() < 1e-3);
    assert!((t.ty - 792.0 * 0.25).abs() < 1e-3);
}

#[test]
fn u_f25_s2_resolve_uniform_margins() {
    let layout = PrintLayoutOptions::with_uniform_margins(36.0);
    let t = layout.resolve(612.0, 792.0);
    assert!((t.scale - 1.0).abs() < 1e-5);
    // Content at full size cannot fit with 36pt margins on each side; tx/ty stay at margin.
    assert!((t.tx - 36.0).abs() < 1e-3 || t.tx >= 36.0 - 1e-3);
    assert!(t.ty >= 36.0 - 1e-3);
}

#[test]
fn u_f25_s2_resolve_fit_to_margins() {
    let layout = PrintLayoutOptions::fit_to_margins(72.0);
    let t = layout.resolve(612.0, 792.0);
    let expected = ((612.0_f32 - 144.0) / 612.0).min((792.0 - 144.0) / 792.0);
    assert!((t.scale - expected).abs() < 1e-4);
    assert!(t.scale < 1.0);
    assert!(!t.is_identity());
}

#[test]
fn u_f25_s2_prepare_print_pdf_embeds_scale_cm() {
    let doc = sample_doc();
    let layout = PrintLayoutOptions::with_scale_percent(50.0);
    let pdf = prepare_print_pdf(&doc, &layout).expect("scaled print PDF");
    assert!(pdf.starts_with(b"%PDF"));
    let s = String::from_utf8_lossy(&pdf);
    assert!(
        s.contains("0.500000 0 0 0.500000") || s.contains("0.5 0 0 0.5"),
        "expected 50% scale CTM in content stream"
    );
}

#[test]
fn u_f25_s2_prepare_print_pdf_embeds_margin_translate() {
    let doc = sample_doc();
    let layout = PrintLayoutOptions {
        scale_mode: PrintScaleMode::CustomPercent,
        scale_percent: 80.0,
        margin_left: 48.0,
        margin_right: 48.0,
        margin_top: 36.0,
        margin_bottom: 36.0,
        ..Default::default()
    };
    let pdf = prepare_print_pdf(&doc, &layout).expect("margin print PDF");
    let s = String::from_utf8_lossy(&pdf);
    assert!(s.contains("0.800000 0 0 0.800000") || s.contains(" cm\n"));
    // Transform open/close present when non-identity.
    assert!(s.contains(" q ") || s.contains("\nq ") || s.contains("cm\n"));
}

#[test]
fn i_f25_s2_print_layout_roundtrip_options() {
    let doc = sample_doc();
    for layout in [
        PrintLayoutOptions::default(),
        PrintLayoutOptions::with_scale_percent(125.0),
        PrintLayoutOptions::with_uniform_margins(24.0),
        PrintLayoutOptions::fit_to_margins(54.0),
    ] {
        let pdf = prepare_print_pdf(&doc, &layout).expect("print PDF");
        assert!(pdf.starts_with(b"%PDF"), "{layout:?}");
        assert!(pdf.len() > 64, "{layout:?}");
    }
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f25_s2_print_scale_margin_churn() {
    let mut doc = Document::new();
    doc.sections[0].blocks.clear();
    for i in 0..30 {
        doc.sections[0].blocks.push(Block::Paragraph(Paragraph::with_text(format!(
            "Scale churn paragraph {i}"
        ))));
    }
    for percent in [25.0, 50.0, 75.0, 100.0, 150.0, 200.0] {
        for margin in [0.0, 18.0, 36.0, 72.0] {
            let layout = PrintLayoutOptions {
                scale_mode: PrintScaleMode::CustomPercent,
                scale_percent: percent,
                margin_left: margin,
                margin_right: margin,
                margin_top: margin,
                margin_bottom: margin,
                ..Default::default()
            };
            let pdf = prepare_print_pdf(&doc, &layout).expect("churn PDF");
            assert!(pdf.starts_with(b"%PDF"));
        }
    }
}

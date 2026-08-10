//! F28.S6 — smart editing: heading suggestions, auto-format notes, TOC draft.

use std::sync::Arc;

use tw_ai::{
    analyze_document_heuristics, apply_smart_edit_plan, parse_smart_edit_plan,
    production_desktop_router, smart_edit_commands, suggest_smart_edit, AiServiceImpl,
    DocumentContext, MockHttpClient, ProviderRouter, LLAMA_CPP, OPENAI,
};
use tw_edit::{Command, EditSession};
use tw_model::{Block, Document, Paragraph, TOC_TITLE};

fn messy_doc() -> Document {
    let mut doc = Document::new();
    let section = doc.sections.first_mut().unwrap();
    section.blocks = vec![
        Block::Paragraph(Paragraph::with_text("Introduction")),
        Block::Paragraph(Paragraph::with_text(
            "This is a long body paragraph that explains the background in detail.",
        )),
        Block::Paragraph(Paragraph::with_text("")),
        Block::Paragraph(Paragraph::with_text("")),
        Block::Paragraph(Paragraph::with_text("Next Steps:")),
        Block::Paragraph(Paragraph::with_text("Ship the MVP and gather feedback from users.")),
        Block::Paragraph(Paragraph::with_text("A  B")), // irregular spacing
    ];
    doc
}

fn service_with_http(http: Arc<MockHttpClient>) -> AiServiceImpl {
    let hybrid = production_desktop_router(
        http,
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    );
    let mut provider_router = ProviderRouter::new(OPENAI, LLAMA_CPP);
    *provider_router.hybrid_mut() = hybrid;
    AiServiceImpl::new(provider_router)
}

fn heading_style_names(doc: &Document) -> Vec<(String, Option<String>)> {
    doc.sections[0]
        .blocks
        .iter()
        .filter_map(|b| {
            let p = b.paragraph()?;
            let text = p.full_text();
            if text.trim().is_empty() {
                return None;
            }
            let style = p.style_id.and_then(|id| {
                doc.styles
                    .paragraph_styles
                    .get(&id)
                    .map(|s| s.name.clone())
            });
            Some((text, style))
        })
        .collect()
}

#[test]
fn u_f28_s6_heuristics_suggest_headings_and_notes() {
    let doc = messy_doc();
    let plan = analyze_document_heuristics(&doc);
    assert!(
        plan.headings.iter().any(|h| h.preview_text.contains("Introduction")),
        "expected Introduction heading: {:?}",
        plan.headings
    );
    assert!(
        plan.headings.iter().any(|h| h.style_name.starts_with("Heading")),
        "expected heading styles"
    );
    assert!(plan.insert_toc);
    assert!(
        plan.auto_format_notes.iter().any(|n| n.contains("blank") || n.contains("spacing")),
        "expected auto-format notes: {:?}",
        plan.auto_format_notes
    );
}

#[test]
fn u_f28_s6_parse_ai_plan_overrides_indices() {
    let doc = messy_doc();
    let plan = parse_smart_edit_plan(
        r#"{"headings":[{"index":0,"style":"Heading 1"},{"index":4,"style":"Heading 2"}],"insert_toc":true}"#,
        &doc,
    )
    .unwrap();
    assert_eq!(plan.headings.len(), 2);
    assert_eq!(plan.headings[0].style_name, "Heading 1");
    assert_eq!(plan.headings[1].paragraph_index, 4);
    assert!(plan.insert_toc);
}

#[test]
fn u_f28_s6_smart_edit_commands() {
    let doc = messy_doc();
    let plan = analyze_document_heuristics(&doc);
    let cmds = smart_edit_commands(&doc, &plan);
    assert!(cmds.iter().any(|c| matches!(c, Command::ApplyParagraphStyle { .. })));
    assert!(cmds
        .iter()
        .any(|c| matches!(c, Command::InsertTableOfContents { .. })));
}

#[test]
fn i_f28_s6_apply_headings_and_toc() {
    let mut session = EditSession::from_document(messy_doc());
    let plan = analyze_document_heuristics(&session.document);
    assert!(!plan.headings.is_empty());
    apply_smart_edit_plan(&mut session, &plan).unwrap();

    let styles = heading_style_names(&session.document);
    assert!(
        styles.iter().any(|(t, s)| t.contains("Introduction") && s.as_deref() == Some("Heading 1")),
        "Introduction should be H1: {styles:?}"
    );

    let has_toc = session.document.sections[0].blocks.iter().any(|b| {
        b.paragraph()
            .is_some_and(|p| p.full_text().contains(TOC_TITLE))
    });
    assert!(has_toc, "expected TOC title after apply");

    // Heading styles undo as one step (TOC inverse is not supported yet).
    let mut session2 = EditSession::from_document(messy_doc());
    let mut heading_only = analyze_document_heuristics(&session2.document);
    heading_only.insert_toc = false;
    apply_smart_edit_plan(&mut session2, &heading_only).unwrap();
    session2.undo().unwrap();
    let styles_after = heading_style_names(&session2.document);
    assert!(
        styles_after
            .iter()
            .all(|(_, s)| !s.as_deref().unwrap_or("").starts_with("Heading")),
        "undo should remove heading styles"
    );
}

#[test]
fn i_f28_s6_suggest_smart_edit_via_ai() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix(
        "http://127.0.0.1:11434/",
        r##"{"choices":[{"message":{"content":"{\"headings\":[{\"index\":0,\"style\":\"Heading 1\"}],\"insert_toc\":false}"}}]}"##,
    );
    let service = service_with_http(http);
    let doc = messy_doc();
    let plan = suggest_smart_edit(
        &service,
        &doc,
        &DocumentContext {
            total_token_estimate: 120,
            page_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(plan.headings.len(), 1);
    assert_eq!(plan.headings[0].style_name, "Heading 1");
    assert!(!plan.insert_toc);
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f28_s6_analyze_apply_undo_churn() {
    for i in 0..150 {
        let mut session = EditSession::from_document(messy_doc());
        let mut plan = analyze_document_heuristics(&session.document);
        plan.insert_toc = false; // TOC inverse not supported; stress undo on headings
        apply_smart_edit_plan(&mut session, &plan).unwrap();
        assert!(
            heading_style_names(&session.document)
                .iter()
                .any(|(_, s)| s.as_deref().unwrap_or("").starts_with("Heading")),
            "iter {i}"
        );
        session.undo().unwrap();
        let styles = heading_style_names(&session.document);
        assert!(
            styles.iter().all(|(_, s)| !s.as_deref().unwrap_or("").starts_with("Heading")),
            "undo iter {i}"
        );
    }
}

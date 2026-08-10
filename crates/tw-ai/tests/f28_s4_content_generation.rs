//! F28.S4 — generate outline / minutes / report into a new document.

use std::sync::Arc;

use tw_ai::{
    document_from_markdown, generate_content, production_desktop_router, AiServiceImpl, ContentKind,
    DocumentContext, MockHttpClient, ProviderRouter, LLAMA_CPP, OPENAI,
};
use tw_model::Block;

fn outline_fixture() -> &'static str {
    r##"{
      "choices": [{"message": {"content": "# Project Plan\n## Goals\n- Ship MVP\n## Timeline\nQ3 kickoff"}}],
      "usage": {"total_tokens": 20}
    }"##
}

fn minutes_fixture() -> &'static str {
    r##"{
      "choices": [{"message": {"content": "# Sprint Sync\n## Attendees\nAlice, Bob\n## Discussion\nScope review\n## Action items\n- Draft design"}}],
      "usage": {"total_tokens": 18}
    }"##
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

fn heading_names(doc: &tw_model::Document) -> Vec<Option<String>> {
    doc.sections[0]
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Paragraph(p) => {
                let name = p.style_id.and_then(|id| {
                    doc.styles
                        .paragraph_styles
                        .get(&id)
                        .map(|s| s.name.clone())
                });
                Some(name)
            }
            _ => None,
        })
        .collect()
}

#[test]
fn u_f28_s4_document_from_markdown_applies_headings() {
    let md = "# Title\n## Section\nBody line\n- bullet";
    let doc = document_from_markdown(md);
    let names = heading_names(&doc);
    assert_eq!(names.len(), 4);
    assert_eq!(names[0].as_deref(), Some("Heading 1"));
    assert_eq!(names[1].as_deref(), Some("Heading 2"));
    assert_eq!(names[2], None);
    let texts: Vec<_> = doc.sections[0]
        .blocks
        .iter()
        .filter_map(|b| b.paragraph().map(|p| p.full_text()))
        .collect();
    assert_eq!(texts[0], "Title");
    assert_eq!(texts[3], "• bullet");
}

#[test]
fn u_f28_s4_content_kind_prompt() {
    let p = ContentKind::Outline.prompt_instruction("launch");
    assert!(p.to_lowercase().contains("outline"));
    assert!(p.contains("launch"));
}

#[test]
fn i_f28_s4_generate_outline_new_document() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", outline_fixture());
    let service = service_with_http(http);
    let ctx = DocumentContext {
        total_token_estimate: 100,
        page_count: 1,
        ..Default::default()
    };
    let generated =
        generate_content(&service, ContentKind::Outline, "Project Plan", &ctx).unwrap();
    assert_eq!(generated.kind, ContentKind::Outline);
    assert!(generated.paragraph_count() >= 3);
    let names = heading_names(&generated.document);
    assert_eq!(names[0].as_deref(), Some("Heading 1"));
    assert!(generated.plain_text().contains("Project Plan"));
    assert!(generated.plain_text().contains("Goals"));
}

#[test]
fn i_f28_s4_generate_minutes_new_document() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", minutes_fixture());
    let service = service_with_http(http);
    let generated = generate_content(
        &service,
        ContentKind::Minutes,
        "Sprint Sync",
        &DocumentContext::default(),
    )
    .unwrap();
    assert_eq!(generated.kind, ContentKind::Minutes);
    assert!(generated.plain_text().contains("Attendees"));
    assert!(generated.plain_text().contains("Action items"));
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f28_s4_generate_churn() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", outline_fixture());
    let service = service_with_http(http);
    let kinds = [
        ContentKind::Outline,
        ContentKind::Minutes,
        ContentKind::Report,
    ];
    for i in 0..150 {
        let kind = kinds[i % kinds.len()];
        let generated = generate_content(
            &service,
            kind,
            &format!("topic-{i}"),
            &DocumentContext {
                total_token_estimate: 80,
                page_count: 1,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(generated.paragraph_count() >= 1, "iter {i}");
        assert!(!generated.markdown.is_empty());
    }
}

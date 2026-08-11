//! F28.S3 — document chunking, RAG retrieval, chat with paragraph citations.

use std::sync::Arc;

use tw_ai::{
    chunk_document, production_desktop_router, ChatRole, DocumentChatSession, DocumentRagIndex,
    MockHttpClient, ProviderRouter, LLAMA_CPP, OPENAI,
};
use tw_model::{Block, Document, Paragraph};

fn sample_doc() -> Document {
    let mut doc = Document::new();
    let section = doc.sections.first_mut().unwrap();
    section.blocks = vec![
        Block::Paragraph(Paragraph::with_text(
            "Budget overview: Q3 spending is under control.",
        )),
        Block::Paragraph(Paragraph::with_text(
            "Risk assessment: supply chain delays remain the top risk.",
        )),
        Block::Paragraph(Paragraph::with_text(
            "Next steps: schedule a vendor review meeting next week.",
        )),
    ];
    doc
}

fn chat_fixture_with_cite(para_id: &str) -> String {
    format!(
        r#"{{
      "choices": [{{"message": {{"content": "Supply chain delays are the top risk. [[cite:{para_id}]]"}}}}],
      "usage": {{"total_tokens": 12}}
    }}"#
    )
}

fn chat_fixture_plain() -> &'static str {
    r#"{
      "choices": [{"message": {"content": "The document covers budget, risk, and next steps."}}],
      "usage": {"total_tokens": 8}
    }"#
}

fn router_with_http(http: Arc<MockHttpClient>) -> ProviderRouter {
    let hybrid = production_desktop_router(
        http,
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    );
    let mut provider_router = ProviderRouter::new(OPENAI, LLAMA_CPP);
    *provider_router.hybrid_mut() = hybrid;
    provider_router
}

#[test]
fn u_f28_s3_chunk_document_by_paragraph() {
    let doc = sample_doc();
    let chunks = chunk_document(&doc);
    assert_eq!(chunks.len(), 3);
    assert!(chunks[0].text.contains("Budget"));
    assert!(chunks[1].text.contains("Risk"));
    assert_ne!(chunks[0].paragraph_id, chunks[1].paragraph_id);
    assert_eq!(chunks[1].block_index, 1);
}

#[test]
fn u_f28_s3_rag_retrieve_by_keywords() {
    let index = DocumentRagIndex::from_document(&sample_doc());
    let hits = index.retrieve("What is the top risk?", 2);
    assert!(!hits.is_empty());
    assert!(
        hits[0].text.to_lowercase().contains("risk"),
        "expected risk paragraph first, got {}",
        hits[0].text
    );
}

#[test]
fn i_f28_s3_chat_returns_paragraph_citations() {
    let doc = sample_doc();
    let risk_id = doc.paragraph_at(0, 1).unwrap().id;
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix(
        "http://127.0.0.1:11434/",
        &chat_fixture_with_cite(&risk_id.to_string()),
    );
    let router = router_with_http(http);
    let mut chat = DocumentChatSession::from_document(&doc);
    let reply = chat.ask(&router, "What is the top risk?").unwrap();

    assert_eq!(reply.role, ChatRole::Assistant);
    assert!(reply.content.to_lowercase().contains("risk"));
    assert!(
        !reply.content.contains("[[cite:"),
        "markers should be stripped from display text"
    );
    assert!(
        reply
            .references
            .iter()
            .any(|r| r.paragraph_id == risk_id),
        "expected citation to risk paragraph"
    );
    assert_eq!(chat.history().len(), 2);
}

#[test]
fn i_f28_s3_chat_fallback_retrieved_citations() {
    let doc = sample_doc();
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", chat_fixture_plain());
    let router = router_with_http(http);
    let mut chat = DocumentChatSession::from_document(&doc);
    let reply = chat
        .ask(&router, "Summarize budget overview spending")
        .unwrap();
    assert!(!reply.references.is_empty());
    assert!(reply.references.iter().any(|r| r.excerpt.contains("Budget")));
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f28_s3_chunk_retrieve_chat_churn() {
    let doc = sample_doc();
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", chat_fixture_plain());
    let router = router_with_http(http);
    let mut chat = DocumentChatSession::from_document(&doc);
    for i in 0..200 {
        let q = if i % 2 == 0 {
            "budget spending overview"
        } else {
            "risk assessment delays"
        };
        let hits = chat.index().retrieve(q, 2);
        assert!(!hits.is_empty(), "iter {i}");
        let reply = chat.ask(&router, q).unwrap();
        assert_eq!(reply.role, ChatRole::Assistant);
        assert!(!reply.content.is_empty());
    }
    assert!(chat.history().len() >= 400);
}

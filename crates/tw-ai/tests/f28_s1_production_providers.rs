//! F28.S1 — production OpenAI / Gemini / llama adapters + HybridRouter.

use std::sync::Arc;

use tw_ai::{
    production_desktop_router, AiProvider, AiRoutingMode, AiService, AiServiceImpl, AiTask,
    CompletionRequest, DocumentContext, GeminiProvider, LlamaCppProvider, MockHttpClient,
    OpenAiProvider, ProviderRouter, GEMINI, LLAMA_CPP, OPENAI,
};

fn openai_fixture() -> &'static str {
    r#"{
      "choices": [{"message": {"content": "cloud summary"}}],
      "usage": {"total_tokens": 42}
    }"#
}

fn gemini_fixture() -> &'static str {
    r#"{
      "candidates": [{"content": {"parts": [{"text": "gemini reply"}]}}],
      "usageMetadata": {"totalTokenCount": 11}
    }"#
}

fn llama_fixture() -> &'static str {
    r#"{
      "choices": [{"message": {"content": "local grammar fix"}}],
      "usage": {"total_tokens": 7}
    }"#
}

fn small_ctx() -> DocumentContext {
    DocumentContext {
        total_token_estimate: 500,
        page_count: 1,
        ..Default::default()
    }
}

fn large_ctx() -> DocumentContext {
    DocumentContext {
        total_token_estimate: 120_000,
        page_count: 200,
        ..Default::default()
    }
}

#[test]
fn u_f28_s1_hybrid_router_local() {
    let http = Arc::new(MockHttpClient::new());
    let router = production_desktop_router(
        http,
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    );
    let id = router.route(AiTask::Grammar, &small_ctx()).unwrap();
    assert_eq!(id, LLAMA_CPP, "grammar must prefer local llama");
}

#[test]
fn u_f28_s1_hybrid_router_cloud() {
    let http = Arc::new(MockHttpClient::new());
    let router = production_desktop_router(
        http,
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    );
    let id = router.route(AiTask::Summarize, &large_ctx()).unwrap();
    assert_eq!(id, OPENAI, "long summarize must prefer cloud OpenAI");
}

#[test]
fn u_f28_s1_openai_adapter_parses_response() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("https://api.openai.com/", openai_fixture());
    let provider = OpenAiProvider::new(http.clone(), "sk-test");
    let resp = provider
        .complete(&CompletionRequest {
            prompt: "Summarize this.".into(),
            system_prompt: Some("Be brief.".into()),
            max_tokens: 64,
            temperature: 0.2,
        })
        .unwrap();
    assert_eq!(resp.text, "cloud summary");
    assert_eq!(resp.tokens_used, 42);
    let call = &http.calls()[0];
    assert!(call
        .headers
        .iter()
        .any(|(k, v)| k == "Authorization" && v.contains("sk-test")));
}

#[test]
fn u_f28_s1_gemini_adapter_parses_response() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("https://generativelanguage.googleapis.com/", gemini_fixture());
    let provider = GeminiProvider::new(http, "gem-key");
    let resp = provider
        .complete(&CompletionRequest {
            prompt: "Hi".into(),
            system_prompt: None,
            max_tokens: 32,
            temperature: 0.5,
        })
        .unwrap();
    assert_eq!(resp.text, "gemini reply");
    assert_eq!(resp.tokens_used, 11);
}

#[test]
fn u_f28_s1_llama_adapter_local() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", llama_fixture());
    let provider = LlamaCppProvider::new(http, "http://127.0.0.1:11434");
    assert!(provider.capabilities().local);
    let resp = provider
        .complete(&CompletionRequest {
            prompt: "Fix grammar.".into(),
            system_prompt: None,
            max_tokens: 32,
            temperature: 0.0,
        })
        .unwrap();
    assert_eq!(resp.text, "local grammar fix");
}

#[test]
fn i_f28_s1_service_summarize_via_openai() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("https://api.openai.com/", openai_fixture());
    let hybrid = production_desktop_router(
        http,
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    );
    // Force cloud for summarize by using large context through ProviderRouter.
    let mut provider_router = ProviderRouter::new(OPENAI, LLAMA_CPP);
    *provider_router.hybrid_mut() = hybrid;
    let service = AiServiceImpl::new(provider_router);
    let mut ctx = large_ctx();
    ctx.selection = Some(tw_ai::ContextSelection {
        text: "Long document text…".into(),
        char_format_summary: String::new(),
        paragraph_style: None,
    });
    let reply = service.summarize(&ctx).unwrap();
    match reply {
        tw_ai::AiResponse::TextSuggestion { text } => assert_eq!(text, "cloud summary"),
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn i_f28_s1_service_grammar_via_llama() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", llama_fixture());
    let hybrid = production_desktop_router(
        http,
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    );
    let mut provider_router = ProviderRouter::new(OPENAI, LLAMA_CPP);
    *provider_router.hybrid_mut() = hybrid;
    let service = AiServiceImpl::new(provider_router);
    let mut ctx = small_ctx();
    ctx.selection = Some(tw_ai::ContextSelection {
        text: "This are wrong.".into(),
        char_format_summary: String::new(),
        paragraph_style: None,
    });
    let reply = service.correct_grammar(&ctx).unwrap();
    match reply {
        tw_ai::AiResponse::TextSuggestion { text } => assert_eq!(text, "local grammar fix"),
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn i_f28_s1_unavailable_without_keys() {
    let http = Arc::new(MockHttpClient::new());
    let router = production_desktop_router(http, None, None, Some("http://127.0.0.1:11434".into()));
    // Cloud providers unavailable without keys; local still available.
    assert!(!router.get_provider(OPENAI).unwrap().is_available());
    assert!(!router.get_provider(GEMINI).unwrap().is_available());
    assert!(router.get_provider(LLAMA_CPP).unwrap().is_available());
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f28_s1_router_and_complete_churn() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("https://api.openai.com/", openai_fixture());
    http.enqueue_prefix("http://127.0.0.1:11434/", llama_fixture());
    let hybrid = production_desktop_router(
        http.clone(),
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    );
    let mut provider_router = ProviderRouter::new(OPENAI, LLAMA_CPP);
    *provider_router.hybrid_mut() = hybrid;
    let service = AiServiceImpl::new(provider_router);
    for i in 0..300 {
        if i % 2 == 0 {
            let id = service
                .router()
                .hybrid()
                .route(AiTask::Grammar, &small_ctx())
                .unwrap();
            assert_eq!(id, LLAMA_CPP);
            let _ = service.correct_grammar(&small_ctx()).unwrap();
        } else {
            let id = service
                .router()
                .hybrid()
                .route(AiTask::Summarize, &large_ctx())
                .unwrap();
            assert_eq!(id, OPENAI);
            let _ = service.summarize(&large_ctx()).unwrap();
        }
    }
    assert!(http.calls().len() >= 300);
}

#[test]
fn u_f28_s1_always_local_mode() {
    let http = Arc::new(MockHttpClient::new());
    let router = production_desktop_router(
        http,
        Some("sk-test".into()),
        Some("gem-test".into()),
        Some("http://127.0.0.1:11434".into()),
    )
    .with_routing_mode(AiRoutingMode::AlwaysLocal);
    assert_eq!(
        router.route(AiTask::Summarize, &large_ctx()).unwrap(),
        LLAMA_CPP
    );
}

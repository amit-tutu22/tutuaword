//! F28.S2 — rewrite selection → DeleteRange / InsertText (undoable).

use std::sync::Arc;

use tw_ai::{
    apply_ai_response, apply_text_suggestion, production_desktop_router, suggestion_to_commands,
    AiResponse, AiService, AiServiceImpl, ContextSelection, DocumentContext, MockHttpClient,
    ProviderRouter, RewriteTone, LLAMA_CPP, OPENAI,
};
use tw_edit::{replace_run_range_commands, Command, EditSession};
use tw_model::NodeId;

fn llama_rewrite_fixture() -> &'static str {
    r#"{
      "choices": [{"message": {"content": "Rewritten prose."}}],
      "usage": {"total_tokens": 9}
    }"#
}

fn first_run_id(session: &EditSession) -> NodeId {
    session.document.paragraph_at(0, 0).unwrap().runs[0].id
}

fn run_text(session: &EditSession) -> String {
    session
        .document
        .paragraph_at(0, 0)
        .unwrap()
        .full_text()
}

fn service_with_llama(http: Arc<MockHttpClient>) -> AiServiceImpl {
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

#[test]
fn u_f28_s2_replace_commands_delete_then_insert() {
    let run = NodeId::new();
    let cmds = replace_run_range_commands(run, 0, 5, "Hello");
    assert_eq!(cmds.len(), 2);
    match &cmds[0] {
        Command::DeleteRange {
            run_id,
            start,
            end,
        } => {
            assert_eq!(*run_id, run);
            assert_eq!((*start, *end), (0, 5));
        }
        other => panic!("expected DeleteRange, got {other:?}"),
    }
    match &cmds[1] {
        Command::InsertText {
            run_id,
            offset,
            text,
        } => {
            assert_eq!(*run_id, run);
            assert_eq!(*offset, 0);
            assert_eq!(text, "Hello");
        }
        other => panic!("expected InsertText, got {other:?}"),
    }
}

#[test]
fn u_f28_s2_suggestion_to_commands() {
    let run = NodeId::new();
    let resp = AiResponse::TextSuggestion {
        text: "new".into(),
    };
    let cmds = suggestion_to_commands(run, 1, 4, &resp).unwrap();
    assert_eq!(cmds.len(), 2);
}

#[test]
fn u_f28_s2_apply_text_suggestion_replaces_run() {
    let mut session = EditSession::new();
    let run_id = first_run_id(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "hello world".into(),
        })
        .unwrap();
    let run_id = first_run_id(&session);
    apply_text_suggestion(&mut session, run_id, 0, 5, "HELLO").unwrap();
    assert_eq!(run_text(&session), "HELLO world");
}

#[test]
fn i_f28_s2_rewrite_selection() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", llama_rewrite_fixture());
    let service = service_with_llama(http);

    let mut session = EditSession::new();
    let run_id = first_run_id(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "original draft".into(),
        })
        .unwrap();
    let run_id = first_run_id(&session);
    let before = run_text(&session);
    assert_eq!(before, "original draft");

    let ctx = DocumentContext {
        selection: Some(ContextSelection {
            text: before.clone(),
            char_format_summary: String::new(),
            paragraph_style: None,
        }),
        total_token_estimate: 50,
        page_count: 1,
        ..Default::default()
    };
    let reply = service.rewrite(&ctx, RewriteTone::Formal).unwrap();
    apply_ai_response(&mut session, run_id, 0, before.chars().count(), &reply).unwrap();
    assert_eq!(run_text(&session), "Rewritten prose.");

    // Single undo restores pre-rewrite text (transaction groups delete+insert).
    session.undo().unwrap();
    assert_eq!(run_text(&session), "original draft");
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f28_s2_rewrite_apply_undo_churn() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix("http://127.0.0.1:11434/", llama_rewrite_fixture());
    let service = service_with_llama(http);

    let mut session = EditSession::new();
    let run_id = first_run_id(&session);
    session
        .apply(Command::InsertText {
            run_id,
            offset: 0,
            text: "seed".into(),
        })
        .unwrap();

    for i in 0..200 {
        let run_id = first_run_id(&session);
        let current = run_text(&session);
        let ctx = DocumentContext {
            selection: Some(ContextSelection {
                text: current.clone(),
                char_format_summary: String::new(),
                paragraph_style: None,
            }),
            total_token_estimate: 40,
            page_count: 1,
            ..Default::default()
        };
        let reply = service.rewrite(&ctx, RewriteTone::Neutral).unwrap();
        apply_ai_response(
            &mut session,
            run_id,
            0,
            current.chars().count(),
            &reply,
        )
        .unwrap();
        assert_eq!(run_text(&session), "Rewritten prose.");
        session.undo().unwrap();
        assert_eq!(run_text(&session), current, "iter {i}");
    }
}

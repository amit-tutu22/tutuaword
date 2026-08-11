//! F28.S5 — suggest diagram / table / timeline and insert as blocks.

use std::sync::Arc;

use tw_ai::{
    apply_visual_suggestion, parse_visual_suggestion, production_desktop_router, suggest_visual,
    visual_suggestion_commands, AiServiceImpl, DocumentContext, MockHttpClient, ProviderRouter,
    VisualKind, VisualSuggestion, LLAMA_CPP, OPENAI,
};
use tw_edit::{Command, EditSession};
use tw_model::{Block, DiagramKind, ShapeKind};

fn first_block_id(session: &EditSession) -> tw_model::NodeId {
    match &session.document.sections[0].blocks[0] {
        Block::Paragraph(p) => p.id,
        Block::Table(t) => t.id,
        Block::ImageBlock(i) => i.id,
        Block::ShapeBlock(s) => s.id,
        _ => panic!("unexpected block type"),
    }
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

#[test]
fn u_f28_s5_parse_table_suggestion() {
    let s = parse_visual_suggestion(
        r#"{"kind":"table","rows":4,"cols":2,"title":"Compare","rationale":"side by side"}"#,
    )
    .unwrap();
    assert_eq!(s.title, "Compare");
    match s.kind {
        VisualKind::Table { rows, cols } => {
            assert_eq!((rows, cols), (4, 2));
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn u_f28_s5_parse_diagram_and_timeline() {
    let d = parse_visual_suggestion(
        r#"{"kind":"diagram","diagram":"hierarchy","title":"Org","rationale":"structure"}"#,
    )
    .unwrap();
    match d.kind {
        VisualKind::Diagram { kind } => assert_eq!(kind, DiagramKind::Hierarchy),
        other => panic!("expected diagram, got {other:?}"),
    }

    let t = parse_visual_suggestion(
        r#"{"kind":"timeline","stages":["Plan","Build","Ship"],"title":"Roadmap","rationale":"phases"}"#,
    )
    .unwrap();
    match t.kind {
        VisualKind::Timeline { stages } => {
            assert_eq!(stages, vec!["Plan", "Build", "Ship"]);
        }
        other => panic!("expected timeline, got {other:?}"),
    }
}

#[test]
fn u_f28_s5_suggestion_to_commands() {
    let after = tw_model::NodeId::new();
    let table = VisualSuggestion {
        kind: VisualKind::Table { rows: 3, cols: 3 },
        title: "T".into(),
        rationale: String::new(),
    };
    let cmds = visual_suggestion_commands(after, &table);
    assert_eq!(cmds.len(), 1);
    assert!(matches!(
        cmds[0],
        Command::InsertTable {
            rows: 3,
            cols: 3,
            ..
        }
    ));

    let diagram = VisualSuggestion {
        kind: VisualKind::Diagram {
            kind: DiagramKind::Process,
        },
        title: "D".into(),
        rationale: String::new(),
    };
    assert!(matches!(
        visual_suggestion_commands(after, &diagram)[0],
        Command::InsertDiagram { .. }
    ));
}

#[test]
fn i_f28_s5_suggest_and_insert_table() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix(
        "http://127.0.0.1:11434/",
        r##"{"choices":[{"message":{"content":"{\"kind\":\"table\",\"rows\":2,\"cols\":3,\"title\":\"Grid\",\"rationale\":\"compare\"}"}}]}"##,
    );
    let service = service_with_http(http);
    let suggestion = suggest_visual(
        &service,
        "comparison matrix",
        &DocumentContext {
            total_token_estimate: 100,
            page_count: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(matches!(suggestion.kind, VisualKind::Table { .. }));

    let mut session = EditSession::new();
    let after = first_block_id(&session);
    apply_visual_suggestion(&mut session, after, &suggestion).unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 2);
    let table = session.document.sections[0].blocks[1].table().unwrap();
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0].cells.len(), 3);

    session.undo().unwrap();
    assert_eq!(session.document.sections[0].blocks.len(), 1);
}

#[test]
fn i_f28_s5_insert_diagram_and_timeline_blocks() {
    let mut session = EditSession::new();
    let after = first_block_id(&session);
    let diagram = VisualSuggestion {
        kind: VisualKind::Diagram {
            kind: DiagramKind::Cycle,
        },
        title: "Cycle".into(),
        rationale: "loop".into(),
    };
    apply_visual_suggestion(&mut session, after, &diagram).unwrap();
    let shape = session.document.sections[0].blocks[1].shape().unwrap();
    assert_eq!(shape.shape.shape_type, ShapeKind::Diagram);
    assert_eq!(shape.diagram_kind, DiagramKind::Cycle);

    let after = session.document.sections[0].blocks[1]
        .shape()
        .unwrap()
        .id;
    let timeline = VisualSuggestion {
        kind: VisualKind::Timeline {
            stages: vec!["A".into(), "B".into(), "C".into()],
        },
        title: "TL".into(),
        rationale: String::new(),
    };
    apply_visual_suggestion(&mut session, after, &timeline).unwrap();
    let table = session.document.sections[0].blocks[2].table().unwrap();
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0].cells.len(), 3);
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn s_f28_s5_suggest_insert_undo_churn() {
    let http = Arc::new(MockHttpClient::new());
    http.enqueue_prefix(
        "http://127.0.0.1:11434/",
        r##"{"choices":[{"message":{"content":"{\"kind\":\"timeline\",\"stages\":[\"A\",\"B\"],\"title\":\"T\",\"rationale\":\"r\"}"}}]}"##,
    );
    let service = service_with_http(http);
    let mut session = EditSession::new();
    for i in 0..120 {
        let suggestion = suggest_visual(
            &service,
            &format!("topic-{i}"),
            &DocumentContext::default(),
        )
        .unwrap();
        let after = first_block_id(&session);
        let before_len = session.document.sections[0].blocks.len();
        apply_visual_suggestion(&mut session, after, &suggestion).unwrap();
        assert_eq!(
            session.document.sections[0].blocks.len(),
            before_len + 1,
            "iter {i}"
        );
        session.undo().unwrap();
        assert_eq!(
            session.document.sections[0].blocks.len(),
            before_len,
            "undo iter {i}"
        );
    }
}

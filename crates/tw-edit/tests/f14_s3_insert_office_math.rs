//! F14.S3 — insert and edit OfficeMath commands.

use tw_edit::{Command, EditSession};
use tw_model::{RunContent, build_display_omath_para, build_inline_omath, omml_run};

#[test]
fn u_f14_s3_insert_inline_office_math() {
    let mut session = EditSession::from_document(tw_model::Document::new());
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    let xml = build_inline_omath(&omml_run("E=mc2"));

    session
        .apply(Command::InsertOfficeMath {
            run_id,
            offset: 0,
            xml: xml.clone(),
        })
        .unwrap();

    let run = &session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0];
    let RunContent::OfficeMath { xml: stored } = &run.content else {
        panic!("expected OfficeMath run");
    };
    assert!(stored.contains("E=mc2"));
}

#[test]
fn u_f14_s3_insert_display_office_math() {
    let mut session = EditSession::from_document(tw_model::Document::new());
    let after = session.document.sections[0].blocks[0].paragraph().unwrap().id;
    let xml = build_display_omath_para("x+y");

    session
        .apply(Command::InsertOfficeMathDisplay {
            after_block_id: after,
            xml: xml.clone(),
        })
        .unwrap();

    assert_eq!(session.document.sections[0].blocks.len(), 2);
    let para = session.document.sections[0].blocks[1].paragraph().unwrap();
    let RunContent::OfficeMath { xml: stored } = &para.runs[0].content else {
        panic!("expected OfficeMath run");
    };
    assert!(stored.contains("<m:oMathPara"));
    assert!(stored.contains("x+y"));
}

#[test]
fn u_f14_s3_set_office_math_undo() {
    let mut session = EditSession::from_document(tw_model::Document::new());
    let run_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .id;
    let first = build_inline_omath(&omml_run("a"));
    let second = build_inline_omath(&omml_run("b"));

    session
        .apply(Command::InsertOfficeMath {
            run_id,
            offset: 0,
            xml: first,
        })
        .unwrap();
    session
        .apply(Command::SetOfficeMath {
            run_id,
            xml: second,
        })
        .unwrap();
    session.undo().unwrap();

    let RunContent::OfficeMath { xml } = &session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .runs[0]
        .content
    else {
        panic!("expected OfficeMath");
    };
    assert!(xml.contains(">a<"));
}

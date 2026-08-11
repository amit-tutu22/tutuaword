use tw_edit::{
    adjust_list_level_command_for, apply, bullet_list_command_for, numbered_list_command_for,
    EditSession,
};
use tw_model::NumberingRef;

#[test]
fn promote_list_level_increments_ilvl() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    apply(&mut session.document, bullet_list_command_for(para_id)).unwrap();

    let cmd = adjust_list_level_command_for(&session.document, para_id, 1).unwrap();
    apply(&mut session.document, cmd).unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(
        para.format.numbering,
        Some(NumberingRef {
            numbering_id: 1,
            level: 1,
        })
    );
}

#[test]
fn demote_at_level_zero_is_no_op() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    apply(&mut session.document, bullet_list_command_for(para_id)).unwrap();

    assert!(adjust_list_level_command_for(&session.document, para_id, -1).is_none());
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().format.numbering,
        Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        })
    );
}

#[test]
fn promote_at_max_level_is_no_op() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    apply(&mut session.document, bullet_list_command_for(para_id)).unwrap();
    let promote = adjust_list_level_command_for(&session.document, para_id, 1).unwrap();
    apply(&mut session.document, promote).unwrap();
    assert!(adjust_list_level_command_for(&session.document, para_id, 1).is_none());

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.format.numbering.unwrap().level, 1);
}

#[test]
fn numbered_list_promote_sets_outline_level() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    apply(&mut session.document, numbered_list_command_for(para_id)).unwrap();
    assert_eq!(
        session.document.paragraph_at(0, 0).unwrap().format.outline_level,
        Some(0)
    );

    let promote = adjust_list_level_command_for(&session.document, para_id, 1).unwrap();
    apply(&mut session.document, promote).unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.format.numbering.unwrap().level, 1);
    assert_eq!(para.format.outline_level, Some(1));
}

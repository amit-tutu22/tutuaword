use tw_edit::{
    apply, bullet_list_command_for, continue_numbering_command_for, restart_numbering_command_for,
    EditSession,
};
use tw_model::NumberingRef;

#[test]
fn restart_numbering_sets_num_restart_flag() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    apply(&mut session.document, bullet_list_command_for(para_id)).unwrap();
    apply(
        &mut session.document,
        restart_numbering_command_for(para_id),
    )
    .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.format.num_restart, Some(true));
    assert_eq!(
        para.format.numbering,
        Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        }),
        "restart should not clear numbering"
    );
}

#[test]
fn continue_numbering_clears_num_restart_flag() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    apply(&mut session.document, bullet_list_command_for(para_id)).unwrap();
    apply(
        &mut session.document,
        restart_numbering_command_for(para_id),
    )
    .unwrap();
    apply(
        &mut session.document,
        continue_numbering_command_for(para_id),
    )
    .unwrap();

    let para = session.document.paragraph_at(0, 0).unwrap();
    assert_eq!(para.format.num_restart, None);
    assert_eq!(
        para.format.numbering,
        Some(NumberingRef {
            numbering_id: 1,
            level: 0,
        })
    );
}

#[test]
fn restart_without_listing_returns_invalid_range() {
    let mut session = EditSession::new();
    let para_id = session.document.sections[0].blocks[0]
        .paragraph()
        .unwrap()
        .id;

    let err = apply(
        &mut session.document,
        restart_numbering_command_for(para_id),
    )
    .unwrap_err();
    assert!(matches!(err, tw_edit::EditError::InvalidRange));
}

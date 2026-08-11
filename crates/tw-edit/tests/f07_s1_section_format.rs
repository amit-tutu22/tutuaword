//! F07.S1 — section format edit command.

use tw_edit::{Command, EditSession};
use tw_model::SectionFormat;

#[test]
fn u_f07_s1_set_section_format_updates_margins() {
    let mut session = EditSession::new();
    let mut patch = SectionFormat::default();
    patch.margin_left = 36.0;
    patch.margin_right = 36.0;

    session
        .apply(Command::SetSectionFormat {
            section_index: 0,
            format: patch,
        })
        .expect("apply narrow margins");

    let format = &session.document.sections[0].format;
    assert_eq!(format.margin_left, 36.0);
    assert_eq!(format.margin_right, 36.0);
    assert_eq!(format.page_width, 612.0);
}

#[test]
fn u_f07_s1_set_section_format_undo_restores_previous() {
    let mut session = EditSession::new();
    assert_eq!(session.document.sections[0].format.margin_left, 72.0);

    let mut patch = SectionFormat::default();
    patch.margin_left = 36.0;
    session
        .apply(Command::SetSectionFormat {
            section_index: 0,
            format: patch,
        })
        .expect("apply");
    assert_eq!(session.document.sections[0].format.margin_left, 36.0);

    session.undo().expect("undo");
    assert_eq!(session.document.sections[0].format.margin_left, 72.0);
}

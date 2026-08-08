//! F08.S3 — section/document flags for first page and odd/even headers.

use tw_edit::{Command, EditSession};
use tw_model::SectionFormat;

#[test]
fn u_f08_s3_set_different_first_page_via_section_format() {
    let mut session = EditSession::new();
    let mut patch = SectionFormat::default();
    patch.different_first_page = true;
    session
        .apply(Command::SetSectionFormat {
            section_index: 0,
            format: patch,
        })
        .unwrap();
    assert!(session.document.sections[0].format.different_first_page);
}

#[test]
fn u_f08_s3_set_even_and_odd_headers_command() {
    let mut session = EditSession::new();
    session
        .apply(Command::SetEvenAndOddHeaders { enabled: true })
        .unwrap();
    assert!(session.document.settings.even_and_odd_headers);

    session.undo().unwrap();
    assert!(!session.document.settings.even_and_odd_headers);
}

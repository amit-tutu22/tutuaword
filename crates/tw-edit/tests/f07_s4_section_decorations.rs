//! F07.S4 — section decoration fields via SetSectionFormat.

use tw_edit::{Command, EditSession};
use tw_model::{Color, LineNumberSettings, SectionFormat, WatermarkSettings};

fn decoration_patch() -> SectionFormat {
    let mut patch = SectionFormat::default();
    patch.page_color = Some(Color {
        r: 240,
        g: 240,
        b: 240,
        a: 255,
    });
    patch.watermark = Some(WatermarkSettings::new("CONFIDENTIAL"));
    patch.line_numbers = LineNumberSettings {
        enabled: true,
        start: 1,
    };
    patch
}

/// U-F07-S4-set-page-color-and-watermark
#[test]
fn u_f07_s4_set_page_color_and_watermark() {
    let mut session = EditSession::new();
    session
        .apply(Command::SetSectionFormat {
            section_index: 0,
            format: decoration_patch(),
        })
        .expect("apply decorations");

    let format = &session.document.sections[0].format;
    assert_eq!(format.page_color, Some(Color { r: 240, g: 240, b: 240, a: 255 }));
    assert_eq!(
        format.watermark,
        Some(WatermarkSettings::new("CONFIDENTIAL"))
    );
    assert_eq!(
        format.line_numbers,
        LineNumberSettings {
            enabled: true,
            start: 1,
        }
    );
}

#[test]
fn u_f07_s4_decorations_preserve_header() {
    let mut session = EditSession::new();
    session.document.sections[0].format.header_text = Some("My Header".into());

    session
        .apply(Command::SetSectionFormat {
            section_index: 0,
            format: decoration_patch(),
        })
        .expect("apply decorations");

    let format = &session.document.sections[0].format;
    assert_eq!(format.header_text.as_deref(), Some("My Header"));
    assert!(format.page_color.is_some());
}

#[test]
fn u_f07_s4_decorations_undo() {
    let mut session = EditSession::new();
    assert!(session.document.sections[0].format.page_color.is_none());

    let mut patch = SectionFormat::default();
    patch.page_color = Some(Color::BLACK);
    session
        .apply(Command::SetSectionFormat {
            section_index: 0,
            format: patch,
        })
        .expect("apply");
    assert_eq!(session.document.sections[0].format.page_color, Some(Color::BLACK));

    session.undo().expect("undo");
    assert!(session.document.sections[0].format.page_color.is_none());
}

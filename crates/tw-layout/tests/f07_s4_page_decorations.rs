//! F07.S4 — page color, watermark, and line numbers.

#[path = "f07_s4_layout_helpers/mod.rs"]
mod helpers;

use helpers::{
    content_line_count, first_content_line_index, full_page_rect_index, gutter_line_count,
    layout, watermark_band_rect_index,
};
use tw_layout::LayoutBox;
use tw_model::{Color, LineNumberSettings, SectionFormat, WatermarkSettings};

/// U-F07-S4-page-color-rect
#[test]
fn u_f07_s4_page_color_rect() {
    let yellow = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
    let mut format = SectionFormat::default();
    format.page_color = Some(yellow);

    let layout = layout(format, "Hello page color");
    let page = &layout.pages[0];
    let margin_left = SectionFormat::default().margin_left;

    let color_idx = full_page_rect_index(page, yellow.to_argb())
        .expect("full-page color rect");
    let body_idx = first_content_line_index(page, margin_left)
        .expect("body text line");
    assert!(
        color_idx < body_idx,
        "page color rect should paint before body text (color={color_idx}, body={body_idx})"
    );
}

/// U-F07-S4-watermark-background
#[test]
fn u_f07_s4_watermark_background() {
    let mut format = SectionFormat::default();
    format.watermark = Some(WatermarkSettings::new("DRAFT"));

    let layout = layout(format.clone(), "Body text");
    let page = &layout.pages[0];
    let margin_left = format.margin_left;

    let wm_idx = watermark_band_rect_index(page, &format).expect("watermark band rect");
    let body_idx = first_content_line_index(page, margin_left).expect("body text line");
    assert!(wm_idx < body_idx, "watermark rect should paint before body text");

    let wm_text_before_body = page.boxes.iter().any(|b| match b {
        LayoutBox::TextLine(line) => {
            line.glyphs.iter().any(|g| g.codepoint == 'D')
                && (line.x - format.watermark_band().0).abs() < 2.0
        }
        _ => false,
    });
    assert!(wm_text_before_body, "watermark label should layout in the band");
}

/// U-F07-S4-line-number-gutter
#[test]
fn u_f07_s4_line_number_gutter() {
    let mut format = SectionFormat::default();
    format.line_numbers = LineNumberSettings {
        enabled: true,
        start: 1,
    };

    let text = "word ".repeat(80);
    let layout = layout(format.clone(), text.trim());
    let page = &layout.pages[0];
    let gutter_x = format.line_number_gutter_x();
    let body_lines = content_line_count(page, format.margin_left);
    let gutter_lines = gutter_line_count(page, gutter_x);

    assert!(body_lines >= 2, "fixture should wrap to multiple body lines");
    assert_eq!(
        gutter_lines, body_lines,
        "each body line should get one gutter number"
    );
}

/// U-F07-S4-line-numbers-continue
#[test]
fn u_f07_s4_line_numbers_continue() {
    let mut format = SectionFormat::default();
    format.page_height = 220.0;
    format.line_numbers = LineNumberSettings {
        enabled: true,
        start: 1,
    };

    let text = "word ".repeat(200);
    let layout = layout(format.clone(), text.trim());
    assert!(
        layout.pages.len() >= 2,
        "short page should paginate for continuation test"
    );

    let page0_gutter = gutter_line_count(&layout.pages[0], format.line_number_gutter_x());
    let page1_gutter = gutter_line_count(&layout.pages[1], format.line_number_gutter_x());
    assert!(page0_gutter >= 1 && page1_gutter >= 1);

    let total_gutter: usize = layout
        .pages
        .iter()
        .map(|p| gutter_line_count(p, format.line_number_gutter_x()))
        .sum();
    let total_body: usize = layout
        .pages
        .iter()
        .map(|p| content_line_count(p, format.margin_left))
        .sum();
    assert_eq!(total_gutter, total_body);

    let next_label = (page0_gutter + 1).to_string();
    let continues_on_page_two = layout.pages[1].boxes.iter().any(|b| match b {
        LayoutBox::TextLine(line) if (line.x - format.line_number_gutter_x()).abs() < 2.0 => {
            let label: String = line.glyphs.iter().map(|g| g.codepoint).collect();
            label == next_label
        }
        _ => false,
    });
    assert!(
        continues_on_page_two,
        "page 2 gutter should start at line {}",
        next_label
    );
}

#[test]
fn u_f07_s4_line_numbers_disabled_no_gutter() {
    let format = SectionFormat::default();
    let layout = layout(format.clone(), "word ".repeat(40).trim());
    assert_eq!(
        gutter_line_count(&layout.pages[0], format.line_number_gutter_x()),
        0
    );
}

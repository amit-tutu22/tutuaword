//! F08.S2 — field evaluation helpers.

use chrono::{TimeZone, Utc};
use tw_model::{
    FieldData, FieldEvalContext, FieldType, evaluate_field,
};

#[test]
fn u_f08_s2_page_field_increments() {
    let field = FieldData {
        field_type: FieldType::Page,
        instruction: Some(" PAGE ".into()),
        display_text: None,
        form: None,
            merge_name: None,
    };
    assert_eq!(
        evaluate_field(&field, &FieldEvalContext::for_page(0, 2)),
        "1"
    );
    assert_eq!(
        evaluate_field(&field, &FieldEvalContext::for_page(1, 2)),
        "2"
    );
}

#[test]
fn u_f08_s2_date_field_formats() {
    let field = FieldData {
        field_type: FieldType::Date,
        instruction: None,
        display_text: None,
        form: None,
            merge_name: None,
    };
    let fixed = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap();
    let ctx = FieldEvalContext::for_page(0, 1).with_fixed_datetime(fixed);
    assert_eq!(evaluate_field(&field, &ctx), "January 15, 2026");
}

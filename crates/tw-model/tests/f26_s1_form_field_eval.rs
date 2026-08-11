//! F26.S1 — form field evaluation helpers.

use tw_model::{
    evaluate_field, form_checkbox_field_data, form_text_field_data, FieldEvalContext,
};

#[test]
fn u_f26_s1_form_text_eval() {
    let field = form_text_field_data(Some("n".into()), "hello");
    assert_eq!(
        evaluate_field(&field, &FieldEvalContext::for_page(0, 1)),
        "hello"
    );
}

#[test]
fn u_f26_s1_form_checkbox_eval() {
    let unchecked = form_checkbox_field_data(None, false);
    assert_eq!(
        evaluate_field(&unchecked, &FieldEvalContext::for_page(0, 1)),
        "☐"
    );
    let checked = form_checkbox_field_data(None, true);
    assert_eq!(
        evaluate_field(&checked, &FieldEvalContext::for_page(0, 1)),
        "☑"
    );
}

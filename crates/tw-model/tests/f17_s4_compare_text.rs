//! F17.S4 — document/text comparison scenarios.

use tw_model::{
    compare_text, CompareChangeKind, DocumentCompareSummary,
};

#[test]
fn u_f17_s4_identical_text_has_no_changes() {
    let summary = compare_text("Hello\nWorld", "Hello\nWorld");
    assert_eq!(summary.insertion_count, 0);
    assert_eq!(summary.deletion_count, 0);
    assert!(summary.changes.iter().all(|c| c.kind == CompareChangeKind::Equal));
}

#[test]
fn u_f17_s4_detects_line_insert_and_delete() {
    let left = "Alpha\nBeta\nGamma";
    let right = "Alpha\nDelta\nGamma";
    let DocumentCompareSummary {
        insertion_count,
        deletion_count,
        changes,
    } = compare_text(left, right);
    assert_eq!(insertion_count, 1);
    assert_eq!(deletion_count, 1);
    assert!(changes.iter().any(|c| c.kind == CompareChangeKind::Delete && c.text == "Beta"));
    assert!(changes.iter().any(|c| c.kind == CompareChangeKind::Insert && c.text == "Delta"));
}

#[test]
fn u_f17_s4_empty_vs_nonempty() {
    let summary = compare_text("", "Only line");
    assert_eq!(summary.insertion_count, 1);
    assert_eq!(summary.deletion_count, 0);
}

#[test]
fn u_f17_s4_multiline_reorder_counts_changes() {
    let left = "One\nTwo\nThree";
    let right = "One\nThree\nTwo";
    let summary = compare_text(left, right);
    assert!(summary.insertion_count >= 1);
    assert!(summary.deletion_count >= 1);
}

#[test]
fn u_f17_s4_large_line_count_uses_bounded_diff() {
    // Product of line counts exceeds the LCS table budget (250_000).
    let left: Vec<String> = (0..600).map(|i| format!("L{i}")).collect();
    let right: Vec<String> = (0..600).map(|i| format!("R{i}")).collect();
    let left_joined = left.join("\n");
    let right_joined = right.join("\n");
    let summary = compare_text(&left_joined, &right_joined);
    assert_eq!(summary.deletion_count, 600);
    assert_eq!(summary.insertion_count, 600);
}

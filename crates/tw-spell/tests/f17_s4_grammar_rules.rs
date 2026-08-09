//! F17.S4 — rule-based grammar checker scenarios.

use tw_spell::{GrammarChecker, GrammarIssue};

#[test]
fn u_f17_s4_flags_modal_verb_of_errors() {
    let checker = GrammarChecker::english();
    let issues = checker.check_text("I could of won if I should of tried and would of finished.");
    let messages: Vec<_> = issues.iter().map(|i| i.message.as_str()).collect();
    assert!(messages.iter().any(|m| m.contains("could have")));
    assert!(messages.iter().any(|m| m.contains("should have")));
    assert!(messages.iter().any(|m| m.contains("would have")));
}

#[test]
fn u_f17_s4_flags_common_homophone_mistakes() {
    let checker = GrammarChecker::english();
    let issues = checker.check_text("Their is a problem. Your welcome. Its a nice day.");
    let messages: Vec<_> = issues.iter().map(|i| i.message.as_str()).collect();
    assert!(messages.iter().any(|m| m.contains("there is")));
    assert!(messages.iter().any(|m| m.contains("you're welcome")));
    assert!(messages.iter().any(|m| m.contains("it's a")));
}

#[test]
fn u_f17_s4_flags_alot_and_double_spaces() {
    let checker = GrammarChecker::english();
    let issues = checker.check_text("I ate alot  of cookies.");
    assert!(issues.iter().any(|i| i.message.contains("a lot")));
    assert!(issues.iter().any(|i| i.message.contains("extra space")));
}

#[test]
fn u_f17_s4_clean_prose_has_no_issues() {
    let checker = GrammarChecker::english();
    assert!(checker
        .check_text("The team could have finished if they had tried.")
        .is_empty());
}

#[test]
fn u_f17_s4_issues_include_suggestions_and_offsets() {
    let checker = GrammarChecker::english();
    let text = "We could of left earlier.";
    let issues = checker.check_text(text);
    let issue = issues
        .iter()
        .find(|i| i.message.contains("could have"))
        .expect("expected could-of issue");
    assert_eq!(issue.suggestion.as_deref(), Some(" could have "));
    assert!(issue.start < issue.end);
    assert!(issue.end <= text.len());
    let _span: GrammarIssue = issue.clone();
}

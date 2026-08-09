//! F17.S1 — Hunspell-compatible suggestions and misspelling detection.

use tw_spell::SpellChecker;

#[test]
fn u_f17_s1_hunspell_suggestions() {
    let checker = SpellChecker::english();

    assert!(!checker.is_correct("teh"));
    assert!(!checker.is_correct("recieved"));

    let teh_suggestions = checker.suggest("teh", 5);
    assert_eq!(
        teh_suggestions.first().map(String::as_str),
        Some("the"),
        "top suggestion for 'teh' should be 'the', got {teh_suggestions:?}"
    );

    let received_suggestions = checker.suggest("recieved", 5);
    assert!(
        received_suggestions.iter().any(|s| s == "received"),
        "expected 'received' in suggestions for 'recieved', got {received_suggestions:?}"
    );

    let issues = checker.check_text("Teh quikc brown fox recieved an invitaion.");
    let words: Vec<_> = issues.iter().map(|i| i.word.as_str()).collect();
    assert!(words.contains(&"Teh"));
    assert!(words.contains(&"quikc"));
    assert!(words.contains(&"recieved"));
    assert!(words.contains(&"invitaion"));

    let teh_issue = issues.iter().find(|i| i.word == "Teh").expect("Teh issue");
    assert!(
        teh_issue.suggestions.first().map(String::as_str) == Some("the"),
        "issue suggestions: {:?}",
        teh_issue.suggestions
    );
}

#[test]
fn u_f17_s1_known_good_words_pass() {
    let checker = SpellChecker::english();
    assert!(checker.check_text("The document editor received an invitation.").is_empty());
}

#[test]
fn u_f17_s1_suggest_respects_limit() {
    let checker = SpellChecker::english();
    let suggestions = checker.suggest("teh", 2);
    assert!(suggestions.len() <= 2);
}

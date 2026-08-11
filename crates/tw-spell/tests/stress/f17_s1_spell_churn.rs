//! Stress: repeated spell check over long typo-heavy documents.

use tw_spell::SpellChecker;

fn typo_paragraph(seed: usize) -> String {
    let mut text = String::from("Teh document ");
    for i in 0..200 {
        text.push_str(&format!("wrds{seed}x{i} "));
    }
    text.push_str("recieved invitaion.");
    text
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f17_s1_spell_check_churn() {
    let checker = SpellChecker::english();
    for seed in 0..50 {
        let text = typo_paragraph(seed);
        let issues = checker.check_text(&text);
        assert!(
            issues.len() >= 3,
            "seed {seed}: expected multiple issues, got {}",
            issues.len()
        );
        for issue in &issues {
            if issue.word.eq_ignore_ascii_case("teh") {
                assert!(
                    issue.suggestions.iter().any(|s| s == "the"),
                    "Teh should suggest 'the': {:?}",
                    issue.suggestions
                );
            }
        }
    }
}

#[test]
#[ignore = "stress: run locally or on nightly CI"]
fn stress_f17_s1_suggest_hot_path() {
    let checker = SpellChecker::english();
    for _ in 0..5_000 {
        let suggestions = checker.suggest("recieved", 5);
        assert!(suggestions.iter().any(|s| s == "received"));
    }
}

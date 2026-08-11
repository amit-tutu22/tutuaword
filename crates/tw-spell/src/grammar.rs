//! Lightweight rule-based grammar checks (F17.S4).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarIssue {
    pub message: String,
    pub start: usize,
    pub end: usize,
    pub suggestion: Option<String>,
}

struct GrammarRule {
    needle: &'static str,
    message: &'static str,
    suggestion: Option<&'static str>,
}

const PHRASE_RULES: &[GrammarRule] = &[
    GrammarRule {
        needle: " could of ",
        message: "Use \"could have\" instead of \"could of\"",
        suggestion: Some(" could have "),
    },
    GrammarRule {
        needle: " should of ",
        message: "Use \"should have\" instead of \"should of\"",
        suggestion: Some(" should have "),
    },
    GrammarRule {
        needle: " would of ",
        message: "Use \"would have\" instead of \"would of\"",
        suggestion: Some(" would have "),
    },
    GrammarRule {
        needle: " alot ",
        message: "Use \"a lot\" instead of \"alot\"",
        suggestion: Some(" a lot "),
    },
    GrammarRule {
        needle: " their is ",
        message: "Did you mean \"there is\"?",
        suggestion: Some(" there is "),
    },
    GrammarRule {
        needle: " their are ",
        message: "Did you mean \"there are\"?",
        suggestion: Some(" there are "),
    },
    GrammarRule {
        needle: " your welcome",
        message: "Did you mean \"you're welcome\"?",
        suggestion: Some(" you're welcome"),
    },
    GrammarRule {
        needle: " its a ",
        message: "Did you mean \"it's a\"?",
        suggestion: Some(" it's a "),
    },
];

pub struct GrammarChecker;

impl GrammarChecker {
    pub fn english() -> Self {
        Self
    }

    pub fn check_text(&self, text: &str) -> Vec<GrammarIssue> {
        let mut issues = Vec::new();
        let padded = format!(" {text} ");
        let lower = padded.to_ascii_lowercase();
        for rule in PHRASE_RULES {
            let mut search_from = 0;
            while let Some(rel) = lower[search_from..].find(rule.needle) {
                let start = search_from + rel;
                let end = start + rule.needle.len();
                issues.push(GrammarIssue {
                    message: rule.message.into(),
                    start: start.saturating_sub(1),
                    end: end.saturating_sub(1).min(text.len()),
                    suggestion: rule.suggestion.map(str::to_string),
                });
                search_from = end;
            }
        }
        issues.extend(check_double_spaces(text));
        issues.sort_by_key(|issue| issue.start);
        issues
    }
}

fn check_double_spaces(text: &str) -> Vec<GrammarIssue> {
    let mut issues = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b' ' && bytes[i + 1] == b' ' {
            let start = i;
            while i + 1 < bytes.len() && bytes[i] == b' ' && bytes[i + 1] == b' ' {
                i += 1;
            }
            issues.push(GrammarIssue {
                message: "Remove extra space".into(),
                start,
                end: (i + 1).min(text.len()),
                suggestion: Some(" ".to_string()),
            });
        }
        i += 1;
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_could_of() {
        let checker = GrammarChecker::english();
        let issues = checker.check_text("I could of done better.");
        assert!(issues.iter().any(|i| i.message.contains("could have")));
    }

    #[test]
    fn flags_double_space() {
        let checker = GrammarChecker::english();
        let issues = checker.check_text("Hello  world");
        assert!(issues.iter().any(|i| i.message.contains("extra space")));
    }

    #[test]
    fn clean_text_has_no_issues() {
        let checker = GrammarChecker::english();
        assert!(checker
            .check_text("The document editor works well.")
            .is_empty());
    }
}

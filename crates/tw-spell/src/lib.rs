use std::collections::HashSet;

use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellIssue {
    pub word: String,
    pub start: usize,
    pub end: usize,
}

pub struct SpellChecker {
    words: HashSet<String>,
    locale: String,
}

impl SpellChecker {
    pub fn english() -> Self {
        let mut words: HashSet<String> = ENGLISH_CORE
            .split_whitespace()
            .map(|w| w.to_ascii_lowercase())
            .collect();
        words.extend(
            INDIC_LATIN_TOKENS
                .split_whitespace()
                .map(|w| w.to_ascii_lowercase()),
        );
        Self {
            words,
            locale: "en".into(),
        }
    }

    pub fn with_extra_words(words: impl IntoIterator<Item = String>) -> Self {
        let mut checker = Self::english();
        checker.words.extend(words.into_iter().map(|w| w.to_ascii_lowercase()));
        checker
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn is_correct(&self, word: &str) -> bool {
        let normalized = normalize_word(word);
        if normalized.is_empty() || normalized.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
        self.words.contains(&normalized)
    }

    pub fn check_text(&self, text: &str) -> Vec<SpellIssue> {
        let mut issues = Vec::new();
        let mut offset = 0usize;
        for word in text.split_word_bounds() {
            let len = word.len();
            if is_word_token(word) && !self.is_correct(word) {
                issues.push(SpellIssue {
                    word: word.to_string(),
                    start: offset,
                    end: offset + len,
                });
            }
            offset += len;
        }
        issues
    }
}

fn normalize_word(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_ascii_lowercase()
}

fn is_word_token(word: &str) -> bool {
    word.chars().any(|c| c.is_alphabetic())
}

const ENGLISH_CORE: &str = include_str!("../data/en_core.txt");

const INDIC_LATIN_TOKENS: &str = "namaste hindi tamil telugu bengali marathi gujarati kannada malayalam punjabi urdu india delhi mumbai chennai kolkata";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_known_misspellings() {
        let checker = SpellChecker::english();
        let issues = checker.check_text(
            "Teh quick brown fox recieved an invitation to teh party.",
        );
        let words: Vec<_> = issues.iter().map(|i| i.word.as_str()).collect();
        assert!(words.contains(&"Teh"));
        assert!(words.contains(&"recieved"));
    }

    #[test]
    fn accepts_valid_words() {
        let checker = SpellChecker::english();
        assert!(checker.check_text("The document editor works well.").is_empty());
    }

    #[test]
    fn accepts_indic_latin_tokens() {
        let checker = SpellChecker::english();
        assert!(checker.is_correct("namaste"));
        assert!(checker.is_correct("Chennai"));
    }

    #[test]
    fn check_text_reports_offsets() {
        let checker = SpellChecker::english();
        let issues = checker.check_text("Teh document");
        assert!(!issues.is_empty());
        assert_eq!(issues[0].word, "Teh");
        assert_eq!(issues[0].start, 0);
        assert_eq!(issues[0].end, 3);
    }

    #[test]
    fn with_extra_words_extends_dictionary() {
        let checker = SpellChecker::with_extra_words(["tutuaword".into()]);
        assert!(checker.is_correct("tutuaword"));
    }

    #[test]
    fn ignores_numbers_and_punctuation_only_tokens() {
        let checker = SpellChecker::english();
        assert!(checker.check_text("2026 — 100%").is_empty());
    }
}

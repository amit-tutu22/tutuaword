mod suggest;
mod grammar;

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use suggest::rank_suggestions;
use unicode_segmentation::UnicodeSegmentation;

pub use grammar::{GrammarChecker, GrammarIssue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellIssue {
    pub word: String,
    pub start: usize,
    pub end: usize,
    pub suggestions: Vec<String>,
}

/// Embedded English spell checker with Hunspell-compatible `check` + `suggest` API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellChecker {
    words: HashSet<String>,
    word_rank: HashMap<String, usize>,
    locale: String,
}

impl SpellChecker {
    pub fn english() -> Self {
        Self::english_cached().clone()
    }

    /// Shared core English dictionary (avoids rebuilding on every call).
    pub fn english_cached() -> &'static SpellChecker {
        static CHECKER: OnceLock<SpellChecker> = OnceLock::new();
        CHECKER.get_or_init(|| Self::from_word_list(ENGLISH_CORE, "en"))
    }

    /// Lazy-loaded en_US dictionary (embedded Hunspell-style word list).
    pub fn english_us() -> &'static SpellChecker {
        static CHECKER: OnceLock<SpellChecker> = OnceLock::new();
        CHECKER.get_or_init(|| Self::from_word_list(ENGLISH_US, "en_US"))
    }

    fn from_word_list(raw: &str, locale: &str) -> Self {
        let mut words = HashSet::new();
        let mut word_rank = HashMap::new();
        for (rank, token) in raw.split_whitespace().enumerate() {
            let word = token.to_ascii_lowercase();
            words.insert(word.clone());
            word_rank.entry(word).or_insert(rank);
        }
        for token in INDIC_LATIN_TOKENS.split_whitespace() {
            let word = token.to_ascii_lowercase();
            words.insert(word.clone());
            word_rank.entry(word).or_insert(usize::MAX);
        }
        Self {
            words,
            word_rank,
            locale: locale.into(),
        }
    }

    pub fn with_extra_words(words: impl IntoIterator<Item = String>) -> Self {
        let mut checker = Self::english_cached().clone();
        for word in words {
            let normalized = word.to_ascii_lowercase();
            checker.words.insert(normalized.clone());
            checker.word_rank.entry(normalized).or_insert(usize::MAX);
        }
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

    /// Hunspell-style suggestions for a misspelled token.
    pub fn suggest(&self, word: &str, limit: usize) -> Vec<String> {
        let normalized = normalize_word(word);
        if normalized.is_empty() || self.is_correct(word) {
            return Vec::new();
        }
        rank_suggestions(
            &normalized,
            self.words.iter().map(String::as_str),
            limit,
            2,
            |candidate| self.word_rank.get(candidate).copied().unwrap_or(usize::MAX),
        )
    }

    pub fn check_text(&self, text: &str) -> Vec<SpellIssue> {
        self.check_text_with_limit(text, 5)
    }

    /// Like [`check_text`], but caps suggestion work per issue (RULES path uses 1).
    pub fn check_text_with_limit(&self, text: &str, suggestion_limit: usize) -> Vec<SpellIssue> {
        let mut issues = Vec::new();
        let mut offset = 0usize;
        for word in text.split_word_bounds() {
            let len = word.len();
            if is_word_token(word) && !self.is_correct(word) {
                issues.push(SpellIssue {
                    word: word.to_string(),
                    start: offset,
                    end: offset + len,
                    suggestions: self.suggest(word, suggestion_limit),
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
const ENGLISH_US: &str = include_str!("../data/en_us.txt");

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

    #[test]
    fn suggest_returns_the_for_teh() {
        let checker = SpellChecker::english();
        let suggestions = checker.suggest("teh", 5);
        assert!(
            suggestions.first().map(String::as_str) == Some("the"),
            "expected 'the' first, got {suggestions:?}"
        );
    }

    #[test]
    fn check_text_attaches_suggestions() {
        let checker = SpellChecker::english();
        let issues = checker.check_text("Teh");
        assert_eq!(issues.len(), 1);
        assert!(
            issues[0].suggestions.iter().any(|s| s == "the"),
            "suggestions: {:?}",
            issues[0].suggestions
        );
    }
}

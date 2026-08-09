//! Edit-distance suggestion ranking (Hunspell-compatible API surface).

use std::collections::HashMap;

/// Optimal string alignment / Damerau-Levenshtein distance.
pub fn damerau_levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let inf = a_len + b_len;
    let mut da: HashMap<char, usize> = HashMap::new();
    let mut d = vec![vec![0usize; b_len + 2]; a_len + 2];
    d[0][0] = inf;
    for i in 0..=a_len {
        d[i + 1][1] = i;
        d[i + 1][0] = inf;
    }
    for j in 0..=b_len {
        d[1][j + 1] = j;
        d[0][j + 1] = inf;
    }

    for i in 1..=a_len {
        let mut db = 0;
        for j in 1..=b_len {
            let i1 = da.get(&b[j - 1]).copied().unwrap_or(0);
            let j1 = db;
            let cost = if a[i - 1] == b[j - 1] {
                db = j;
                0
            } else {
                1
            };
            d[i + 1][j + 1] = (d[i][j] + cost)
                .min(d[i + 1][j] + 1)
                .min(d[i][j + 1] + 1)
                .min(d[i1][j1] + (i - i1 - 1) + 1 + (j - j1 - 1));
        }
        da.insert(a[i - 1], i);
    }
    d[a_len + 1][b_len + 1]
}

pub fn rank_suggestions<'a>(
    word: &str,
    dictionary: impl Iterator<Item = &'a str>,
    limit: usize,
    max_distance: usize,
    mut rank_key: impl FnMut(&str) -> usize,
) -> Vec<String> {
    if word.is_empty() || limit == 0 {
        return Vec::new();
    }
    let word_len = word.chars().count();
    let mut ranked: Vec<(usize, usize, String)> = dictionary
        .filter_map(|candidate| {
            let candidate_len = candidate.chars().count();
            if word_len.abs_diff(candidate_len) > max_distance {
                return None;
            }
            let distance = damerau_levenshtein(word, candidate);
            if distance <= max_distance {
                Some((distance, rank_key(candidate), candidate.to_string()))
            } else {
                None
            }
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)).then_with(|| a.2.cmp(&b.2)));
    ranked
        .into_iter()
        .map(|(_, _, word)| word)
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transposition_counts_as_one_edit() {
        assert_eq!(damerau_levenshtein("teh", "the"), 1);
    }

    #[test]
    fn rank_prefers_closer_matches() {
        let dict = ["the", "tea", "then", "tech"];
        let rank = |word: &str| match word {
            "the" => 0,
            "tea" => 1,
            _ => 2,
        };
        let suggestions = rank_suggestions("teh", dict.into_iter(), 3, 2, rank);
        assert_eq!(suggestions.first().map(String::as_str), Some("the"));
    }
}

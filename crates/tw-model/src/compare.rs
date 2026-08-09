//! Text and document comparison helpers (F17.S4).

use crate::Document;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareChangeKind {
    Equal,
    Insert,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompareChange {
    pub kind: CompareChangeKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentCompareSummary {
    pub changes: Vec<CompareChange>,
    pub insertion_count: usize,
    pub deletion_count: usize,
}

pub fn compare_text(left: &str, right: &str) -> DocumentCompareSummary {
    let left_lines: Vec<&str> = if left.is_empty() {
        vec![]
    } else {
        left.lines().collect()
    };
    let right_lines: Vec<&str> = if right.is_empty() {
        vec![]
    } else {
        right.lines().collect()
    };
    let ops = diff_lines(&left_lines, &right_lines);
    let mut insertion_count = 0;
    let mut deletion_count = 0;
    for change in &ops {
        match change.kind {
            CompareChangeKind::Insert => insertion_count += 1,
            CompareChangeKind::Delete => deletion_count += 1,
            CompareChangeKind::Equal => {}
        }
    }
    DocumentCompareSummary {
        changes: ops,
        insertion_count,
        deletion_count,
    }
}

pub fn compare_documents(left: &Document, right: &Document) -> DocumentCompareSummary {
    compare_text(&document_plain_text(left), &document_plain_text(right))
}

pub fn document_plain_text(doc: &Document) -> String {
    doc.sections
        .iter()
        .flat_map(|section| section.blocks.iter())
        .filter_map(|block| block.paragraph())
        .map(|para| para.visible_text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn diff_lines(left: &[&str], right: &[&str]) -> Vec<CompareChange> {
    let lcs = longest_common_subsequence(left, right);
    let mut out = Vec::new();
    let mut li = 0usize;
    let mut ri = 0usize;
    for &(l_idx, r_idx) in &lcs {
        while li < l_idx {
            out.push(CompareChange {
                kind: CompareChangeKind::Delete,
                text: left[li].to_string(),
            });
            li += 1;
        }
        while ri < r_idx {
            out.push(CompareChange {
                kind: CompareChangeKind::Insert,
                text: right[ri].to_string(),
            });
            ri += 1;
        }
        out.push(CompareChange {
            kind: CompareChangeKind::Equal,
            text: left[l_idx].to_string(),
        });
        li = l_idx + 1;
        ri = r_idx + 1;
    }
    while li < left.len() {
        out.push(CompareChange {
            kind: CompareChangeKind::Delete,
            text: left[li].to_string(),
        });
        li += 1;
    }
    while ri < right.len() {
        out.push(CompareChange {
            kind: CompareChangeKind::Insert,
            text: right[ri].to_string(),
        });
        ri += 1;
    }
    out
}

fn longest_common_subsequence(left: &[&str], right: &[&str]) -> Vec<(usize, usize)> {
    let mut dp = vec![vec![0usize; right.len() + 1]; left.len() + 1];
    for (i, l_line) in left.iter().enumerate() {
        for (j, r_line) in right.iter().enumerate() {
            if l_line == r_line {
                dp[i + 1][j + 1] = dp[i][j] + 1;
            } else {
                dp[i + 1][j + 1] = dp[i + 1][j].max(dp[i][j + 1]);
            }
        }
    }
    let mut pairs = Vec::new();
    let mut i = left.len();
    let mut j = right.len();
    while i > 0 && j > 0 {
        if left[i - 1] == right[j - 1] {
            pairs.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    pairs.reverse();
    pairs
}

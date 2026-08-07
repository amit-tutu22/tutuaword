use std::collections::{BTreeMap, HashMap, HashSet};

/// Tracks OOXML elements encountered during import vs those retained in the model.
#[derive(Debug, Clone, Default)]
pub struct ImportRetentionReport {
    encountered: HashMap<String, usize>,
    retained: HashMap<String, usize>,
}

impl ImportRetentionReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_encountered(&mut self, element: &str) {
        *self.encountered.entry(element.to_string()).or_default() += 1;
    }

    pub fn record_retained(&mut self, element: &str) {
        *self.retained.entry(element.to_string()).or_default() += 1;
    }

    pub fn encountered_count(&self, element: &str) -> usize {
        self.encountered.get(element).copied().unwrap_or(0)
    }

    pub fn retained_count(&self, element: &str) -> usize {
        self.retained.get(element).copied().unwrap_or(0)
    }

    pub fn encountered(&self) -> &HashMap<String, usize> {
        &self.encountered
    }

    pub fn retained(&self) -> &HashMap<String, usize> {
        &self.retained
    }

    /// Element types seen in XML but not mapped to the document model.
    pub fn missing_types(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for (element, count) in &self.encountered {
            if self.retained.get(element).copied().unwrap_or(0) < *count {
                missing.push(element.clone());
            }
        }
        missing.sort();
        missing
    }

    /// Human-readable summary for test failures.
    pub fn summary(&self) -> String {
        let mut lines = vec!["Import retention report:".to_string()];
        let mut keys: Vec<_> = self.encountered.keys().collect();
        keys.sort();
        for key in keys {
            let enc = self.encountered[key];
            let ret = self.retained.get(key).copied().unwrap_or(0);
            lines.push(format!("  {key}: encountered={enc}, retained={ret}"));
        }
        let missing = self.missing_types();
        if !missing.is_empty() {
            lines.push(format!("  missing from model: {}", missing.join(", ")));
        }
        lines.join("\n")
    }
}

/// Scan XML for known OOXML element tags (w:* namespace) and record counts.
pub fn scan_ooxml_elements(xml: &str, report: &mut ImportRetentionReport) {
    let mut seen = HashSet::new();
    let mut rest = xml;
    while let Some(i) = rest.find("<w:") {
        rest = &rest[i + 3..];
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
            .unwrap_or(rest.len());
        let tag = &rest[..end];
        if seen.insert(tag.to_string()) {
            let count = count_element(xml, tag);
            for _ in 0..count {
                report.record_encountered(tag);
            }
        }
    }
}

fn count_element(xml: &str, tag: &str) -> usize {
    let open = format!("<w:{tag}");
    xml.matches(&open).count()
}

/// Compare corpus XML tags against retention; returns unmapped tags with counts.
pub fn unmapped_corpus_tags(
    encountered: &HashMap<String, usize>,
    retained: &HashMap<String, usize>,
    known_mapped: &[&str],
) -> BTreeMap<String, usize> {
    let mapped: HashSet<&str> = known_mapped.iter().copied().collect();
    let mut out = BTreeMap::new();
    for (tag, count) in encountered {
        if mapped.contains(tag.as_str()) {
            continue;
        }
        let ret = retained.get(tag).copied().unwrap_or(0);
        if ret < *count {
            out.insert(tag.clone(), count - ret);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_types_lists_unretained_elements() {
        let mut report = ImportRetentionReport::new();
        report.record_encountered("hyperlink");
        report.record_encountered("hyperlink");
        report.record_retained("hyperlink");
        assert_eq!(report.missing_types(), vec!["hyperlink"]);
    }
}

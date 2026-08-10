//! External hyperlink relationships for DOCX import/export (F19.S3).

use std::collections::HashMap;

use tw_model::{Block, Document, RunContent};

use crate::xml_util::{read_own_attr, split_elements};
use crate::DocxPackage;

const HYPERLINK_RELATIONSHIP_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
const RELS_PART: &str = "word/_rels/document.xml.rels";

/// Maps external hyperlink URLs to relationship ids for export.
pub struct HyperlinkRels {
    /// Canonical URL → relationship id.
    url_to_rid: HashMap<String, String>,
    /// Newly allocated relationships `(id, target URL)`.
    new_rels: Vec<(String, String)>,
}

impl HyperlinkRels {
    pub fn build(doc: &Document, package: &DocxPackage) -> Self {
        let existing = package
            .parts
            .get(RELS_PART)
            .map(|bytes| parse_relationship_targets(&String::from_utf8_lossy(bytes)))
            .unwrap_or_default();

        let mut next_relationship = existing
            .keys()
            .filter_map(|id| id.strip_prefix("rId")?.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
            + 1;

        // Prefer reusing an existing external hyperlink rel when Target matches.
        let mut target_to_rid: HashMap<String, String> = HashMap::new();
        for (id, (target, external)) in &existing {
            if *external {
                target_to_rid.insert(target.clone(), id.clone());
            }
        }

        let mut url_to_rid = HashMap::new();
        let mut new_rels = Vec::new();

        for_each_hyperlink_url(doc, |url| {
            if !is_external_url(url) {
                return;
            }
            if url_to_rid.contains_key(url) {
                return;
            }
            if let Some(id) = target_to_rid.get(url) {
                url_to_rid.insert(url.to_string(), id.clone());
                return;
            }
            let id = format!("rId{next_relationship}");
            next_relationship += 1;
            new_rels.push((id.clone(), url.to_string()));
            target_to_rid.insert(url.to_string(), id.clone());
            url_to_rid.insert(url.to_string(), id);
        });

        Self {
            url_to_rid,
            new_rels,
        }
    }

    pub fn rid_for(&self, url: &str) -> Option<&str> {
        self.url_to_rid.get(url).map(String::as_str)
    }

    pub fn commit(self, package: &mut DocxPackage) {
        if self.new_rels.is_empty() {
            return;
        }
        let existing = package
            .parts
            .get(RELS_PART)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_else(|| "<Relationships/>".to_string());
        let updated = append_hyperlink_relationships(&existing, &self.new_rels);
        package
            .parts
            .insert(RELS_PART.into(), updated.into_bytes());
        package.mark_modified(RELS_PART.into());
    }
}

/// Resolve `r:id:…` hyperlink targets to their relationship Target URLs.
pub fn resolve_hyperlink_targets(doc: &mut Document, relationships: &HashMap<String, String>) {
    let resolve_run = |content: &mut RunContent| {
        if let RunContent::Hyperlink { target, .. } = content {
            if let Some(rid) = target.url.strip_prefix("r:id:") {
                if let Some(url) = relationships.get(rid) {
                    target.url = url.clone();
                }
            }
        }
    };
    walk_document_runs_mut(doc, resolve_run);
}

pub fn is_external_url(url: &str) -> bool {
    if url.is_empty() || url == "#" || url.starts_with('#') || url.starts_with("r:id:") {
        return false;
    }
    true
}

pub fn hyperlink_anchor(target_url: &str, anchor: &Option<String>) -> Option<String> {
    if let Some(a) = anchor {
        if !a.is_empty() {
            return Some(a.clone());
        }
    }
    target_url
        .strip_prefix('#')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn for_each_hyperlink_url(doc: &Document, mut visit: impl FnMut(&str)) {
    let visit_blocks = |blocks: &[Block], visit: &mut dyn FnMut(&str)| {
        for block in blocks {
            visit_block(block, visit);
        }
    };
    for section in &doc.sections {
        visit_blocks(&section.blocks, &mut visit);
        for hf in section.headers.values().chain(section.footers.values()) {
            visit_blocks(&hf.blocks, &mut visit);
        }
    }
    for note in &doc.footnotes {
        visit_blocks(&note.blocks, &mut visit);
    }
}

fn visit_block(block: &Block, visit: &mut dyn FnMut(&str)) {
    match block {
        Block::Paragraph(para) => {
            for run in &para.runs {
                if let RunContent::Hyperlink { target, .. } = &run.content {
                    visit(&target.url);
                }
            }
        }
        Block::Table(table) => {
            for row in &table.rows {
                for cell in &row.cells {
                    for child in &cell.blocks {
                        visit_block(child, visit);
                    }
                }
            }
        }
        _ => {}
    }
}

fn walk_document_runs_mut(doc: &mut Document, mut visit: impl FnMut(&mut RunContent)) {
    for section in &mut doc.sections {
        walk_blocks_mut(&mut section.blocks, &mut visit);
        for hf in section
            .headers
            .values_mut()
            .chain(section.footers.values_mut())
        {
            walk_blocks_mut(&mut hf.blocks, &mut visit);
        }
    }
    for note in &mut doc.footnotes {
        walk_blocks_mut(&mut note.blocks, &mut visit);
    }
}

fn walk_blocks_mut(blocks: &mut [Block], visit: &mut dyn FnMut(&mut RunContent)) {
    for block in blocks {
        match block {
            Block::Paragraph(para) => {
                for run in &mut para.runs {
                    visit(&mut run.content);
                }
            }
            Block::Table(table) => {
                for row in &mut table.rows {
                    for cell in &mut row.cells {
                        walk_blocks_mut(&mut cell.blocks, visit);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Id → (Target, is_external_hyperlink).
fn parse_relationship_targets(xml: &str) -> HashMap<String, (String, bool)> {
    let mut map = HashMap::new();
    for element in split_elements(xml, "Relationship") {
        let (Some(id), Some(target)) = (
            read_own_attr(element, "Id"),
            read_own_attr(element, "Target"),
        ) else {
            continue;
        };
        let rel_type = read_own_attr(element, "Type").unwrap_or_default();
        let mode = read_own_attr(element, "TargetMode").unwrap_or_default();
        let external = rel_type.contains("/hyperlink")
            || mode.eq_ignore_ascii_case("External");
        map.insert(id.to_string(), (target.to_string(), external));
    }
    map
}

fn append_hyperlink_relationships(xml: &str, relationships: &[(String, String)]) -> String {
    let mut additions = String::new();
    for (id, target) in relationships {
        additions.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{HYPERLINK_RELATIONSHIP_TYPE}" Target="{}" TargetMode="External"/>"#,
            xml_escape_attr(target)
        ));
    }

    if let Some(index) = xml.rfind("</Relationships>") {
        let mut out = String::with_capacity(xml.len() + additions.len());
        out.push_str(&xml[..index]);
        out.push_str(&additions);
        out.push_str(&xml[index..]);
        return out;
    }
    if let Some(index) = xml.rfind("/>") {
        let mut out = String::with_capacity(xml.len() + additions.len() + 20);
        out.push_str(&xml[..index]);
        out.push('>');
        out.push_str(&additions);
        out.push_str("</Relationships>");
        return out;
    }
    format!("<Relationships>{additions}</Relationships>")
}

fn xml_escape_attr(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

//! SmartArt / diagram OPC export for editor-inserted placeholders (F12.S3 hardening).

use std::collections::{HashMap, HashSet};

use tw_model::{NodeId, ShapeBlock};

use crate::xml_util::{read_own_attr, split_elements};
use crate::DocxPackage;

const DIAGRAM_DATA_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData";
const DIAGRAM_LAYOUT_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramLayout";
const RELS_PART: &str = "word/_rels/document.xml.rels";
const CONTENT_TYPES_PART: &str = "[Content_Types].xml";
const DIAGRAM_DATA_CONTENT: &str =
    "application/vnd.openxmlformats-officedocument.drawingml.diagramData+xml";
const DIAGRAM_LAYOUT_CONTENT: &str =
    "application/vnd.openxmlformats-officedocument.drawingml.diagramLayout+xml";

const DEFAULT_DATA_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<dgm:dataModel xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram">
  <dgm:ptLst><dgm:pt modelId="inserted-diagram"/></dgm:ptLst>
</dgm:dataModel>"#;

const DEFAULT_LAYOUT_XML: &str = r#"<dgm:layoutDef xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram"/>"#;

pub fn resolve_diagram_data_part(para_xml: &str, package: &DocxPackage) -> Option<String> {
    let data_rel = crate::xml_util::read_attr_value(para_xml, "dgm:relIds", "r:dm")?;
    resolve_rel_target("word", &data_rel, package)
}

pub fn resolve_diagram_layout_part(para_xml: &str, package: &DocxPackage) -> Option<String> {
    let layout_rel = crate::xml_util::read_attr_value(para_xml, "dgm:relIds", "r:lo")?;
    resolve_rel_target("word", &layout_rel, package)
}

fn resolve_rel_target(base: &str, rel_id: &str, package: &DocxPackage) -> Option<String> {
    let doc_rels = package
        .parts
        .get(RELS_PART)
        .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
        .unwrap_or_default();
    let target = doc_rels.get(rel_id)?;
    Some(normalize_part_path(base, target))
}

#[derive(Debug, Clone)]
pub struct DiagramRelationships {
    pub data_rel: String,
    pub layout_rel: String,
}

pub struct DiagramWriter {
    targets: HashMap<String, String>,
    referenced: HashMap<NodeId, DiagramRelationships>,
    taken: HashSet<String>,
    parts: Vec<(String, Vec<u8>)>,
    relationships: Vec<(String, String, &'static str)>,
    next_relationship: u32,
    next_diagram: u32,
}

impl DiagramWriter {
    pub fn new(package: &DocxPackage) -> Self {
        let existing = package
            .parts
            .get(RELS_PART)
            .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
            .unwrap_or_default();

        let next_relationship = existing
            .keys()
            .filter_map(|id| id.strip_prefix("rId")?.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
            + 1;

        let targets = existing
            .into_iter()
            .map(|(id, target)| (normalize_target(&target), id))
            .collect();

        Self {
            targets,
            referenced: HashMap::new(),
            taken: package.parts.keys().cloned().collect(),
            parts: Vec::new(),
            relationships: Vec::new(),
            next_relationship,
            next_diagram: 1,
        }
    }

    pub fn reference(&mut self, shape: &ShapeBlock) -> Option<&DiagramRelationships> {
        if shape.shape.shape_type != tw_model::ShapeKind::Diagram {
            return None;
        }
        if self.referenced.contains_key(&shape.id) {
            return self.referenced.get(&shape.id);
        }

        let (data_part, layout_part) = self.allocate_diagram_parts(shape);
        let data_target = data_part.strip_prefix("word/").unwrap_or(&data_part).to_string();
        let layout_target = layout_part
            .strip_prefix("word/")
            .unwrap_or(&layout_part)
            .to_string();

        let data_rel = self.relationship_for_target(&data_target, DIAGRAM_DATA_REL);
        let layout_rel = self.relationship_for_target(&layout_target, DIAGRAM_LAYOUT_REL);

        self.parts
            .push((data_part, DEFAULT_DATA_XML.as_bytes().to_vec()));
        self.parts
            .push((layout_part, DEFAULT_LAYOUT_XML.as_bytes().to_vec()));

        let rels = DiagramRelationships {
            data_rel: data_rel.clone(),
            layout_rel: layout_rel.clone(),
        };
        self.referenced.insert(shape.id, rels);
        self.referenced.get(&shape.id)
    }

    pub fn relationships_for(&self, shape_id: &NodeId) -> Option<&DiagramRelationships> {
        self.referenced.get(shape_id)
    }

    pub fn commit(self, package: &mut DocxPackage) {
        if self.parts.is_empty() {
            return;
        }

        let part_names: Vec<String> = self.parts.iter().map(|(n, _)| n.clone()).collect();
        for (name, bytes) in self.parts {
            package.parts.insert(name.clone(), bytes);
            package.mark_modified(name);
        }

        if !self.relationships.is_empty() {
            let existing = package
                .parts
                .get(RELS_PART)
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
                .unwrap_or_else(|| DEFAULT_RELS.to_string());
            let updated = append_diagram_relationships(&existing, &self.relationships);
            package.parts.insert(RELS_PART.into(), updated.into_bytes());
            package.mark_modified(RELS_PART.into());
        }

        let content_types = package
            .parts
            .get(CONTENT_TYPES_PART)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_else(|| DEFAULT_CONTENT_TYPES.to_string());
        if let Some(updated) = ensure_part_overrides(
            &content_types,
            &part_names,
            &[
                ("data", DIAGRAM_DATA_CONTENT),
                ("layout", DIAGRAM_LAYOUT_CONTENT),
            ],
        ) {
            package
                .parts
                .insert(CONTENT_TYPES_PART.into(), updated.into_bytes());
            package.mark_modified(CONTENT_TYPES_PART.into());
        }
    }

    fn relationship_for_target(&mut self, target: &str, rel_type: &'static str) -> String {
        if let Some(id) = self.targets.get(target) {
            return id.clone();
        }
        let id = format!("rId{}", self.next_relationship);
        self.next_relationship += 1;
        self.relationships
            .push((id.clone(), target.to_string(), rel_type));
        self.targets.insert(target.to_string(), id.clone());
        id
    }

    fn allocate_diagram_parts(&mut self, shape: &ShapeBlock) -> (String, String) {
        if let Some(data_part) = shape
            .diagram_data_part
            .as_ref()
            .filter(|p| p.starts_with("word/diagrams/"))
        {
            let layout_part = shape.diagram_layout_part.clone().unwrap_or_else(|| {
                data_part.replace("data", "layout")
            });
            self.taken.insert(data_part.clone());
            self.taken.insert(layout_part.clone());
            return (data_part.clone(), layout_part);
        }

        loop {
            let idx = self.next_diagram;
            self.next_diagram += 1;
            let data = format!("word/diagrams/data{idx}.xml");
            let layout = format!("word/diagrams/layout{idx}.xml");
            if self.taken.insert(data.clone()) && self.taken.insert(layout.clone()) {
                return (data, layout);
            }
        }
    }
}

pub fn ensure_part_overrides(
    xml: &str,
    part_names: &[String],
    suffix_content_types: &[(&str, &str)],
) -> Option<String> {
    let mut additions = String::new();
    for part in part_names {
        let override_path = if part.starts_with('/') {
            part.clone()
        } else {
            format!("/{part}")
        };
        if xml.contains(&override_path) {
            continue;
        }
        let content_type = suffix_content_types
            .iter()
            .find(|(suffix, _)| part.contains(suffix))
            .map(|(_, ct)| *ct)
            .unwrap_or("application/xml");
        additions.push_str(&format!(
            r#"<Override PartName="{override_path}" ContentType="{content_type}"/>"#
        ));
    }
    if additions.is_empty() {
        return None;
    }
    if let Some(index) = xml.rfind("</Types>") {
        let mut out = String::with_capacity(xml.len() + additions.len());
        out.push_str(&xml[..index]);
        out.push_str(&additions);
        out.push_str(&xml[index..]);
        return Some(out);
    }
    None
}

fn normalize_part_path(base: &str, target: &str) -> String {
    let mut path = base.to_string();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if let Some(idx) = path.rfind('/') {
                    path.truncate(idx);
                }
            }
            part => {
                if !path.is_empty() {
                    path.push('/');
                }
                path.push_str(part);
            }
        }
    }
    path
}

fn parse_relationships(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for element in split_elements(xml, "Relationship") {
        let (Some(id), Some(target)) = (
            read_own_attr(element, "Id"),
            read_own_attr(element, "Target"),
        ) else {
            continue;
        };
        map.insert(id.to_string(), target.to_string());
    }
    map
}

fn normalize_target(target: &str) -> String {
    target.trim_start_matches("./").to_string()
}

fn append_diagram_relationships(
    xml: &str,
    relationships: &[(String, String, &'static str)],
) -> String {
    let mut additions = String::new();
    for (id, target, rel_type) in relationships {
        additions.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{rel_type}" Target="{target}"/>"#
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
    format!("{DEFAULT_RELS_OPEN}{additions}</Relationships>")
}

const DEFAULT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#;

const DEFAULT_RELS_OPEN: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#;

const DEFAULT_CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"></Types>"#;

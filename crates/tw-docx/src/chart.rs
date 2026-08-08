//! Chart XML parse/serialize and export wiring (F13.S3).

use std::collections::{HashMap, HashSet};

use tw_model::{ChartData, ChartSeries, NodeId, ShapeBlock};

use crate::xml_util::{read_own_attr, split_elements};
use crate::DocxPackage;

const CHART_RELATIONSHIP_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart";
const RELS_PART: &str = "word/_rels/document.xml.rels";
const CONTENT_TYPES_PART: &str = "[Content_Types].xml";

pub fn resolve_chart_part_path(para_xml: &str, package: &DocxPackage) -> Option<String> {
    let chart_rel = crate::xml_util::read_attr_value(para_xml, "c:chart", "r:id")?;
    let doc_rels = package
        .parts
        .get(RELS_PART)
        .map(|bytes| parse_relationships(&String::from_utf8_lossy(bytes)))
        .unwrap_or_default();
    let target = doc_rels.get(&chart_rel)?;
    Some(normalize_part_path("word", target))
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

pub fn parse_chart_data(xml: &str) -> Option<ChartData> {
    let mut categories = Vec::new();
    let mut series = Vec::new();

    for ser in split_elements(xml, "c:ser") {
        if categories.is_empty() {
            if let Some(cat) = first_tag_section(ser, "c:cat") {
                categories = parse_cache_values(&cat, "c:strCache");
                if categories.is_empty() {
                    categories = parse_cache_values(&cat, "c:numCache");
                }
            }
        }

        let name = first_tag_section(ser, "c:tx")
            .and_then(|tx| parse_cache_values(&tx, "c:strCache").into_iter().next())
            .unwrap_or_else(|| "Series 1".into());
        let values = first_tag_section(ser, "c:val")
            .map(|val| parse_cache_values(&val, "c:numCache"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.parse::<f64>().ok())
            .collect::<Vec<_>>();

        if !values.is_empty() {
            series.push(ChartSeries { name, values });
        }
    }

    if categories.is_empty() || series.is_empty() {
        return None;
    }
    Some(ChartData { categories, series })
}

pub fn serialize_chart_xml(data: &ChartData) -> String {
    let cat_cache = serialize_str_cache(&data.categories);
    let mut series_xml = String::new();
    for (idx, series) in data.series.iter().enumerate() {
        let tx_cache = serialize_str_cache(&[series.name.clone()]);
        let val_cache = serialize_num_cache(&series.values);
        series_xml.push_str(&format!(
            r#"<c:ser><c:idx val="{idx}"/><c:tx><c:strRef><c:f/><c:strCache>{tx_cache}</c:strCache></c:strRef></c:tx><c:cat><c:strRef><c:f/><c:strCache>{cat_cache}</c:strCache></c:strRef></c:cat><c:val><c:numRef><c:f/><c:numCache>{val_cache}</c:numCache></c:numRef></c:val></c:ser>"#
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
  <c:chart>
    <c:plotArea>
      <c:barChart>{series_xml}<c:axId val="1"/><c:axId val="2"/></c:barChart>
      <c:catAx><c:axId val="1"/></c:catAx>
      <c:valAx><c:axId val="2"/></c:valAx>
    </c:plotArea>
  </c:chart>
</c:chartSpace>"#
    )
}

pub struct ChartWriter {
    targets: HashMap<String, String>,
    referenced: HashMap<NodeId, String>,
    taken: HashSet<String>,
    parts: Vec<(String, Vec<u8>)>,
    relationships: Vec<(String, String)>,
    next_relationship: u32,
    next_chart: u32,
}

impl ChartWriter {
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
            next_chart: 1,
        }
    }

    pub fn reference(&mut self, shape: &ShapeBlock, data: &ChartData) -> Option<String> {
        if shape.shape.shape_type != tw_model::ShapeKind::Chart {
            return None;
        }

        let part_name = shape
            .chart_part
            .clone()
            .filter(|part| part.starts_with("word/charts/"))
            .unwrap_or_else(|| self.allocate_chart_part());

        let target = part_name
            .strip_prefix("word/")
            .unwrap_or(&part_name)
            .to_string();
        let relationship_id = if let Some(id) = self.referenced.get(&shape.id) {
            id.clone()
        } else {
            match self.targets.get(&target) {
                Some(id) => id.clone(),
                None => {
                    let id = format!("rId{}", self.next_relationship);
                    self.next_relationship += 1;
                    self.relationships.push((id.clone(), target.clone()));
                    self.targets.insert(target, id.clone());
                    id
                }
            }
        };

        self.parts
            .push((part_name, serialize_chart_xml(data).into_bytes()));
        self.referenced.insert(shape.id, relationship_id.clone());
        Some(relationship_id)
    }

    pub fn relationship_for(&self, shape_id: &NodeId) -> Option<&str> {
        self.referenced.get(shape_id).map(String::as_str)
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
            let updated = append_chart_relationships(&existing, &self.relationships);
            package.parts.insert(RELS_PART.into(), updated.into_bytes());
            package.mark_modified(RELS_PART.into());
        }

        let content_types = package
            .parts
            .get(CONTENT_TYPES_PART)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_else(|| DEFAULT_CONTENT_TYPES.to_string());
        if let Some(updated) = crate::diagram::ensure_part_overrides(
            &content_types,
            &part_names,
            &[("charts/", "application/vnd.openxmlformats-officedocument.drawingml.chart+xml")],
        ) {
            package
                .parts
                .insert(CONTENT_TYPES_PART.into(), updated.into_bytes());
            package.mark_modified(CONTENT_TYPES_PART.into());
        }
    }

    fn allocate_chart_part(&mut self) -> String {
        loop {
            let name = format!("word/charts/chart{}.xml", self.next_chart);
            self.next_chart += 1;
            if self.taken.insert(name.clone()) {
                return name;
            }
        }
    }
}

fn first_tag_section(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = xml.find(&open)?;
    let after = &xml[start..];
    let close = format!("</{tag}>");
    let end = after.find(&close)? + close.len();
    Some(after[..end].to_string())
}

fn parse_cache_values(section: &str, cache_tag: &str) -> Vec<String> {
    let open = format!("<{cache_tag}");
    let Some(start) = section.find(&open) else {
        return Vec::new();
    };
    let close = format!("</{cache_tag}>");
    let Some(end_rel) = section[start..].find(&close) else {
        return Vec::new();
    };
    let cache = &section[start..start + end_rel];
    let mut points = Vec::new();
    for pt in split_elements(cache, "c:pt") {
        let Some(idx) = read_own_attr(pt, "idx").and_then(|v| v.parse::<u32>().ok()) else {
            continue;
        };
        let Some(value) = read_element_text(pt, "c:v") else {
            continue;
        };
        points.push((idx, value));
    }
    points.sort_by_key(|(idx, _)| *idx);
    points.into_iter().map(|(_, value)| value).collect()
}

fn read_element_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

fn serialize_str_cache(values: &[String]) -> String {
    let mut out = format!(r#"<c:ptCount val="{}"/>"#, values.len());
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(
            r#"<c:pt idx="{idx}"><c:v>{}</c:v></c:pt>"#,
            escape_xml(value)
        ));
    }
    out
}

fn serialize_num_cache(values: &[f64]) -> String {
    let mut out = format!(r#"<c:ptCount val="{}"/>"#, values.len());
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&format!(r#"<c:pt idx="{idx}"><c:v>{value}</c:v></c:pt>"#));
    }
    out
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

fn append_chart_relationships(xml: &str, relationships: &[(String, String)]) -> String {
    let mut additions = String::new();
    for (id, target) in relationships {
        additions.push_str(&format!(
            r#"<Relationship Id="{id}" Type="{CHART_RELATIONSHIP_TYPE}" Target="{target}"/>"#
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_serialize_round_trip() {
        let xml = serialize_chart_xml(&ChartData::sample_bar());
        let parsed = parse_chart_data(&xml).expect("parsed chart data");
        assert_eq!(parsed, ChartData::sample_bar());
    }
}

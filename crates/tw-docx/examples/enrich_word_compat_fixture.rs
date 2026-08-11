//! Inject real F10–F19 objects into the Word-compatible feature fixture.
//!
//! Run: `cargo run -p tw-docx --example enrich_word_compat_fixture`

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tw_docx::chart::serialize_chart_xml;
use tw_model::ChartData;
use zip::read::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

const OMML_TOKEN: &str = "OMML_FEATURE_FIXTURE";

const IMAGE_PLACEHOLDER_PARA: &str = r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>[ IMAGE PLACEHOLDER ]</w:t></w:r></w:p>"#;

const IMAGE_PARAGRAPH: &str = r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="914400" cy="457200"/><wp:docPr id="100" name="FeatureFixtureImage"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="100" name="image1.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rId11"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="457200"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

const SHAPE_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>[RECTANGLE]    [CIRCLE]    → ARROW →    [CALLOUT]</w:t></w:r></w:p>"#;

const SHAPE_PARAGRAPH: &str = r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="1828800" cy="914400"/><wp:docPr id="42" name="BlueRectangle"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:cNvPr id="42" name="BlueRectangle"/></wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

const DIAGRAM_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Test SmartArt-like process, hierarchy, cycle, relationship, matrix, and organization-chart objects.</w:t></w:r></w:p>"#;

const DIAGRAM_PARAGRAPH: &str = r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="5486400" cy="2743200"/><wp:docPr id="2" name="SmartArt Diagram"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/diagram"><dgm:relIds xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram" r:dm="rId12" r:lo="rId13"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

const CHART_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Insert bar, line, pie, and scatter charts using the table above. Test editable data, legends, axes, labels, and formatting.</w:t></w:r></w:p>"#;

const CHART_PARAGRAPH: &str = r#"<w:p><w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="5486400" cy="2743200"/><wp:docPr id="3" name="Chart 1"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" r:id="rId14"/></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

const OMML_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>E = mc²</w:t></w:r></w:p>"#;

const OMML_PARAGRAPH: &str = r#"<w:p><w:r><w:t>Before </w:t></w:r><m:oMath><m:r><m:t>OMML_FEATURE_FIXTURE</m:t></m:r></m:oMath><w:r><w:t> after (E = mc² preview)</w:t></w:r></w:p>"#;

const FOOTNOTE_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Footnote test sentence: This sentence should receive a footnote marker and footnote content.</w:t></w:r></w:p>"#;

const FOOTNOTE_PARAGRAPH: &str = r#"<w:p><w:r><w:t>Footnote test sentence: This sentence should receive a footnote marker</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r><w:r><w:t> and footnote content.</w:t></w:r></w:p>"#;

const TRACK_CHANGES_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Track Changes test sentence: Original wording should be modified, deleted, and replaced while revisions remain visible.</w:t></w:r></w:p>"#;

const TRACK_CHANGES_PARAGRAPH: &str = r#"<w:p><w:ins w:id="1" w:author="Fixture Author" w:date="2026-01-01T00:00:00Z"><w:r><w:t>Added revision. </w:t></w:r></w:ins><w:del w:id="2" w:author="Fixture Author" w:date="2026-01-01T00:00:00Z"><w:r><w:t>removed </w:t></w:r></w:del><w:r><w:t>Track Changes test sentence: Original wording should be modified, deleted, and replaced while revisions remain visible.</w:t></w:r></w:p>"#;

const COMMENT_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Comment test: Select this sentence and add a comment such as “Please verify this figure.”</w:t></w:r></w:p>"#;

const COMMENT_PARAGRAPH: &str = r#"<w:p><w:r><w:t>Comment test: Select this sentence</w:t></w:r><w:r><w:commentReference w:id="1"/></w:r><w:r><w:t> and add a comment such as “Please verify this figure.”</w:t></w:r></w:p>"#;

const BOOKMARK_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Bookmark Target: IMPORTANT_BOOKMARK</w:t></w:r></w:p>"#;

const BOOKMARK_PARAGRAPH: &str = r#"<w:p><w:r><w:t>Bookmark Target: </w:t></w:r><w:bookmarkStart w:id="42" w:name="IMPORTANT_BOOKMARK"/><w:r><w:t>IMPORTANT_BOOKMARK</w:t></w:r><w:bookmarkEnd w:id="42"/></w:p>"#;

const HYPERLINK_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Hyperlink Target: https://example.com</w:t></w:r></w:p>"#;

const HYPERLINK_PARAGRAPH: &str = r#"<w:p xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:r><w:t>Hyperlink Target: </w:t></w:r><w:hyperlink r:id="rId16"><w:r><w:t>https://example.com</w:t></w:r></w:hyperlink></w:p>"#;

const BIBLIOGRAPHY_PLACEHOLDER_PARA: &str = r#"<w:p><w:r><w:t>Bibliography sample:</w:t></w:r></w:p>"#;

const BIBLIOGRAPHY_PARAGRAPH: &str = r#"<w:p><w:r><w:t>Bibliography sample: </w:t></w:r><w:fldSimple w:instr=" CITATION Smith2020 \l 1033 "><w:r><w:t>(Smith, 2020)</w:t></w:r></w:fldSimple></w:p>"#;

const FOOTNOTES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote>
  <w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote>
  <w:footnote w:id="1"><w:p><w:r><w:t>Fixture footnote body text.</w:t></w:r></w:p></w:footnote>
</w:footnotes>"#;

const COMMENTS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:comment w:id="1" w:author="Fixture Reviewer" w:date="2026-01-01T00:00:00Z" w:initials="FR">
    <w:p><w:r><w:t>Please verify this figure.</w:t></w:r></w:p>
  </w:comment>
</w:comments>"#;

const BIBLIOGRAPHY_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<b:Sources xmlns:b="http://schemas.openxmlformats.org/officeDocument/2006/bibliography">
  <b:Source>
    <b:Tag>Smith2020</b:Tag>
    <b:Author><b:Author><b:NameList><b:Person><b:Last>Smith</b:Last><b:First>John</b:First></b:Person></b:NameList></b:Author></b:Author>
    <b:Title>Example Research</b:Title>
    <b:Year>2020</b:Year>
  </b:Source>
</b:Sources>"#;

const DIAGRAM_DATA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<dgm:dataModel xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram">
  <dgm:ptLst><dgm:pt modelId="word-compat-feature-fixture"/></dgm:ptLst>
</dgm:dataModel>"#;

const DIAGRAM_LAYOUT: &str = "<dgm:layoutDef/>";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn fixture_paths() -> [PathBuf; 2] {
    let root = repo_root();
    [
        root.join("crates/tw-docx/tests/corpus/word_compatible_feature_test.docx"),
        root.join("app/test/fixtures/word_compatible_feature_test.docx"),
    ]
}

fn read_zip(path: &Path) -> HashMap<String, Vec<u8>> {
    let file = std::fs::File::open(path).expect("open fixture");
    let mut archive = ZipArchive::new(file).expect("read zip");
    let mut parts = HashMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("zip entry");
        let name = entry.name().to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        parts.insert(name, data);
    }
    parts
}

fn write_zip(path: &Path, parts: &HashMap<String, Vec<u8>>) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    let file = std::fs::File::create(path).expect("create output zip");
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default();
    let mut names: Vec<_> = parts.keys().cloned().collect();
    names.sort();
    for name in names {
        zip.start_file(&name, opts).expect("start zip entry");
        zip.write_all(&parts[&name]).expect("write zip entry");
    }
    zip.finish().expect("finish zip");
}

fn replace_once(haystack: &str, needle: &str, replacement: &str) -> (String, bool) {
    if let Some(idx) = haystack.find(needle) {
        let mut out = String::with_capacity(haystack.len() - needle.len() + replacement.len());
        out.push_str(&haystack[..idx]);
        out.push_str(replacement);
        out.push_str(&haystack[idx + needle.len()..]);
        (out, true)
    } else {
        (haystack.to_string(), false)
    }
}

fn ensure_relationship(rels_xml: &str, id: &str, rel_type: &str, target: &str) -> String {
    if rels_xml.contains(&format!("Id=\"{id}\"")) {
        return rels_xml.to_string();
    }
    let rel = format!(
        r#"<Relationship Id="{id}" Type="{rel_type}" Target="{target}"/>"#
    );
    rels_xml.replace("</Relationships>", &format!("{rel}</Relationships>"))
}

fn ensure_content_type_override(content_types: &str, part_name: &str, content_type: &str) -> String {
    let marker = format!("PartName=\"{part_name}\"");
    if content_types.contains(&marker) {
        return content_types.to_string();
    }
    let ov = format!(
        r#"<Override PartName="{part_name}" ContentType="{content_type}"/>"#
    );
    content_types.replace("</Types>", &format!("{ov}</Types>"))
}

fn ensure_png_default(content_types: &str) -> String {
    if content_types.contains(r#"Extension="png""#) {
        return content_types.to_string();
    }
    let def = r#"<Default Extension="png" ContentType="image/png"/>"#;
    content_types.replace(
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
        &format!(
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">{def}"#
        ),
    )
}

fn enrich_parts(parts: &mut HashMap<String, Vec<u8>>) {
    let document_key = "word/document.xml";
    let rels_key = "word/_rels/document.xml.rels";
    let content_types_key = "[Content_Types].xml";

    let mut document = String::from_utf8(parts[document_key].clone()).expect("document utf8");
    let mut changed = 0usize;

    for (needle, replacement) in [
        (IMAGE_PLACEHOLDER_PARA, IMAGE_PARAGRAPH),
        (SHAPE_PLACEHOLDER_PARA, SHAPE_PARAGRAPH),
        (DIAGRAM_PLACEHOLDER_PARA, DIAGRAM_PARAGRAPH),
        (CHART_PLACEHOLDER_PARA, CHART_PARAGRAPH),
        (OMML_PLACEHOLDER_PARA, OMML_PARAGRAPH),
    ] {
        let (next, did) = replace_once(&document, needle, replacement);
        document = next;
        if did {
            changed += 1;
        }
    }

    if !document.contains(OMML_TOKEN) && !document.contains("r:embed=\"rId11\"") {
        eprintln!("warning: no F10–F14 placeholders replaced; fixture may already be enriched");
    } else {
        eprintln!("replaced {changed} placeholder paragraph(s) in document.xml");
    }

    parts.insert(document_key.to_string(), document.into_bytes());

    parts.insert("word/media/image1.png".to_string(), PNG_1X1.to_vec());
    parts.insert(
        "word/charts/chart1.xml".to_string(),
        serialize_chart_xml(&ChartData::sample_bar()).into_bytes(),
    );
    parts.insert(
        "word/diagrams/data1.xml".to_string(),
        DIAGRAM_DATA.as_bytes().to_vec(),
    );
    parts.insert(
        "word/diagrams/layout1.xml".to_string(),
        DIAGRAM_LAYOUT.as_bytes().to_vec(),
    );

    let rels = String::from_utf8(
        parts
            .get(rels_key)
            .expect("document rels")
            .clone(),
    )
    .expect("rels utf8");
    let rels = ensure_relationship(
        &rels,
        "rId11",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
        "media/image1.png",
    );
    let rels = ensure_relationship(
        &rels,
        "rId12",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData",
        "diagrams/data1.xml",
    );
    let rels = ensure_relationship(
        &rels,
        "rId13",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramLayout",
        "diagrams/layout1.xml",
    );
    let rels = ensure_relationship(
        &rels,
        "rId14",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart",
        "charts/chart1.xml",
    );
    parts.insert(rels_key.to_string(), rels.into_bytes());

    let mut content_types = String::from_utf8(
        parts
            .get(content_types_key)
            .expect("content types")
            .clone(),
    )
    .expect("content types utf8");
    content_types = ensure_png_default(&content_types);
    content_types = ensure_content_type_override(
        &content_types,
        "/word/charts/chart1.xml",
        "application/vnd.openxmlformats-officedocument.drawingml.chart+xml",
    );
    content_types = ensure_content_type_override(
        &content_types,
        "/word/diagrams/data1.xml",
        "application/vnd.openxmlformats-officedocument.drawingml.diagramData+xml",
    );
    content_types = ensure_content_type_override(
        &content_types,
        "/word/diagrams/layout1.xml",
        "application/vnd.openxmlformats-officedocument.drawingml.diagramLayout+xml",
    );
    parts.insert(content_types_key.to_string(), content_types.into_bytes());

    enrich_f16_f19(parts);
}

fn enrich_f16_f19(parts: &mut HashMap<String, Vec<u8>>) {
    let document_key = "word/document.xml";
    let rels_key = "word/_rels/document.xml.rels";
    let content_types_key = "[Content_Types].xml";

    let mut document = String::from_utf8(parts[document_key].clone()).expect("document utf8");
    let mut changed = 0usize;

    for (needle, replacement) in [
        (FOOTNOTE_PLACEHOLDER_PARA, FOOTNOTE_PARAGRAPH),
        (TRACK_CHANGES_PLACEHOLDER_PARA, TRACK_CHANGES_PARAGRAPH),
        (COMMENT_PLACEHOLDER_PARA, COMMENT_PARAGRAPH),
        (BOOKMARK_PLACEHOLDER_PARA, BOOKMARK_PARAGRAPH),
        (HYPERLINK_PLACEHOLDER_PARA, HYPERLINK_PARAGRAPH),
        (BIBLIOGRAPHY_PLACEHOLDER_PARA, BIBLIOGRAPHY_PARAGRAPH),
    ] {
        let (next, did) = replace_once(&document, needle, replacement);
        document = next;
        if did {
            changed += 1;
        }
    }

    if changed > 0 {
        eprintln!("replaced {changed} F16–F19 placeholder paragraph(s) in document.xml");
    } else if !document.contains("<w:footnoteReference") {
        eprintln!("warning: no F16–F19 placeholders replaced; fixture may already be enriched");
    }

    parts.insert(document_key.to_string(), document.into_bytes());
    parts.insert(
        "word/footnotes.xml".to_string(),
        FOOTNOTES_XML.as_bytes().to_vec(),
    );
    parts.insert(
        "word/comments.xml".to_string(),
        COMMENTS_XML.as_bytes().to_vec(),
    );
    parts.insert(
        "word/bibliography.xml".to_string(),
        BIBLIOGRAPHY_XML.as_bytes().to_vec(),
    );

    let rels = String::from_utf8(
        parts
            .get(rels_key)
            .expect("document rels")
            .clone(),
    )
    .expect("rels utf8");
    let rels = ensure_relationship(
        &rels,
        "rId15",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        "footnotes.xml",
    );
    let rels = ensure_external_hyperlink_relationship(&rels, "rId16", "https://example.com");
    let rels = ensure_relationship(
        &rels,
        "rId17",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        "comments.xml",
    );
    parts.insert(rels_key.to_string(), rels.into_bytes());

    let mut content_types = String::from_utf8(
        parts
            .get(content_types_key)
            .expect("content types")
            .clone(),
    )
    .expect("content types utf8");
    content_types = ensure_content_type_override(
        &content_types,
        "/word/footnotes.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
    );
    content_types = ensure_content_type_override(
        &content_types,
        "/word/comments.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
    );
    content_types = ensure_content_type_override(
        &content_types,
        "/word/bibliography.xml",
        "application/xml",
    );
    parts.insert(content_types_key.to_string(), content_types.into_bytes());
}

fn ensure_external_hyperlink_relationship(rels_xml: &str, id: &str, target: &str) -> String {
    if rels_xml.contains(&format!("Id=\"{id}\"")) {
        return rels_xml.to_string();
    }
    let rel = format!(
        r#"<Relationship Id="{id}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{target}" TargetMode="External"/>"#
    );
    rels_xml.replace("</Relationships>", &format!("{rel}</Relationships>"))
}

fn main() {
    let source = fixture_paths()[0].clone();
    if !source.exists() {
        eprintln!("missing fixture: {}", source.display());
        std::process::exit(1);
    }

    let mut parts = read_zip(&source);
    enrich_parts(&mut parts);

    for path in fixture_paths() {
        write_zip(&path, &parts);
        println!("wrote {}", path.display());
    }

    let file = std::fs::read(&source).expect("read enriched");
    let out = tw_docx::import(&file).expect("import enriched fixture");
    let mut images = 0;
    let mut shapes = 0;
    for section in &out.document.sections {
        for block in &section.blocks {
            match block {
                tw_model::Block::ImageBlock(_) => images += 1,
                tw_model::Block::ShapeBlock(_) => shapes += 1,
                _ => {}
            }
        }
    }
    println!("sanity: images={images} shapes={shapes}");
    assert!(images >= 1, "expected at least one image block");
    assert!(shapes >= 3, "expected shape/diagram/chart blocks");
}

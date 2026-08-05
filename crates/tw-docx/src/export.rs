//! Serializes a [`Document`] back to `word/document.xml`, together with any
//! media parts and relationships its images need.
//!
//! Element order inside `w:pPr`, `w:rPr`, `w:tcPr` and `w:sectPr` follows the
//! OOXML schema sequences. Word rejects properties that appear out of order.

use tw_model::{
    Alignment, Block, BorderSpec, CellFormat, CharFormat, Color, Document, ImageBlock, LineSpacing,
    NodeId, Paragraph, ParaFormat, Run, RunContent, SectionFormat, StyleId, TabAlignment, Table,
    TableCell, TableRow, TextWrap, UnderlineStyle, VerticalAlign,
};

use std::collections::HashMap;

use crate::media::MediaWriter;
use crate::{DocxError, DocxPackage, MINIMAL_CONTENT_TYPES};

const TWIPS_PER_POINT: f32 = 20.0;
/// English Metric Units per point (914400 per inch, 72 points per inch).
const EMU_PER_POINT: f32 = 12700.0;
/// `w:sz` on a border is measured in eighths of a point.
const BORDER_EIGHTHS_PER_POINT: f32 = 8.0;

/// `w:sectPr` children we do not model. Copied across from the source document
/// so that headers, footers, and column setup survive a save.
const PRESERVED_SECTION_CHILDREN: &[&str] = &[
    "w:headerReference",
    "w:footerReference",
    "w:footnotePr",
    "w:endnotePr",
    "w:type",
    "w:paperSrc",
    "w:pgBorders",
    "w:lnNumType",
    "w:pgNumType",
    "w:cols",
    "w:formProt",
    "w:vAlign",
    "w:noEndnote",
    "w:titlePg",
    "w:textDirection",
    "w:bidi",
    "w:rtlGutter",
    "w:docGrid",
];

pub fn export_docx(doc: &Document, package: &DocxPackage) -> Result<Vec<u8>, DocxError> {
    let mut pkg = package.clone();
    let mut media = MediaWriter::new(&pkg);
    let document_xml = serialize_document_xml(doc, package, &mut media);

    pkg.parts
        .insert("word/document.xml".into(), document_xml.into_bytes());
    pkg.mark_modified("word/document.xml".into());

    let numbering_xml = crate::numbering::serialize_numbering_xml(&doc.settings.numbering);
    if !numbering_xml.is_empty() {
        pkg.parts
            .insert("word/numbering.xml".into(), numbering_xml.into_bytes());
        pkg.mark_modified("word/numbering.xml".into());
        ensure_numbering_content_type(&mut pkg);
        ensure_numbering_relationship(&mut pkg);
    }

    media.commit(&mut pkg);

    crate::opc::repack(&pkg)
}

fn ensure_numbering_content_type(pkg: &mut DocxPackage) {
    let part = "[Content_Types].xml";
    let bytes = pkg.parts.get(part).cloned().unwrap_or_else(|| {
        MINIMAL_CONTENT_TYPES.to_vec()
    });
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    let override_tag = r#"<Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>"#;
    if !xml.contains("/word/numbering.xml") {
        if let Some(end) = xml.rfind("</Types>") {
            xml.insert_str(end, override_tag);
        } else {
            xml.push_str(override_tag);
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn ensure_numbering_relationship(pkg: &mut DocxPackage) {
    let part = "word/_rels/document.xml.rels";
    let bytes = pkg.parts.get(part).cloned().unwrap_or_else(|| b"<Relationships/>".to_vec());
    let mut xml = String::from_utf8_lossy(&bytes).into_owned();
    if !xml.contains("numbering.xml") {
        let rel = r#"<Relationship Id="rIdNumbering" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>"#;
        if let Some(end) = xml.rfind("</Relationships>") {
            xml.insert_str(end, rel);
        } else {
            xml = format!("<Relationships>{rel}</Relationships>");
        }
        pkg.parts.insert(part.into(), xml.into_bytes());
        pkg.mark_modified(part.into());
    }
}

fn serialize_document_xml(doc: &Document, source: &DocxPackage, media: &mut MediaWriter) -> String {
    let original_sect_pr = original_section_properties(source);
    let mut body = String::new();
    let mut revision_ids = RevisionIdAllocator::new();

    let last = doc.sections.len().saturating_sub(1);
    for (index, section) in doc.sections.iter().enumerate() {
        for block in &section.blocks {
            body.push_str(&serialize_block(block, doc, media, &mut revision_ids));
        }
        let sect_pr = serialize_section_properties(&section.format, original_sect_pr.as_deref());
        if index == last {
            body.push_str(&sect_pr);
        } else {
            // A section break before the last section is carried by the
            // paragraph that ends the section, which is how Word records it.
            body.push_str(&format!("<w:p><w:pPr>{sect_pr}</w:pPr></w:p>"));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
  <w:body>{body}</w:body>
</w:document>"#
    )
}

fn serialize_block(
    block: &Block,
    doc: &Document,
    media: &mut MediaWriter,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    match block {
        Block::Paragraph(para) => serialize_paragraph(para, doc, revision_ids),
        Block::Table(table) => serialize_table(table, doc, media, revision_ids),
        Block::ImageBlock(image) => serialize_image_paragraph(image, media),
    }
}

// -- paragraphs ---------------------------------------------------------------

fn serialize_paragraph(
    para: &Paragraph,
    doc: &Document,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let mut xml = String::from("<w:p>");
    xml.push_str(&serialize_paragraph_properties(para, doc));
    for run in &para.runs {
        xml.push_str(&serialize_run(run, revision_ids));
    }
    xml.push_str("</w:p>");
    xml
}

fn serialize_paragraph_properties(para: &Paragraph, doc: &Document) -> String {
    let format = &para.format;
    let mut props = String::new();

    if let Some(style_id) = para.style_id.and_then(|id| ooxml_style_id(doc, id)) {
        props.push_str(&format!(
            r#"<w:pStyle w:val="{}"/>"#,
            escape_xml(&style_id)
        ));
    }
    if format.keep_together == Some(true) {
        props.push_str("<w:keepLines/>");
    }
    if format.keep_with_next == Some(true) {
        props.push_str("<w:keepNext/>");
    }
    if let Some(widow) = format.widow_orphan_control {
        if widow {
            props.push_str("<w:widowControl/>");
        } else {
            props.push_str(r#"<w:widowControl w:val="0"/>"#);
        }
    }
    if format.page_break_before == Some(true) {
        props.push_str("<w:pageBreakBefore/>");
    }
    if let Some(numbering) = format.numbering {
        props.push_str(&format!(
            r#"<w:numPr><w:ilvl w:val="{}"/><w:numId w:val="{}"/></w:numPr>"#,
            numbering.level, numbering.numbering_id
        ));
    }
    props.push_str(&serialize_spacing(format));
    props.push_str(&serialize_indent(format));
    props.push_str(&serialize_tab_stops(format));
    if let Some(alignment) = format.alignment {
        props.push_str(&format!(
            r#"<w:jc w:val="{}"/>"#,
            alignment_value(alignment)
        ));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{props}</w:pPr>")
    }
}

fn serialize_spacing(format: &ParaFormat) -> String {
    let mut attrs = String::new();
    if let Some(before) = format.space_before {
        attrs.push_str(&format!(r#" w:before="{}""#, to_twips(before)));
    }
    if let Some(after) = format.space_after {
        attrs.push_str(&format!(r#" w:after="{}""#, to_twips(after)));
    }
    match &format.line_spacing {
        // `w:line` is in 240ths of a line for the `auto` rule, and in twips for
        // the two fixed rules.
        Some(LineSpacing::Single) => attrs.push_str(r#" w:line="240" w:lineRule="auto""#),
        Some(LineSpacing::Double) => attrs.push_str(r#" w:line="480" w:lineRule="auto""#),
        Some(LineSpacing::Multiple(m)) => attrs.push_str(&format!(
            r#" w:line="{}" w:lineRule="auto""#,
            (m * 240.0).round() as i32
        )),
        Some(LineSpacing::AtLeast(points)) => attrs.push_str(&format!(
            r#" w:line="{}" w:lineRule="atLeast""#,
            to_twips(*points)
        )),
        Some(LineSpacing::Exactly(points)) => attrs.push_str(&format!(
            r#" w:line="{}" w:lineRule="exact""#,
            to_twips(*points)
        )),
        None => {}
    }
    if attrs.is_empty() {
        String::new()
    } else {
        format!("<w:spacing{attrs}/>")
    }
}

fn serialize_indent(format: &ParaFormat) -> String {
    let mut attrs = String::new();
    if let Some(left) = format.indent_left {
        attrs.push_str(&format!(r#" w:left="{}""#, to_twips(left)));
    }
    if let Some(right) = format.indent_right {
        attrs.push_str(&format!(r#" w:right="{}""#, to_twips(right)));
    }
    // A negative first-line indent is a hanging indent; the two are exclusive.
    match format.indent_first_line {
        Some(first) if first < 0.0 => {
            attrs.push_str(&format!(r#" w:hanging="{}""#, to_twips(-first)))
        }
        Some(first) => attrs.push_str(&format!(r#" w:firstLine="{}""#, to_twips(first))),
        None => {}
    }
    if attrs.is_empty() {
        String::new()
    } else {
        format!("<w:ind{attrs}/>")
    }
}

fn serialize_tab_stops(format: &ParaFormat) -> String {
    if format.tab_stops.is_empty() {
        return String::new();
    }
    let mut xml = String::from("<w:tabs>");
    for stop in &format.tab_stops {
        let align = match stop.alignment {
            TabAlignment::Center => "center",
            TabAlignment::Right => "right",
            TabAlignment::Decimal => "decimal",
            TabAlignment::Bar => "bar",
            TabAlignment::Left => "left",
        };
        xml.push_str(&format!(
            r#"<w:tab w:val="{align}" w:pos="{}"/>"#,
            to_twips(stop.position)
        ));
    }
    xml.push_str("</w:tabs>");
    xml
}

/// The `w:styleId` the style was imported under, so a saved document keeps
/// referring to the style definition already in `styles.xml`.
fn ooxml_style_id(doc: &Document, style_id: StyleId) -> Option<String> {
    if let Some((ooxml_id, _)) = doc
        .styles
        .ooxml_style_ids
        .iter()
        .find(|(_, id)| **id == style_id)
    {
        return Some(ooxml_id.clone());
    }
    doc.styles
        .paragraph_styles
        .get(&style_id)
        .map(|style| style.name.replace(' ', ""))
}

// -- runs ---------------------------------------------------------------------

fn serialize_run(run: &Run, revision_ids: &mut RevisionIdAllocator) -> String {
    let deleted = matches!(
        run.revision.as_ref().map(|r| r.revision_type),
        Some(tw_model::RevisionType::Delete)
    );

    let content = match &run.content {
        RunContent::Text(text) => serialize_text(text, deleted),
        RunContent::Tab => "<w:tab/>".to_string(),
        RunContent::Break(tw_model::BreakType::Page) => r#"<w:br w:type="page"/>"#.to_string(),
        RunContent::Break(tw_model::BreakType::Column) => r#"<w:br w:type="column"/>"#.to_string(),
        RunContent::Break(tw_model::BreakType::Line) => "<w:br/>".to_string(),
    };

    let xml = format!(
        "<w:r>{}{}</w:r>",
        serialize_run_properties(&run.format),
        content
    );

    // Revision marks wrap the run; they are not run properties.
    match &run.revision {
        Some(rev) => {
            let tag = if deleted { "w:del" } else { "w:ins" };
            format!(
                r#"<{tag} w:id="{}" w:author="{}" w:date="{}">{xml}</{tag}>"#,
                revision_ids.id_for(&rev.id),
                escape_xml(&rev.author),
                rev.timestamp.to_rfc3339()
            )
        }
        None => xml,
    }
}

/// Text content of a run. Tabs and newlines have to become `w:tab` and `w:br`
/// elements: left as literal characters inside `w:t`, Word renders them as
/// ordinary spaces.
fn serialize_text(text: &str, deleted: bool) -> String {
    let tag = if deleted { "w:delText" } else { "w:t" };
    let mut out = String::new();
    let mut pending = String::new();

    for ch in text.chars() {
        let element = match ch {
            '\t' => "<w:tab/>",
            '\n' => "<w:br/>",
            _ => {
                pending.push(ch);
                continue;
            }
        };
        if !pending.is_empty() {
            // `xml:space` keeps leading and trailing spaces, which Word would
            // otherwise collapse.
            out.push_str(&format!(
                r#"<{tag} xml:space="preserve">{}</{tag}>"#,
                escape_xml(&pending)
            ));
            pending.clear();
        }
        out.push_str(element);
    }

    if !pending.is_empty() || out.is_empty() {
        out.push_str(&format!(
            r#"<{tag} xml:space="preserve">{}</{tag}>"#,
            escape_xml(&pending)
        ));
    }
    out
}

struct RevisionIdAllocator {
    next: u32,
    ids: HashMap<NodeId, u32>,
}

impl RevisionIdAllocator {
    fn new() -> Self {
        Self {
            next: 1,
            ids: HashMap::new(),
        }
    }

    fn id_for(&mut self, node_id: &NodeId) -> u32 {
        *self
            .ids
            .entry(*node_id)
            .or_insert_with(|| {
                let id = self.next;
                self.next += 1;
                id
            })
    }
}

fn serialize_run_properties(format: &CharFormat) -> String {
    let mut props = String::new();

    if let Some(family) = &format.font_family {
        let family = escape_xml(family);
        props.push_str(&format!(
            r#"<w:rFonts w:ascii="{family}" w:hAnsi="{family}" w:cs="{family}"/>"#
        ));
    }
    // An explicit `false` has to be written out: it turns off a toggle the
    // paragraph style switched on.
    props.push_str(&toggle("w:b", format.bold));
    props.push_str(&toggle("w:i", format.italic));
    props.push_str(&toggle("w:strike", format.strikethrough));
    if let Some(color) = format.color {
        props.push_str(&format!(r#"<w:color w:val="{}"/>"#, hex_rgb(color)));
    }
    if let Some(size) = format.font_size {
        let half_points = (size * 2.0).round() as i32;
        props.push_str(&format!(
            r#"<w:sz w:val="{half_points}"/><w:szCs w:val="{half_points}"/>"#
        ));
    }
    if let Some(name) = format.highlight.and_then(highlight_name) {
        props.push_str(&format!(r#"<w:highlight w:val="{name}"/>"#));
    }
    if let Some(underline) = format.underline {
        props.push_str(&format!(
            r#"<w:u w:val="{}"/>"#,
            underline_value(underline)
        ));
    }
    if format.superscript == Some(true) {
        props.push_str(r#"<w:vertAlign w:val="superscript"/>"#);
    } else if format.subscript == Some(true) {
        props.push_str(r#"<w:vertAlign w:val="subscript"/>"#);
    }
    if let Some(language) = &format.language {
        props.push_str(&format!(r#"<w:lang w:val="{}"/>"#, escape_xml(language)));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{props}</w:rPr>")
    }
}

fn toggle(tag: &str, value: Option<bool>) -> String {
    match value {
        Some(true) => format!("<{tag}/>"),
        Some(false) => format!(r#"<{tag} w:val="0"/>"#),
        None => String::new(),
    }
}

// -- tables -------------------------------------------------------------------

fn serialize_table(
    table: &Table,
    doc: &Document,
    media: &mut MediaWriter,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let widths = &table.format.column_widths;
    let mut xml = String::from("<w:tbl><w:tblPr>");

    let width = table
        .format
        .width
        .unwrap_or_else(|| widths.iter().sum::<f32>());
    xml.push_str(&format!(
        r#"<w:tblW w:w="{}" w:type="dxa"/>"#,
        to_twips(width)
    ));
    if let Some(border) = table.format.border {
        xml.push_str(&serialize_table_borders(border));
    }
    xml.push_str("</w:tblPr><w:tblGrid>");
    for column in widths {
        xml.push_str(&format!(r#"<w:gridCol w:w="{}"/>"#, to_twips(*column)));
    }
    xml.push_str("</w:tblGrid>");

    for row in &table.rows {
        xml.push_str(&serialize_table_row(row, widths, doc, media, revision_ids));
    }
    xml.push_str("</w:tbl>");
    xml
}

fn serialize_table_borders(border: BorderSpec) -> String {
    let size = (border.width * BORDER_EIGHTHS_PER_POINT).round().max(1.0) as i32;
    let color = hex_rgb(border.color);
    let edge = |name: &str| {
        format!(r#"<w:{name} w:val="single" w:sz="{size}" w:space="0" w:color="{color}"/>"#)
    };
    format!(
        "<w:tblBorders>{}{}{}{}{}{}</w:tblBorders>",
        edge("top"),
        edge("left"),
        edge("bottom"),
        edge("right"),
        edge("insideH"),
        edge("insideV"),
    )
}

fn serialize_table_row(
    row: &TableRow,
    widths: &[f32],
    doc: &Document,
    media: &mut MediaWriter,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let mut xml = String::from("<w:tr>");
    if let Some(height) = row.height {
        xml.push_str(&format!(
            r#"<w:trPr><w:trHeight w:val="{}"/></w:trPr>"#,
            to_twips(height)
        ));
    }

    let mut column = 0usize;
    for cell in &row.cells {
        let span = cell.format.colspan.max(1) as usize;
        let width: f32 = widths
            .iter()
            .skip(column)
            .take(span)
            .copied()
            .sum::<f32>();
        xml.push_str(&serialize_table_cell(cell, width, doc, media, revision_ids));
        column += span;
    }
    xml.push_str("</w:tr>");
    xml
}

fn serialize_table_cell(
    cell: &TableCell,
    width: f32,
    doc: &Document,
    media: &mut MediaWriter,
    revision_ids: &mut RevisionIdAllocator,
) -> String {
    let mut xml = String::from("<w:tc>");
    xml.push_str(&serialize_cell_properties(&cell.format, width));

    let mut has_paragraph = false;
    for block in &cell.blocks {
        has_paragraph |= matches!(block, Block::Paragraph(_));
        xml.push_str(&serialize_block(block, doc, media, revision_ids));
    }
    // A cell must end with a paragraph or Word treats the file as corrupt.
    if !has_paragraph {
        xml.push_str("<w:p/>");
    }

    xml.push_str("</w:tc>");
    xml
}

fn serialize_cell_properties(format: &CellFormat, width: f32) -> String {
    let mut props = String::new();
    if width > 0.0 {
        props.push_str(&format!(
            r#"<w:tcW w:w="{}" w:type="dxa"/>"#,
            to_twips(width)
        ));
    }
    if format.colspan > 1 {
        props.push_str(&format!(r#"<w:gridSpan w:val="{}"/>"#, format.colspan));
    }
    if format.rowspan > 1 {
        props.push_str(r#"<w:vMerge w:val="restart"/>"#);
    }
    if let Some(border) = format.border {
        let size = (border.width * BORDER_EIGHTHS_PER_POINT).round().max(1.0) as i32;
        let color = hex_rgb(border.color);
        let edge = |name: &str| {
            format!(r#"<w:{name} w:val="single" w:sz="{size}" w:space="0" w:color="{color}"/>"#)
        };
        props.push_str(&format!(
            "<w:tcBorders>{}{}{}{}</w:tcBorders>",
            edge("top"),
            edge("left"),
            edge("bottom"),
            edge("right"),
        ));
    }
    if let Some(background) = format.background {
        props.push_str(&format!(
            r#"<w:shd w:val="clear" w:color="auto" w:fill="{}"/>"#,
            hex_rgb(background)
        ));
    }
    if format.vertical_align != VerticalAlign::Top {
        let value = match format.vertical_align {
            VerticalAlign::Middle => "center",
            VerticalAlign::Bottom => "bottom",
            VerticalAlign::Top => "top",
        };
        props.push_str(&format!(r#"<w:vAlign w:val="{value}"/>"#));
    }

    if props.is_empty() {
        String::new()
    } else {
        format!("<w:tcPr>{props}</w:tcPr>")
    }
}

// -- images -------------------------------------------------------------------

fn serialize_image_paragraph(image: &ImageBlock, media: &mut MediaWriter) -> String {
    // Without bytes there is nothing to point a relationship at, so the block
    // can only survive as the empty paragraph it occupies.
    let Some(relationship_id) = media.reference(&image.data) else {
        return "<w:p/>".to_string();
    };

    let cx = to_emu(image.display_width);
    let cy = to_emu(image.display_height);
    let name = escape_xml(&image.data.asset_id);
    let id = media.next_drawing_id();

    let graphic = format!(
        r#"<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="{id}" name="{name}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{relationship_id}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic>"#
    );
    let doc_pr = format!(r#"<wp:docPr id="{id}" name="Picture {id}"/><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect="1"/></wp:cNvGraphicFramePr>"#);

    let drawing = match image.anchor {
        Some(anchor) => {
            let behind = matches!(image.wrap, TextWrap::Behind);
            format!(
                r#"<wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="1" behindDoc="{}" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="{}"><wp:posOffset>{}</wp:posOffset></wp:positionH><wp:positionV relativeFrom="{}"><wp:posOffset>{}</wp:posOffset></wp:positionV><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/>{}{doc_pr}{graphic}</wp:anchor>"#,
                if behind { 1 } else { 0 },
                anchor_origin_value(anchor.origin_x),
                to_emu(anchor.x),
                anchor_origin_value(anchor.origin_y),
                to_emu(anchor.y),
                wrap_element(image.wrap),
            )
        }
        None => format!(
            r#"<wp:inline distT="0" distB="0" distL="0" distR="0"><wp:extent cx="{cx}" cy="{cy}"/><wp:effectExtent l="0" t="0" r="0" b="0"/>{doc_pr}{graphic}</wp:inline>"#
        ),
    };

    format!("<w:p><w:r><w:drawing>{drawing}</w:drawing></w:r></w:p>")
}

fn wrap_element(wrap: TextWrap) -> &'static str {
    match wrap {
        TextWrap::Square => r#"<wp:wrapSquare wrapText="bothSides"/>"#,
        TextWrap::TopBottom => "<wp:wrapTopAndBottom/>",
        TextWrap::Inline | TextWrap::Behind | TextWrap::InFront => "<wp:wrapNone/>",
    }
}

fn anchor_origin_value(origin: tw_model::AnchorOrigin) -> &'static str {
    match origin {
        tw_model::AnchorOrigin::Column => "column",
        tw_model::AnchorOrigin::Page => "page",
        tw_model::AnchorOrigin::Margin => "margin",
    }
}

// -- sections -----------------------------------------------------------------

fn serialize_section_properties(format: &SectionFormat, original: Option<&str>) -> String {
    let mut xml = String::from("<w:sectPr>");

    // Header and footer references point at parts we pass through untouched,
    // so they have to be carried over rather than regenerated.
    if let Some(original) = original {
        xml.push_str(&copy_children(original, PRESERVED_SECTION_CHILDREN));
    }
    xml.push_str(&format!(
        r#"<w:pgSz w:w="{}" w:h="{}"/>"#,
        to_twips(format.page_width),
        to_twips(format.page_height)
    ));
    xml.push_str(&format!(
        r#"<w:pgMar w:top="{}" w:right="{}" w:bottom="{}" w:left="{}" w:header="720" w:footer="720" w:gutter="0"/>"#,
        to_twips(format.margin_top),
        to_twips(format.margin_right),
        to_twips(format.margin_bottom),
        to_twips(format.margin_left)
    ));
    xml.push_str("</w:sectPr>");
    xml
}

fn original_section_properties(package: &DocxPackage) -> Option<String> {
    let bytes = package.parts.get("word/document.xml")?;
    let xml = String::from_utf8_lossy(bytes);
    let start = xml.find("<w:sectPr")?;
    let end = xml[start..].find("</w:sectPr>")? + start + "</w:sectPr>".len();
    Some(xml[start..end].to_string())
}

/// Copies the named child elements out of an element, start tag to end tag,
/// preserving whatever attributes and content they carried.
fn copy_children(xml: &str, tags: &[&str]) -> String {
    let mut out = String::new();
    for tag in tags {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let mut rest = xml;
        while let Some(start) = rest.find(&open) {
            let after = &rest[start..];
            let Some(head_end) = after.find('>') else {
                break;
            };
            // Distinguish `<w:cols/>` from `<w:cols>...</w:cols>`, and skip
            // tags that only share a name prefix such as `w:type` vs `w:typo`.
            if !matches!(
                after.as_bytes().get(open.len()),
                Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r')
            ) {
                rest = &after[head_end + 1..];
                continue;
            }
            let end = if after[..head_end].ends_with('/') {
                head_end + 1
            } else {
                match after.find(&close) {
                    Some(i) => i + close.len(),
                    None => break,
                }
            };
            out.push_str(&after[..end]);
            rest = &after[end..];
        }
    }
    out
}

// -- primitives ---------------------------------------------------------------

fn alignment_value(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Left => "left",
        Alignment::Center => "center",
        Alignment::Right => "right",
        Alignment::Justify => "both",
    }
}

fn underline_value(style: UnderlineStyle) -> &'static str {
    match style {
        UnderlineStyle::None => "none",
        UnderlineStyle::Single => "single",
        UnderlineStyle::Double => "double",
        UnderlineStyle::Dotted => "dotted",
        UnderlineStyle::Dashed => "dash",
        UnderlineStyle::Wave => "wave",
    }
}

/// Word's highlight is a named palette, not a colour; anything outside it
/// cannot be expressed as `w:highlight`.
fn highlight_name(color: Color) -> Option<&'static str> {
    match (color.r, color.g, color.b) {
        (255, 255, 0) => Some("yellow"),
        (0, 255, 0) => Some("green"),
        (0, 255, 255) => Some("cyan"),
        (255, 0, 255) => Some("magenta"),
        (255, 0, 0) => Some("red"),
        (0, 0, 255) => Some("blue"),
        _ => None,
    }
}

fn hex_rgb(color: Color) -> String {
    format!("{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

pub(crate) fn to_twips(points: f32) -> i32 {
    (points * TWIPS_PER_POINT).round() as i32
}

fn to_emu(points: f32) -> i64 {
    (points * EMU_PER_POINT).round() as i64
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_model::Document;

    fn xml_for(doc: &Document) -> String {
        let package = DocxPackage::minimal();
        let mut media = MediaWriter::new(&package);
        serialize_document_xml(doc, &package, &mut media)
    }

    #[test]
    fn serializes_bold_run() {
        let mut doc = Document::with_paragraph("Hello");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].format = CharFormat {
                bold: Some(true),
                ..Default::default()
            };
        }
        let xml = xml_for(&doc);
        assert!(xml.contains("<w:b/>"));
        assert!(xml.contains("Hello"));
    }

    #[test]
    fn bold_turned_off_is_written_out() {
        // A run that overrides its style's bold has to say so explicitly.
        let mut doc = Document::with_paragraph("Plain");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].format.bold = Some(false);
        }
        assert!(xml_for(&doc).contains(r#"<w:b w:val="0"/>"#));
    }

    #[test]
    fn leading_whitespace_is_preserved() {
        let doc = Document::with_paragraph("  indented by spaces");
        assert!(xml_for(&doc).contains(r#"xml:space="preserve""#));
    }

    #[test]
    fn a_deletion_wraps_the_run_and_uses_del_text() {
        let mut doc = Document::with_paragraph("Gone");
        if let Block::Paragraph(para) = &mut doc.sections[0].blocks[0] {
            para.runs[0].revision = Some(tw_model::Revision::delete("Reviewer"));
        }
        let xml = xml_for(&doc);
        assert!(xml.contains("<w:del "), "expected a wrapping w:del");
        assert!(xml.contains("<w:delText"), "deleted text uses w:delText");
        assert!(
            xml.find("<w:del ").unwrap() < xml.find("<w:r>").unwrap(),
            "w:del must wrap the run, not sit inside w:rPr"
        );
    }

    #[test]
    fn copy_children_takes_whole_elements() {
        let sect = r#"<w:sectPr><w:headerReference r:id="rId7"/><w:cols w:num="2"><w:col/></w:cols><w:pgSz w:w="1"/></w:sectPr>"#;
        let copied = copy_children(sect, &["w:headerReference", "w:cols"]);
        assert!(copied.contains(r#"<w:headerReference r:id="rId7"/>"#));
        assert!(copied.contains("<w:cols w:num=\"2\"><w:col/></w:cols>"));
        assert!(!copied.contains("pgSz"));
    }
}

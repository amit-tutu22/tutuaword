//! Build the seven F24 starter `.docx` templates into `app/assets/templates/`.
//!
//! Each template stamps `settings.template_name` and a bound `DocumentTheme`
//! (F24.S2). Run from repo root:
//! `cargo run -p tw-docx --example build_starter_templates`

use std::fs;
use std::path::PathBuf;

use tw_docx::{export, DocxPackage};
use tw_model::{Block, Document, DocumentTheme, Paragraph, Table};

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../app/assets/templates")
}

fn heading_para(doc: &Document, name: &str, text: &str) -> Paragraph {
    let mut para = Paragraph::with_text(text);
    if let Some(style) = doc.styles.find_style_by_name(name) {
        para.style_id = Some(style.id);
    }
    para
}

fn write_template(
    file_name: &str,
    template_name: &str,
    theme_name: &str,
    build: impl FnOnce(&Document) -> Vec<Block>,
) {
    let mut doc = Document::new();
    doc.settings.template_name = Some(template_name.into());
    doc.settings.theme = DocumentTheme::by_name(theme_name)
        .unwrap_or_else(|| panic!("unknown theme {theme_name}"));
    let blocks = build(&doc);
    doc.sections[0].blocks = blocks;
    let bytes = export(&doc, &DocxPackage::minimal()).expect("export template");
    let path = out_dir().join(file_name);
    fs::write(&path, bytes).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    println!("wrote {} (theme={theme_name})", path.display());
}

fn resume() {
    write_template("resume.docx", "Resume", "Facet", |doc| {
        vec![
            Block::Paragraph(heading_para(doc, "Heading 1", "Professional Resume")),
            Block::Paragraph(Paragraph::with_text("Your Name")),
            Block::Paragraph(Paragraph::with_text("city@email.example · (555) 010-0100")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Experience")),
            Block::Paragraph(Paragraph::with_text(
                "Role Title — Company Name (20XX–Present)",
            )),
            Block::Paragraph(Paragraph::with_text(
                "Describe impact with measurable outcomes.",
            )),
            Block::Paragraph(heading_para(doc, "Heading 2", "Education")),
            Block::Paragraph(Paragraph::with_text("Degree — University")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Skills")),
            Block::Paragraph(Paragraph::with_text(
                "Communication · Leadership · Tools",
            )),
        ]
    });
}

fn letter() {
    write_template("letter.docx", "Letter", "Office", |doc| {
        vec![
            Block::Paragraph(heading_para(doc, "Heading 1", "Business Letter")),
            Block::Paragraph(Paragraph::with_text("[Your Name]")),
            Block::Paragraph(Paragraph::with_text("[Street Address]")),
            Block::Paragraph(Paragraph::with_text("[City, ST ZIP]")),
            Block::Paragraph(Paragraph::with_text("")),
            Block::Paragraph(Paragraph::with_text("[Date]")),
            Block::Paragraph(Paragraph::with_text("")),
            Block::Paragraph(Paragraph::with_text("[Recipient Name]")),
            Block::Paragraph(Paragraph::with_text("[Company]")),
            Block::Paragraph(Paragraph::with_text("")),
            Block::Paragraph(Paragraph::with_text("Dear [Recipient],")),
            Block::Paragraph(Paragraph::with_text(
                "Opening paragraph stating purpose.",
            )),
            Block::Paragraph(Paragraph::with_text(
                "Supporting details and call to action.",
            )),
            Block::Paragraph(Paragraph::with_text("Sincerely,")),
            Block::Paragraph(Paragraph::with_text("[Your Name]")),
        ]
    });
}

fn invoice() {
    write_template("invoice.docx", "Invoice", "Ion", |doc| {
        let mut table = Table::new(3, 3);
        let labels = [
            ["Item", "Qty", "Amount"],
            ["Service A", "1", "$100.00"],
            ["Service B", "2", "$50.00"],
        ];
        for (ri, row) in labels.iter().enumerate() {
            for (ci, text) in row.iter().enumerate() {
                table.rows[ri].cells[ci].blocks =
                    vec![Block::Paragraph(Paragraph::with_text(*text))];
            }
        }
        vec![
            Block::Paragraph(heading_para(doc, "Heading 1", "Invoice")),
            Block::Paragraph(Paragraph::with_text("Invoice #: INV-0001")),
            Block::Paragraph(Paragraph::with_text("Bill To: [Client Name]")),
            Block::Table(table),
            Block::Paragraph(Paragraph::with_text("Total Due: $200.00")),
            Block::Paragraph(Paragraph::with_text("Thank you for your business.")),
        ]
    });
}

fn brochure() {
    write_template("brochure.docx", "Brochure", "Facet", |doc| {
        vec![
            Block::Paragraph(heading_para(doc, "Heading 1", "Product Brochure")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Headline Benefit")),
            Block::Paragraph(Paragraph::with_text(
                "Short pitch describing the product or service.",
            )),
            Block::Paragraph(heading_para(doc, "Heading 2", "Features")),
            Block::Paragraph(Paragraph::with_text("• Feature one")),
            Block::Paragraph(Paragraph::with_text("• Feature two")),
            Block::Paragraph(Paragraph::with_text("• Feature three")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Call to Action")),
            Block::Paragraph(Paragraph::with_text("Contact us today.")),
        ]
    });
}

fn newsletter() {
    write_template("newsletter.docx", "Newsletter", "Ion", |doc| {
        vec![
            Block::Paragraph(heading_para(doc, "Heading 1", "Team Newsletter")),
            Block::Paragraph(Paragraph::with_text("Issue Date · Volume 1")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Lead Story")),
            Block::Paragraph(Paragraph::with_text(
                "Summarize the main update for readers.",
            )),
            Block::Paragraph(heading_para(doc, "Heading 2", "Quick Hits")),
            Block::Paragraph(Paragraph::with_text("• Announcement one")),
            Block::Paragraph(Paragraph::with_text("• Announcement two")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Upcoming")),
            Block::Paragraph(Paragraph::with_text("List events and deadlines.")),
        ]
    });
}

fn business_proposal() {
    write_template(
        "business_proposal.docx",
        "Business Proposal",
        "Office",
        |doc| {
            vec![
                Block::Paragraph(heading_para(doc, "Heading 1", "Business Proposal")),
                Block::Paragraph(Paragraph::with_text("Prepared for: [Client]")),
                Block::Paragraph(heading_para(doc, "Heading 2", "Executive Summary")),
                Block::Paragraph(Paragraph::with_text(
                    "Outline the opportunity and recommended approach.",
                )),
                Block::Paragraph(heading_para(doc, "Heading 2", "Scope of Work")),
                Block::Paragraph(Paragraph::with_text("Deliverable 1")),
                Block::Paragraph(Paragraph::with_text("Deliverable 2")),
                Block::Paragraph(heading_para(doc, "Heading 2", "Investment")),
                Block::Paragraph(Paragraph::with_text("Proposed fee and timeline.")),
            ]
        },
    );
}

fn research_paper() {
    write_template("research_paper.docx", "Research Paper", "Facet", |doc| {
        vec![
            Block::Paragraph(heading_para(doc, "Heading 1", "Research Paper")),
            Block::Paragraph(Paragraph::with_text("Author Name · Affiliation")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Abstract")),
            Block::Paragraph(Paragraph::with_text(
                "Brief summary of research question, method, and findings.",
            )),
            Block::Paragraph(heading_para(doc, "Heading 2", "Introduction")),
            Block::Paragraph(Paragraph::with_text(
                "Background and motivation for the study.",
            )),
            Block::Paragraph(heading_para(doc, "Heading 2", "Methods")),
            Block::Paragraph(Paragraph::with_text("Describe materials and procedure.")),
            Block::Paragraph(heading_para(doc, "Heading 2", "Results")),
            Block::Paragraph(Paragraph::with_text("Present key results.")),
            Block::Paragraph(heading_para(doc, "Heading 2", "References")),
            Block::Paragraph(Paragraph::with_text("[1] Author. Title. Year.")),
        ]
    });
}

fn main() {
    let dir = out_dir();
    fs::create_dir_all(&dir).expect("create templates dir");
    resume();
    letter();
    invoice();
    brochure();
    newsletter();
    business_proposal();
    research_paper();
    println!("F24 templates ready in {}", dir.display());
}

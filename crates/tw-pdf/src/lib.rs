use std::collections::{BTreeMap, BTreeSet, HashMap};

use image::GenericImageView;
use thiserror::Error;
use tw_layout::{LayoutBox, LayoutEngine, PageLayout, TextLine};
use tw_model::Document;
use tw_render::DisplayListBuilder;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("pdf export failed: {0}")]
    ExportFailed(String),
}

/// PDF fidelity gate (see `docs/risk-mitigation.md`).
///
/// Structural export uses Helvetica + layout positions (and images).
/// VisualMatch embeds document faces (`/FontFile2`) for WYSIWYG export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PdfFidelity {
    /// Glyph positions from layout; Standard 14 Helvetica — not a Word visual match.
    #[default]
    Structural,
    /// Font-embedded visual match (full faces via `/FontFile2`).
    VisualMatch,
}

/// How page content is scaled onto the printable area (F25.S2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintScaleMode {
    /// Use [`PrintLayoutOptions::scale_percent`] (100 = actual size).
    #[default]
    ActualSize,
    /// Shrink uniformly so the page fits inside the margin box.
    FitToMargins,
    /// Explicit percent via [`PrintLayoutOptions::scale_percent`].
    CustomPercent,
}

/// Duplex / two-sided print mode (F25.S4). Platform attribute; also hints booklet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintDuplexMode {
    #[default]
    Simplex,
    /// Flip on long edge (typical booklet / portrait duplex).
    LongEdge,
    /// Flip on short edge.
    ShortEdge,
}

/// Scale + printer margins + sheet attributes for print PDF (F25.S2/S4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintLayoutOptions {
    pub scale_mode: PrintScaleMode,
    /// Percent of actual size (used by ActualSize / CustomPercent). Clamped 10..=400.
    pub scale_percent: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    /// Two-sided printing (F25.S4) — primarily a platform print attribute.
    pub duplex: PrintDuplexMode,
    /// Pages per physical sheet: 1, 2, 4, 6, 9, or 16 (F25.S4 N-up).
    pub pages_per_sheet: u8,
    /// Booklet imposition (2-up + signature order); implies duplex LongEdge (F25.S4).
    pub booklet: bool,
}

impl Default for PrintLayoutOptions {
    fn default() -> Self {
        Self {
            scale_mode: PrintScaleMode::ActualSize,
            scale_percent: 100.0,
            margin_left: 0.0,
            margin_right: 0.0,
            margin_top: 0.0,
            margin_bottom: 0.0,
            duplex: PrintDuplexMode::Simplex,
            pages_per_sheet: 1,
            booklet: false,
        }
    }
}

/// One physical sheet after N-up / booklet planning (F25.S4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintSheet {
    /// Source page indices (None = blank slot), length = pages_per_sheet.
    pub slots: Vec<Option<usize>>,
}

/// Planned sheets for an N-page document (F25.S4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintSheetPlan {
    pub pages_per_sheet: u8,
    pub cols: u8,
    pub rows: u8,
    pub duplex: PrintDuplexMode,
    pub booklet: bool,
    pub sheets: Vec<PrintSheet>,
}

/// Normalize N-up to a supported grid size.
pub fn normalize_pages_per_sheet(n: u8) -> u8 {
    match n {
        0 | 1 => 1,
        2 => 2,
        3 | 4 => 4,
        5 | 6 => 6,
        7 | 8 | 9 => 9,
        _ => 16,
    }
}

/// Grid dimensions for an N-up sheet `(cols, rows)`.
pub fn nup_grid(pages_per_sheet: u8) -> (u8, u8) {
    match normalize_pages_per_sheet(pages_per_sheet) {
        1 => (1, 1),
        2 => (2, 1),
        4 => (2, 2),
        6 => (3, 2),
        9 => (3, 3),
        _ => (4, 4),
    }
}

/// Classic saddle-stitch booklet page order (0-based), padded to a multiple of 4.
///
/// Each group of 4 indices is one physical sheet (front L/R, back L/R) for 2-up duplex.
pub fn booklet_page_order(page_count: usize) -> Vec<Option<usize>> {
    if page_count == 0 {
        return Vec::new();
    }
    let padded = page_count.div_ceil(4) * 4;
    let sheets = padded / 4;
    let mut out = Vec::with_capacity(padded);
    for i in 0..sheets {
        let a = padded - 1 - 2 * i;
        let b = 2 * i;
        let c = 2 * i + 1;
        let d = padded - 2 - 2 * i;
        for idx in [a, b, c, d] {
            out.push(if idx < page_count { Some(idx) } else { None });
        }
    }
    out
}

/// Plan physical sheets for N-up and/or booklet printing (F25.S4).
pub fn build_print_sheet_plan(page_count: usize, layout: &PrintLayoutOptions) -> PrintSheetPlan {
    let booklet = layout.booklet;
    let pages_per = if booklet {
        2
    } else {
        normalize_pages_per_sheet(layout.pages_per_sheet)
    };
    let (cols, rows) = nup_grid(pages_per);
    let duplex = if booklet {
        PrintDuplexMode::LongEdge
    } else {
        layout.duplex
    };

    let sheets = if booklet {
        booklet_page_order(page_count)
            .chunks(2)
            .map(|pair| PrintSheet {
                slots: pair.to_vec(),
            })
            .collect()
    } else if pages_per <= 1 {
        (0..page_count)
            .map(|i| PrintSheet {
                slots: vec![Some(i)],
            })
            .collect()
    } else {
        let pps = pages_per as usize;
        let sheet_count = if page_count == 0 {
            0
        } else {
            page_count.div_ceil(pps)
        };
        (0..sheet_count)
            .map(|s| {
                let mut slots = Vec::with_capacity(pps);
                for k in 0..pps {
                    let idx = s * pps + k;
                    slots.push(if idx < page_count { Some(idx) } else { None });
                }
                PrintSheet { slots }
            })
            .collect()
    };

    PrintSheetPlan {
        pages_per_sheet: pages_per,
        cols,
        rows,
        duplex,
        booklet,
        sheets,
    }
}

/// Cell placement transform for slot `slot` on an N-up sheet (PDF user space).
pub fn nup_cell_transform(
    sheet_w: f32,
    sheet_h: f32,
    cols: u8,
    rows: u8,
    slot: usize,
    page_w: f32,
    page_h: f32,
) -> PrintContentTransform {
    let cols = cols.max(1) as f32;
    let rows = rows.max(1) as f32;
    let cell_w = sheet_w / cols;
    let cell_h = sheet_h / rows;
    let col = (slot as f32) % cols;
    let row_from_top = ((slot as f32) / cols).floor();
    // PDF y grows upward; row 0 is top of sheet.
    let row_from_bottom = rows - 1.0 - row_from_top;
    let page_w = page_w.max(1.0);
    let page_h = page_h.max(1.0);
    let scale = (cell_w / page_w).min(cell_h / page_h);
    let content_w = page_w * scale;
    let content_h = page_h * scale;
    let tx = col * cell_w + (cell_w - content_w) * 0.5;
    let ty = row_from_bottom * cell_h + (cell_h - content_h) * 0.5;
    PrintContentTransform { scale, tx, ty }
}

/// Resolved CTM for placing page content (PDF user space, origin bottom-left).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintContentTransform {
    pub scale: f32,
    pub tx: f32,
    pub ty: f32,
}

impl PrintContentTransform {
    pub fn is_identity(self) -> bool {
        (self.scale - 1.0).abs() < 1e-6 && self.tx.abs() < 1e-6 && self.ty.abs() < 1e-6
    }
}

impl PrintLayoutOptions {
    pub fn with_scale_percent(percent: f32) -> Self {
        Self {
            scale_mode: PrintScaleMode::CustomPercent,
            scale_percent: percent,
            ..Self::default()
        }
    }

    pub fn with_uniform_margins(points: f32) -> Self {
        let m = points.max(0.0);
        Self {
            margin_left: m,
            margin_right: m,
            margin_top: m,
            margin_bottom: m,
            ..Self::default()
        }
    }

    pub fn fit_to_margins(margins: f32) -> Self {
        let m = margins.max(0.0);
        Self {
            scale_mode: PrintScaleMode::FitToMargins,
            scale_percent: 100.0,
            margin_left: m,
            margin_right: m,
            margin_top: m,
            margin_bottom: m,
            ..Self::default()
        }
    }

    pub fn with_nup(pages_per_sheet: u8) -> Self {
        Self {
            pages_per_sheet: normalize_pages_per_sheet(pages_per_sheet),
            ..Self::default()
        }
    }

    pub fn with_booklet() -> Self {
        Self {
            booklet: true,
            duplex: PrintDuplexMode::LongEdge,
            pages_per_sheet: 2,
            ..Self::default()
        }
    }

    pub fn with_duplex(duplex: PrintDuplexMode) -> Self {
        Self {
            duplex,
            ..Self::default()
        }
    }

    /// Duplex mode after booklet override.
    pub fn effective_duplex(&self) -> PrintDuplexMode {
        if self.booklet {
            PrintDuplexMode::LongEdge
        } else {
            self.duplex
        }
    }

    /// Resolve scale/translation for a page of size `(page_w, page_h)`.
    pub fn resolve(&self, page_w: f32, page_h: f32) -> PrintContentTransform {
        let page_w = page_w.max(1.0);
        let page_h = page_h.max(1.0);
        let ml = self.margin_left.max(0.0).min(page_w * 0.45);
        let mr = self.margin_right.max(0.0).min(page_w * 0.45);
        let mt = self.margin_top.max(0.0).min(page_h * 0.45);
        let mb = self.margin_bottom.max(0.0).min(page_h * 0.45);
        let avail_w = (page_w - ml - mr).max(1.0);
        let avail_h = (page_h - mt - mb).max(1.0);

        let scale = match self.scale_mode {
            PrintScaleMode::FitToMargins => (avail_w / page_w).min(avail_h / page_h),
            PrintScaleMode::ActualSize | PrintScaleMode::CustomPercent => {
                (self.scale_percent / 100.0).clamp(0.1, 4.0)
            }
        };

        let content_w = page_w * scale;
        let content_h = page_h * scale;
        let tx = ml + (avail_w - content_w).max(0.0) * 0.5;
        let ty = mb + (avail_h - content_h).max(0.0) * 0.5;
        PrintContentTransform { scale, tx, ty }
    }
}

#[derive(Debug, Clone)]
pub struct PdfExportOptions {
    pub fidelity: PdfFidelity,
    /// When true, embed TrueType faces used by the layout (`/FontFile2`).
    pub embed_fonts: bool,
    /// Print scale / margins (F25.S2). Ignored for ordinary Save-as-PDF.
    pub print_layout: PrintLayoutOptions,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            fidelity: PdfFidelity::Structural,
            embed_fonts: false,
            print_layout: PrintLayoutOptions::default(),
        }
    }
}

impl PdfExportOptions {
    /// Options for OS print (F25.S1/S2): VisualMatch / font-embedded PDF.
    pub fn for_print() -> Self {
        Self {
            fidelity: PdfFidelity::VisualMatch,
            embed_fonts: true,
            print_layout: PrintLayoutOptions::default(),
        }
    }

    pub fn for_print_with_layout(layout: PrintLayoutOptions) -> Self {
        Self {
            fidelity: PdfFidelity::VisualMatch,
            embed_fonts: true,
            print_layout: layout,
        }
    }
}

/// Build [`PrintLayoutOptions`] from FFI / WASM numeric discriminants (F25.S2/S4).
pub fn print_layout_from_codes(
    scale_mode: i32,
    scale_percent: f32,
    margin_left: f32,
    margin_right: f32,
    margin_top: f32,
    margin_bottom: f32,
    duplex: i32,
    pages_per_sheet: i32,
    booklet: i32,
) -> PrintLayoutOptions {
    PrintLayoutOptions {
        scale_mode: match scale_mode {
            1 => PrintScaleMode::FitToMargins,
            2 => PrintScaleMode::CustomPercent,
            _ => PrintScaleMode::ActualSize,
        },
        scale_percent,
        margin_left,
        margin_right,
        margin_top,
        margin_bottom,
        duplex: match duplex {
            1 => PrintDuplexMode::LongEdge,
            2 => PrintDuplexMode::ShortEdge,
            _ => PrintDuplexMode::Simplex,
        },
        pages_per_sheet: normalize_pages_per_sheet(pages_per_sheet.max(0) as u8),
        booklet: booklet != 0,
    }
}

/// Build a print-ready PDF: VisualMatch first, structural fallback (F25.S1/S2).
pub fn prepare_print_pdf(
    doc: &Document,
    layout: &PrintLayoutOptions,
) -> Result<Vec<u8>, PdfError> {
    let exporter = DisplayListPdfExporter;
    let mut opts = PdfExportOptions::for_print_with_layout(*layout);
    match exporter.export(doc, &opts) {
        Ok(bytes) => Ok(bytes),
        Err(_) => {
            opts.fidelity = PdfFidelity::Structural;
            opts.embed_fonts = false;
            exporter.export(doc, &opts)
        }
    }
}

/// Print only the selected range (F25.S3): clone subset → [`prepare_print_pdf`].
pub fn prepare_print_pdf_selection(
    doc: &Document,
    range: &tw_edit::DocRange,
    layout: &PrintLayoutOptions,
) -> Result<Vec<u8>, PdfError> {
    let subset = tw_edit::document_from_range(doc, range)
        .map_err(|e| PdfError::ExportFailed(e.to_string()))?;
    prepare_print_pdf(&subset, layout)
}

pub trait PdfExporter: Send + Sync {
    fn export(&self, doc: &Document, options: &PdfExportOptions) -> Result<Vec<u8>, PdfError>;
}

pub struct DisplayListPdfExporter;

impl PdfExporter for DisplayListPdfExporter {
    fn export(&self, doc: &Document, options: &PdfExportOptions) -> Result<Vec<u8>, PdfError> {
        let embed =
            options.embed_fonts || options.fidelity == PdfFidelity::VisualMatch;

        let mut engine = LayoutEngine::new();
        let layout = engine.layout_document(doc);
        let fonts = engine.shaper().fonts();

        let mut used_font_keys = BTreeSet::new();
        if embed {
            collect_font_keys(&layout.pages, &mut used_font_keys);
        }

        let mut pdf = PdfBuilder::new();
        let catalog_id = pdf.alloc();
        let pages_id = pdf.alloc();

        let mut embedded: BTreeMap<u32, EmbeddedFontRefs> = BTreeMap::new();
        if embed {
            for key in &used_font_keys {
                let data = fonts.load_embeddable_sfnt_by_key(*key).ok_or_else(|| {
                    PdfError::ExportFailed(format!(
                        "PDF VisualMatch / embed_fonts cannot load embeddable face bytes for font key {key}"
                    ))
                })?;
                let family = fonts
                    .family_name_for_key(*key)
                    .unwrap_or_else(|| format!("Face{key}"));
                embedded.insert(*key, write_embedded_font(&mut pdf, *key, &family, &data)?);
            }
        }

        let standard14 = if embed {
            None
        } else {
            Some(write_standard14_fonts(&mut pdf))
        };

        let sheet_plan = build_print_sheet_plan(layout.pages.len(), &options.print_layout);
        let impose = sheet_plan.pages_per_sheet > 1 || sheet_plan.booklet;

        let mut page_ids = Vec::new();
        if !impose {
            page_ids.reserve(layout.pages.len());
            for page in &layout.pages {
                let list = DisplayListBuilder::from_page(page, engine.atlas(), 1);
                let transform = options.print_layout.resolve(page.width, page.height);
                let page_id = write_page(
                    &mut pdf,
                    doc,
                    pages_id,
                    page,
                    &list,
                    embed,
                    &embedded,
                    standard14.as_ref(),
                    transform,
                )?;
                page_ids.push(page_id);
            }
        } else {
            let lists: Vec<_> = layout
                .pages
                .iter()
                .map(|page| DisplayListBuilder::from_page(page, engine.atlas(), 1))
                .collect();
            let sheet_w = layout.pages.first().map(|p| p.width).unwrap_or(612.0);
            let sheet_h = layout.pages.first().map(|p| p.height).unwrap_or(792.0);
            page_ids.reserve(sheet_plan.sheets.len());
            for sheet in &sheet_plan.sheets {
                let page_id = write_imposed_sheet(
                    &mut pdf,
                    doc,
                    pages_id,
                    &layout.pages,
                    &lists,
                    sheet,
                    sheet_plan.cols,
                    sheet_plan.rows,
                    sheet_w,
                    sheet_h,
                    &options.print_layout,
                    embed,
                    &embedded,
                    standard14.as_ref(),
                )?;
                page_ids.push(page_id);
            }
        }

        let kids = page_ids
            .iter()
            .map(|id| format!("{id} 0 R"))
            .collect::<Vec<_>>()
            .join(" ");
        pdf.put(
            pages_id,
            format!(
                "{pages_id} 0 obj<< /Type /Pages /Kids [{kids}] /Count {} >>endobj\n",
                page_ids.len()
            )
            .into_bytes(),
        );
        pdf.put(
            catalog_id,
            format!("{catalog_id} 0 obj<< /Type /Catalog /Pages {pages_id} 0 R >>endobj\n")
                .into_bytes(),
        );

        Ok(pdf.finish())
    }
}

/// Legacy name kept for compatibility.
pub struct PrintPdfExporter;

impl PdfExporter for PrintPdfExporter {
    fn export(&self, doc: &Document, options: &PdfExportOptions) -> Result<Vec<u8>, PdfError> {
        DisplayListPdfExporter.export(doc, options)
    }
}

struct EmbeddedFontRefs {
    type0_id: u32,
    resource_name: String,
}

struct PdfBuilder {
    next_id: u32,
    objects: HashMap<u32, Vec<u8>>,
}

impl PdfBuilder {
    fn new() -> Self {
        Self {
            next_id: 1,
            objects: HashMap::new(),
        }
    }

    fn alloc(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn put(&mut self, id: u32, bytes: Vec<u8>) {
        self.objects.insert(id, bytes);
    }

    fn finish(self) -> Vec<u8> {
        let mut ids: Vec<u32> = self.objects.keys().copied().collect();
        ids.sort_unstable();
        let mut pdf = Vec::from(&b"%PDF-1.4\n"[..]);
        let mut offsets = vec![0usize; self.next_id as usize];
        for id in ids {
            offsets[id as usize] = pdf.len();
            let obj = self.objects.get(&id).expect("object present");
            pdf.extend_from_slice(obj);
            if !pdf.ends_with(b"\n") {
                pdf.push(b'\n');
            }
        }
        let xref_start = pdf.len();
        let size = self.next_id;
        pdf.extend_from_slice(format!("xref\n0 {size}\n").as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for id in 1..size {
            let pos = offsets[id as usize];
            pdf.extend_from_slice(format!("{pos:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!("trailer<< /Size {size} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF\n")
                .as_bytes(),
        );
        pdf
    }
}

struct Standard14Fonts {
    regular: u32,
    bold: u32,
    italic: u32,
    bold_italic: u32,
}

fn write_standard14_fonts(pdf: &mut PdfBuilder) -> Standard14Fonts {
    Standard14Fonts {
        regular: write_type1_font(pdf, "Helvetica"),
        bold: write_type1_font(pdf, "Helvetica-Bold"),
        italic: write_type1_font(pdf, "Helvetica-Oblique"),
        bold_italic: write_type1_font(pdf, "Helvetica-BoldOblique"),
    }
}

fn write_type1_font(pdf: &mut PdfBuilder, base: &str) -> u32 {
    let id = pdf.alloc();
    pdf.put(
        id,
        format!("{id} 0 obj<< /Type /Font /Subtype /Type1 /BaseFont /{base} >>endobj\n")
            .into_bytes(),
    );
    id
}

fn write_helvetica(pdf: &mut PdfBuilder) -> u32 {
    write_type1_font(pdf, "Helvetica")
}

fn pdf_name_escape(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '+') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "Embedded".into()
    } else {
        out
    }
}

fn write_embedded_font(
    pdf: &mut PdfBuilder,
    key: u32,
    family: &str,
    face_bytes: &[u8],
) -> Result<EmbeddedFontRefs, PdfError> {
    let base = format!("TW{}+{}", key, pdf_name_escape(family));
    let file_id = pdf.alloc();
    let desc_id = pdf.alloc();
    let cid_id = pdf.alloc();
    let type0_id = pdf.alloc();

    let is_cff = face_bytes.starts_with(b"OTTO");
    let is_ttf = matches!(
        face_bytes.get(0..4),
        Some(&[0x00, 0x01, 0x00, 0x00] | b"true" | b"typ1")
    );
    if !is_cff && !is_ttf {
        return Err(PdfError::ExportFailed(format!(
            "PDF VisualMatch / embed_fonts: unsupported SFNT magic for font key {key}"
        )));
    }

    let mut file_obj = if is_ttf {
        format!(
            "{file_id} 0 obj<< /Length {} /Length1 {} >>stream\n",
            face_bytes.len(),
            face_bytes.len()
        )
        .into_bytes()
    } else {
        format!(
            "{file_id} 0 obj<< /Length {} /Subtype /OpenType >>stream\n",
            face_bytes.len()
        )
        .into_bytes()
    };
    file_obj.extend_from_slice(face_bytes);
    file_obj.extend_from_slice(b"\nendstream\nendobj\n");
    pdf.put(file_id, file_obj);

    let font_file_key = if is_ttf { "FontFile2" } else { "FontFile3" };
    let cid_subtype = if is_ttf { "CIDFontType2" } else { "CIDFontType0" };

    pdf.put(
        desc_id,
        format!(
            "{desc_id} 0 obj<< /Type /FontDescriptor /FontName /{base} \
             /Flags 32 /FontBBox [-1024 -1024 3072 3072] /ItalicAngle 0 \
             /Ascent 800 /Descent -200 /CapHeight 700 /StemV 80 \
             /{font_file_key} {file_id} 0 R >>endobj\n"
        )
        .into_bytes(),
    );

    let cid_extra = if is_ttf {
        " /CIDToGIDMap /Identity".to_string()
    } else {
        String::new()
    };

    pdf.put(
        cid_id,
        format!(
            "{cid_id} 0 obj<< /Type /Font /Subtype /{cid_subtype} /BaseFont /{base} \
             /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> \
             /FontDescriptor {desc_id} 0 R /DW 1000{cid_extra} >>endobj\n"
        )
        .into_bytes(),
    );

    pdf.put(
        type0_id,
        format!(
            "{type0_id} 0 obj<< /Type /Font /Subtype /Type0 /BaseFont /{base} \
             /Encoding /Identity-H /DescendantFonts [{cid_id} 0 R] >>endobj\n"
        )
        .into_bytes(),
    );

    Ok(EmbeddedFontRefs {
        type0_id,
        resource_name: format!("F{key}"),
    })
}

fn write_image_xobject(pdf: &mut PdfBuilder, encoded: &[u8]) -> Result<Option<u32>, PdfError> {
    if encoded.is_empty() {
        return Ok(None);
    }

    if encoded.starts_with(&[0xFF, 0xD8, 0xFF]) {
        let img = image::load_from_memory(encoded).map_err(|e| {
            PdfError::ExportFailed(format!("jpeg decode for PDF failed: {e}"))
        })?;
        let (w, h) = img.dimensions();
        let id = pdf.alloc();
        let mut obj = format!(
            "{id} 0 obj<< /Type /XObject /Subtype /Image /Width {w} /Height {h} \
             /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode \
             /Length {} >>stream\n",
            encoded.len()
        )
        .into_bytes();
        obj.extend_from_slice(encoded);
        obj.extend_from_slice(b"\nendstream\nendobj\n");
        pdf.put(id, obj);
        return Ok(Some(id));
    }

    let img = image::load_from_memory(encoded)
        .map_err(|e| PdfError::ExportFailed(format!("image decode for PDF failed: {e}")))?;
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let raw = rgb.into_raw();
    let id = pdf.alloc();
    let mut obj = format!(
        "{id} 0 obj<< /Type /XObject /Subtype /Image /Width {w} /Height {h} \
         /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>stream\n",
        raw.len()
    )
    .into_bytes();
    obj.extend_from_slice(&raw);
    obj.extend_from_slice(b"\nendstream\nendobj\n");
    pdf.put(id, obj);
    Ok(Some(id))
}

fn append_display_list_graphics(
    content: &mut String,
    pdf: &mut PdfBuilder,
    page: &PageLayout,
    list: &tw_render::DisplayList,
    xobject_names: &mut Vec<(String, u32)>,
) -> Result<(), PdfError> {
    for chunk in list.rect_batch.rects.chunks(4) {
        if chunk.len() == 4 {
            let [x, y, w, h] = [chunk[0], chunk[1], chunk[2], chunk[3]];
            content.push_str(&format!(
                "q 0.9 0.9 0.9 rg {} {} {} {} re f Q\n",
                x,
                page.height - y - h,
                w,
                h
            ));
        }
    }

    for chunk in list.path_batch.points.chunks(4) {
        if chunk.len() == 4 {
            content.push_str(&format!(
                "q 0 0 0 RG 0.5 w {} {} m {} {} l S Q\n",
                chunk[0],
                page.height - chunk[1],
                chunk[2],
                page.height - chunk[3]
            ));
        }
    }

    let n = list.image_batch.payloads.len();
    for i in 0..n {
        let Some(img_id) = write_image_xobject(pdf, &list.image_batch.payloads[i])? else {
            continue;
        };
        let name = format!("Im{}", xobject_names.len());
        let x = list.image_batch.transforms.get(i * 2).copied().unwrap_or(0.0);
        let y = list
            .image_batch
            .transforms
            .get(i * 2 + 1)
            .copied()
            .unwrap_or(0.0);
        let w = list.image_batch.sizes.get(i * 2).copied().unwrap_or(1.0);
        let h = list.image_batch.sizes.get(i * 2 + 1).copied().unwrap_or(1.0);
        let pdf_y = page.height - y - h;
        content.push_str(&format!(
            "q {w:.2} 0 0 {h:.2} {x:.2} {pdf_y:.2} cm /{name} Do Q\n"
        ));
        xobject_names.push((name, img_id));
    }
    Ok(())
}

fn write_imposed_sheet(
    pdf: &mut PdfBuilder,
    doc: &Document,
    pages_id: u32,
    pages: &[PageLayout],
    lists: &[tw_render::DisplayList],
    sheet: &PrintSheet,
    cols: u8,
    rows: u8,
    sheet_w: f32,
    sheet_h: f32,
    layout: &PrintLayoutOptions,
    embed: bool,
    embedded: &BTreeMap<u32, EmbeddedFontRefs>,
    standard14: Option<&Standard14Fonts>,
) -> Result<u32, PdfError> {
    let mut content = String::new();
    let mut xobject_names: Vec<(String, u32)> = Vec::new();

    for (slot, source) in sheet.slots.iter().enumerate() {
        let Some(page_index) = *source else {
            continue;
        };
        let Some(page) = pages.get(page_index) else {
            continue;
        };
        let Some(list) = lists.get(page_index) else {
            continue;
        };
        let cell = nup_cell_transform(sheet_w, sheet_h, cols, rows, slot, page.width, page.height);
        let inner = layout.resolve(page.width, page.height);
        content.push_str(&format!(
            "q {:.6} 0 0 {:.6} {:.6} {:.6} cm\n",
            cell.scale, cell.scale, cell.tx, cell.ty
        ));
        if !inner.is_identity() {
            content.push_str(&format!(
                "q {:.6} 0 0 {:.6} {:.6} {:.6} cm\n",
                inner.scale, inner.scale, inner.tx, inner.ty
            ));
        }
        append_page_text(&mut content, doc, page, embed, embedded, standard14);
        append_display_list_graphics(&mut content, pdf, page, list, &mut xobject_names)?;
        if !inner.is_identity() {
            content.push_str("Q\n");
        }
        content.push_str("Q\n");
    }

    let content_id = pdf.alloc();
    let page_id = pdf.alloc();

    let mut font_res = String::new();
    if let Some(std14) = standard14 {
        font_res.push_str(&format!("/F1 {} 0 R ", std14.regular));
        font_res.push_str(&format!("/F2 {} 0 R ", std14.bold));
        font_res.push_str(&format!("/F3 {} 0 R ", std14.italic));
        font_res.push_str(&format!("/F4 {} 0 R ", std14.bold_italic));
    }
    for font in embedded.values() {
        font_res.push_str(&format!("/{} {} 0 R ", font.resource_name, font.type0_id));
    }

    let mut xobj_res = String::new();
    for (name, id) in &xobject_names {
        xobj_res.push_str(&format!("/{name} {id} 0 R "));
    }

    let resources = format!(
        "/Resources << /Font << {font_res}>> /XObject << {xobj_res}>> >>"
    );

    pdf.put(
        content_id,
        {
            let mut obj = format!(
                "{content_id} 0 obj<< /Length {} >>stream\n",
                content.len()
            )
            .into_bytes();
            obj.extend_from_slice(content.as_bytes());
            obj.extend_from_slice(b"\nendstream\nendobj\n");
            obj
        },
    );

    pdf.put(
        page_id,
        format!(
            "{page_id} 0 obj<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 {:.2} {:.2}] \
             /Contents {content_id} 0 R {resources} >>endobj\n",
            sheet_w, sheet_h
        )
        .into_bytes(),
    );

    Ok(page_id)
}

fn write_page(
    pdf: &mut PdfBuilder,
    doc: &Document,
    pages_id: u32,
    page: &PageLayout,
    list: &tw_render::DisplayList,
    embed: bool,
    embedded: &BTreeMap<u32, EmbeddedFontRefs>,
    standard14: Option<&Standard14Fonts>,
    transform: PrintContentTransform,
) -> Result<u32, PdfError> {
    let mut content = String::new();
    if !transform.is_identity() {
        content.push_str(&format!(
            "q {:.6} 0 0 {:.6} {:.6} {:.6} cm\n",
            transform.scale, transform.scale, transform.tx, transform.ty
        ));
    }
    append_page_text(&mut content, doc, page, embed, embedded, standard14);

    let mut xobject_names: Vec<(String, u32)> = Vec::new();
    append_display_list_graphics(&mut content, pdf, page, list, &mut xobject_names)?;

    if !transform.is_identity() {
        content.push_str("Q\n");
    }

    let content_id = pdf.alloc();
    let page_id = pdf.alloc();

    let mut font_res = String::new();
    if let Some(std14) = standard14 {
        font_res.push_str(&format!("/F1 {} 0 R ", std14.regular));
        font_res.push_str(&format!("/F2 {} 0 R ", std14.bold));
        font_res.push_str(&format!("/F3 {} 0 R ", std14.italic));
        font_res.push_str(&format!("/F4 {} 0 R ", std14.bold_italic));
    }
    for font in embedded.values() {
        font_res.push_str(&format!("/{} {} 0 R ", font.resource_name, font.type0_id));
    }

    let mut xobj_res = String::new();
    for (name, id) in &xobject_names {
        xobj_res.push_str(&format!("/{name} {id} 0 R "));
    }

    let resources = format!(
        "/Resources << /Font << {font_res}>> /XObject << {xobj_res}>> >>"
    );

    pdf.put(
        content_id,
        {
            let mut obj = format!(
                "{content_id} 0 obj<< /Length {} >>stream\n",
                content.len()
            )
            .into_bytes();
            obj.extend_from_slice(content.as_bytes());
            obj.extend_from_slice(b"\nendstream\nendobj\n");
            obj
        },
    );

    pdf.put(
        page_id,
        format!(
            "{page_id} 0 obj<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 {:.2} {:.2}] \
             /Contents {content_id} 0 R {resources} >>endobj\n",
            page.width, page.height
        )
        .into_bytes(),
    );

    Ok(page_id)
}

fn collect_font_keys(pages: &[PageLayout], out: &mut BTreeSet<u32>) {
    for page in pages {
        for layout_box in &page.boxes {
            collect_font_keys_in_box(layout_box, out);
        }
    }
}

fn collect_font_keys_in_box(layout_box: &LayoutBox, out: &mut BTreeSet<u32>) {
    match layout_box {
        LayoutBox::TextLine(line) => {
            for g in &line.glyphs {
                out.insert(g.font_id);
            }
        }
        LayoutBox::Table(table) => {
            for cell in &table.cells {
                for line in &cell.lines {
                    for g in &line.glyphs {
                        out.insert(g.font_id);
                    }
                }
                for nested in &cell.nested_tables {
                    for cell in &nested.cells {
                        for line in &cell.lines {
                            for g in &line.glyphs {
                                out.insert(g.font_id);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn escape_pdf_text(text: &str) -> String {
    text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)")
}

fn font_size_for_glyph(doc: &Document, line: &TextLine, glyph_x: f32) -> f32 {
    for (x0, x1, run_id, _) in &line.run_map {
        if glyph_x >= *x0 && glyph_x < *x1 {
            if let Some(run) = doc.run_by_id(*run_id) {
                return run.format.font_size.unwrap_or(12.0);
            }
        }
    }
    if let Some((_, _, run_id, _)) = line.run_map.first() {
        if let Some(run) = doc.run_by_id(*run_id) {
            return run.format.font_size.unwrap_or(12.0);
        }
    }
    12.0
}

fn append_page_text(
    content: &mut String,
    doc: &Document,
    page: &PageLayout,
    embed: bool,
    embedded: &BTreeMap<u32, EmbeddedFontRefs>,
    standard14: Option<&Standard14Fonts>,
) {
    for layout_box in &page.boxes {
        match layout_box {
            LayoutBox::TextLine(line) => {
                append_line_text(content, doc, page.height, line, embed, embedded, standard14)
            }
            LayoutBox::Table(table) => {
                for cell in &table.cells {
                    for line in &cell.lines {
                        append_line_text(content, doc, page.height, line, embed, embedded, standard14);
                    }
                }
            }
            _ => {}
        }
    }
}

fn append_line_text(
    content: &mut String,
    doc: &Document,
    page_height: f32,
    line: &TextLine,
    embed: bool,
    embedded: &BTreeMap<u32, EmbeddedFontRefs>,
    standard14: Option<&Standard14Fonts>,
) {
    if line.glyphs.is_empty() {
        return;
    }

    if embed {
        for glyph in &line.glyphs {
            if glyph.codepoint.is_control() {
                continue;
            }
            let Some(font) = embedded.get(&glyph.font_id) else {
                continue;
            };
            let size = font_size_for_glyph(doc, line, glyph.x);
            let pdf_y = page_height - line.y;
            content.push_str(&format!(
                "BT /{} {:.2} Tf 1 0 0 1 {:.2} {:.2} Tm <{:04X}> Tj ET\n",
                font.resource_name,
                size,
                glyph.x,
                pdf_y,
                glyph.glyph_id.min(0xFFFF) as u16
            ));
        }
        return;
    }

    let mut text = String::new();
    let mut size = 12.0_f32;
    for glyph in &line.glyphs {
        if glyph.codepoint.is_control() {
            continue;
        }
        if text.is_empty() {
            size = font_size_for_glyph(doc, line, glyph.x);
        }
        text.push(glyph.codepoint);
    }
    if text.is_empty() {
        return;
    }
    let (bold, italic) = line
        .run_map
        .first()
        .and_then(|(_, _, run_id, _)| doc.run_by_id(*run_id))
        .map(|run| {
            (
                run.format.bold == Some(true),
                run.format.italic == Some(true),
            )
        })
        .unwrap_or((false, false));
    let font_name = match (bold, italic) {
        (true, true) => "F4",
        (true, false) => "F2",
        (false, true) => "F3",
        (false, false) => "F1",
    };
    if standard14.is_none() && font_name != "F1" {
        // Fallback when only legacy single-font path is used.
    }
    let pdf_y = page_height - line.y;
    content.push_str(&format!(
        "BT /{font_name} {:.2} Tf 1 0 0 1 {:.2} {:.2} Tm ({}) Tj ET\n",
        size,
        line.x,
        pdf_y,
        escape_pdf_text(&text)
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tw_edit::{apply, Command, EditSession};
    use tw_model::{Block, Paragraph};

    #[test]
    fn export_document_with_table_produces_pdf() {
        let mut session = EditSession::new();
        let block_id = session.document.sections[0].blocks[0]
            .paragraph()
            .unwrap()
            .id;
        apply(
            &mut session.document,
            Command::InsertTable {
                after_block_id: block_id,
                rows: 3,
                cols: 3,
            },
        )
        .unwrap();

        let exporter = DisplayListPdfExporter;
        let pdf = exporter
            .export(&session.document, &PdfExportOptions::default())
            .unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 100);
    }

    #[test]
    fn u_f23_s2_embed_fonts_writes_fontfile2() {
        let mut doc = Document::new();
        doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text(
            "Embedded face sample",
        ))];
        let pdf = DisplayListPdfExporter
            .export(
                &doc,
                &PdfExportOptions {
                    fidelity: PdfFidelity::Structural,
                    embed_fonts: true,
                },
            )
            .expect("embed_fonts should succeed when system faces resolve");
        let s = String::from_utf8_lossy(&pdf);
        assert!(
            s.contains("/FontFile2"),
            "expected /FontFile2 in embedded export"
        );
        assert!(s.contains("/Identity-H"));
    }

    #[test]
    fn u_f23_s2_visual_match_exports_when_ready() {
        let mut doc = Document::new();
        doc.sections[0].blocks = vec![Block::Paragraph(Paragraph::with_text("VisualMatch"))];
        let pdf = DisplayListPdfExporter
            .export(
                &doc,
                &PdfExportOptions {
                    fidelity: PdfFidelity::VisualMatch,
                    embed_fonts: false,
                },
            )
            .expect("VisualMatch ready once faces embed");
        let s = String::from_utf8_lossy(&pdf);
        assert!(s.contains("/FontFile2"));
    }
}

use thiserror::Error;
use tw_docx::DocxPackage;
use tw_model::Document;
use tw_odt::OdtPackage;

use crate::import::{detect_format, DetectedFormat, ImportError};

#[derive(Debug, Clone)]
pub struct ImportBundle {
    pub document: Document,
    pub source_format: DetectedFormat,
    pub docx_package: Option<DocxPackage>,
    pub odt_package: Option<OdtPackage>,
}

#[derive(Debug, Clone, Default)]
pub struct FormatContext {
    pub source_format: DetectedFormat,
    pub docx_package: Option<DocxPackage>,
    pub odt_package: Option<OdtPackage>,
    pub save_format: DetectedFormat,
    pub path_hint: Option<String>,
}

impl FormatContext {
    pub fn from_bundle(bundle: ImportBundle, path_hint: Option<String>) -> Self {
        let save_format = path_hint
            .as_deref()
            .and_then(crate::import::extension_from_path)
            .and_then(|ext| crate::import::format_from_extension(&ext))
            .unwrap_or(bundle.source_format);
        Self {
            source_format: bundle.source_format,
            docx_package: bundle.docx_package,
            odt_package: bundle.odt_package,
            save_format,
            path_hint,
        }
    }

    pub fn new_document() -> Self {
        Self {
            source_format: DetectedFormat::Twdoc,
            save_format: DetectedFormat::Twdoc,
            ..Default::default()
        }
    }

    pub fn mark_document_modified(&mut self) {
        if self.docx_package.is_some() {
            if let Some(pkg) = &mut self.docx_package {
                pkg.mark_modified("word/document.xml".into());
            }
        }
        if self.odt_package.is_some() {
            if let Some(pkg) = &mut self.odt_package {
                pkg.mark_modified("content.xml".into());
            }
        }
    }

    /// Tier B: force `word/numbering.xml` to be re-serialized on the next save.
    pub fn mark_numbering_modified(&mut self) {
        if let Some(pkg) = &mut self.docx_package {
            pkg.mark_modified("word/numbering.xml".into());
            pkg.source_numbering_fingerprint = None;
        }
    }

    /// Tier B: force `word/styles.xml` to be re-serialized when a serializer exists.
    pub fn mark_styles_modified(&mut self) {
        if let Some(pkg) = &mut self.docx_package {
            pkg.mark_modified("word/styles.xml".into());
            pkg.source_styles_fingerprint = None;
        }
    }
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("export not supported for format")]
    UnsupportedFormat,
    #[error("docx package missing for docx export")]
    MissingDocxPackage,
    #[error("odt package missing for odt export")]
    MissingOdtPackage,
    #[error("native format error: {0}")]
    Native(#[from] tw_native::NativeError),
    #[error("docx error: {0}")]
    Docx(#[from] tw_docx::DocxError),
    #[error("odt error: {0}")]
    Odt(#[from] tw_odt::OdtError),
    #[error("markdown error: {0}")]
    Markdown(#[from] tw_markdown::MarkdownError),
    #[error("html error: {0}")]
    Html(#[from] tw_html::HtmlError),
}

pub fn export_document(doc: &Document, ctx: &FormatContext) -> Result<Vec<u8>, ExportError> {
    match ctx.save_format {
        DetectedFormat::Twdoc | DetectedFormat::PlainText => Ok(tw_native::NativeFormat::export(doc)?),
        DetectedFormat::Docx => {
            let package = ctx
                .docx_package
                .as_ref()
                .cloned()
                .unwrap_or_else(tw_docx::DocxPackage::minimal);
            Ok(tw_docx::export(doc, &package)?)
        }
        DetectedFormat::Odt => {
            let package = ctx
                .odt_package
                .as_ref()
                .cloned()
                .unwrap_or_else(tw_odt::OdtPackage::minimal);
            Ok(tw_odt::export(doc, &package)?)
        }
        DetectedFormat::Markdown => Ok(tw_markdown::export(doc)?),
        DetectedFormat::Html => Ok(tw_html::export(doc)?),
        DetectedFormat::Rtf | DetectedFormat::LegacyDoc | DetectedFormat::Unknown => {
            Err(ExportError::UnsupportedFormat)
        }
    }
}

pub fn import_document_bundle(
    data: &[u8],
    path_hint: Option<&str>,
) -> Result<ImportBundle, ImportError> {
    let format = detect_format(data, path_hint);
    match format {
        DetectedFormat::Twdoc => Ok(ImportBundle {
            document: tw_native::NativeFormat::import(data)?,
            source_format: format,
            docx_package: None,
            odt_package: None,
        }),
        DetectedFormat::Docx => {
            let result = tw_docx::import(data).map_err(|e| match e {
                tw_docx::DocxError::PasswordProtected => ImportError::PasswordProtected,
                other => ImportError::Docx(other),
            })?;
            let mut document = result.document;
            tw_render::normalize_document_images(&mut document);
            Ok(ImportBundle {
                document,
                source_format: format,
                docx_package: Some(result.package),
                odt_package: None,
            })
        }
        DetectedFormat::Odt => {
            let result = tw_odt::import(data)?;
            Ok(ImportBundle {
                document: result.document,
                source_format: format,
                docx_package: None,
                odt_package: Some(result.package),
            })
        }
        DetectedFormat::Rtf => Ok(ImportBundle {
            document: tw_rtf::import(data)?,
            source_format: format,
            docx_package: None,
            odt_package: None,
        }),
        DetectedFormat::Html => Ok(ImportBundle {
            document: tw_html::import(data)?,
            source_format: format,
            docx_package: None,
            odt_package: None,
        }),
        DetectedFormat::Markdown => Ok(ImportBundle {
            document: tw_markdown::import(data)?,
            source_format: format,
            docx_package: None,
            odt_package: None,
        }),
        DetectedFormat::PlainText => Ok(ImportBundle {
            document: tw_native::NativeFormat::import_plain_text(data)?,
            source_format: format,
            docx_package: None,
            odt_package: None,
        }),
        DetectedFormat::LegacyDoc => Err(ImportError::LegacyDocNotSupported),
        DetectedFormat::Unknown => Err(ImportError::UnknownFormat),
    }
}

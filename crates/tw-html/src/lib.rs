use thiserror::Error;
use tw_model::Document;

#[derive(Debug, Error)]
pub enum HtmlError {
    #[error("not a valid html document")]
    InvalidFormat,
    #[error("html export failed")]
    ExportFailed,
}

mod export;
mod import;

pub use export::export_html;
pub use import::sanitize_html;

pub fn import(source: &[u8]) -> Result<Document, HtmlError> {
    import::import_html(source)
}

pub fn export(doc: &Document) -> Result<Vec<u8>, HtmlError> {
    Ok(export_html(doc).into_bytes())
}

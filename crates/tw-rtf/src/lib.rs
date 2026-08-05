use thiserror::Error;
use tw_model::Document;

#[derive(Debug, Error)]
pub enum RtfError {
    #[error("not a valid rtf document")]
    InvalidFormat,
}

mod import;

pub fn import(source: &[u8]) -> Result<Document, RtfError> {
    import::import_rtf(source)
}

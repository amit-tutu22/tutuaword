//! OPC ZIP repack for DOCX passthrough export (ADR-0008).

use std::io::{Cursor, Write};

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::{DocxError, DocxPackage};

pub fn repack(package: &DocxPackage) -> Result<Vec<u8>, DocxError> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let mut names: Vec<_> = package.parts.keys().cloned().collect();
        names.sort();

        for name in names {
            let data = package.parts.get(&name).expect("part exists");
            zip.start_file(&name, options)?;
            zip.write_all(data)?;
        }
        zip.finish()?;
    }
    Ok(buf)
}

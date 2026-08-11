//! VBA / macro OPC parts — preserve on passthrough, never execute (ADR-0008 Tier C).

use crate::DocxPackage;

/// Part paths and name fragments that indicate embedded VBA or macro payloads.
pub fn is_vba_part(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("vbaproject")
        || lower.contains("vbadata")
        || (lower.ends_with(".bin")
            && (lower.contains("word/vba") || lower.contains("vba")))
        || lower == "word/activex"
        || lower.starts_with("word/activex/")
}

/// Returns true when the package still carries VBA-related OPC parts.
pub fn package_has_vba_parts(package: &DocxPackage) -> bool {
    package.parts.keys().any(|name| is_vba_part(name))
}

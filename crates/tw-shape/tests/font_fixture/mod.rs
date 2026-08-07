//! Real font bytes for tests that exercise the injected-font path.
//!
//! The repository vendors no font binary, so the bytes are read from the host at
//! test time. Nothing here goes through `fontdb`: the point of these tests is
//! that the engine under test never consults a system font database, so the
//! harness must not hand it one by the back door.
//!
//! Returns `None` on a machine with no fonts, so the suite skips rather than
//! fails.

use std::path::{Path, PathBuf};

/// Directories scanned for a usable face, in preference order.
const FONT_DIRS: &[&str] = &[
    // macOS
    "/System/Library/Fonts/Supplemental",
    "/System/Library/Fonts",
    "/Library/Fonts",
    // Linux
    "/usr/share/fonts/truetype/dejavu",
    "/usr/share/fonts/truetype/liberation",
    "/usr/share/fonts/truetype",
    "/usr/share/fonts",
    "/usr/local/share/fonts",
    // Windows
    "C:\\Windows\\Fonts",
];

/// Bytes of some single-face TrueType/OpenType file installed on this machine,
/// or `None` if there are none to be found.
pub fn any_font_bytes() -> Option<Vec<u8>> {
    let path = find_font_file()?;
    std::fs::read(path).ok()
}

/// Prints why a test did nothing, so a skip is visible in CI output rather than
/// looking like a pass.
pub fn skipped(test: &str) {
    eprintln!("{test}: skipped — no TrueType font found on this host");
}

fn find_font_file() -> Option<PathBuf> {
    FONT_DIRS
        .iter()
        .map(Path::new)
        .filter(|dir| dir.is_dir())
        .find_map(|dir| first_font_in(dir, 0))
}

fn first_font_in(dir: &Path, depth: usize) -> Option<PathBuf> {
    const MAX_DEPTH: usize = 3;

    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();
    // Directory order is not stable across filesystems; a fixed order keeps the
    // chosen face the same from run to run.
    entries.sort();

    if let Some(file) = entries.iter().find(|path| is_usable_font(path)) {
        return Some(file.clone());
    }
    if depth >= MAX_DEPTH {
        return None;
    }
    entries
        .iter()
        .filter(|path| path.is_dir())
        .find_map(|sub| first_font_in(sub, depth + 1))
}

fn is_usable_font(path: &Path) -> bool {
    // Collections (`.ttc`) need a face index, and `.dfont` is a Mac resource
    // fork container neither ttf-parser nor swash reads.
    let is_font = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ttf") || ext.eq_ignore_ascii_case("otf"));
    is_font && path.is_file()
}

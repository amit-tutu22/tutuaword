// R3.1 exit gate: open a real .docx through the wasm engine from JavaScript.
//
// This harness exists because compiling for wasm32 proves almost nothing about
// the web path: thread::spawn, thread::sleep and the system font scan all
// compile for wasm32 and only fail when they run. Driving a document through
// the engine from Node is what actually exercises the inline executor and the
// byte-slice font provider.
//
// Run via scripts/wasm-smoke.sh, which builds the module and points
// TW_WASM_PKG at the wasm-bindgen output directory.

import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const pkgDir = process.env.TW_WASM_PKG;
if (!pkgDir) {
  console.error("TW_WASM_PKG is not set; run scripts/wasm-smoke.sh");
  process.exit(1);
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const wasm = await import(path.join(pkgDir, "tw_wasm.js"));

let failures = 0;
function check(label, condition, detail) {
  if (condition) {
    console.log(`  ok   ${label}`);
  } else {
    failures += 1;
    console.log(`  FAIL ${label}${detail ? ` — ${detail}` : ""}`);
  }
}

// A web host has no system font directory, so it must hand the engine font
// bytes itself. We borrow one from the CI host rather than vendoring a binary.
const FONT_CANDIDATES = [
  "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
  "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
  "/usr/share/fonts/dejavu/DejaVuSans.ttf",
  "/System/Library/Fonts/Supplemental/Arial.ttf",
  "/Library/Fonts/Arial.ttf",
];

function findFont() {
  for (const candidate of FONT_CANDIDATES) {
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

console.log("tw-wasm smoke test");

const engine = new wasm.TwEngine();
check("engine constructs without spawning a thread", engine !== undefined);

const fontPath = findFont();
if (fontPath) {
  engine.register_font("Arial", false, false, readFileSync(fontPath));
  console.log(`  info registered font ${path.basename(fontPath)} as "Arial"`);
} else {
  console.log("  info no host font found; exercising the unregistered-family fallback");
}

const docxPath = path.join(repoRoot, "crates/tw-docx/tests/corpus/bold_heading.docx");
const docx = readFileSync(docxPath);
check("fixture is a real docx", docx.length > 0 && docx[0] === 0x50 && docx[1] === 0x4b);

engine.open_document(docx);

const pageCount = engine.page_count();
check("opened document reports at least one page", pageCount >= 1, `page_count=${pageCount}`);

const text = engine.text();
check("laid-out document exposes its text", text.length > 0, `text=${JSON.stringify(text.slice(0, 80))}`);

if (fontPath) {
  // With a real face registered, layout must produce positioned glyphs rather
  // than silently measuring everything as zero-width.
  const advance = engine.first_line_advance();
  check("shaping produced non-zero advances", advance > 0, `advance=${advance}`);
}

console.log(failures === 0 ? "\nsmoke test passed" : `\nsmoke test failed (${failures})`);
process.exit(failures === 0 ? 0 : 1);

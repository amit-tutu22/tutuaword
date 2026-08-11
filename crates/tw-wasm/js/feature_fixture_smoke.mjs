// Open the Word-compatible feature fixture through the wasm engine (Chrome/web path).
// Run after scripts/wasm-smoke.sh build, or via scripts/wasm-feature-fixture-smoke.sh.

import { readFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const pkgDir = process.env.TW_WASM_PKG;
if (!pkgDir) {
  console.error("TW_WASM_PKG is not set");
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

const FONT_CANDIDATES = [
  "/System/Library/Fonts/Supplemental/Arial.ttf",
  "/Library/Fonts/Arial.ttf",
  "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

function findFont() {
  for (const candidate of FONT_CANDIDATES) {
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

console.log("tw-wasm word-compatible feature fixture");

const engine = new wasm.TwEngine();
const fontPath = findFont();
if (fontPath) {
  engine.register_font("Arial", false, false, readFileSync(fontPath));
}

const docxPath = path.join(
  repoRoot,
  "crates/tw-docx/tests/corpus/word_compatible_feature_test.docx",
);
const docx = readFileSync(docxPath);
check("fixture present", docx.length > 0);

engine.open_document_with_path(docx, "word_compatible_feature_test.docx");

const pageCount = engine.page_count();
check("page_count >= 2", pageCount >= 2, `page_count=${pageCount}`);

const text = engine.text();
for (const marker of [
  "Microsoft Word-Compatible Feature Test Document",
  "PROJECT-ALPHA-2026",
  "CUSTOMER_NAME",
  "The quick brown fox jumps over the lazy dog",
  "Left-aligned paragraph",
  "FINAL TEST CHECKLIST",
  "10. Images",
  "14. Equations",
  "Before ",
  "[math]",
  "Item",
  "Quarter",
]) {
  check(`text contains ${marker}`, text.includes(marker));
}

if (typeof engine.find_matches === "function") {
  const raw = engine.find_matches("PROJECT-ALPHA-2026", true, false, false, "");
  let matches = [];
  try {
    matches = JSON.parse(raw);
  } catch {
    matches = [];
  }
  check(
    "find case-sensitive match",
    Array.isArray(matches) && matches.length >= 1,
    `len=${matches?.length} raw=${String(raw).slice(0, 80)}`,
  );
}

console.log(failures === 0 ? "\nfeature fixture wasm smoke passed" : `\nfailed (${failures})`);
process.exit(failures === 0 ? 0 : 1);

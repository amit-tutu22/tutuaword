# Testing Strategy

Testing approach for a Word-class document editor. Covers unit tests, golden-image layout regression, DOCX round-trip fidelity, performance benchmarks, and end-to-end UI tests.

## Test Pyramid

```
                    ┌───────────┐
                    │  E2E UI   │  (few, slow, high confidence)
                   ┌┴───────────┴┐
                   │ Integration  │  (moderate count, cross-crate)
                  ┌┴─────────────┴┐
                  │  Golden Image   │  (layout/render regression)
                 ┌┴───────────────┴┐
                 │   Unit Tests    │  (many, fast, isolated)
                 └─────────────────┘
```

## Unit Tests

Every crate has unit tests in `src/` (inline `#[cfg(test)]`) and integration tests in `tests/`.

### Coverage Targets

| Crate | Target Coverage | Focus Areas |
|-------|----------------|-------------|
| `tw-model` | 90% | Node creation, style resolution, serialization round-trip |
| `tw-text` | 90% | Rope insert/delete, grapheme boundaries, run splitting |
| `tw-edit` | 95% | Every Command variant, undo/redo, run normalization |
| `tw-shape` | 80% | Font fallback, shaping output for known inputs |
| `tw-layout` | 85% | Line breaking, pagination, table layout |
| `tw-render` | 80% | Display list serialization/deserialization |
| `tw-docx` | 85% | XML parsing, part mapping, passthrough preservation |
| `tw-core` | 70% | Session lifecycle, command routing |

### Key Unit Test Categories

**Model invariants** — after every `apply()`:
- All NodeIds are unique
- Every paragraph has at least one run
- Adjacent runs with identical formatting are merged
- Section structure is valid (blocks belong to sections)

**Command round-trip** — for every Command:
- `apply(command)` then `apply(inverse)` restores original state
- Undo then redo returns to post-edit state

**Style resolution** — for known style hierarchies:
- `based_on` chain resolves correctly
- Direct formatting overrides style
- Document defaults apply when no style specified

**Serialization** — for every model type:
- `serialize(deserialize(x)) == x` (lossless round-trip)

## Golden-Image Layout Tests

**Honesty note (risk-mitigation):** Today CI uses **layout fingerprint hashes** (`crates/tw-layout/tests/golden_layout.rs`) for regression. That is **self-consistency**, not Word fidelity. Word (fixed-version) screenshot baselines are the S2 fidelity gate; PNG golden pipelines below are the target architecture.

### Target flow (Word baselines / PNG)

```
Test fixture (.twdoc or .docx)
      │
      ▼
  Layout Engine → PageLayout
      │
      ▼
  Display List Builder → DisplayList
      │
      ▼
  Software Renderer (tiny-skia or swash)
      │
      ▼
  PNG output
      │
      ▼
  Compare vs reference PNG (pixel diff)
      │
      ▼
  Pass if diff < 2% of pixels
```

### Test Fixtures

Organized by feature in `crates/tw-layout/tests/golden/`:

```
golden/
├── basic/
│   ├── single_paragraph.twdoc
│   ├── single_paragraph.png          ← reference
│   ├── bold_italic.twdoc
│   ├── bold_italic.png
│   ├── multi_font.twdoc
│   └── multi_font.png
├── paragraph/
│   ├── alignment_left_center_right.twdoc
│   ├── line_spacing.twdoc
│   ├── indentation.twdoc
│   └── borders_shading.twdoc
├── lists/
│   ├── bullet_list.twdoc
│   ├── numbered_list.twdoc
│   └── multi_level_list.twdoc
├── tables/
│   ├── simple_3x3.twdoc
│   ├── merged_cells.twdoc
│   ├── nested_table.twdoc
│   └── table_borders.twdoc
├── images/
│   ├── inline_image.twdoc
│   ├── wrapped_image.twdoc
│   └── multiple_images.twdoc
├── pagination/
│   ├── page_break.twdoc
│   ├── headers_footers.twdoc
│   └── multi_section.twdoc
├── scripts/
│   ├── arabic_rtl.twdoc
│   ├── hindi_devanagari.twdoc
│   ├── chinese_cjk.twdoc
│   └── mixed_direction.twdoc
└── complex/
    ├── academic_paper.twdoc
    ├── business_letter.twdoc
    └── resume.twdoc
```

### Updating References

When a layout change is intentional:

```bash
cargo test --test golden_layout -- --update-golden
```

This regenerates reference PNGs. The PR must include the updated images for review.

### Software Renderer

Golden-image tests use a software renderer (not Flutter/GPU) for CI compatibility:

- **Renderer:** `tiny-skia` (pure Rust, no GPU required)
- **Output:** RGBA buffer → PNG via `png` crate
- **Comparison:** pixel-by-pixel with 2% tolerance (anti-aliasing differences)

The software renderer consumes the same `DisplayList` format as the Flutter painter — if the golden test passes, the Flutter render should match.

## DOCX Round-Trip Tests

Automated tests verifying DOCX import/export fidelity using a curated corpus of real-world documents.

### Test Corpus

100+ DOCX files organized by category (see [docx-compatibility.md](architecture/docx-compatibility.md#test-corpus)):

```
crates/tw-docx/tests/corpus/
├── simple/           (10 files)
├── styled/           (15 files)
├── tables/           (15 files)
├── images/           (10 files)
├── lists/            (10 files)
├── headers_footers/  (5 files)
├── complex/          (10 files)
├── track_changes/    (5 files)
├── comments/         (5 files)
├── real_world/       (15 files)
└── edge_cases/       (10 files)
```

### Test Procedures

**Import test:**
```rust
#[test]
fn import_corpus() {
    for docx_path in corpus_files() {
        let result = tw_docx::import(&read_file(docx_path));
        assert!(result.is_ok(), "Failed to import {}", docx_path);
        let warnings = result.unwrap().warnings;
        assert!(warnings.iter().all(|w| !w.is_error()), "{:?}", warnings);

        // Render all pages, compare against reference
        let pages = layout_all_pages(&result.unwrap().document);
        for (i, page) in pages.iter().enumerate() {
            let png = render_page(page);
            let reference = reference_png(docx_path, i);
            assert_pixel_diff_below(&png, &reference, 0.02);
        }
    }
}
```

**Round-trip test:**
```rust
#[test]
fn round_trip_corpus() {
    for docx_path in corpus_files() {
        let original_bytes = read_file(docx_path);
        let imported = tw_docx::import(&original_bytes).unwrap();
        let exported = tw_docx::export(&imported.document, &imported.passthrough).unwrap();
        let reimported = tw_docx::import(&exported).unwrap();

        // Tier A: model comparison
        assert_model_equal(&imported.document, &reimported.document, Tier::A);

        // Tier B/C: byte preservation
        assert_passthrough_preserved(&imported.passthrough, &reimported.passthrough);
    }
}
```

**Performance test:**
```rust
#[test]
fn open_500_page_document() {
    let start = Instant::now();
    let result = tw_core::open("corpus/edge_cases/500_pages.docx");
    let elapsed = start.elapsed();
    assert!(result.is_ok());
    assert!(elapsed < Duration::from_secs(2));
}
```

## Performance Benchmarks

Using `criterion` for Rust benchmarks:

```
crates/tw-core/benches/
├── typing_latency.rs       1000× InsertText + layout
├── open_document.rs        Open various sizes
├── layout_page.rs          Layout single page (text, tables, images)
├── layout_document.rs      Layout full document (parallel)
├── search.rs               Search across document sizes
├── save_document.rs        Serialize and write
└── display_list_build.rs   Layout → display list pipeline

crates/tw-shape/benches/
├── shape_paragraph.rs      Shape typical paragraph
├── shape_complex_script.rs Shape Arabic, Hindi, CJK
└── atlas_rasterize.rs      Rasterize glyph batch

crates/tw-docx/benches/
├── import_docx.rs            Parse DOCX files of various sizes
├── export_docx.rs            Export with passthrough
└── round_trip.rs             Full import → export cycle
```

Benchmark baselines stored in `benches/baselines/`. CI compares against baselines and flags >10% regressions.

## End-to-End UI Tests

Flutter integration tests for user-facing workflows:

### Test Framework

- **Flutter integration tests** for widget-level testing
- **Patrol** or **Maestro** for native UI automation (desktop)
- Screenshot comparison for visual regression

### E2E Test Scenarios

| Scenario | Steps | Verification |
|----------|-------|-------------|
| Create and type | New doc → type text → verify rendering | Text visible, cursor positioned |
| Format text | Select text → bold → verify | Bold rendering in screenshot |
| Undo/redo | Type → undo → redo | Content restored |
| Save/load | Type → save → close → open | Content identical |
| Paste | Copy from external → paste | Formatted content inserted |
| Scroll | Open 100-page doc → scroll | 60 FPS, pages render |
| DOCX open | Open .docx file | Content renders correctly |
| Search | Open doc → Ctrl+F → search | Results highlighted |

### CI Pipeline

```yaml
# Test stages (future CI configuration)
stages:
  - unit-tests          # cargo test (all crates)
  - golden-images       # layout golden-image comparison
  - docx-corpus         # DOCX import/round-trip tests
  - benchmarks          # criterion benchmarks (nightly)
  - e2e-ui              # Flutter integration tests (nightly)
```

Unit tests and golden-image tests run on every PR. Benchmarks and E2E tests run nightly.

## Fuzz Testing

Format parsers are fuzz-tested to prevent crashes on malformed input:

```
crates/tw-docx/fuzz/
├── fuzz_targets/
│   ├── fuzz_import.rs      Random bytes → import (should not panic)
│   └── fuzz_opc_parse.rs   Random bytes → OPC parse

crates/tw-native/fuzz/
├── fuzz_targets/
│   └── fuzz_import.rs      Random bytes → JSON/ZIP parse
```

Run with `cargo fuzz run fuzz_import` (requires `cargo-fuzz`).

## Test Data Management

- Golden reference images are committed to the repository (Git LFS if >100 MB total)
- DOCX corpus files are committed (most are <1 MB each)
- Large test files (>10 MB) are generated by test setup scripts, not committed
- Benchmark baselines are committed and updated intentionally

## Coverage Reporting

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

Coverage reports uploaded to CI artifact storage. Coverage targets enforced per crate (see Unit Tests section above).

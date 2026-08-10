# DOCX corpus (F23.S1 continuous gate)

Synthetic fixtures are generated at test time by `corpus_render.rs` / `tests/common`
(`ensure_corpus`). Gate-eligible files exclude `_`-prefixed benchmarks and
`password_protected_*.docx`. Add real-world `.docx` files here for stronger
Word-fidelity coverage.

**Word-compatible feature fixture (F10–F14):** `word_compatible_feature_test.docx` is enriched with real image, shape, SmartArt diagram, chart, and OMML objects. Regenerate after editing placeholders with:

```bash
cargo run -p tw-docx --example enrich_word_compat_fixture
```

This writes both `crates/tw-docx/tests/corpus/word_compatible_feature_test.docx` and `app/test/fixtures/word_compatible_feature_test.docx`.

**Password-protected fixture (F22.S1):** `password_protected_standard.docx` is an ECMA-376 Standard-encrypted OOXML sample from the [office-crypto](https://github.com/udbhav1/office-crypto) test suite (MIT). Password: `Password1234_`. Excluded from the open/round-trip gate (tested by F22.S1).

## Gates (CI via `cargo test -p tw-docx`)

| Gate | Test | Pass criteria |
|------|------|----------------|
| Open + render | `corpus_render_gate_passes_95_percent` | ≥50 gate fixtures; ≥95% import + layout ≥1 page |
| Round-trip 50 | `i_f23_s1_roundtrip_50` | First 50 gate fixtures: import → export → re-import outline stable |
| Tier A categories | `tier_a_numbering_styles` | Numbering `lvlText`/start/`rPr`, styles, tables, char formats, images |
| Structural PDF | `tw-pdf` defaults | `PdfFidelity::Structural` only; VisualMatch errors until fonts embed |

## Categories (corpus growth)

See `docs/risk-mitigation.md` corpus table: styles, numbering, tables, images, HF/sections, floats, track changes, scripts, real-world, edge.

`ensure_corpus` seeds category fixtures (`cat_*.docx`) plus `synth_paragraph_NN.docx` padding to reach 50 gate-eligible files.

**Word screenshot baselines** are the fidelity gate for S2; self-goldens/fingerprints are regression-only.

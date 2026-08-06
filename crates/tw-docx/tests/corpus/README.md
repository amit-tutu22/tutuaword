# DOCX corpus (S1 continuous gate)

Synthetic fixtures are generated at test time by `corpus_render.rs` (`ensure_corpus`).
Add real-world `.docx` files here for stronger Word-fidelity coverage.

## Gates (CI via `cargo test -p tw-docx`)

| Gate | Test | Pass criteria |
|------|------|----------------|
| Open + render | `corpus_render_gate_passes_95_percent` | ≥95% of `.docx` files import and produce a non-empty display list |
| Round-trip Tier A fields | `tier_a_numbering_styles` | Custom `lvlText` / start / level `rPr` + styles survive export→import |
| Structural PDF | `tw-pdf` defaults | `PdfFidelity::Structural` only; VisualMatch errors until fonts embed |

## Categories (planned growth)

See `docs/risk-mitigation.md` corpus table: styles, numbering, tables, images, HF/sections, floats, track changes, scripts, real-world, edge.

**Word screenshot baselines** are the fidelity gate for S2; self-goldens/fingerprints are regression-only.

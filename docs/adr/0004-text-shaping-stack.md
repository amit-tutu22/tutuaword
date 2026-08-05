# ADR-0004: Pure-Rust Text Shaping Stack

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 1

## Context

Text shaping (converting character sequences to positioned glyphs) and line breaking are core to layout fidelity. The traditional approach uses C libraries: HarfBuzz for shaping, ICU for line breaking and segmentation. These require cmake, pkg-config, and per-platform native builds. WASM compilation of C libraries adds further complexity.

The development machine does not have cmake installed. WASM is a first-class target platform.

## Decision

Use a **pure-Rust text processing stack**:

| Function | Crate | Replaces |
|----------|-------|----------|
| Text shaping | `rustybuzz` | HarfBuzz |
| Glyph rasterization | `swash` | FreeType (subset) |
| Font enumeration | `fontdb` | Fontconfig / Core Text |
| Line breaking (UAX #14) | `unicode-linebreak` | ICU line break |
| Word segmentation (UAX #29) | `icu_segmenter` (icu4x) | ICU break iterator |
| Bidirectional text (UAX #9) | `unicode-bidi` | ICU bidi |
| Text buffer | `ropey` | — |

No C/C++ library dependencies for text processing.

## Consequences

**Positive:**
- No cmake, pkg-config, or native library builds required
- Identical shaping on all platforms including WASM
- `cargo build` works out of the box on any platform with Rust installed
- Faster CI builds (no C compilation step)
- Memory-safe text processing throughout

**Negative:**
- `rustybuzz` may lag behind HarfBuzz for edge-case complex scripts
- `swash` rasterization quality may differ from FreeType at small sizes
- `unicode-linebreak` may not support all locale-specific line breaking rules that ICU does
- Performance may be 10–20% slower than native HarfBuzz (acceptable within budget)

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **HarfBuzz + ICU (C libraries)** | Requires cmake and native builds; complicates WASM target; build friction on developer machines |
| **Skia text shaping** | Ties shaping to Skia rendering (see ADR-0003 — rendering is separate from shaping) |
| **Platform-native shaping (Core Text, DirectWrite)** | Different results per platform; no WASM support; cannot share layout between platforms |
| **All-in on icu4x** | icu4x is Rust but its segmenter alone doesn't cover shaping; still need rustybuzz |

## Fallback Plan

If `rustybuzz` proves insufficient for a specific script (e.g., complex Indic conjuncts), a targeted HarfBuzz FFI binding can be added behind a feature flag (`--features harfbuzz-shaping`) without changing the architecture. The `tw-shape` crate abstracts the shaper implementation.

# Sample DOCX smoke suite

Headless open / layout / edit / round-trip checks for real-world sample documents (from [sample-files.com](https://sample-files.com/) style fixtures). Complements the committed DOCX corpus in `crates/tw-docx/tests/corpus/` — these files are larger and live outside the repo by default.

## Harness

| Item | Location |
|------|----------|
| Integration test | [`crates/tw-core/tests/sample_files_smoke.rs`](../crates/tw-core/tests/sample_files_smoke.rs) |
| Fixture directory | `$SAMPLE_DOCX_DIR` if set, otherwise `$HOME/Downloads` |
| CI behavior | Skips when no matching fixtures are present (does not fail empty CI) |

## How to run

Place the sample files in a directory (or use Downloads), then:

```bash
SAMPLE_DOCX_DIR=/path/to/samples \
  cargo test -p tw-core --test sample_files_smoke -- --nocapture
```

Expected filenames (exact):

- `sample-files.com-basic-text.docx`
- `sample-files.com-formatted-report.docx`
- `sample-files.com-image-document.docx`
- `sample-files.com-table-document.docx`
- `sample-files.com-template.docx`
- `sample-files.com-lists.docx`
- `sample-files.com-tracked-changes.docx`
- `sample-files.com-multi-column.docx`
- `sample-files.com-large-document.docx`

## What each file is checked for

For every present file the harness asserts:

1. **Import** — `import_document_bundle` succeeds  
2. **Layout** — `SyncSession::relayout` produces at least the expected page count  
3. **Display list** — non-empty bytes that decode with a positive page size  
4. **Hit-test** — caret placement near the content origin (soft for image-only docs)  
5. **Edit** — insert a marker string and see it in plaintext  
6. **Round-trip** — export DOCX → re-import succeeds  

Feature-specific expectations:

| File | Feature gate |
|------|----------------|
| `basic-text` | Non-trivial plaintext |
| `formatted-report` | Non-trivial plaintext |
| `image-document` | ≥1 image (block or inline) |
| `table-document` | ≥1 table |
| `template` | Opens / lays out (tables optional) |
| `lists` | List paragraphs **or** list paragraph styles (`ListBullet*` / `ListNumber*`) |
| `tracked-changes` | ≥1 revision-marked run |
| `multi-column` | Section column count ≥ 2 |
| `large-document` | ≥2 pages and substantial plaintext |

## Baseline results (2026-08-11)

Run against Downloads fixtures on macOS (`cargo test -p tw-core --test sample_files_smoke`). All nine files **PASS**.

| File | Pages | Notable model counts | Notes |
|------|------:|----------------------|-------|
| basic-text | 2 | ~867 chars | Open/edit/round-trip OK |
| formatted-report | 4 | ~2.4k chars | Open/edit/round-trip OK |
| image-document | 4 | 6 images, 1 table | ~11 MB media; OK |
| table-document | 4 | 5 tables | OK |
| template | 2 | 3 tables | OK |
| lists | 4 | 57 list-style paras, 0 direct `numPr` | See caveat below |
| tracked-changes | 2 | 12 revisions | OK |
| multi-column | 12 | 2 columns | OK |
| large-document | 53 | ~63k chars, 28 tables | ~10.5s debug layout |

## Known caveat: style-linked lists

`sample-files.com-lists.docx` (and many Word docs) apply bullets/numbering via paragraph styles (`w:pStyle` = `ListBullet` / `ListNumber` / …) **without** a direct `w:numPr` on each paragraph. `numbering.xml` is imported into `DocumentSettings.numbering`, and styles are preserved, but `ParaFormat.numbering` stays empty until style→numbering resolve lands.

| Capability today | Status |
|------------------|--------|
| Open / layout / display list | Works |
| Plaintext + insert edit | Works |
| DOCX round-trip | Works |
| Detect “in list” via `format.numbering` | Misses style-only lists |
| Ribbon promote / demote on those paras | May no-op until resolve |

The smoke test counts OOXML list style ids / names so open stays green, and emits a **WARN** when lists are style-linked only.

## Related docs

- [Testing strategy](testing-strategy.md) — corpus, golden images, E2E  
- [DOCX compatibility](architecture/docx-compatibility.md) — Tier A/B/C fidelity  
- [Risk mitigation](risk-mitigation.md) — open-without-crash gates  

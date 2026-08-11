# Vision

## Product Goal

A professional document editor that:

- Opens and edits Microsoft Word (`.docx`) files with high fidelity
- Works on Windows, macOS, Linux, Android, iOS, and Web
- Is fully offline capable
- Includes AI as a built-in assistant rather than an add-on
- Supports enterprise deployment
- Offers real-time collaboration
- Has a plugin ecosystem

Positioning: **Microsoft Word + Notion AI + Grammarly + Git versioning + Local AI** — not a clone, but a next-generation document platform that maintains strong Word compatibility while differentiating through AI, collaboration, and modern architecture.

## Core Principles

1. **Microsoft Word compatibility first.** Users must be able to open, edit, and save `.docx` files without losing formatting. This is the table-stakes requirement.

2. **Offline first.** The editor must be fully functional without network access. Cloud sync, AI, and collaboration are enhancements, not dependencies.

3. **AI-assisted writing.** AI is a built-in capability — rewrite, summarize, translate, generate, review — not a bolt-on plugin. Local model support for privacy-sensitive users.

4. **Fast editing.** Typing latency under 10 ms. Opening a 500-page document under 2 seconds. 60 FPS scrolling and zoom.

5. **Open document format.** A native on-disk format that is human-readable, diffable, and version-control friendly. DOCX is the interchange format, not the source of truth.

6. **Real-time collaboration.** Live cursors, comments, suggestions, version history — powered by CRDTs, not operational transforms.

7. **Enterprise deployment.** SSO, audit logs, data residency, on-premises hosting, admin controls, compliance features.

8. **Plugin ecosystem.** Third-party extensions for grammar, citations, diagrams, integrations, and custom workflows.

## Explicit Non-Goals (Phase 1–3)

These are out of scope for the first 12 months:

- **Spreadsheet engine.** Tables are in scope; a full Excel-compatible spreadsheet is not.
- **Presentation engine.** PowerPoint export is a future feature; slide editing is not.
- **Legacy `.doc` (binary) format.** Only OOXML (`.docx`) and open formats (ODT, Markdown, HTML, RTF) are supported initially.
- **Macro/VBA support.** No execution of embedded macros or VBA scripts; `vbaProject.bin` and related parts are **preserved** on DOCX import/export (Tier C passthrough).
- **ActiveX controls.** Legacy embedded controls are preserved verbatim but not executed.
- **100% Word feature parity.** The goal is 99% formatting compatibility on common document types, not replication of every legacy feature Word has accumulated over 30 years.

## Differentiators

What makes this product competitive beyond Word compatibility:

| Capability | Description |
|------------|-------------|
| AI writing assistant | Inline suggestions, tone adjustment, grammar, while typing |
| AI document chat | Ask questions about the document, extract action items, find risks |
| AI templates | Generate resumes, proposals, contracts, meeting notes from prompts |
| Local AI mode | Run models on-device for privacy-sensitive users |
| Git-style versioning | Branch, merge, diff documents like code |
| Agent workflows | AI agents that review contracts, check compliance, prepare briefs |
| Natural language editing | "Make this proposal more persuasive and reduce it to two pages" |
| Interactive documents | Embed live charts, code execution, forms, mini-apps |

## Target Users

- **Individual professionals** — writers, lawyers, academics, consultants who need Word compatibility and AI assistance
- **Teams** — collaborative document editing with comments, suggestions, and version history
- **Enterprises** — self-hosted deployment, local AI, compliance, SSO, admin controls
- **Developers** — plugin authors, automation via APIs, document processing pipelines

## Business Model (Future)

| Tier | Features |
|------|----------|
| Free Personal | Offline editing, basic formatting, PDF export |
| Pro Subscription | Advanced AI, premium templates, cloud sync, collaboration |
| Enterprise | Self-hosted, local AI, compliance, SSO, admin controls, APIs |
| Developer Platform | Paid plugin marketplace and document automation APIs |

## Open Questions

These decisions are deferred and must be resolved before the relevant phase begins:

| Question | Impact | Resolve By |
|----------|--------|------------|
| **Product name** — "tutuaword" (repo) vs "Zyvor Docs" (vision) | Branding, crate naming, app bundle IDs | Before Phase 1 UI work |
| **Cloud backend provider** — self-hosted vs AWS/GCP/Azure | Collaboration, sync, AI routing | Before Phase 5 |
| **Default AI provider** — resolved: OpenAI (cloud), llama.cpp (desktop local) | AI feature quality, cost, privacy | Phase 4 (see [ADR-0010](../adr/0010-ai-provider-abstraction.md)) |
| **Plugin sandbox model** — WASM vs native process isolation | Security, performance, SDK complexity | Before Phase 6 |
| **Mobile input model** — touch selection, Apple Pencil, stylus | Mobile UX design | Before mobile Phase 2 |

## Team Sizing

| Scope | Engineers | Timeline |
|-------|-----------|----------|
| Lean MVP | 6–8 | Months 1–6 |
| Ambitious commercial | 20 | Months 1–24 |
| Full commercial-grade | 30 | Months 1–24+ |

See [roadmap.md](roadmap.md) for phase-by-phase team allocation guidance.

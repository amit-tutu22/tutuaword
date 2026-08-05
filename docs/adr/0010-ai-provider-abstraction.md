# ADR-0010: AI Provider Abstraction

**Status:** Accepted
**Date:** 2026-08-04
**Phase:** 4

## Context

AI is a core product differentiator. Users need access to cloud models (OpenAI, Gemini, Claude) for quality, in-process local models (llama.cpp) for privacy without a separate service, and on-device models (ONNX, Apple Foundation Models) for mobile. Enterprise customers may require specific providers, self-hosted models, or local-only policies.

Hardcoding a single AI provider (e.g., OpenAI only) limits the product's reach, increases cost, and violates the privacy-first principle.

## Decision

Implement a **two-layer provider-agnostic AI stack**:

1. **`AiProvider`** — transport adapter (HTTP, llama.cpp bindings, ONNX session)
2. **`AiService`** — UI-facing capability API (`summarize`, `rewrite`, `translate`, `generate`, `explain`, `correct_grammar`)

A **`HybridRouter`** selects the provider based on:
- **Task complexity** (`AiTask`) — grammar → local, long summarize → cloud
- **User routing mode** — Always Local, Always Cloud, Automatic (default)
- **Platform** — desktop local default = llama.cpp; mobile = ONNX / Apple FM / LiteRT
- **Enterprise policy** — allow/block cloud, per-provider blocklist, classification rules
- **Provider availability** — fallback if primary is down
- **Installed AI modules** — smallest capable module from marketplace

AI capabilities are implemented as prompt templates, not hardcoded provider calls. The Flutter UI never references a provider ID.

## Consequences

**Positive:**
- Users choose cloud, local, or hybrid — no vendor lock-in
- Desktop local AI works without installing Ollama or any background service
- Enterprise admins enforce AI policy without code changes
- New providers added by implementing `AiProvider` — no changes to AI capabilities
- Hybrid routing balances cost, privacy, quality, and latency automatically
- On-demand AI modules avoid loading one monolithic model

**Negative:**
- Prompt templates must be provider-agnostic
- Quality varies between providers — must test each capability against each provider
- Local model quality may be insufficient for some tasks — cloud fallback needed
- Provider API changes require adapter updates
- Hybrid routing adds complexity vs a single-provider design

## Rejected Alternatives

| Alternative | Why Rejected |
|-------------|-------------|
| **OpenAI only** | Vendor lock-in; no offline/privacy option; enterprise customers require choice |
| **Local only** | Quality gap vs cloud models for long-document tasks |
| **Ollama as default local** | Requires separate install and background process; poor fit for commercial desktop app |
| **LangChain/LlamaIndex framework** | Python-centric; heavy dependency; over-abstracted for our use case |
| **Hardcoded prompts per provider** | Maintenance nightmare; adding a provider requires duplicating all capabilities |
| **AI as Flutter plugin** | AI needs document context from Rust model; cross-language context building is expensive |
| **Single monolithic model download** | Wastes disk and memory; users only need modules for features they use |

## Provider Priority

| Priority | Provider | Role |
|----------|----------|------|
| 1 | llama.cpp | Default desktop local (in-process) |
| 2 | OpenAI | Default cloud at launch |
| 3 | Google Gemini | Alternative cloud |
| 4 | ONNX Runtime | Mobile on-device |
| 5 | Apple Foundation Models | macOS/iOS native |
| 6 | Anthropic Claude | Alternative cloud |
| 7 | Mistral / OpenRouter | Multi-model cloud access |
| 8 | Ollama | Optional local (dev/advanced users) |
| 9 | Custom endpoint | Enterprise self-hosted |

## Hybrid Routing

`HybridRouter` is the routing authority. Example defaults in Automatic mode:

| Task | Route |
|------|-------|
| Spell check | Local rules (no LLM) |
| Grammar | Small local model |
| Rewrite | Local model |
| Summarize 200 pages | Cloud |
| Generate legal contract | Cloud |
| Document chat | Local with cloud fallback |

User override: **Always Local**, **Always Cloud**, **Automatic**.

See [ai-platform.md](../architecture/ai-platform.md) for full routing table and module marketplace design.

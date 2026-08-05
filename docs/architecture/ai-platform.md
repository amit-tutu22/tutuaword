# AI Platform

> **Implementation status:** Routing, policy, and `AiProvider` trait exist in `tw-ai`; no production provider implementations yet (tests use mocks only). See [Long-Tail Gaps](../long-tail-gaps.md) §6.

The AI platform (`tw-ai`) provides document intelligence as a built-in capability. This is the primary product differentiator. The editor is **not tied to a single AI engine** — a provider-agnostic abstraction layer lets users switch between cloud APIs, in-process local models, and platform-native on-device inference.

## Design Principles

1. **Provider-agnostic.** Cloud, local, and on-device providers implement the same interface. The UI never knows which model is running.
2. **Context-aware.** AI operations receive structured document context, not raw text dumps.
3. **Non-destructive.** AI suggestions are previews; the user accepts or rejects. Original content is preserved until explicitly replaced.
4. **Offline-capable.** Desktop local AI via in-process llama.cpp; mobile via ONNX and platform-native frameworks.
5. **Hybrid by default.** Route tasks to the smallest capable model — local for grammar, cloud for long-document summarization.
6. **Extensible.** New capabilities are prompt templates; new providers are adapter implementations.

## Architecture

```
Word Application (Flutter UI)
        │
        ▼
  tw-ai::AiService          ← UI calls capability methods only
  ├── ContextBuilder
  ├── PromptTemplate
  ├── HybridRouter          ← task + policy + platform routing
  └── ResponseParser
        │
        ├──────────┬──────────┬──────────┬──────────┐
        ▼          ▼          ▼          ▼          ▼
     OpenAI     Gemini    llama.cpp    ONNX     Apple FM
     Claude     Mistral   Ollama*      LiteRT   Custom
     OpenRouter
```

\* Ollama is supported as an optional adapter for developers and power users; it is not the default local provider for commercial desktop builds.

## Two-Layer Abstraction

### Transport layer — `AiProvider`

Low-level adapter for a specific inference backend (HTTP API, llama.cpp bindings, ONNX session):

```rust
pub trait AiProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn capabilities(&self) -> AiCapabilities;
    fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, AiError>;
    fn stream(&self, request: &CompletionRequest) -> Result<CompletionStream, AiError>;
    fn is_available(&self) -> bool;
}

pub struct AiCapabilities {
    pub max_context_tokens: u32,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub local: bool,
}
```

### Capability layer — `AiService`

UI-facing API. Each method maps to an `AiTask`, prompt template, and hybrid routing decision:

```rust
pub enum AiTask {
    SpellCheck,       // rule-based, no LLM
    Grammar,          // small local model
    Rewrite,
    Translate,
    Summarize,
    Generate,
    Explain,
    CorrectGrammar,
    Chat,
}

pub enum AiRoutingMode {
    AlwaysLocal,
    AlwaysCloud,
    Automatic,  // default
}

pub trait AiService {
    fn summarize(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError>;
    fn rewrite(&self, ctx: &DocumentContext, tone: RewriteTone) -> Result<AiResponse, AiError>;
    fn translate(&self, ctx: &DocumentContext, lang: &str) -> Result<AiResponse, AiError>;
    fn generate(&self, ctx: &DocumentContext, prompt: &str) -> Result<AiResponse, AiError>;
    fn explain(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError>;
    fn correct_grammar(&self, ctx: &DocumentContext) -> Result<AiResponse, AiError>;
}
```

The Flutter UI calls `AiService` methods only — never a provider ID.

## Provider Comparison

| AI Option | Best For | Offline | Cost | Quality | Speed |
|-----------|----------|---------|------|---------|-------|
| OpenAI API | Premium AI features | No | Pay per use | High | High |
| Google Gemini API | Cloud AI alternative | No | Pay per use | High | High |
| Anthropic Claude | Cloud AI alternative | No | Pay per use | High | High |
| Mistral / OpenRouter | Multi-model cloud access | No | Pay per use | High | High |
| llama.cpp | Desktop local (default) | Yes | Free | High | High |
| Ollama | Dev / power-user local | Yes | Free | High | Medium |
| ONNX Runtime | Mobile on-device | Yes | Free | Medium | High |
| Apple Foundation Models | macOS / iOS native | Yes | Free | High | High |
| Google AI Edge / LiteRT | Android on-device | Yes | Free | Medium | High |
| Custom endpoint | Enterprise self-hosted | Varies | Varies | Varies | Varies |

## Platform Recommendations

### Desktop local AI — llama.cpp (default)

For a commercial desktop application, **llama.cpp is preferred over Ollama**:

- No separate background service or user install step
- Models bundled with the app or downloaded in-app
- Direct control over memory, performance, and updates
- In-process inference via Rust bindings

Ollama remains available as an optional adapter for developers who already run it locally.

### Mobile local AI — avoid large LLMs

Do not run 70B-class models on phones. Use:

- **ONNX Runtime** — quantized small models (<500 MB)
- **Apple Foundation Models** — on supported macOS/iOS devices
- **Google AI Edge / LiteRT** — on Android where appropriate

### Cloud AI — multi-provider

Support multiple cloud providers to avoid vendor lock-in:

- OpenAI (default cloud at launch)
- Google Gemini
- Anthropic Claude
- Mistral
- OpenRouter (access to many models through one API)
- Self-hosted inference servers (enterprise)

## Supported Providers (Phase 4)

| Provider | Platform | Library/API | Default Role |
|----------|----------|-------------|--------------|
| llama.cpp | Desktop | Native bindings | **Default local desktop** |
| OpenAI | Cloud | REST API | Default cloud |
| Google Gemini | Cloud | REST API | Alternative cloud |
| Anthropic Claude | Cloud | REST API | Alternative cloud |
| Mistral | Cloud | REST API | Alternative cloud |
| OpenRouter | Cloud | REST API | Multi-model gateway |
| ONNX Runtime | Mobile / desktop | `ort` crate | On-device inference |
| Apple Foundation Models | macOS / iOS | Platform API | Native Apple devices |
| Google AI Edge / LiteRT | Android | Platform API | Mobile on-device |
| Ollama | Desktop | HTTP API (localhost) | Optional (not default) |
| Custom | Enterprise | User-configured | Self-hosted |

Provider IDs (for routing internals): `llama_cpp`, `openai`, `gemini`, `claude`, `mistral`, `openrouter`, `onnx`, `apple_fm`, `litert`, `ollama`, `custom`.

## Hybrid Routing

`HybridRouter` selects a provider based on `AiTask`, `AiRoutingMode`, platform, and enterprise `AiPolicy`.

### Task routing defaults (Automatic mode)

| Task | Default route | Rationale |
|------|---------------|-----------|
| Spell check | Local rules | No LLM needed |
| Grammar / CorrectGrammar | Small local model (3B–8B) | Low latency, privacy |
| Rewrite sentence | Local model | Fast, offline-capable |
| Translate paragraph | Local or cloud | User/policy dependent |
| Summarize long document | Cloud | Context window + quality |
| Generate complex content | Cloud | Quality + safety |
| Document chat | Local for privacy; cloud fallback | User routing mode |
| Explain selection | Local or cloud | Selection size dependent |

### User routing modes

| Mode | Behavior |
|------|----------|
| **Automatic** (default) | Route by task complexity and platform |
| **Always Local** | Force local providers; fail if unavailable |
| **Always Cloud** | Force cloud providers; fail if policy blocks |

Enterprise admins can enforce local-only AI or block specific cloud providers via `AiPolicy`. See [security.md](security.md).

```rust
pub struct HybridRouter {
    providers: HashMap<String, Box<dyn AiProvider>>,
    default_cloud: String,
    default_local: String,
    platform: AiPlatform,
    routing_mode: AiRoutingMode,
    policy: AiPolicy,
    modules: AiModuleRegistry,
}

pub struct AiPolicy {
    pub allow_cloud: bool,
    pub allow_local: bool,
    pub blocked_providers: HashSet<String>,
    pub require_local_for_classification: Option<DataClassification>,
    pub max_tokens_per_request: u32,
    pub rate_limit: Option<RateLimit>,
}
```

## Small Local Models

Many writing tasks do not require flagship cloud models. Compact models (3B–8B parameters) work well for:

- Grammar correction
- Tone adjustment
- Rewriting
- Title generation
- Email drafting

Recommended compact models for desktop local inference:

| Model | Size class | Tasks |
|-------|------------|-------|
| Gemma 3 (small) | 1B–4B | Grammar, rewrite |
| Qwen 2.5 / 3 (small) | 3B–7B | Translate, rewrite |
| Llama 3.2 | 1B / 3B | Grammar, completion |
| Phi-4 Mini | ~3B | Grammar, explain |

## AI Module Marketplace

Rather than loading one monolithic model, users download AI modules on demand:

| Module | Approx size | Capabilities |
|--------|-------------|--------------|
| Grammar | 200 MB | CorrectGrammar, spell-adjacent |
| Translation | 300 MB | Translate |
| Writing | 700 MB | Rewrite, Expand, Shorten |
| Reasoning | 4 GB | Summarize long docs, complex chat |
| Coding | 3 GB | Code blocks, technical writing |

`AiModuleRegistry` tracks installed modules. `HybridRouter` selects the smallest installed module capable of handling each `AiTask`.

```rust
pub struct AiModule {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub tasks: Vec<AiTask>,
    pub model_path: Option<PathBuf>,
    pub installed: bool,
}
```

## AI Feature Catalog

### Writing Assistant

| Feature | AiService method | Default route |
|---------|------------------|---------------|
| Rewrite | `rewrite()` | Local |
| Shorten | `rewrite(Shorten)` | Local |
| Expand | `rewrite(Expand)` | Local |
| Improve tone | `rewrite(tone)` | Local |
| Grammar correction | `correct_grammar()` | Local |
| Translate | `translate()` | Local or cloud |

### Document Intelligence

| Feature | AiService method | Default route |
|---------|------------------|---------------|
| Summarize document | `summarize()` | Cloud (long docs) |
| Extract action items | `generate()` | Cloud |
| Explain sections | `explain()` | Local or cloud |
| Generate table of contents | `generate()` | Local or cloud |
| Find inconsistencies | `generate()` | Cloud |

### Smart Editing

| Feature | Output |
|---------|--------|
| Auto-format headings | DocumentOperations |
| Convert paragraphs to lists | DocumentOperations |
| Generate tables | DocumentOperations |
| Suggest citations | TextSuggestion |

### Productivity Templates

| Feature | AiService method |
|---------|------------------|
| Meeting minutes | `generate()` |
| Proposal generation | `generate()` |
| Resume builder | `generate()` |
| Contract drafting | `generate()` |
| Report generation | `generate()` |

## Context Building

AI operations receive structured context extracted from the document, not the entire document as a text blob.

```rust
pub struct DocumentContext {
    pub selection: Option<ContextSelection>,
    pub surrounding_paragraphs: Vec<ParagraphSummary>,
    pub document_metadata: DocumentMetadata,
    pub style_info: Option<StyleContext>,
    pub total_token_estimate: u32,
    pub page_count: u32,
    pub classification: Option<DataClassification>,
}
```

Context building respects token limits — for large documents, it includes the selection plus surrounding paragraphs up to the provider's context window.

## Inline Suggestions (While Typing)

| Trigger | Suggestion Type | Latency Target | Route |
|---------|----------------|----------------|-------|
| Pause >500 ms after sentence | Grammar correction | <500 ms | Local |
| Pause >1 s mid-paragraph | Sentence completion | <1 s | Local |
| Formatting inconsistency | Style suggestion | <200 ms | Rule-based (no AI) |
| Passive voice detected | Active voice rewrite | <500 ms | Local |

Inline suggestions appear as ghost text. User presses Tab to accept, Escape to dismiss.

## Document Chat

```rust
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub references: Vec<DocumentReference>,
}
```

Example queries: "What is this document about?", "List all action items", "Find risks in section 3", "Explain the table on page 12".

## AI Review

| Check | Method | Route |
|-------|--------|-------|
| Grammar | AI + rule-based | Local |
| Readability | Flesch-Kincaid + AI | Local or cloud |
| Duplicate content | Text similarity | Local (no LLM) |
| Missing citations | Pattern matching + AI | Cloud |
| Inconsistent formatting | Rule-based | No AI |
| Accessibility | Rule-based | No AI |

## Prompt Templates

```rust
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub user_prompt_template: String,
    pub response_format: ResponseFormat,
    pub max_tokens: u32,
    pub temperature: f32,
    pub preferred_task: AiTask,
}
```

Templates are stored as JSON files. Users and enterprises can customize templates.

## Response Handling

AI responses that modify the document produce `Command` objects (same as user edits):

```rust
pub enum AiResponse {
    TextSuggestion { text: String, replace_range: DocRange },
    DocumentOperations { commands: Vec<Command> },
    ChatReply { message: String, references: Vec<DocumentReference> },
    ReviewResults { issues: Vec<ReviewIssue> },
}
```

The UI shows a preview before applying. User confirms → commands go through `tw-edit::apply()` → undoable.

## Local AI Configuration

```rust
pub struct LocalAiConfig {
    pub model_path: PathBuf,
    pub model_format: ModelFormat,  // GGUF, ONNX
    pub context_length: u32,
    pub gpu_layers: u32,
    pub threads: u32,
    pub backend: LocalBackend,  // LlamaCpp, Onnx, AppleFm, LiteRt, Ollama
}

pub enum LocalBackend {
    LlamaCpp,   // default desktop
    Onnx,
    AppleFm,
    LiteRt,
    Ollama,     // optional
}
```

Desktop default: in-process **llama.cpp**. Mobile default: **ONNX** or **Apple Foundation Models** depending on platform.

## FFI Surface

Flutter accesses AI via the session facade — capability methods, not provider IDs:

```dart
// Capability methods (preferred)
Future<AiResponse> aiSummarize(int docId, {String? selection});
Future<AiResponse> aiRewrite(int docId, RewriteTone tone, {String? selection});
Future<AiResponse> aiTranslate(int docId, String lang, {String? selection});
Future<AiResponse> aiCorrectGrammar(int docId, {String? selection});

// Routing and provider settings
Future<void> aiSetRoutingMode(AiRoutingMode mode);
Future<void> aiSetPreferredCloudProvider(String providerId);
Future<List<AiModule>> aiListModules();
Future<void> aiInstallModule(String moduleId);

// Streaming
Stream<String> aiStream(int docId, AiTask task, {String? selection});

// Document chat
Stream<ChatMessage> aiChat(int docId, String message, {List<ChatMessage>? history});
```

Settings UI (Phase 4): provider picker, routing mode (Always Local / Always Cloud / Automatic), module marketplace.

## Metrics and Telemetry

| Metric | Purpose |
|--------|---------|
| Request latency (p50, p99) | Provider performance |
| Token usage per request | Cost tracking |
| Accept/reject rate | Suggestion quality |
| Provider availability | Fallback decisions |
| Local vs cloud ratio | Privacy compliance |
| Module download success | Marketplace health |

Telemetry is opt-in for personal edition, configurable by enterprise admin.

## Phase 4 Exit Criteria

| Criterion | Measurement |
|-----------|-------------|
| Local AI rewrite works offline with llama.cpp | Integration test |
| Automatic routing sends grammar to local, long summarize to cloud | Unit test on HybridRouter |
| Switch provider / routing mode without restart | Integration test |
| User can choose Always Local / Always Cloud / Automatic | Settings UI test |
| Inline grammar suggestion <500 ms (local) | Latency benchmark |
| Cloud rewrite <3 s | Latency benchmark |
| AI sidebar responds to 10 standard prompts | Prompt test suite |

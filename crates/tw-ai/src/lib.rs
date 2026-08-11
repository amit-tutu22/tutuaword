mod apply;
mod chat;
mod context;
mod generate;
mod http;
mod module;
mod policy;
mod provider;
mod providers;
mod router;
mod service;
mod smart;
mod task;
mod visual;

pub use apply::{apply_ai_response, apply_text_suggestion, suggestion_to_commands};
pub use chat::{
    chunk_document, ChatMessage, ChatRole, DocumentChatSession, DocumentChunk, DocumentRagIndex,
    DocumentReference,
};
pub use generate::{
    document_from_markdown, generate_content, ContentKind, GeneratedDocument,
};
pub use smart::{
    analyze_document_heuristics, apply_smart_edit_plan, parse_smart_edit_plan, smart_edit_commands,
    suggest_smart_edit, HeadingSuggestion, SmartEditPlan,
};
pub use visual::{
    apply_visual_suggestion, parse_visual_suggestion, suggest_visual, visual_suggestion_commands,
    VisualKind, VisualSuggestion,
};
pub use context::{
    ContextSelection, DocumentContext, DocumentMetadata, ParagraphSummary,
};
pub use http::{HttpClient, MockHttpClient, MockHttpCall};
pub use module::{AiModule, AiModuleRegistry};
pub use policy::{AiPolicy, DataClassification, RateLimit};
pub use provider::{
    AiCapabilities, AiError, AiProvider, CompletionRequest, CompletionResponse, CompletionStream,
    APPLE_FM, CLAUDE, CUSTOM, GEMINI, LITERT, LLAMA_CPP, MISTRAL, OLLAMA, ONNX, OPENAI,
    OPENROUTER, RULES,
};
pub use providers::{
    production_desktop_router, register_production_providers, GeminiProvider, LlamaCppProvider,
    OpenAiProvider,
};
pub use router::{HybridRouter, ProviderRouter};
pub use service::{AiResponse, AiService, AiServiceImpl, AiSession, PromptTemplate};
pub use task::{AiPlatform, AiRoutingMode, AiTask, RewriteTone};

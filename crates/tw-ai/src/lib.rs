mod context;
mod module;
mod policy;
mod provider;
mod router;
mod service;
mod task;

pub use context::{
    ContextSelection, DocumentContext, DocumentMetadata, ParagraphSummary,
};
pub use module::{AiModule, AiModuleRegistry};
pub use policy::{AiPolicy, DataClassification, RateLimit};
pub use provider::{
    AiCapabilities, AiError, AiProvider, CompletionRequest, CompletionResponse, CompletionStream,
    APPLE_FM, CLAUDE, CUSTOM, GEMINI, LITERT, LLAMA_CPP, MISTRAL, OLLAMA, ONNX, OPENAI,
    OPENROUTER, RULES,
};
pub use router::{HybridRouter, ProviderRouter};
pub use service::{AiResponse, AiService, AiServiceImpl, AiSession, PromptTemplate};
pub use task::{AiPlatform, AiRoutingMode, AiTask, RewriteTone};

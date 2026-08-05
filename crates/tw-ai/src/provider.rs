use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("provider not available: {0}")]
    ProviderUnavailable(String),
    #[error("completion failed: {0}")]
    CompletionFailed(String),
    #[error("policy violation: {0}")]
    PolicyViolation(String),
    #[error("not implemented")]
    NotImplemented,
}

#[derive(Debug, Clone, Default)]
pub struct AiCapabilities {
    pub max_context_tokens: u32,
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub local: bool,
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub text: String,
    pub tokens_used: u32,
}

pub struct CompletionStream;

pub trait AiProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn capabilities(&self) -> AiCapabilities;
    fn complete(&self, request: &CompletionRequest) -> Result<CompletionResponse, AiError>;
    fn stream(&self, request: &CompletionRequest) -> Result<CompletionStream, AiError>;
    fn is_available(&self) -> bool;
}

// Provider ID constants
pub const LLAMA_CPP: &str = "llama_cpp";
pub const OPENAI: &str = "openai";
pub const GEMINI: &str = "gemini";
pub const CLAUDE: &str = "claude";
pub const MISTRAL: &str = "mistral";
pub const OPENROUTER: &str = "openrouter";
pub const ONNX: &str = "onnx";
pub const APPLE_FM: &str = "apple_fm";
pub const LITERT: &str = "litert";
pub const OLLAMA: &str = "ollama";
pub const CUSTOM: &str = "custom";
pub const RULES: &str = "rules";

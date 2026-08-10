//! Production AI provider adapters (F28.S1).

mod gemini;
mod llama;
mod openai;

pub use gemini::GeminiProvider;
pub use llama::LlamaCppProvider;
pub use openai::OpenAiProvider;

use crate::http::HttpClient;
use crate::provider::{LLAMA_CPP, OPENAI};
use crate::router::HybridRouter;
use crate::task::AiPlatform;
use std::sync::Arc;

/// Build a desktop HybridRouter with OpenAI, Gemini, and local llama adapters.
pub fn production_desktop_router(
    http: Arc<dyn HttpClient>,
    openai_api_key: Option<String>,
    gemini_api_key: Option<String>,
    llama_endpoint: Option<String>,
) -> HybridRouter {
    let mut router = HybridRouter::new(OPENAI, LLAMA_CPP, AiPlatform::Desktop);
    router.register(Box::new(OpenAiProvider::new(
        http.clone(),
        openai_api_key.unwrap_or_default(),
    )));
    router.register(Box::new(GeminiProvider::new(
        http.clone(),
        gemini_api_key.unwrap_or_default(),
    )));
    router.register(Box::new(LlamaCppProvider::new(
        http,
        llama_endpoint.unwrap_or_else(|| "http://127.0.0.1:11434".into()),
    )));
    router
}

/// Convenience: register production providers onto an existing router.
pub fn register_production_providers(
    router: &mut HybridRouter,
    http: Arc<dyn HttpClient>,
    openai_api_key: impl Into<String>,
    gemini_api_key: impl Into<String>,
    llama_endpoint: impl Into<String>,
) {
    router.register(Box::new(OpenAiProvider::new(http.clone(), openai_api_key)));
    router.register(Box::new(GeminiProvider::new(http.clone(), gemini_api_key)));
    router.register(Box::new(LlamaCppProvider::new(http, llama_endpoint)));
}
